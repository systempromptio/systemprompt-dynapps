//! File content through the shared file layer, against a simulated Graph
//! whose `/content` answers 302 to a separate download host (as Graph
//! does): the redirect is followed without the bearer (the Belgian
//! `d365_get_file_content` returned `null` here), small text comes back
//! inline, anything else is kept as the caller's document and returned as a
//! `FileOut` with no base64 and no pre-signed URL.

use serde_json::{Value, json};
use systemprompt::identifiers::McpExecutionId;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_microsoft_365::tools::drive_content::{
    INLINE_TEXT_MAX_BYTES, download_file, get_file_content,
};
use systemprompt_mcp_shared::files::{FileAction, FileCall};

use crate::files_support::{ALICE, Harness, exec_id, harness, signed_in};
use crate::m365_pim_support::{args, graph};
use crate::m365_support::{Reply, Sim};

pub(crate) fn as_caller<'a>(ctx: &'a RequestContext, exec: &'a McpExecutionId) -> FileCall<'a> {
    FileCall {
        ctx,
        tool: "d365_files_test",
        exec_id: exec,
        hop: None,
    }
}

// A drive whose one item has `metadata`; its `/content` redirects to
// `download`, which serves `bytes`.
async fn drive_with(metadata: Value, bytes: Vec<u8>) -> (Sim, Sim) {
    let download = Sim::start(move |_, _| Reply::bytes(200, &bytes)).await;
    let target = format!("{}/dl/item?sig=presigned", download.base);
    let graph_sim = Sim::start(move |seen, _| {
        if seen.path.ends_with("/content") {
            Reply::bytes(302, b"").with_header("location", target.clone())
        } else {
            Reply::json(200, &metadata)
        }
    })
    .await;
    (graph_sim, download)
}

fn stored(h: &Harness) -> Vec<(String, String, Vec<u8>)> {
    h.store.with(|state| {
        state
            .records
            .values()
            .map(|(record, bytes)| (record.owner.to_string(), record.name.clone(), bytes.clone()))
            .collect()
    })
}

fn data_of(reply: &impl serde::Serialize) -> Value {
    serde_json::to_value(reply).unwrap()["data"].clone()
}

