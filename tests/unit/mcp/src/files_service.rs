//! `McpFiles` over an in-memory store: an upload round-trips to the same
//! bytes, a reference resolves only for its owner, expired and unfinished
//! uploads are refused before any byte is read, size limits hold even when
//! the store does not know the size, downloads become the caller's
//! documents, and every operation (refusals included) leaves one audit row.

use chrono::{Duration, Utc};
use futures_util::StreamExt;
use systemprompt::identifiers::{ContentHash, DocumentUploadId, UserId};
use systemprompt_mcp_shared::files::{
    DownloadedFile, FileAction, FileCall, FilePolicy, FileRecord, FileRef, FileState, FilesError,
    StoredRef, UploadRequest,
};

use crate::files_support::{
    ALICE, BOB, UPLOAD_TOOL, anonymous, document_of, exec_id, harness, harness_with, signed_in,
};

const PDF: &[u8] = b"%PDF-1.7 tiny test file";
const SHA: &str = "4f2b4cbf4ad7f0a9a24a2d56c9c3bd0bd5a1c9a0e6b2c1b0d6b0c3a1f2e4d5c6";

fn upload_request(size: u64) -> UploadRequest {
    UploadRequest {
        name: "contract.pdf".to_owned(),
        mime: "application/pdf".to_owned(),
        size,
        sha256: SHA.to_uppercase(),
        upload_id: String::new(),
    }
}

#[tokio::test]
async fn files_upload_then_resolve_round_trips_the_same_bytes() {
    let h = harness();
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_send",
        exec_id: &exec,
        hop: None,
    };
    let ticket = h
        .files
        .request_upload(call, &upload_request(PDF.len() as u64))
        .await
        .expect("ticket");
    assert_eq!(ticket.file_ref, FileRef::upload(&ticket.upload_id));
    assert_eq!(ticket.part_count, 1);
    assert!(ticket.parts_url.ends_with("/parts/{n}"));
    assert!(ticket.expires_at <= Utc::now() + Duration::minutes(15));
    assert!(ticket.instructions.contains(&ticket.token_header));
    assert!(ticket.instructions.contains(UPLOAD_TOOL));
    assert!(!format!("{ticket:?}").contains(&ticket.upload_token));

    h.store
        .complete(&ticket.upload_id, PDF, Duration::minutes(1));
    let file = h
        .files
        .resolve_ref(call, &ticket.file_ref)
        .await
        .expect("resolves");
    assert_eq!(file.name, "contract.pdf");
    assert_eq!(file.mime, "application/pdf");
    assert_eq!(file.read_all().await.expect("bytes"), PDF);

    let actions: Vec<_> = h.audit.events().iter().map(|e| e.action).collect();
    assert_eq!(actions, [FileAction::UploadOpened, FileAction::Read]);
    let events = h.audit.events();
    assert_eq!(events[1].file, format!("upload:{}", ticket.upload_id));
    assert_eq!(events[1].tool, "test_send");
}

#[tokio::test]
async fn files_another_users_document_is_refused_before_any_byte_is_read() {
    let h = harness();
    let (record, id) = document_of(BOB, "salaries.xlsx", "application/vnd.ms-excel", 10);
    h.store.put(record, b"secret row");
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_send",
        exec_id: &exec,
        hop: None,
    };
    let reference = FileRef {
        document_id: id.as_str().to_owned(),
        ..FileRef::default()
    };
    let error = h
        .files
        .resolve_ref(call, &reference)
        .await
        .expect_err("not Alice's");
    assert!(matches!(error, FilesError::NotFound { .. }), "{error}");
    assert_eq!(h.store.with(|state| state.opened), 0);
    let events = h.audit.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].action, FileAction::Refused);
    assert_eq!(events[0].reason.as_deref(), Some("not_found"));
    assert_eq!(events[0].user_id.as_ref().map(UserId::as_str), Some(ALICE));
}

