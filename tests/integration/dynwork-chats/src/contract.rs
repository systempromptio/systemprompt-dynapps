//! The store contract. Each scenario takes a [`Fixture`] and runs unchanged
//! on the memory store and on Postgres: `suite!` stamps one test per scenario
//! into each of the two modules below.

use dynwork_chats::ids::{Cursor, MessageId, ThreadId};
use dynwork_chats::store::{
    AppendOutcome, CompletedReply, CreateOutcome, EventRecord, MessageRecord, ModelRef, NewEvent,
    NewMessage, NewThread, Role, ThreadEdit, ThreadPatch, ThreadRecord, WriteOutcome,
};
use serde_json::json;
use systemprompt::identifiers::UserId;

use crate::fixture::{Fixture, at, user};

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

fn message(role: Role, text: &str) -> NewMessage {
    let id = MessageId::generate();
    NewMessage {
        info: json!({ "id": id.as_str(), "role": role.as_str() }),
        parts: json!([{ "type": "text", "text": text }]),
        completed_at: None,
        role,
        id,
    }
}

async fn append(
    f: &Fixture,
    owner: &UserId,
    thread: &ThreadId,
    new: &NewMessage,
    secs: i64,
) -> MessageRecord {
    match f
        .store
        .append_message(owner, thread, new, at(secs))
        .await
        .expect("append a message")
    {
        AppendOutcome::Appended(record) => record,
        other => panic!("expected an appended message, got {other:?}"),
    }
}

fn edit(expected_revision: i64, patch: ThreadPatch) -> ThreadEdit {
    ThreadEdit {
        expected_revision,
        patch,
    }
}

fn rename(expected_revision: i64, title: &str) -> ThreadEdit {
    edit(
        expected_revision,
        ThreadPatch {
            title: Some(title.to_owned()),
            ..ThreadPatch::default()
        },
    )
}

fn event(message: Option<&MessageId>, kind: &str, text: &str) -> NewEvent {
    NewEvent {
        message: message.cloned(),
        kind: kind.to_owned(),
        data: json!({ "text": text }),
    }
}

async fn feed(f: &Fixture, owner: &UserId, since: Option<Cursor>) -> (Vec<ThreadRecord>, Cursor) {
    let page = f
        .store
        .changes_since(owner, since, 100)
        .await
        .expect("changes");
    (page.threads, page.cursor)
}

pub(crate) async fn a_thread_is_created_read_and_renamed_with_a_revision_bump(f: &Fixture) {
    let model = ModelRef {
        provider_id: "anthropic".to_owned(),
        model_id: "claude-sonnet-5-5".to_owned(),
    };
    let created = f
        .store
        .create_thread(
            &f.alice,
            &NewThread {
                id: ThreadId::generate(),
                title: "First".to_owned(),
                model: Some(model.clone()),
            },
            at(0),
        )
        .await
        .expect("create");
    let CreateOutcome::Created(created) = created else {
        panic!("expected created");
    };
    assert_eq!(created.revision, 1);
    assert_eq!(created.title, "First");
    assert_eq!(created.model, Some(model));
    assert_eq!(created.user, f.alice);
    assert_eq!((created.created_at, created.updated_at), (at(0), at(0)));
    assert_eq!((created.archived_at, created.deleted_at), (None, None));
    assert_eq!(created.last_event_seq, 0);

    let read = f
        .store
        .get_thread(&f.alice, &created.id)
        .await
        .expect("get");
    assert_eq!(read.as_ref(), Some(&created));

    let renamed = f
        .store
        .update_thread(&f.alice, &created.id, &rename(1, "Renamed"), at(10))
        .await
        .expect("update");
    let WriteOutcome::Done(renamed) = renamed else {
        panic!("expected done");
    };
    assert_eq!(renamed.revision, 2);
    assert_eq!(renamed.title, "Renamed");
    assert_eq!(renamed.updated_at, at(10));
    assert_eq!(renamed.created_at, at(0));
    assert!(renamed.change_seq > created.change_seq);
    assert_eq!(
        renamed.model, created.model,
        "a field the patch omits is kept"
    );
}

