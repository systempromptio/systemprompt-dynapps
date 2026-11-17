# dynwork-chats

DynWork cloud chats: a systemprompt extension that stores chat threads, their messages and the live events of a
running reply in the user's own instance, so DynWork Mobile and every DynWork desktop read the same chats whether a
desktop is on or not. The server is the source of truth; clients only cache.

The contract this crate implements is `contract.md` of the cloud-chats plan (the OpenCode wire shapes, the
`/api/dynwork/chats/v1` API, the change feed and the `chats.changed` hint). It is delivered in phases; this README
describes what exists today.

## What exists

The store only. Nothing is mounted on HTTP yet: no routes, no auth, no replies, no SSE. Those come in the next
phases (threads and the changes feed, replies and the `/log` stream, models and the inactivity job, the relay hint).

- `ChatsExtension` (`dynwork-chats`, "DynWork Chats"), depends on `users`; registers the schema and migrations.
- `store::ChatStore`, an async trait with two implementations: `store::postgres::PgChatStore` (production) and
  `store::memory::MemoryChatStore` (the seam later suites use to run the HTTP layer without a database). Both pass
  the same contract suite in `tests/integration/dynwork-chats`.
- `ids`: `ThreadId` (`ses_` and a ULID), `MessageId` (`msg_` and a ULID), and `Cursor`, the opaque cursor of the
  changes feed (base64url of a `change_seq`, never parsed by clients).

## Tables

`schema/01_dynwork_chats.sql`, all prefixed `dynwork_chat_`:

| Table | Holds |
| --- | --- |
| `dynwork_chat_threads` | `id`, `user_id`, `title`, `model`, `revision`, `archived_at`, `deleted_at` (tombstone), `created_at`, `updated_at`, `change_seq`, and the per-thread counters `last_message_seq` and `last_event_seq`. Index `(user_id, change_seq)`. |
| `dynwork_chat_messages` | One OpenCode message per row: `role`, `info` and `parts` as `jsonb` (never interpreted beyond `role`), `seq` per thread, `completed_at`. |
| `dynwork_chat_events` | The live events of a reply, `seq` per thread, so a late reader resumes after a `seq`. `compacted_at` marks the events of a finished reply. |

`change_seq` comes from the sequence `dynwork_chat_change_seq`. It is taken while the writer holds the owner's
`users` row lock, so for one user the sequence order is the commit order and a cursor never skips a change.

This is product data, separate from core's 30-day `ai_requests` audit, which still records every model call.

## Store contract

Every method takes the owner's `UserId`; no method reads across users. Times are arguments, so the injected clock
decides.

- **Threads.** `create_thread` (idempotent on the id: `Exists` for the caller's own, `IdTaken` for another user's),
  `get_thread`, `update_thread` and `delete_thread`. Both take the `If-Match` revision: a mismatch is
  `Conflict(current row)`. `revision` starts at 1 and every edit by a person (rename, archive, model, delete) bumps
  it together with `change_seq`. Activity (a message, the end of a reply) moves `change_seq` and `updated_at` but not
  `revision`, so a rename is never refused because a reply was streaming. An empty patch writes nothing.
- **Tombstones.** Deleting keeps the row with `deleted_at` set and an empty title, and removes the thread's messages
  and events. `get_thread` and `changes_since` still show it; every other method treats it as absent.
- **Changes feed.** `changes_since(cursor, limit)` returns threads changed after the cursor, oldest change first,
  with the new cursor and `has_more`. No cursor means the full list of live threads. Pages are capped at
  `store::MAX_PAGE` (500).
- **Messages.** `append_message` gives a per-thread `seq` from 1; a message id already in the thread returns
  `Duplicate(stored row)` and writes nothing, which makes a prompt retry idempotent. `list_messages` pages newest
  first with `before=<message id>`.
- **Events.** `append_events` gives each event a per-thread `seq` that never repeats, even after pruning, and does
  not change the thread's `change_seq`. `events_after(seq, limit)` reads them oldest first.
- **Ending a reply.** `compact_reply` stores the final info and parts in the assistant message, stamps it completed,
  marks the reply's events compacted and gives the thread its next `change_seq`. `prune_events(before)` removes the
  events of replies compacted before `before`, and events that belong to no reply and are older than it. Events of a
  reply still running are never pruned.

## Queries

`PgChatStore` uses the compile-time checked `sqlx::query!` family, so the 22 queries have entries in the workspace's
`.sqlx/` cache (`query-*.json`). After changing a query, regenerate them against a database that has the schema:
run `cargo check -p dynwork-chats` with `DATABASE_URL` set, `SQLX_OFFLINE=false` and `SQLX_OFFLINE_DIR` pointing at an
empty directory, and copy the resulting `query-*.json` files into `.sqlx/`. The Postgres suite then runs every query
against the real schema.

## Tests

- `tests/unit/dynwork-chats`: ids, the cursor, error keys, the extension's registration, the memory store.
- `tests/integration/dynwork-chats`: the store contract, stamped once onto the memory store and once onto Postgres
  (`db_or_skip!`-style throwaway database; with `CI=1` a missing database is a failure, not a skip).
