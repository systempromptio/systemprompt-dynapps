//! Mail attachments through the shared file layer against a simulated Graph
//! and an in-memory store: a download is read raw from `/$value` (so one
//! whose base64 would pass the response limit still fits), kept as the
//! caller's document and never returned as base64, an item attachment is
//! passed through as Graph describes it, an attachment comes in by reference or
//! as small inline base64, a large one goes through an upload session this
//! server fills without the user's bearer, and an upload URL this server
//! did not open for the caller or a file of another user never gets a byte
//! (the session binding itself is in `m365_mail_attach_session`).

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use systemprompt::identifiers::McpExecutionId;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_microsoft_365::graph::GraphLimits;
use systemprompt_mcp_microsoft_365::graph::upload::UPLOAD_CHUNK_BYTES;
use systemprompt_mcp_microsoft_365::tools::mail_attach::{
    add_mail_attachment, delete_mail_attachment, get_mail_attachment,
};
use systemprompt_mcp_microsoft_365::tools::mail_upload::{
    create_upload_session, upload_attachment_chunk,
};
use systemprompt_mcp_shared::files::{FileAction, FileCall};

use crate::files_support::{ALICE, BOB, document_of, exec_id, harness, signed_in};
use crate::m365_support::{FakeTokens, Reply, Seen, Sim, client, fast_limits};

fn parse<T: serde::de::DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

fn as_caller<'a>(ctx: &'a RequestContext, exec: &'a McpExecutionId) -> FileCall<'a> {
    FileCall {
        ctx,
        tool: "d365_mail_test",
        exec_id: exec,
        hop: None,
    }
}

// Graph for attachment writes: a session whose uploadUrl is on the
// simulated Graph's own origin; a chunk PUT answers 200 until the range
// reaches the total, then 201 with the attachment.
fn attachment_graph(seen: &Seen, _: usize) -> Reply {
    let host = seen.header("host").unwrap_or_default().to_owned();
    match (seen.method.as_str(), seen.path.as_str()) {
        ("POST", path) if path.ends_with("/createUploadSession") => Reply::json(
            201,
            &json!({"uploadUrl": format!("http://{host}/upload/s1?authtoken=sas")}),
        ),
        ("PUT", _) if last_range(seen) => Reply::json(201, &json!({"id": "att-big"})),
        ("PUT", _) => Reply::json(200, &json!({"nextExpectedRanges": ["more"]})),
        ("DELETE", _) => Reply::bytes(204, b""),
        _ => Reply::json(201, &json!({"id": "att1"})),
    }
}

fn last_range(seen: &Seen) -> bool {
    let range = seen.header("content-range").unwrap_or_default();
    let (span, total) = range.trim_start_matches("bytes ").split_once('/').unwrap();
    let end: u64 = span.split_once('-').unwrap().1.parse().unwrap();
    end + 1 == total.parse::<u64>().unwrap()
}

// Graph for one attachment: `metadata` at its path and `raw` at `/$value`.
async fn attachment_with(metadata: Value, raw: Vec<u8>) -> Sim {
    Sim::start(move |seen, _| {
        if seen.path.ends_with("/$value") {
            Reply::bytes(200, &raw)
        } else {
            Reply::json(200, &metadata)
        }
    })
    .await
}