pub(crate) async fn a_stale_revision_is_a_conflict_that_carries_the_current_row(f: &Fixture) {
    let thread = create(f, &f.alice, "Original", 0).await;
    let WriteOutcome::Done(current) = f
        .store
        .update_thread(&f.alice, &thread.id, &rename(1, "Winner"), at(1))
        .await
        .expect("update")
    else {
        panic!("expected done");
    };

    let stale = f
        .store
        .update_thread(&f.alice, &thread.id, &rename(1, "Loser"), at(2))
        .await
        .expect("update");
    assert_eq!(stale, WriteOutcome::Conflict(current.clone()));
    let deleting = f
        .store
        .delete_thread(&f.alice, &thread.id, 1, at(3))
        .await
        .expect("delete");
    assert_eq!(deleting, WriteOutcome::Conflict(current.clone()));
    let after = f.store.get_thread(&f.alice, &thread.id).await.expect("get");
    assert_eq!(after, Some(current), "a refused write changes nothing");
}

pub(crate) async fn archiving_and_a_model_change_are_edits_and_an_empty_patch_is_not(f: &Fixture) {
    let thread = create(f, &f.alice, "T", 0).await;
    let WriteOutcome::Done(archived) = f
        .store
        .update_thread(
            &f.alice,
            &thread.id,
            &edit(
                1,
                ThreadPatch {
                    archived: Some(true),
                    model: Some(ModelRef {
                        provider_id: "openai".to_owned(),
                        model_id: "gpt-x".to_owned(),
                    }),
                    ..ThreadPatch::default()
                },
            ),
            at(5),
        )
        .await
        .expect("update")
    else {
        panic!("expected done");
    };
    assert_eq!(archived.archived_at, Some(at(5)));
    assert_eq!(
        archived.model.as_ref().map(|m| m.model_id.as_str()),
        Some("gpt-x")
    );
    assert_eq!(archived.revision, 2);

    let WriteOutcome::Done(unchanged) = f
        .store
        .update_thread(
            &f.alice,
            &thread.id,
            &edit(2, ThreadPatch::default()),
            at(6),
        )
        .await
        .expect("update")
    else {
        panic!("expected done");
    };
    assert_eq!(unchanged, archived, "an empty patch writes nothing");

    let WriteOutcome::Done(unarchived) = f
        .store
        .update_thread(
            &f.alice,
            &thread.id,
            &edit(
                2,
                ThreadPatch {
                    archived: Some(false),
                    ..ThreadPatch::default()
                },
            ),
            at(7),
        )
        .await
        .expect("update")
    else {
        panic!("expected done");
    };
    assert_eq!(unarchived.archived_at, None);
    assert_eq!(unarchived.revision, 3);
}

pub(crate) async fn creating_an_existing_id_returns_it_or_refuses_another_users(f: &Fixture) {
    let thread = create(f, &f.alice, "Mine", 0).await;
    let again = f
        .store
        .create_thread(
            &f.alice,
            &NewThread {
                id: thread.id.clone(),
                title: "Retry".to_owned(),
                model: None,
            },
            at(1),
        )
        .await
        .expect("create");
    assert_eq!(again, CreateOutcome::Exists(thread.clone()));

    let taken = f
        .store
        .create_thread(
            &f.bob,
            &NewThread {
                id: thread.id.clone(),
                title: "Squatter".to_owned(),
                model: None,
            },
            at(2),
        )
        .await
        .expect("create");
    assert_eq!(taken, CreateOutcome::IdTaken);
    assert_eq!(
        f.store.get_thread(&f.alice, &thread.id).await.expect("get"),
        Some(thread),
        "the owner's row is untouched"
    );
}