#[tokio::test]
async fn files_renewing_another_users_upload_is_refused() {
    let h = harness();
    let bob = signed_in(BOB);
    let alice = signed_in(ALICE);
    let exec = exec_id();
    let as_bob = FileCall {
        ctx: &bob,
        tool: "test_upload",
        exec_id: &exec,
        hop: None,
    };
    let ticket = h
        .files
        .request_upload(as_bob, &upload_request(20))
        .await
        .expect("Bob's ticket");
    let renew = UploadRequest {
        upload_id: ticket.upload_id.as_str().to_owned(),
        ..UploadRequest::default()
    };
    let as_alice = FileCall {
        ctx: &alice,
        ..as_bob
    };
    let error = h
        .files
        .request_upload(as_alice, &renew)
        .await
        .expect_err("not Alice's upload");
    assert!(matches!(error, FilesError::NotFound { .. }), "{error}");
    let renewed = h
        .files
        .request_upload(as_bob, &renew)
        .await
        .expect("Bob renews his own");
    assert_eq!(renewed.upload_id, ticket.upload_id);
}

#[tokio::test]
async fn files_unfinished_expired_and_stale_uploads_are_refused() {
    let h = harness();
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_send",
        exec_id: &exec,
        hop: None,
    };
    let pending = h
        .files
        .request_upload(call, &upload_request(20))
        .await
        .expect("ticket");
    let error = h
        .files
        .resolve_ref(call, &pending.file_ref)
        .await
        .expect_err("not complete");
    assert!(matches!(error, FilesError::NotReady { .. }), "{error}");

    let lapsed = DocumentUploadId::generate();
    h.store.put(
        FileRecord {
            owner: ctx.user_id().clone(),
            name: "late.pdf".to_owned(),
            mime: "application/pdf".to_owned(),
            size_bytes: Some(20),
            state: FileState::Pending {
                expires_at: Utc::now() - Duration::minutes(1),
            },
            pinned: StoredRef::Upload(lapsed.clone()),
        },
        b"",
    );
    let error = h
        .files
        .resolve_ref(call, &FileRef::upload(&lapsed))
        .await
        .expect_err("session expired");
    assert!(matches!(error, FilesError::Expired { .. }), "{error}");
    assert!(error.to_string().contains(UPLOAD_TOOL), "{error}");

    let stale = h
        .files
        .request_upload(call, &upload_request(PDF.len() as u64))
        .await
        .expect("ticket");
    h.store.complete(&stale.upload_id, PDF, Duration::hours(25));
    let error = h
        .files
        .resolve_ref(call, &stale.file_ref)
        .await
        .expect_err("past the ref TTL");
    assert!(matches!(error, FilesError::Expired { .. }), "{error}");
    assert_eq!(h.store.with(|state| state.opened), 0);
}

