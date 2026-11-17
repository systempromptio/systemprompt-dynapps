//! The inactivity sweep, on both stores: which threads it tombstones, which it
//! keeps, and what a client sees afterwards.

use chrono::Duration;
use dynwork_chats::ids::{Cursor, MessageId, ThreadId};
use dynwork_chats::retention::sweep;
use dynwork_chats::store::{
    AppendOutcome, CreateOutcome, NewEvent, NewMessage, NewThread, Role, ThreadEdit, ThreadPatch,
    ThreadRecord, WriteOutcome,
};
use serde_json::json;
use systemprompt::identifiers::UserId;

use crate::contract::suite;
use crate::fixture::{Fixture, at};

const WINDOW_DAYS: i64 = 15;
const DAY: i64 = 86_400;
const BATCH: u32 = 50;
const BATCHES: u32 = 10;

async fn create(f: &Fixture, owner: &UserId, title: &str, secs: i64) -> ThreadRecord {
    let outcome = f
        .store
        .create_thread(
            owner,
            &NewThread {
                id: ThreadId::generate(),
                title: title.to_owned(),
                model: None,
            },
            at(secs),
        )
        .await
        .expect("create a thread");
    match outcome {
        CreateOutcome::Created(record) => record,
        other => panic!("expected a created thread, got {other:?}"),
    }
}

async fn run(f: &Fixture, now: i64) -> dynwork_chats::retention::Sweep {
    sweep(
        f.store.as_ref(),
        Some(Duration::days(WINDOW_DAYS)),
        at(now),
        BATCH,
        BATCHES,
    )
    .await
    .expect("sweep")
}

async fn live_titles(f: &Fixture, owner: &UserId) -> Vec<String> {
    let page = f
        .store
        .changes_since(owner, None, 100)
        .await
        .expect("changes");
    let mut titles: Vec<String> = page.threads.into_iter().map(|t| t.title).collect();
    titles.sort();
    titles
}

async fn say(f: &Fixture, owner: &UserId, thread: &ThreadId, secs: i64) {
    let id = MessageId::generate();
    let message = NewMessage {
        info: json!({ "id": id.as_str(), "role": "user" }),
        parts: json!([{ "type": "text", "text": "hello" }]),
        completed_at: None,
        role: Role::User,
        id,
    };
    let outcome = f
        .store
        .append_message(owner, thread, &message, at(secs))
        .await
        .expect("append");
    assert!(matches!(outcome, AppendOutcome::Appended(_)), "{outcome:?}");
}

async fn event(f: &Fixture, owner: &UserId, thread: &ThreadId, secs: i64) {
    let written = f
        .store
        .append_events(
            owner,
            thread,
            &[NewEvent {
                message: None,
                kind: "session.text.delta".to_owned(),
                data: json!({ "delta": "..." }),
            }],
            at(secs),
        )
        .await
        .expect("append events");
    assert!(written.is_some());
}

async fn without_a_window_nothing_is_deleted(f: &Fixture) {
    let old = create(f, &f.alice, "ancient", 0).await;
    let done = sweep(f.store.as_ref(), None, at(400 * DAY), BATCH, BATCHES)
        .await
        .expect("sweep");
    assert_eq!(done.tombstoned, 0);
    assert!(!done.more);
    let kept = f.store.get_thread(&f.alice, &old.id).await.expect("get");
    assert!(kept.expect("still there").deleted_at.is_none());
}