pub(crate) async fn deleting_writes_a_tombstone_and_drops_the_content(f: &Fixture) {
    let thread = create(f, &f.alice, "Secret plans", 0).await;
    let user_message = message(Role::User, "hello");
    append(f, &f.alice, &thread.id, &user_message, 1).await;
    f.store
        .append_events(
            &f.alice,
            &thread.id,
            &[event(None, "session.created", "")],
            at(2),
        )
        .await
        .expect("events");
    let current = f
        .store
        .get_thread(&f.alice, &thread.id)
        .await
        .expect("get")
        .expect("exists");

    let WriteOutcome::Done(tombstone) = f
        .store
        .delete_thread(&f.alice, &thread.id, current.revision, at(10))
        .await
        .expect("delete")
    else {
        panic!("expected done");
    };
    assert_eq!(tombstone.deleted_at, Some(at(10)));
    assert_eq!(tombstone.title, "", "a tombstone keeps no content");
    assert_eq!(tombstone.revision, current.revision + 1);
    assert!(tombstone.change_seq > current.change_seq);
    assert_eq!(
        f.store.get_thread(&f.alice, &thread.id).await.expect("get"),
        Some(tombstone.clone()),
        "the tombstone is still readable"
    );

    assert!(
        f.store
            .list_messages(&f.alice, &thread.id, None, 10)
            .await
            .expect("list")
            .is_none()
    );
    assert!(
        f.store
            .events_after(&f.alice, &thread.id, 0, 10)
            .await
            .expect("events")
            .is_none()
    );
    let again = f
        .store
        .delete_thread(&f.alice, &thread.id, tombstone.revision, at(11))
        .await
        .expect("delete");
    assert_eq!(again, WriteOutcome::NotFound);
    let rename = f
        .store
        .update_thread(
            &f.alice,
            &thread.id,
            &rename(tombstone.revision, "Zombie"),
            at(12),
        )
        .await
        .expect("update");
    assert_eq!(rename, WriteOutcome::NotFound);
    assert_eq!(
        f.store
            .append_message(&f.alice, &thread.id, &message(Role::User, "late"), at(13))
            .await
            .expect("append"),
        AppendOutcome::NoThread
    );
}

pub(crate) async fn the_feed_lists_changes_in_order_with_tombstones_and_a_cursor(f: &Fixture) {
    let first = create(f, &f.alice, "one", 0).await;
    let second = create(f, &f.alice, "two", 1).await;
    let third = create(f, &f.alice, "three", 2).await;

    let (full, cursor) = feed(f, &f.alice, None).await;
    let ids: Vec<_> = full.iter().map(|t| t.id.clone()).collect();
    assert_eq!(ids, [first.id.clone(), second.id.clone(), third.id.clone()]);
    assert!(
        full.windows(2)
            .all(|pair| pair[0].change_seq < pair[1].change_seq)
    );

    let (quiet, same) = feed(f, &f.alice, Some(cursor)).await;
    assert!(quiet.is_empty());
    assert_eq!(same, cursor);

    f.store
        .update_thread(&f.alice, &first.id, &rename(1, "one, renamed"), at(10))
        .await
        .expect("update");
    f.store
        .delete_thread(&f.alice, &second.id, 1, at(11))
        .await
        .expect("delete");

    let (changed, next) = feed(f, &f.alice, Some(cursor)).await;
    assert_eq!(changed.len(), 2);
    assert_eq!(changed[0].id, first.id);
    assert_eq!(changed[0].title, "one, renamed");
    assert_eq!(changed[1].id, second.id);
    assert!(
        changed[1].deleted_at.is_some(),
        "the feed carries the tombstone"
    );
    assert!(next > cursor);

    let (full_again, _) = feed(f, &f.alice, None).await;
    let live: Vec<_> = full_again.iter().map(|t| t.id.clone()).collect();
    assert_eq!(
        live,
        [third.id, first.id],
        "a full list has no tombstones, in change order"
    );
}