#[tokio::test]
async fn m365_mail_get_mail_attachment_reads_the_raw_value_into_a_document_not_base64() {
    let pdf = b"%PDF-1.7 tiny".to_vec();
    // A Graph that ignored `$select` and sent the base64 along with the metadata.
    let metadata = json!({
        "@odata.type": "#microsoft.graph.fileAttachment",
        "id": "att1", "name": "report.pdf", "contentType": "application/pdf",
        "size": pdf.len(), "contentBytes": STANDARD.encode(&pdf)
    });
    let sim = attachment_with(metadata, pdf.clone()).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let input = parse(json!({"message_id": "m1", "attachment_id": "att1"}));
    let reply = get_mail_attachment(&graph, &h.files, as_caller(&ctx, &exec), input)
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2, "{calls:#?}");
    assert_eq!(calls[0].path, "/v1.0/me/messages/m1/attachments/att1");
    let select = calls[0].query.get("$select").cloned().unwrap_or_default();
    assert!(
        select.contains("name") && !select.contains("contentBytes"),
        "the metadata is read without the base64: {select:?}"
    );
    assert_eq!(
        (calls[1].method.as_str(), calls[1].path.as_str()),
        ("GET", "/v1.0/me/messages/m1/attachments/att1/$value")
    );
    let data = serde_json::to_value(&reply).unwrap()["data"].clone();
    assert!(data.get("contentBytes").is_none(), "{data}");
    assert_eq!(data["name"], "report.pdf");
    assert_eq!(data["file"]["name"], "report.pdf");
    assert_eq!(data["file"]["mime"], "application/pdf");
    assert_eq!(data["file"]["size_bytes"], pdf.len());
    assert!(
        data["file"]["file_ref"]["document_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    );
    assert!(
        data["file"]["download_url"]
            .as_str()
            .unwrap()
            .starts_with("https://api.test/")
    );
    let stored = h.store.with(|state| {
        state
            .records
            .values()
            .find(|(record, _)| record.name == "report.pdf")
            .map(|(record, bytes)| (record.owner.to_string(), bytes.clone()))
    });
    assert_eq!(stored, Some((ALICE.to_owned(), pdf)));
    let events = h.audit.events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].action, FileAction::Stored));
}

#[tokio::test]
async fn m365_mail_get_mail_attachment_keeps_one_whose_base64_would_pass_the_response_limit() {
    let size = 20 * 1024 * 1024;
    let zip: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
    assert!(
        STANDARD.encode(&zip).len() > GraphLimits::default().max_response_bytes,
        "as base64 in JSON this attachment cannot be read"
    );
    let metadata = json!({
        "@odata.type": "#microsoft.graph.fileAttachment",
        "id": "big", "name": "plans.zip", "contentType": "application/zip", "size": size
    });
    let sim = attachment_with(metadata, zip.clone()).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let input = parse(json!({"message_id": "m1", "attachment_id": "big"}));
    let reply = get_mail_attachment(&graph, &h.files, as_caller(&ctx, &exec), input)
        .await
        .unwrap();

    let data = serde_json::to_value(&reply).unwrap()["data"].clone();
    assert_eq!(data["file"]["size_bytes"], size, "{data}");
    assert_eq!(data["file"]["mime"], "application/zip");
    let stored = h.store.with(|state| {
        state
            .records
            .values()
            .find(|(record, _)| record.name == "plans.zip")
            .map(|(_, bytes)| bytes == &zip)
    });
    assert_eq!(stored, Some(true));
}

#[tokio::test]
async fn m365_mail_get_mail_attachment_with_base64_but_no_type_is_still_read_raw() {
    let csv = b"a,b\n1,2\n".to_vec();
    let metadata = json!({
        "id": "att3", "name": "data.csv", "contentType": "text/csv",
        "contentBytes": STANDARD.encode(&csv)
    });
    let sim = attachment_with(metadata, csv.clone()).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let input = parse(json!({"message_id": "m1", "attachment_id": "att3"}));
    let reply = get_mail_attachment(&graph, &h.files, as_caller(&ctx, &exec), input)
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2, "{calls:#?}");
    assert!(calls[1].path.ends_with("/attachments/att3/$value"));
    let data = serde_json::to_value(&reply).unwrap()["data"].clone();
    assert!(data.get("contentBytes").is_none(), "{data}");
    assert_eq!(data["file"]["size_bytes"], csv.len());
}

#[tokio::test]
async fn m365_mail_get_mail_attachment_passes_an_item_attachment_through() {
    let item =
        json!({"@odata.type": "#microsoft.graph.itemAttachment", "id": "att2", "name": "Fwd"});
    let body = item.clone();
    let sim = Sim::start(move |_, _| Reply::json(200, &body)).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let input = parse(json!({"message_id": "m1", "attachment_id": "att2"}));
    let reply = get_mail_attachment(&graph, &h.files, as_caller(&ctx, &exec), input)
        .await
        .unwrap();

    assert_eq!(serde_json::to_value(&reply).unwrap()["data"], item);
    assert_eq!(sim.calls().len(), 1, "an item attachment is not downloaded");
    assert!(h.store.with(|state| state.stored.is_empty()));
}