#[tokio::test]
async fn files_size_limits_hold_on_requests_and_while_reading() {
    let mut policy = FilePolicy::new(UPLOAD_TOOL);
    policy.max_file_bytes = 16;
    let h = harness_with(policy);
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_send",
        exec_id: &exec,
        hop: None,
    };
    let error = h
        .files
        .request_upload(call, &upload_request(17))
        .await
        .expect_err("declared over the limit");
    assert_eq!(
        error,
        FilesError::TooLarge {
            size: 17,
            limit: 16
        }
    );
    assert!(h.store.with(|state| state.uploads.is_empty()));

    let blob = ContentHash::try_new("ab".repeat(32)).expect("hash");
    h.store.put(
        FileRecord {
            owner: ctx.user_id().clone(),
            name: String::new(),
            mime: "application/octet-stream".to_owned(),
            size_bytes: None,
            state: FileState::Ready { since: None },
            pinned: StoredRef::Blob(blob.clone()),
        },
        &[0u8; 40],
    );
    let reference = FileRef {
        blob_ref: blob.as_str().to_owned(),
        ..FileRef::default()
    };
    let error = h
        .files
        .resolve_ref(call, &reference)
        .await
        .expect_err("size unknown up front, over the limit once read");
    assert!(
        matches!(error, FilesError::TooLarge { limit: 16, .. }),
        "{error}"
    );
    let refused = h.audit.events().pop().expect("audited");
    assert_eq!(refused.action, FileAction::Refused);
    assert_eq!(refused.reason.as_deref(), Some("too_large"));

    let (understated, id) = document_of(ALICE, "big.bin", "application/octet-stream", 10);
    h.store.put(understated, &[0u8; 40]);
    let reference = FileRef {
        document_id: id.as_str().to_owned(),
        ..FileRef::default()
    };
    let file = h
        .files
        .resolve_ref(call, &reference)
        .await
        .expect("the store reported 10 bytes");
    let error = file.read_all().await.expect_err("capped while reading");
    assert!(
        matches!(error, FilesError::TooLarge { limit: 16, .. }),
        "{error}"
    );
    let file = h.files.resolve_ref(call, &reference).await.expect("again");
    let mut stream = file.into_stream();
    let first = stream.next().await.expect("one chunk");
    assert!(first.is_err(), "the stream stops at the limit too");
}

#[tokio::test]
async fn files_a_stored_type_outside_the_allowlist_cannot_be_relabelled() {
    let mut policy = FilePolicy::new(UPLOAD_TOOL);
    policy.allowed_mime = vec!["application/pdf".to_owned()];
    let h = harness_with(policy);
    let (record, id) = document_of(ALICE, "setup.exe", "application/x-msdownload", 4);
    h.store.put(record, b"MZ\x90\x00");
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_send",
        exec_id: &exec,
        hop: None,
    };
    let relabelled = FileRef {
        document_id: id.as_str().to_owned(),
        mime: "application/pdf".to_owned(),
        ..FileRef::default()
    };
    let error = h
        .files
        .resolve_ref(call, &relabelled)
        .await
        .expect_err("stored type not admitted");
    assert!(
        matches!(error, FilesError::MimeNotAllowed { .. }),
        "{error}"
    );
    assert_eq!(h.store.with(|state| state.opened), 0);
}

#[tokio::test]
async fn files_requests_with_bad_names_types_or_checksums_never_reach_the_store() {
    let h = harness();
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_upload",
        exec_id: &exec,
        hop: None,
    };
    let cases = [
        (
            UploadRequest {
                name: "../../etc/passwd".to_owned(),
                ..upload_request(10)
            },
            "invalid_name",
        ),
        (
            UploadRequest {
                mime: "pdf".to_owned(),
                ..upload_request(10)
            },
            "invalid_mime",
        ),
        (
            UploadRequest {
                sha256: "abc".to_owned(),
                ..upload_request(10)
            },
            "invalid_checksum",
        ),
        (upload_request(0), "empty"),
    ];
    for (request, code) in cases {
        let error = h
            .files
            .request_upload(call, &request)
            .await
            .expect_err(code);
        assert_eq!(error.code(), code);
    }
    assert!(h.store.with(|state| state.uploads.is_empty()));
    assert_eq!(h.audit.events().len(), 4);
    assert!(
        h.audit
            .events()
            .iter()
            .all(|e| e.action == FileAction::Refused && e.file == "upload")
    );
}

#[tokio::test]
async fn files_inline_content_resolves_and_anonymous_calls_are_refused() {
    let h = harness();
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_send",
        exec_id: &exec,
        hop: None,
    };
    let inline = FileRef {
        content_base64: "aGVsbG8gd29ybGQ=".to_owned(),
        name: "hello.txt".to_owned(),
        mime: "text/plain".to_owned(),
        ..FileRef::default()
    };
    let file = h.files.resolve_ref(call, &inline).await.expect("inline");
    assert!(file.file_ref.content_base64.is_empty());
    assert_eq!(file.read_all().await.expect("bytes"), b"hello world");

    let nobody = anonymous();
    let as_nobody = FileCall {
        ctx: &nobody,
        ..call
    };
    let error = h
        .files
        .resolve_ref(as_nobody, &inline)
        .await
        .expect_err("anonymous");
    assert_eq!(error, FilesError::Anonymous);
    let events = h.audit.events();
    assert_eq!(events[0].file, "inline");
    assert_eq!(events[1].reason.as_deref(), Some("anonymous"));
    assert_eq!(
        events[1].user_id, None,
        "an anonymous refusal is never pinned to the context's actor id"
    );
}