pub(crate) async fn the_feed_pages_and_says_when_more_follows(f: &Fixture) {
    for n in 0..5 {
        create(f, &f.alice, &format!("t{n}"), n).await;
    }
    let first = f
        .store
        .changes_since(&f.alice, None, 2)
        .await
        .expect("page");
    assert_eq!(first.threads.len(), 2);
    assert!(first.has_more);
    let second = f
        .store
        .changes_since(&f.alice, Some(first.cursor), 2)
        .await
        .expect("page");
    assert_eq!(second.threads.len(), 2);
    assert!(second.has_more);
    let last = f
        .store
        .changes_since(&f.alice, Some(second.cursor), 2)
        .await
        .expect("page");
    assert_eq!(last.threads.len(), 1);
    assert!(!last.has_more);
    let titles: Vec<_> = first
        .threads
        .iter()
        .chain(&second.threads)
        .chain(&last.threads)
        .map(|t| t.title.clone())
        .collect();
    assert_eq!(titles, ["t0", "t1", "t2", "t3", "t4"]);
}

pub(crate) async fn messages_get_a_per_thread_seq_and_a_retry_is_idempotent(f: &Fixture) {
    let one = create(f, &f.alice, "one", 0).await;
    let two = create(f, &f.alice, "two", 1).await;
    let hello = message(Role::User, "hello");
    let stored = append(f, &f.alice, &one.id, &hello, 2).await;
    assert_eq!(stored.seq, 1);
    assert_eq!(stored.role, Role::User);
    assert_eq!(stored.thread, one.id);
    assert_eq!(stored.created_at, at(2));
    assert_eq!(stored.completed_at, None);
    assert_eq!(stored.parts, hello.parts);
    assert_eq!(stored.info, hello.info);

    let reply = message(Role::Assistant, "");
    assert_eq!(append(f, &f.alice, &one.id, &reply, 3).await.seq, 2);
    assert_eq!(
        append(f, &f.alice, &two.id, &message(Role::User, "other"), 4)
            .await
            .seq,
        1
    );

    let retry = f
        .store
        .append_message(&f.alice, &one.id, &hello, at(9))
        .await
        .expect("append");
    assert_eq!(
        retry,
        AppendOutcome::Duplicate(stored),
        "the retry returns the stored row"
    );
    let history = f
        .store
        .list_messages(&f.alice, &one.id, None, 10)
        .await
        .expect("list")
        .expect("thread");
    assert_eq!(history.len(), 2, "the retry wrote nothing");
    assert_eq!(
        append(f, &f.alice, &one.id, &message(Role::User, "next"), 10)
            .await
            .seq,
        3
    );
}

pub(crate) async fn history_pages_newest_first_before_a_message(f: &Fixture) {
    let thread = create(f, &f.alice, "history", 0).await;
    let mut ids = Vec::new();
    for n in 0..5 {
        let new = message(
            if n % 2 == 0 {
                Role::User
            } else {
                Role::Assistant
            },
            &format!("m{n}"),
        );
        ids.push(new.id.clone());
        append(f, &f.alice, &thread.id, &new, 1 + n).await;
    }
    let page = f
        .store
        .list_messages(&f.alice, &thread.id, None, 2)
        .await
        .expect("list")
        .expect("thread");
    let seqs: Vec<_> = page.iter().map(|m| m.seq).collect();
    assert_eq!(seqs, [5, 4], "newest first");

    let older = f
        .store
        .list_messages(&f.alice, &thread.id, Some(&page[1].id), 10)
        .await
        .expect("list")
        .expect("thread");
    let seqs: Vec<_> = older.iter().map(|m| m.seq).collect();
    assert_eq!(seqs, [3, 2, 1]);

    let none_before_first = f
        .store
        .list_messages(&f.alice, &thread.id, Some(&ids[0]), 10)
        .await
        .expect("list")
        .expect("thread");
    assert!(none_before_first.is_empty());
    let unknown = f
        .store
        .list_messages(&f.alice, &thread.id, Some(&MessageId::generate()), 10)
        .await
        .expect("list")
        .expect("thread");
    assert!(unknown.is_empty(), "an unknown `before` matches nothing");
}