#[tokio::test]
async fn m365_mail_add_mail_attachment_posts_small_inline_files_whole() {
    let sim = Sim::start(attachment_graph).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let input = parse(json!({
        "message_id": "d1", "filename": "note.txt",
        "content_base64": STANDARD.encode(b"hello"), "content_type": "text/plain"
    }));
    let reply = add_mail_attachment(&graph, &h.files, as_caller(&ctx, &exec), input)
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(
        (call.method.as_str(), call.path.as_str()),
        ("POST", "/v1.0/me/messages/d1/attachments")
    );
    assert_eq!(
        call.json(),
        json!({
            "@odata.type": "#microsoft.graph.fileAttachment",
            "name": "note.txt",
            "contentType": "text/plain",
            "contentBytes": STANDARD.encode(b"hello")
        })
    );
    assert_eq!(serde_json::to_value(&reply).unwrap()["data"]["id"], "att1");
}

#[tokio::test]
async fn m365_mail_add_mail_attachment_streams_a_large_document_through_an_upload_session() {
    let sim = Sim::start(attachment_graph).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let h = harness();
    let size = UPLOAD_CHUNK_BYTES + 100;
    let bytes: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
    let (record, document) = document_of(ALICE, "plans.zip", "application/zip", size as u64);
    h.store.put(record, &bytes);
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let input = parse(json!({
        "message_id": "d1", "filename": "plans.zip", "file": {"document_id": document.to_string()}
    }));
    let reply = add_mail_attachment(&graph, &h.files, as_caller(&ctx, &exec), input)
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls.len(), 3, "{calls:#?}");
    assert_eq!(
        calls[0].path,
        "/v1.0/me/messages/d1/attachments/createUploadSession"
    );
    assert_eq!(
        calls[0].json(),
        json!({"AttachmentItem": {"attachmentType": "file", "name": "plans.zip", "size": size, "contentType": "application/zip"}})
    );
    for (put, range, len) in [
        (
            &calls[1],
            format!("bytes 0-{}/{size}", UPLOAD_CHUNK_BYTES - 1),
            UPLOAD_CHUNK_BYTES,
        ),
        (
            &calls[2],
            format!("bytes {UPLOAD_CHUNK_BYTES}-{}/{size}", size - 1),
            100,
        ),
    ] {
        assert_eq!(
            (put.method.as_str(), put.path.as_str()),
            ("PUT", "/upload/s1")
        );
        assert_eq!(put.header("content-range"), Some(range.as_str()));
        assert_eq!(
            put.header("authorization"),
            None,
            "the session URL never gets the bearer"
        );
        assert_eq!(put.body.len(), len);
    }
    assert_eq!(calls[2].body, bytes[UPLOAD_CHUNK_BYTES..]);
    let data = serde_json::to_value(&reply).unwrap()["data"].clone();
    assert_eq!(data["upload_session"], true);
    assert_eq!(data["attachment"], json!({"id": "att-big"}));
}

#[tokio::test]
async fn m365_mail_add_mail_attachment_refuses_bad_sources_before_graph() {
    let sim = Sim::start(attachment_graph).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let h = harness();
    let (record, bobs) = document_of(BOB, "salaries.xlsx", "application/pdf", 4);
    h.store.put(record, b"%PDF");
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let big_inline = STANDARD.encode(vec![7u8; 70 * 1024]);
    for bad in [
        json!({"message_id": "d1", "filename": "x"}),
        json!({"message_id": "d1", "filename": "x", "content_base64": "aGk=", "file": {"document_id": bobs.to_string()}}),
        json!({"message_id": "d1", "filename": "x", "file": {"document_id": bobs.to_string()}}),
        json!({"message_id": "d1", "filename": "x", "content_base64": big_inline}),
        json!({"message_id": "d/../1", "filename": "x", "content_base64": "aGk="}),
    ] {
        let input = parse(bad.clone());
        let result = add_mail_attachment(&graph, &h.files, as_caller(&ctx, &exec), input).await;
        assert!(result.is_err(), "{bad}");
    }
    assert!(sim.calls().is_empty());
    assert_eq!(
        h.store.with(|state| state.opened),
        0,
        "Bob's file was never opened"
    );
}