#[tokio::test]
async fn files_downloads_become_the_callers_documents_with_a_link() {
    let h = harness();
    h.store
        .with(|state| state.taken_names.insert("invoice.pdf".to_owned()));
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_get_attachment",
        exec_id: &exec,
        hop: None,
    };
    let out = h
        .files
        .store_download(
            call,
            DownloadedFile {
                name: "attachments/invoice.pdf".to_owned(),
                mime: "Application/PDF".to_owned(),
                bytes: PDF.to_vec(),
            },
        )
        .await
        .expect("stored");
    assert_eq!(out.name, "invoice (2).pdf");
    assert_eq!(out.mime, "application/pdf");
    assert_eq!(out.size_bytes, PDF.len() as u64);
    assert!(
        out.download_url
            .as_deref()
            .is_some_and(|url| url.contains(&out.file_ref.document_id))
    );
    assert!(!out.file_ref.revision_id.is_empty());

    let back = h
        .files
        .resolve_ref(call, &out.file_ref)
        .await
        .expect("the stored document is Alice's");
    assert_eq!(back.read_all().await.expect("bytes"), PDF);
    let events = h.audit.events();
    assert_eq!(events[0].action, FileAction::Stored);
    assert_eq!(
        events[0].file,
        format!("document:{}", out.file_ref.document_id)
    );
    assert!(!events[0].description().contains("invoice"));
}

#[tokio::test]
async fn files_a_download_whose_numbered_names_are_all_taken_gets_a_unique_tag() {
    let h = harness();
    h.store.with(|state| {
        for taken in [
            "notes.txt",
            "notes (2).txt",
            "notes (3).txt",
            "notes (4).txt",
        ] {
            state.taken_names.insert(taken.to_owned());
        }
    });
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_get_attachment",
        exec_id: &exec,
        hop: None,
    };
    let out = h
        .files
        .store_download(
            call,
            DownloadedFile {
                name: "notes.txt".to_owned(),
                mime: "text/plain".to_owned(),
                bytes: b"note".to_vec(),
            },
        )
        .await
        .expect("stored under a unique name");
    let tag = out
        .name
        .strip_prefix("notes (")
        .and_then(|rest| rest.strip_suffix(").txt"))
        .expect("tagged name");
    assert_eq!(tag.len(), 8, "{}", out.name);
    assert!(tag.chars().all(|c| c.is_ascii_hexdigit()), "{}", out.name);
    assert_eq!(h.store.with(|state| state.stored.len()), 1);
}

#[tokio::test]
async fn files_a_download_is_kept_even_when_no_link_can_be_signed() {
    let h = harness();
    h.store.with(|state| state.links_down = true);
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_get_attachment",
        exec_id: &exec,
        hop: None,
    };
    let out = h
        .files
        .store_download(
            call,
            DownloadedFile {
                name: String::new(),
                mime: String::new(),
                bytes: b"raw".to_vec(),
            },
        )
        .await
        .expect("stored without a link");
    assert_eq!(out.name, "download");
    assert_eq!(out.mime, "application/octet-stream");
    assert!(out.download_url.is_none() && out.expires_at.is_none());

    let error = h
        .files
        .request_upload(call, &upload_request(3))
        .await
        .expect_err("an upload is useless without its links");
    assert_eq!(error, FilesError::LinkUnavailable);
}
