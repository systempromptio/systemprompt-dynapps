use chrono::{TimeZone, Utc};
use dynwork_chats::ids::{Cursor, ThreadId};
use dynwork_chats::store::memory::MemoryChatStore;
use dynwork_chats::store::{ChatStore, CreateOutcome, NewThread};
use systemprompt::identifiers::UserId;

async fn seed(store: &MemoryChatStore, user: &UserId, count: usize) {
    let now = Utc.with_ymd_and_hms(2026, 11, 17, 9, 0, 0).unwrap();
    for _ in 0..count {
        let created = store
            .create_thread(
                user,
                &NewThread {
                    id: ThreadId::generate(),
                    title: String::new(),
                    model: None,
                },
                now,
            )
            .await
            .expect("create");
        assert!(matches!(created, CreateOutcome::Created(_)));
    }
}

#[tokio::test]
async fn a_page_limit_of_zero_still_returns_one_row() {
    let store = MemoryChatStore::new();
    let user = UserId::new("alice".to_owned());
    seed(&store, &user, 3).await;
    let page = store.changes_since(&user, None, 0).await.expect("changes");
    assert_eq!(page.threads.len(), 1);
    assert!(page.has_more);
}

#[tokio::test]
async fn an_empty_feed_hands_back_the_cursor_it_was_given() {
    let store = MemoryChatStore::new();
    let user = UserId::new("alice".to_owned());
    seed(&store, &user, 1).await;
    let head = store
        .changes_since(&user, None, 10)
        .await
        .expect("changes")
        .cursor;
    let again = store
        .changes_since(&user, Some(head), 10)
        .await
        .expect("changes");
    assert!(again.threads.is_empty());
    assert!(!again.has_more);
    assert_eq!(again.cursor, head);

    let nothing = store
        .changes_since(&UserId::new("bob".to_owned()), None, 10)
        .await
        .expect("changes");
    assert_eq!(nothing.cursor, Cursor::ZERO);
}