#[tokio::test]
async fn m365_mail_create_mail_attachment_upload_session_names_the_attachment() {
    let sim = Sim::start(attachment_graph).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let reply = create_upload_session(
        &graph,
        &h.files,
        as_caller(&ctx, &exec),
        parse(json!({"message_id": "d1", "filename": "big.mov", "size_bytes": 9_000_000})),
    )
    .await
    .unwrap();
    let error = create_upload_session(
        &graph,
        &h.files,
        as_caller(&ctx, &exec),
        parse(json!({"message_id": "d1", "filename": "x", "size_bytes": 0})),
    )
    .await;

    let calls = sim.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].json(),
        json!({"AttachmentItem": {"attachmentType": "file", "name": "big.mov", "size": 9_000_000, "contentType": "application/octet-stream"}})
    );
    let data = serde_json::to_value(&reply).unwrap()["data"].clone();
    assert!(data["uploadUrl"].as_str().unwrap().contains("/upload/s1"));
    assert!(error.is_err());
    assert_eq!(
        graph.upload_sessions().len(),
        1,
        "only the session Graph opened is kept"
    );
}

#[tokio::test]
async fn m365_mail_upload_attachment_chunk_puts_to_admitted_session_urls_only() {
    let sim = Sim::start(attachment_graph).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let session = create_upload_session(
        &graph,
        &h.files,
        as_caller(&ctx, &exec),
        parse(json!({"message_id": "d1", "filename": "big.bin", "size_bytes": 10})),
    )
    .await
    .unwrap();
    let own = serde_json::to_value(&session).unwrap()["data"]["uploadUrl"]
        .as_str()
        .unwrap()
        .to_owned();
    let input = parse(json!({
        "upload_url": own, "chunk_base64": STANDARD.encode(b"abcd"),
        "byte_range_start": 4, "total_size": 10
    }));
    let reply = upload_attachment_chunk(&graph, &h.files, as_caller(&ctx, &exec), input)
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1].method, "PUT");
    assert_eq!(calls[1].header("content-range"), Some("bytes 4-7/10"));
    assert_eq!(calls[1].header("authorization"), None);
    assert_eq!(calls[1].body, b"abcd");
    assert_eq!(
        serde_json::to_value(&reply).unwrap()["data"],
        json!({"status_code": 200, "response": {"nextExpectedRanges": ["more"]}})
    );

    for (url, start, total) in [
        ("https://evil.example.com/upload?authtoken=sas", 0, 4),
        ("http://outlook.office.com/api/v2.0/sessions/1", 0, 4),
        ("https://outlook.office.com.evil.example/api", 0, 4),
        ("https://user:pw@outlook.office.com/api", 0, 4),
        ("https://evilsharepoint.com/upload", 0, 4),
        (own.as_str(), 8, 10),
        (own.as_str(), -1, 10),
    ] {
        let input = parse(json!({
            "upload_url": url, "chunk_base64": STANDARD.encode(b"abcd"),
            "byte_range_start": start, "total_size": total
        }));
        let result = upload_attachment_chunk(&graph, &h.files, as_caller(&ctx, &exec), input).await;
        assert!(result.is_err(), "{url} {start}/{total}");
    }
    assert_eq!(sim.calls().len(), 2);
}

#[tokio::test]
async fn m365_mail_delete_mail_attachment_deletes_by_message_and_attachment() {
    let sim = Sim::start(attachment_graph).await;
    let graph = client(&sim.graph_url(), FakeTokens::new(&["t1"]), fast_limits());
    let reply = delete_mail_attachment(
        &graph,
        parse(json!({"message_id": "d1", "attachment_id": "att1"})),
    )
    .await
    .unwrap();
    let error = delete_mail_attachment(
        &graph,
        parse(json!({"message_id": "d1", "attachment_id": "a\\b"})),
    )
    .await;

    let calls = sim.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("DELETE", "/v1.0/me/messages/d1/attachments/att1")
    );
    assert_eq!(
        serde_json::to_value(&reply).unwrap()["data"],
        json!({"deleted": true, "message_id": "d1", "attachment_id": "att1"})
    );
    assert!(error.is_err());
}
