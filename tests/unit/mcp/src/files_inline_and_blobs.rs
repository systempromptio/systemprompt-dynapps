//! Two edges of the file layer's promises: a `blob_ref` carries no lifetime
//! of its own here (core reports no date for a blob, so it lives as long as
//! core keeps the blob, while a reference the store dates expires after
//! `ref_ttl` like an `upload_id`), and bytes a server hands back inline are
//! audited like a kept download.

use chrono::{Duration, Utc};
use systemprompt::identifiers::{ContentHash, McpServerId, UserId};
use systemprompt_mcp_shared::files::{
    FileAction, FileAuditEvent, FileCall, FileRecord, FileRef, FileState, FilesError, StoredRef,
};

use crate::files_support::{ALICE, anonymous, exec_id, harness, signed_in};

const PDF: &[u8] = b"%PDF-1.7 blob";

fn blob_record(hash: &ContentHash, since: Option<chrono::DateTime<Utc>>) -> FileRecord {
    FileRecord {
        owner: UserId::new(ALICE),
        name: String::new(),
        mime: "application/pdf".to_owned(),
        size_bytes: None,
        state: FileState::Ready { since },
        pinned: StoredRef::Blob(hash.clone()),
    }
}

fn blob_ref(hash: &ContentHash) -> FileRef {
    FileRef {
        blob_ref: hash.as_str().to_owned(),
        ..FileRef::default()
    }
}

#[tokio::test]
async fn files_a_blob_ref_expires_only_when_the_store_dates_it() {
    let h = harness();
    let ctx = signed_in(ALICE);
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_send",
        exec_id: &exec,
        hop: None,
    };
    let undated = ContentHash::try_new("ab".repeat(32)).expect("hash");
    let dated = ContentHash::try_new("cd".repeat(32)).expect("hash");
    h.store.put(blob_record(&undated, None), PDF);
    h.store.put(
        blob_record(&dated, Some(Utc::now() - Duration::hours(25))),
        PDF,
    );

    let kept = h
        .files
        .resolve_ref(call, &blob_ref(&undated))
        .await
        .expect("an undated blob lives as long as the store keeps it");
    assert_eq!(kept.read_all().await.expect("bytes"), PDF);
    let error = h
        .files
        .resolve_ref(call, &blob_ref(&dated))
        .await
        .expect_err("a blob dated past ref_ttl is refused like an upload");
    assert!(matches!(error, FilesError::Expired { .. }), "{error}");
    assert_eq!(
        h.store.with(|state| state.opened),
        1,
        "the expired blob was never opened"
    );
}

#[tokio::test]
async fn files_an_inline_download_is_audited_like_a_stored_one() {
    let h = harness();
    let ctx = signed_in(ALICE);
    let nobody = anonymous();
    let exec = exec_id();
    let call = FileCall {
        ctx: &ctx,
        tool: "test_read_text",
        exec_id: &exec,
        hop: None,
    };
    h.files
        .return_inline(call, 42)
        .await
        .expect("a signed-in caller gets the text");
    let refused = h
        .files
        .return_inline(
            FileCall {
                ctx: &nobody,
                ..call
            },
            7,
        )
        .await
        .expect_err("a call with no user gets nothing");
    assert!(matches!(refused, FilesError::Anonymous), "{refused}");

    let events = h.audit.events();
    assert_eq!(events.len(), 2, "{events:?}");
    assert_eq!(
        events[0],
        FileAuditEvent {
            user_id: Some(UserId::new(ALICE)),
            server: McpServerId::try_new("files-test").expect("server id"),
            tool: "test_read_text".to_owned(),
            action: FileAction::Returned,
            file: "inline".to_owned(),
            size_bytes: Some(42),
            reason: None,
        }
    );
    assert_eq!(events[0].action.activity(), FileAction::Stored.activity());
    assert_eq!(
        events[0].description(),
        "Returned file inline (42 bytes) on files-test for 'test_read_text'"
    );
    assert_eq!(events[1].action, FileAction::Refused);
    assert_eq!(events[1].user_id, None);
    assert_eq!(events[1].reason.as_deref(), Some("anonymous"));
    assert!(
        h.store.with(|state| state.stored.is_empty()),
        "nothing is kept"
    );
}