pub(crate) async fn activity_moves_the_feed_but_not_the_revision(f: &Fixture) {
    let thread = create(f, &f.alice, "active", 0).await;
    let (_, cursor) = feed(f, &f.alice, None).await;
    append(f, &f.alice, &thread.id, &message(Role::User, "ping"), 5).await;

    let (changed, _) = feed(f, &f.alice, Some(cursor)).await;
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].id, thread.id);
    assert_eq!(changed[0].updated_at, at(5));
    assert_eq!(
        changed[0].revision, 1,
        "a message is not an edit: an If-Match holder is not refused"
    );

    let (_, cursor) = feed(f, &f.alice, None).await;
    f.store
        .append_events(
            &f.alice,
            &thread.id,
            &[event(None, "session.created", "")],
            at(6),
        )
        .await
        .expect("events");
    let (quiet, _) = feed(f, &f.alice, Some(cursor)).await;
    assert!(quiet.is_empty(), "live events do not touch the feed");
}

pub(crate) async fn events_get_a_per_thread_seq_and_are_read_after_a_seq(f: &Fixture) {
    let one = create(f, &f.alice, "one", 0).await;
    let two = create(f, &f.alice, "two", 1).await;
    let reply = MessageId::generate();
    let batch = [
        event(Some(&reply), "session.execution.started", ""),
        event(Some(&reply), "session.text.started", ""),
        event(Some(&reply), "session.text.delta", "Hel"),
        event(Some(&reply), "session.text.delta", "lo"),
    ];
    let written = f
        .store
        .append_events(&f.alice, &one.id, &batch, at(2))
        .await
        .expect("events")
        .expect("thread");
    let seqs: Vec<_> = written.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, [1, 2, 3, 4]);
    assert_eq!(written[2].kind, "session.text.delta");
    assert_eq!(written[2].data, json!({ "text": "Hel" }));
    assert_eq!(written[2].message.as_ref(), Some(&reply));
    assert_eq!(written[0].created_at, at(2));

    let more = f
        .store
        .append_events(
            &f.alice,
            &one.id,
            &[event(Some(&reply), "session.text.ended", "")],
            at(3),
        )
        .await
        .expect("events")
        .expect("thread");
    assert_eq!(more[0].seq, 5, "the next batch continues the log");
    let other = f
        .store
        .append_events(
            &f.alice,
            &two.id,
            &[event(None, "session.created", "")],
            at(3),
        )
        .await
        .expect("events")
        .expect("thread");
    assert_eq!(other[0].seq, 1, "another thread has its own log");

    let tail: Vec<EventRecord> = f
        .store
        .events_after(&f.alice, &one.id, 3, 100)
        .await
        .expect("events")
        .expect("thread");
    let seqs: Vec<_> = tail.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, [4, 5], "a late reader resumes after its seq");
    let capped = f
        .store
        .events_after(&f.alice, &one.id, 0, 2)
        .await
        .expect("events")
        .expect("thread");
    assert_eq!(capped.len(), 2);
    let head = f
        .store
        .get_thread(&f.alice, &one.id)
        .await
        .expect("get")
        .expect("thread");
    assert_eq!(head.last_event_seq, 5);
}