#[tokio::test]
async fn m365_files_get_file_content_follows_the_redirect_and_inlines_small_text() {
    let metadata = json!({
        "id": "T1", "name": "notes.txt", "size": 11,
        "file": {"mimeType": "text/plain"}, "webUrl": "https://contoso/notes.txt"
    });
    let (sim, download) = drive_with(metadata, b"hello world".to_vec()).await;
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let reply = get_file_content(
        &graph(&sim),
        &h.files,
        as_caller(&ctx, &exec),
        args(json!({"item_id": "T1"})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/drive/items/T1");
    assert_eq!(calls[1].path, "/v1.0/me/drive/items/T1/content");
    let fetched = download.calls();
    assert_eq!(fetched.len(), 1);
    assert_eq!(
        fetched[0].header("authorization"),
        None,
        "the pre-signed URL never gets the bearer"
    );
    assert_eq!(
        data_of(&reply),
        json!({"content": "hello world", "mimeType": "text/plain"})
    );
    assert!(stored(&h).is_empty(), "inline text is not kept");
    assert!(h.audit.events().is_empty());
}

#[tokio::test]
async fn m365_files_get_file_content_keeps_a_binary_file_as_the_callers_document() {
    let pdf = b"%PDF-1.7 drive".to_vec();
    let metadata = json!({
        "id": "P1", "name": "report.pdf", "size": pdf.len(),
        "file": {"mimeType": "application/pdf"}, "webUrl": "https://contoso/report.pdf",
        "@microsoft.graph.downloadUrl": "https://contoso/presigned"
    });
    let (sim, _download) = drive_with(metadata, pdf.clone()).await;
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let reply = get_file_content(
        &graph(&sim),
        &h.files,
        as_caller(&ctx, &exec),
        args(json!({"item_id": "P1", "drive_id": "b!d"})),
    )
    .await
    .unwrap();

    assert_eq!(sim.calls()[1].path, "/v1.0/drives/b!d/items/P1/content");
    let data = data_of(&reply);
    assert!(data.get("content").is_none(), "{data}");
    assert_eq!(data["name"], "report.pdf");
    assert_eq!(data["mimeType"], "application/pdf");
    assert_eq!(data["webUrl"], "https://contoso/report.pdf");
    assert_eq!(data["file"]["size_bytes"], pdf.len());
    assert!(data["file"]["file_ref"]["document_id"].is_string());
    assert!(!data.to_string().contains("presigned"), "{data}");
    assert_eq!(
        stored(&h),
        vec![(ALICE.to_owned(), "report.pdf".to_owned(), pdf)]
    );
    let events = h.audit.events();
    assert_eq!(events.len(), 1);
    assert!(matches!(events[0].action, FileAction::Stored));
}

#[tokio::test]
async fn m365_files_get_file_content_keeps_text_over_the_inline_limit_as_a_document() {
    let csv = vec![b'a'; INLINE_TEXT_MAX_BYTES + 1];
    let metadata = json!({
        "id": "C1", "name": "big.csv", "size": csv.len(), "file": {"mimeType": "text/csv"}
    });
    let (sim, _download) = drive_with(metadata, csv.clone()).await;
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let reply = get_file_content(
        &graph(&sim),
        &h.files,
        as_caller(&ctx, &exec),
        args(json!({"item_id": "C1"})),
    )
    .await
    .unwrap();

    let data = data_of(&reply);
    assert!(data.get("content").is_none());
    assert_eq!(data["file"]["name"], "big.csv");
    assert_eq!(
        stored(&h),
        vec![(ALICE.to_owned(), "big.csv".to_owned(), csv)]
    );
}

#[tokio::test]
async fn m365_files_get_file_content_refuses_a_folder_before_downloading() {
    let metadata = json!({"id": "F1", "name": "Docs", "folder": {"childCount": 2}});
    let (sim, download) = drive_with(metadata, Vec::new()).await;
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let result = get_file_content(
        &graph(&sim),
        &h.files,
        as_caller(&ctx, &exec),
        args(json!({"item_id": "F1"})),
    )
    .await;

    assert!(result.is_err());
    assert_eq!(sim.calls().len(), 1, "only the metadata was read");
    assert!(download.calls().is_empty());
    assert!(stored(&h).is_empty());
}

#[tokio::test]
async fn m365_files_download_file_returns_a_document_not_the_presigned_url() {
    let zip = vec![9u8; 2048];
    let metadata = json!({
        "id": "Z1", "name": "plans.zip", "size": zip.len(),
        "file": {"mimeType": "application/zip"},
        "@microsoft.graph.downloadUrl": "https://contoso/presigned"
    });
    let (sim, download) = drive_with(metadata, zip.clone()).await;
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let reply = download_file(
        &graph(&sim),
        &h.files,
        as_caller(&ctx, &exec),
        args(json!({"item_id": "Z1"})),
    )
    .await
    .unwrap();

    assert_eq!(download.calls()[0].header("authorization"), None);
    let data = data_of(&reply);
    assert_eq!(data["name"], "plans.zip");
    assert_eq!(data["size"], 2048);
    assert_eq!(data["mimeType"], "application/zip");
    assert!(data.get("downloadUrl").is_none(), "{data}");
    assert!(
        data["file"]["download_url"]
            .as_str()
            .is_some_and(|url| url.starts_with("https://api.test/"))
    );
    assert_eq!(
        stored(&h),
        vec![(ALICE.to_owned(), "plans.zip".to_owned(), zip)]
    );
}

#[tokio::test]
async fn m365_files_get_file_content_keeps_text_that_is_not_utf8_as_a_document() {
    let latin1 = vec![b'c', b'a', b'f', 0xE9, 0xFF];
    let metadata = json!({
        "id": "L1", "name": "legacy.txt", "size": latin1.len(), "file": {"mimeType": "text/plain"}
    });
    let (sim, _download) = drive_with(metadata, latin1.clone()).await;
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let reply = get_file_content(
        &graph(&sim),
        &h.files,
        as_caller(&ctx, &exec),
        args(json!({"item_id": "L1"})),
    )
    .await
    .unwrap();

    let data = data_of(&reply);
    assert!(data.get("content").is_none(), "{data}");
    assert_eq!(data["file"]["mime"], "text/plain");
    assert_eq!(
        stored(&h),
        vec![(ALICE.to_owned(), "legacy.txt".to_owned(), latin1)]
    );
}

#[tokio::test]
async fn m365_files_download_file_without_a_mime_type_keeps_it_as_octet_stream() {
    let bytes = vec![1u8, 2, 3, 4];
    let metadata = json!({"id": "B1", "name": "blob.bin", "size": 4, "file": {}});
    let (sim, _download) = drive_with(metadata, bytes.clone()).await;
    let h = harness();
    let (ctx, exec) = (signed_in(ALICE), exec_id());
    let reply = download_file(
        &graph(&sim),
        &h.files,
        as_caller(&ctx, &exec),
        args(json!({"item_id": "B1"})),
    )
    .await
    .unwrap();

    let data = data_of(&reply);
    assert_eq!(data["file"]["mime"], "application/octet-stream");
    let mimes: Vec<String> = h.store.with(|state| {
        state
            .records
            .values()
            .map(|(record, _)| record.mime.clone())
            .collect()
    });
    assert_eq!(mimes, vec!["application/octet-stream".to_owned()]);
    assert_eq!(
        stored(&h),
        vec![(ALICE.to_owned(), "blob.bin".to_owned(), bytes)]
    );
}
