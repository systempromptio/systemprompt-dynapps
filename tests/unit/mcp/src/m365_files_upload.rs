//! `d365_upload_file` through the shared file layer: small inline base64 is
//! one PUT of the exact bytes to the encoded drive path, a document of 4 MiB
//! or more streams through an upload session without the user's bearer,
//! and a bad source, another user's file or a name that is a path never
//! reaches Graph or opens the file.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::json;
use systemprompt_mcp_microsoft_365::graph::upload::UPLOAD_CHUNK_BYTES;
use systemprompt_mcp_microsoft_365::tools::drive_content::{SIMPLE_UPLOAD_MAX_BYTES, upload_file};

use crate::files_support::{ALICE, BOB, document_of, exec_id, harness, signed_in};
use crate::m365_files_content::as_caller;
use crate::m365_pim_support::{args, graph};
use crate::m365_support::{Reply, Seen, Sim};

// Graph for drive uploads: a session whose uploadUrl is on the simulated
// Graph's own origin; a chunk PUT answers 202 until the range reaches the
// total, then 201 with the item; a simple PUT answers the item.
fn upload_graph(seen: &Seen, _: usize) -> Reply {
    let host = seen.header("host").unwrap_or_default().to_owned();
    let range = seen.header("content-range").unwrap_or_default().to_owned();
    match (seen.method.as_str(), seen.path.as_str()) {
        ("POST", path) if path.ends_with(":/createUploadSession") => Reply::json(
            200,
            &json!({"uploadUrl": format!("http://{host}/upload/drive-s1?tempauth=x")}),
        ),
        ("PUT", "/upload/drive-s1") if is_last(&range) => {
            Reply::json(201, &json!({"id": "big-item"}))
        },
        ("PUT", "/upload/drive-s1") => Reply::json(202, &json!({"nextExpectedRanges": ["x-"]})),
        _ => Reply::json(201, &json!({"id": "small-item", "name": "notes.txt"})),
    }
}

fn is_last(range: &str) -> bool {
    let (span, total) = range.trim_start_matches("bytes ").split_once('/').unwrap();
    let end: u64 = span.split_once('-').unwrap().1.parse().unwrap();
    end + 1 == total.parse::<u64>().unwrap()
}

#[tokio::test]
async fn m365_files_upload_file_puts_small_inline_bytes_to_the_encoded_path() {
    let sim = Sim::start(upload_graph).await;
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let input = args(json!({
        "folder_path": "/Reports/Q1 #2/", "filename": "notes 1.txt",
        "content_base64": STANDARD.encode(b"hello drive")
    }));
    let reply = upload_file(&graph(&sim), &h.files, as_caller(&ctx, &exec), input)
        .await
        .unwrap();
    let root = args(
        json!({"folder_path": "/", "filename": "a.txt", "content_base64": "aGk=", "drive_id": "b!d"}),
    );
    upload_file(&graph(&sim), &h.files, as_caller(&ctx, &exec), root)
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        (
            "PUT",
            "/v1.0/me/drive/root:/Reports/Q1%20%232/notes%201.txt:/content"
        )
    );
    assert_eq!(calls[0].body, b"hello drive");
    assert_eq!(
        calls[0].header("content-type"),
        Some("application/octet-stream")
    );
    assert_eq!(calls[0].header("authorization"), Some("Bearer graph-token"));
    assert_eq!(calls[1].path, "/v1.0/drives/b!d/root:/a.txt:/content");
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": {"id": "small-item", "name": "notes.txt"}, "message": "success"})
    );
}

#[tokio::test]
async fn m365_files_upload_file_streams_a_large_document_through_an_upload_session() {
    let sim = Sim::start(upload_graph).await;
    let h = harness();
    let size = SIMPLE_UPLOAD_MAX_BYTES + 100;
    let bytes: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
    let (record, document) = document_of(ALICE, "plans.zip", "application/zip", size as u64);
    h.store.put(record, &bytes);
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let input = args(json!({
        "folder_path": "Big", "filename": "plans.zip", "drive_id": "b!d",
        "file": {"document_id": document.to_string()}
    }));
    let reply = upload_file(&graph(&sim), &h.files, as_caller(&ctx, &exec), input)
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls.len(), 3, "{calls:#?}");
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        (
            "POST",
            "/v1.0/drives/b!d/root:/Big/plans.zip:/createUploadSession"
        )
    );
    assert_eq!(
        calls[0].json(),
        json!({"item": {"@microsoft.graph.conflictBehavior": "replace"}})
    );
    let ranges = [
        format!("bytes 0-{}/{size}", UPLOAD_CHUNK_BYTES - 1),
        format!("bytes {UPLOAD_CHUNK_BYTES}-{}/{size}", size - 1),
    ];
    for (put, range) in calls[1..].iter().zip(&ranges) {
        assert_eq!(
            (put.method.as_str(), put.path.as_str()),
            ("PUT", "/upload/drive-s1")
        );
        assert_eq!(put.header("content-range"), Some(range.as_str()));
        assert_eq!(
            put.header("authorization"),
            None,
            "no bearer on the session"
        );
    }
    assert_eq!(calls[2].body, bytes[UPLOAD_CHUNK_BYTES..]);
    assert_eq!(
        serde_json::to_value(&reply).unwrap()["data"],
        json!({"id": "big-item"})
    );
}

#[tokio::test]
async fn m365_files_upload_file_refuses_bad_sources_and_names_before_graph() {
    let sim = Sim::start(upload_graph).await;
    let h = harness();
    let (record, bobs) = document_of(BOB, "salaries.xlsx", "application/pdf", 4);
    h.store.put(record, b"%PDF");
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let big_inline = STANDARD.encode(vec![7_u8; 70 * 1024]);
    let bob = json!({"document_id": bobs.to_string()});
    for bad in [
        json!({"folder_path": "/", "filename": "x"}),
        json!({"folder_path": "/", "filename": "x", "content_base64": "aGk=", "file": bob}),
        json!({"folder_path": "/", "filename": "x", "file": bob}),
        json!({"folder_path": "/", "filename": "x", "content_base64": big_inline}),
        json!({"folder_path": "a/../..", "filename": "x", "content_base64": "aGk="}),
        json!({"folder_path": "/", "filename": "../x", "content_base64": "aGk="}),
        json!({"folder_path": "/", "filename": "a/b.txt", "content_base64": "aGk="}),
        json!({"folder_path": "/", "filename": "x", "content_base64": "aGk=", "drive_id": "b/../c"}),
    ] {
        let result = upload_file(
            &graph(&sim),
            &h.files,
            as_caller(&ctx, &exec),
            args(bad.clone()),
        )
        .await;
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
async fn m365_files_upload_file_fails_when_graph_opens_no_upload_session() {
    let sim = Sim::start(|_, _| Reply::json(200, &json!({"expirationDateTime": "soon"}))).await;
    let h = harness();
    let bytes = vec![5_u8; SIMPLE_UPLOAD_MAX_BYTES];
    let (record, document) = document_of(
        ALICE,
        "big.bin",
        "application/octet-stream",
        bytes.len() as u64,
    );
    h.store.put(record, &bytes);
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let input = args(json!({
        "folder_path": "/", "filename": "big.bin", "file": {"document_id": document.to_string()}
    }));
    let error = upload_file(&graph(&sim), &h.files, as_caller(&ctx, &exec), input)
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 1, "no chunk is sent without an uploadUrl");
    assert!(calls[0].path.ends_with(":/createUploadSession"));
    assert!(error.message.contains("upload session"), "{error:?}");
}