async fn a_thread_just_past_the_window_is_tombstoned_and_shows_in_changes(f: &Fixture) {
    let cursor_before = f
        .store
        .changes_since(&f.alice, None, 10)
        .await
        .expect("changes")
        .cursor;
    let old = create(f, &f.alice, "old", 0).await;
    let page = f
        .store
        .changes_since(&f.alice, Some(Cursor::ZERO), 10)
        .await
        .expect("changes");
    let seen = page.cursor;
    assert_ne!(seen, cursor_before);

    let done = run(f, WINDOW_DAYS * DAY + 1).await;
    assert_eq!((done.tombstoned, done.skipped, done.more), (1, 0, false));

    let after = f
        .store
        .changes_since(&f.alice, Some(seen), 10)
        .await
        .expect("changes");
    assert_eq!(after.threads.len(), 1);
    let tombstone = &after.threads[0];
    assert_eq!(tombstone.id, old.id);
    assert_eq!(tombstone.deleted_at, Some(at(WINDOW_DAYS * DAY + 1)));
    assert!(tombstone.title.is_empty());
    assert!(tombstone.change_seq > old.change_seq);
    assert_eq!(tombstone.revision, old.revision + 1);
    assert_eq!(live_titles(f, &f.alice).await, Vec::<String>::new());
}

async fn a_thread_just_inside_the_window_is_kept(f: &Fixture) {
    let edge = create(f, &f.alice, "edge", 0).await;
    let done = run(f, WINDOW_DAYS * DAY).await;
    assert_eq!(
        done.tombstoned, 0,
        "idle for exactly the window is not past it"
    );
    let inside = run(f, WINDOW_DAYS * DAY - 1).await;
    assert_eq!(inside.tombstoned, 0);
    let kept = f.store.get_thread(&f.alice, &edge.id).await.expect("get");
    assert!(kept.expect("kept").deleted_at.is_none());
    assert_eq!(live_titles(f, &f.alice).await, vec!["edge".to_owned()]);
}

async fn a_message_restarts_the_window(f: &Fixture) {
    let thread = create(f, &f.alice, "chatty", 0).await;
    say(f, &f.alice, &thread.id, 10 * DAY).await;
    let early = run(f, WINDOW_DAYS * DAY + 5).await;
    assert_eq!(
        early.tombstoned, 0,
        "the last message was 5 days into the window"
    );
    let late = run(f, 10 * DAY + WINDOW_DAYS * DAY + 1).await;
    assert_eq!(late.tombstoned, 1);
}

async fn a_running_reply_counts_as_activity(f: &Fixture) {
    let busy = create(f, &f.alice, "streaming", 0).await;
    let quiet = create(f, &f.alice, "quiet", 0).await;
    let now = 40 * DAY;
    event(f, &f.alice, &busy.id, now - 30).await;
    let done = run(f, now).await;
    assert_eq!(done.tombstoned, 1);
    assert_eq!(live_titles(f, &f.alice).await, vec!["streaming".to_owned()]);
    let gone = f.store.get_thread(&f.alice, &quiet.id).await.expect("get");
    assert!(gone.expect("tombstone").deleted_at.is_some());

    let stale = run(f, now + WINDOW_DAYS * DAY + 1).await;
    assert_eq!(stale.tombstoned, 1, "the reply's last event is old now");
}

async fn old_events_alone_do_not_keep_a_thread(f: &Fixture) {
    let thread = create(f, &f.alice, "finished", 0).await;
    event(f, &f.alice, &thread.id, 1).await;
    let done = run(f, WINDOW_DAYS * DAY + 2).await;
    assert_eq!(done.tombstoned, 1);
}

async fn an_edit_by_a_person_restarts_the_window(f: &Fixture) {
    let thread = create(f, &f.alice, "before", 0).await;
    let renamed = f
        .store
        .update_thread(
            &f.alice,
            &thread.id,
            &ThreadEdit {
                expected_revision: thread.revision,
                patch: ThreadPatch {
                    title: Some("after".to_owned()),
                    ..ThreadPatch::default()
                },
            },
            at(12 * DAY),
        )
        .await
        .expect("rename");
    assert!(matches!(renamed, WriteOutcome::Done(_)));
    let done = run(f, WINDOW_DAYS * DAY + 1).await;
    assert_eq!(done.tombstoned, 0);
    assert_eq!(live_titles(f, &f.alice).await, vec!["after".to_owned()]);
}

