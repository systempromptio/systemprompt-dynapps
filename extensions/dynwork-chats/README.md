# dynwork-chats

DynWork cloud chats: a systemprompt extension that stores chat threads, their messages and the live events of a
running reply in the user's own instance, so DynWork Mobile and every DynWork desktop read the same chats whether a
desktop is on or not. The server is the source of truth; clients only cache.

The contract this crate implements is `contract.md` of the cloud-chats plan (the OpenCode wire shapes, the
`/api/dynwork/chats/v1` API, the change feed and the `chats.changed` hint). It is delivered in phases; this README
describes what exists today.

## What exists

The store and the first half of the HTTP API: threads, the changes feed and the message history. Replies (`prompt`,
`interrupt`, the `/log` stream), `models` and the relay change hint come in the next phases.

- `ChatsExtension` (`dynwork-chats`, "DynWork Chats"), depends on `users`; registers the schema and migrations and
  mounts `chats_router` at `EXTENSION_BASE_PATH` (`/api/dynwork/chats/v1`) as a public router: it does its own auth.
  The state (`ChatsState`: store, token authority, device check, clock) is built on core's first `router` call and
  kept in the extension value.
- `store::ChatStore`, an async trait with two implementations: `store::postgres::PgChatStore` (production) and
  `store::memory::MemoryChatStore` (the seam the suites use to run the HTTP layer without a database). Both pass
  the same contract suite in `tests/integration/dynwork-chats`.
- `ids`: `ThreadId` (`ses_` and a ULID), `MessageId` (`msg_` and a ULID), and `Cursor`, the opaque cursor of the
  changes feed (base64url of a `change_seq`, never parsed by clients).

## Routes

All under `/api/dynwork/chats/v1`, under `/api/` for the relay's reason: core's session middleware treats every other
path as a tracked site page. The base path is the chats API itself because core nests each extension at its own base
path and the relay already holds `/api/dynwork`.

| Method and path | Headers | Body | Answer |
| --- | --- | --- | --- |
| `GET /changes?since=<cursor>&limit=<n>` | | | `200 {threads: [thread], cursor, more}` |
| `PUT /threads/{id}` | `If-Match: <revision>` (absent = create) | `{title?, model?, archived?}` | `200 thread`, `ETag: "<revision>"` |
| `DELETE /threads/{id}` | `If-Match: <revision>` (required) | | `200 thread` (the tombstone) |
| `GET /threads/{id}/messages?before=<message id>&limit=<n>` | | | `200 {messages: [message], more, before}` |

Every answer carries `Cache-Control: no-store`. Bodies are capped at 64 KiB (`router::MAX_BODY_BYTES`).

- **Thread.** `{id, title, model, revision, time: {created, updated, archived?}, deleted, created_at, updated_at,
  archived_at, deleted_at}`. `time.*` are epoch milliseconds, the subset of OpenCode's session info the phone reducer
  reads; the `*_at` fields are the same instants in RFC 3339, as the routines feed writes them (the desktop's Chats
  view reads those). `model` is OpenCode's `{providerID, id}` or `null`. A tombstone has `deleted: true`, an empty
  title, and `deleted_at`.
- **Feed.** `since` is the `cursor` of the previous answer; without it the answer is the full list of live threads.
  With it, every thread changed after the cursor, oldest change first, tombstones included. `limit` defaults to 200,
  at most 500; `more: true` means another page is ready now. A page with no changes hands back the same cursor.
  Same shape as `dynwork-routines`' feed (`threads` in place of `routines`).
- **Writes.** A `PUT` without `If-Match` creates the thread at revision 1 (a title defaults to empty; a new thread
  cannot be archived). An id that already exists answers `409` with the caller's row in `current`, or `id_taken` with
  no row when it is another user's. With `If-Match` it renames (`title`), archives or unarchives (`archived`) or changes
  the model (`model`); fields left out stay, an empty body changes nothing. Each change adds one to `revision`.
  `If-Match` takes the bare number or an entity tag (`"3"`, `W/"3"`). A stale revision, or any write to a tombstone,
  answers `409` with `current`. `DELETE` without `If-Match` is `428`.
- **Messages.** Newest first. Each message is the stored OpenCode message document (`info`), with its `id` and `type`
  (`user`/`assistant`) taken from the row. `limit` defaults to 50, at most 200; `more: true` means older ones remain
  and `before` is the id to ask with for them (`null` otherwise). A `before` id the thread does not have answers an
  empty page.
- **Isolation.** Every row read or written belongs to the token's user. Another user's thread is `404`, never `403`,
  and its row is never shown.

### Credentials

`Authorization: Bearer <token>`, checked before a path, a query or a body is read. The routes accept two tokens and
nothing else, verified by the relay's own functions against the instance's token authority (the crate depends on
`dynwork-relay` for them and copies nothing):

- the DynWork desktop JWT: `dynwork_relay::auth::verify_desktop` (audience `bridge` or `api`, user type `user` or
  `admin`, no `act`, no `plugin_id`, no `purpose`, not expired);
- the phone's mobile JWT: `dynwork_relay::mobile_auth::jwt::verify` (audience and purpose `dynwork-mobile`, one hour).
  Its device (the session id) must also be a live phone row of the user in `dynwork_relay_devices`
  (`auth::RelayDevices`, through the relay's `PgRelayStore::list_devices`), so revoking a phone closes its chats at once
  rather than when the JWT expires. That costs one indexed query per phone request.

Both map to the token's subject, so a person's desktop and phone see the same threads.

### Errors

JSON `{"error": {"key": <key>, "message": <text>}}`; a server error answers the fixed message `internal error` and is
logged. The status mapping follows the relay's.

| Status | `key` | When |
| --- | --- | --- |
| 400 | `bad_request` | a malformed body (not JSON, wrong types, a title over 512 characters, a model part empty or over 128 bytes, a create with `archived: true`), `If-Match`, `limit` or query string |
| 400 | `invalid_id` | a thread id or a `before` message id that is not `ses_`/`msg_` and 8 to 40 letters or digits |
| 400 | `invalid_cursor` | a `since` this server did not issue |
| 401 | `unauthorized` | a missing, malformed, forged, expired or wrong-audience token; a token that is not a signed-in person's; a revoked or unknown phone; a deleted account |
| 404 | `not_found` | a thread that is not the caller's or does not exist; an unknown route |
| 409 | `revision_conflict` | a stale `If-Match`, a write to a tombstone, a create of an existing id; the body carries `current` |
| 409 | `id_taken` | a create of an id another user holds |
| 413 | `too_large` | a body over 64 KiB |
| 428 | `precondition_required` | a `DELETE` without `If-Match` |
| 500 | `internal` | the store or the device check failed |

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

- `tests/unit/dynwork-chats`: ids, the cursor, error keys and statuses, the extension's registration, the memory
  store, `If-Match` and page sizes, the thread body, the wire shapes.
- `tests/integration/dynwork-chats`: the store contract, and the HTTP suites (auth, threads, the feed, history) driven
  in process through the router as core mounts it, beside a stand-in for the relay's `/api/dynwork` mount, with real
  RS256 desktop and mobile tokens. Each is stamped once onto the memory store and once onto Postgres (`db_or_skip!`
  throwaway database, dropped at the end of each test; with `CI=1` a missing database is a failure, not a skip). On
  Postgres the phone check runs against the relay's real device table.