pub(crate) async fn ending_a_reply_compacts_it_into_its_message_and_pruning_keeps_the_seq(
    f: &Fixture,
) {
    let thread = create(f, &f.alice, "reply", 0).await;
    append(f, &f.alice, &thread.id, &message(Role::User, "hi"), 1).await;
    let assistant = message(Role::Assistant, "");
    append(f, &f.alice, &thread.id, &assistant, 2).await;
    let loose = event(None, "session.created", "");
    f.store
        .append_events(&f.alice, &thread.id, &[loose], at(2))
        .await
        .expect("events");
    f.store
        .append_events(
            &f.alice,
            &thread.id,
            &[
                event(Some(&assistant.id), "session.text.delta", "Hel"),
                event(Some(&assistant.id), "session.text.delta", "lo"),
            ],
            at(3),
        )
        .await
        .expect("events");
    let (_, cursor) = feed(f, &f.alice, None).await;

    let final_parts = json!([{ "type": "text", "text": "Hello" }]);
    let done = f
        .store
        .compact_reply(
            &f.alice,
            &thread.id,
            &CompletedReply {
                message: assistant.id.clone(),
                info: json!({ "id": assistant.id.as_str(), "role": "assistant", "finish": "stop" }),
                parts: final_parts.clone(),
            },
            at(10),
        )
        .await
        .expect("compact")
        .expect("the message is in the thread");
    assert_eq!(done.parts, final_parts);
    assert_eq!(done.completed_at, Some(at(10)));
    assert_eq!(done.seq, 2, "the message keeps its place");
    assert_eq!(done.info["finish"], "stop");
    let history = f
        .store
        .list_messages(&f.alice, &thread.id, None, 10)
        .await
        .expect("list")
        .expect("thread");
    assert_eq!(history[0], done, "history shows the final message");
    let (changed, _) = feed(f, &f.alice, Some(cursor)).await;
    assert_eq!(
        changed.len(),
        1,
        "the end of a reply is a change of the thread"
    );

    let pruned_early = f.store.prune_events(at(10)).await.expect("prune");
    assert_eq!(
        pruned_early, 1,
        "only the reply-less event older than the cutoff goes; the reply is compacted at the cutoff"
    );
    let pruned = f.store.prune_events(at(11)).await.expect("prune");
    assert_eq!(
        pruned, 2,
        "the compacted reply's events go once the cutoff passes it"
    );
    let remaining = f
        .store
        .events_after(&f.alice, &thread.id, 0, 10)
        .await
        .expect("events")
        .expect("thread");
    assert!(remaining.is_empty());

    let next = f
        .store
        .append_events(
            &f.alice,
            &thread.id,
            &[event(None, "session.renamed", "")],
            at(20),
        )
        .await
        .expect("events")
        .expect("thread");
    assert_eq!(next[0].seq, 4, "a seq never repeats after pruning");
}

pub(crate) async fn a_running_reply_keeps_its_events_through_pruning(f: &Fixture) {
    let thread = create(f, &f.alice, "running", 0).await;
    let assistant = message(Role::Assistant, "");
    append(f, &f.alice, &thread.id, &assistant, 1).await;
    f.store
        .append_events(
            &f.alice,
            &thread.id,
            &[event(
                Some(&assistant.id),
                "session.text.delta",
                "still going",
            )],
            at(2),
        )
        .await
        .expect("events");
    assert_eq!(f.store.prune_events(at(100_000)).await.expect("prune"), 0);
    let kept = f
        .store
        .events_after(&f.alice, &thread.id, 0, 10)
        .await
        .expect("events")
        .expect("thread");
    assert_eq!(kept.len(), 1);

    let missing = f
        .store
        .compact_reply(
            &f.alice,
            &thread.id,
            &CompletedReply {
                message: MessageId::generate(),
                info: json!({}),
                parts: json!([]),
            },
            at(3),
        )
        .await
        .expect("compact");
    assert_eq!(missing, None);
}