async fn every_users_idle_threads_go_and_each_sees_its_own_tombstone(f: &Fixture) {
    let a = create(f, &f.alice, "alice old", 0).await;
    let b = create(f, &f.bob, "bob old", 1).await;
    create(f, &f.bob, "bob fresh", WINDOW_DAYS * DAY).await;
    let done = run(f, WINDOW_DAYS * DAY + 10).await;
    assert_eq!(done.tombstoned, 2);
    let alice = f
        .store
        .changes_since(&f.alice, Some(Cursor::ZERO), 10)
        .await
        .expect("changes");
    assert_eq!(alice.threads.len(), 1);
    assert_eq!(alice.threads[0].id, a.id);
    assert!(alice.threads[0].deleted_at.is_some());
    let bob = f
        .store
        .changes_since(&f.bob, Some(Cursor::ZERO), 10)
        .await
        .expect("changes");
    let deleted: Vec<&ThreadRecord> = bob
        .threads
        .iter()
        .filter(|t| t.deleted_at.is_some())
        .collect();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].id, b.id);
    assert_eq!(live_titles(f, &f.bob).await, vec!["bob fresh".to_owned()]);
}

async fn archived_threads_age_out_too_and_a_second_sweep_finds_nothing(f: &Fixture) {
    let thread = create(f, &f.alice, "shelved", 0).await;
    let archived = f
        .store
        .update_thread(
            &f.alice,
            &thread.id,
            &ThreadEdit {
                expected_revision: thread.revision,
                patch: ThreadPatch {
                    archived: Some(true),
                    ..ThreadPatch::default()
                },
            },
            at(1),
        )
        .await
        .expect("archive");
    assert!(matches!(archived, WriteOutcome::Done(_)));
    let now = WINDOW_DAYS * DAY + 2;
    assert_eq!(run(f, now).await.tombstoned, 1);
    let again = run(f, now + 1).await;
    assert_eq!((again.tombstoned, again.skipped), (0, 0));
}

async fn the_sweep_works_in_batches_and_says_when_more_remain(f: &Fixture) {
    for n in 0..5 {
        create(f, &f.alice, &format!("old {n}"), n).await;
    }
    let now = WINDOW_DAYS * DAY + 100;
    let window = Some(Duration::days(WINDOW_DAYS));
    let first = sweep(f.store.as_ref(), window, at(now), 2, 1)
        .await
        .expect("sweep");
    assert_eq!((first.tombstoned, first.more), (2, true));
    assert_eq!(live_titles(f, &f.alice).await.len(), 3);

    let rest = sweep(f.store.as_ref(), window, at(now + 1), 2, 10)
        .await
        .expect("sweep");
    assert_eq!((rest.tombstoned, rest.more), (3, false));
    assert!(live_titles(f, &f.alice).await.is_empty());
}

async fn a_tombstoned_thread_drops_its_messages(f: &Fixture) {
    let thread = create(f, &f.alice, "with history", 0).await;
    say(f, &f.alice, &thread.id, 1).await;
    run(f, WINDOW_DAYS * DAY + 2).await;
    let history = f
        .store
        .list_messages(&f.alice, &thread.id, None, 10)
        .await
        .expect("history");
    assert!(history.is_none(), "a tombstone has no history");
}

suite!(
    without_a_window_nothing_is_deleted,
    a_thread_just_past_the_window_is_tombstoned_and_shows_in_changes,
    a_thread_just_inside_the_window_is_kept,
    a_message_restarts_the_window,
    a_running_reply_counts_as_activity,
    old_events_alone_do_not_keep_a_thread,
    an_edit_by_a_person_restarts_the_window,
    every_users_idle_threads_go_and_each_sees_its_own_tombstone,
    archived_threads_age_out_too_and_a_second_sweep_finds_nothing,
    the_sweep_works_in_batches_and_says_when_more_remain,
    a_tombstoned_thread_drops_its_messages,
);