pub(crate) async fn users_never_reach_each_others_threads(f: &Fixture) {
    let mine = create(f, &f.alice, "alice's", 0).await;
    let hello = message(Role::User, "private");
    append(f, &f.alice, &mine.id, &hello, 1).await;
    f.store
        .append_events(
            &f.alice,
            &mine.id,
            &[event(None, "session.created", "")],
            at(1),
        )
        .await
        .expect("events");
    let theirs = create(f, &f.bob, "bob's", 2).await;

    assert_eq!(
        f.store.get_thread(&f.bob, &mine.id).await.expect("get"),
        None
    );
    assert!(
        f.store
            .list_messages(&f.bob, &mine.id, None, 10)
            .await
            .expect("list")
            .is_none()
    );
    assert!(
        f.store
            .events_after(&f.bob, &mine.id, 0, 10)
            .await
            .expect("events")
            .is_none()
    );
    assert!(
        f.store
            .append_events(
                &f.bob,
                &mine.id,
                &[event(None, "session.created", "")],
                at(3)
            )
            .await
            .expect("events")
            .is_none()
    );
    assert_eq!(
        f.store
            .append_message(&f.bob, &mine.id, &message(Role::User, "intruder"), at(3))
            .await
            .expect("append"),
        AppendOutcome::NoThread
    );
    assert_eq!(
        f.store
            .update_thread(&f.bob, &mine.id, &rename(1, "hijacked"), at(3))
            .await
            .expect("update"),
        WriteOutcome::NotFound
    );
    assert_eq!(
        f.store
            .delete_thread(&f.bob, &mine.id, 1, at(3))
            .await
            .expect("delete"),
        WriteOutcome::NotFound
    );
    assert_eq!(
        f.store
            .compact_reply(
                &f.bob,
                &mine.id,
                &CompletedReply {
                    message: hello.id.clone(),
                    info: json!({}),
                    parts: json!([]),
                },
                at(3),
            )
            .await
            .expect("compact"),
        None
    );

    let (alice_feed, _) = feed(f, &f.alice, None).await;
    let (bob_feed, _) = feed(f, &f.bob, None).await;
    assert_eq!(
        alice_feed.iter().map(|t| &t.id).collect::<Vec<_>>(),
        [&mine.id]
    );
    assert_eq!(
        bob_feed.iter().map(|t| &t.id).collect::<Vec<_>>(),
        [&theirs.id]
    );
    let intact = f
        .store
        .get_thread(&f.alice, &mine.id)
        .await
        .expect("get")
        .expect("thread");
    assert_eq!(intact.title, "alice's");
    assert_eq!(intact.deleted_at, None);

    let stranger = user("nobody");
    assert!(
        f.store
            .get_thread(&stranger, &mine.id)
            .await
            .expect("get")
            .is_none(),
        "a user with no threads sees none"
    );
}

macro_rules! suite {
    ($($name:ident),* $(,)?) => {
        mod memory_store {
            $(
                #[tokio::test]
                async fn $name() {
                    super::$name(&crate::fixture::memory()).await;
                }
            )*
        }

        mod postgres_store {
            $(
                #[tokio::test]
                async fn $name() {
                    let db = dynapps_test_common::db_or_skip!();
                    let fixture = crate::fixture::postgres(db).await;
                    super::$name(&fixture).await;
                    fixture.finish().await;
                }
            )*
        }
    };
}

suite!(
    a_thread_is_created_read_and_renamed_with_a_revision_bump,
    a_stale_revision_is_a_conflict_that_carries_the_current_row,
    archiving_and_a_model_change_are_edits_and_an_empty_patch_is_not,
    creating_an_existing_id_returns_it_or_refuses_another_users,
    deleting_writes_a_tombstone_and_drops_the_content,
    the_feed_lists_changes_in_order_with_tombstones_and_a_cursor,
    the_feed_pages_and_says_when_more_follows,
    messages_get_a_per_thread_seq_and_a_retry_is_idempotent,
    history_pages_newest_first_before_a_message,
    activity_moves_the_feed_but_not_the_revision,
    events_get_a_per_thread_seq_and_are_read_after_a_seq,
    ending_a_reply_compacts_it_into_its_message_and_pruning_keeps_the_seq,
    a_running_reply_keeps_its_events_through_pruning,
    users_never_reach_each_others_threads,
);
