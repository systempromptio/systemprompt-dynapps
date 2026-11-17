# dynwork-chats

DynWork cloud chats: a systemprompt extension that stores chat threads, their messages and the live events of a
running reply in the user's own instance, so DynWork Mobile and every DynWork desktop read the same chats whether a
desktop is on or not. The server is the source of truth; clients only cache.

The contract this crate implements is `contract.md` of the cloud-chats plan (the OpenCode wire shapes, the
`/api/dynwork/chats/v1` API, the change feed and the `chats.changed` hint). It is delivered in phases; this README
describes what exists today.

## What exists

The store and the HTTP API: threads, the changes feed, the message history, server-side replies (`prompt`,
`interrupt`, the `/log` stream), the models list and the inactivity job. The relay change hint comes in the next
phase.

- `ChatsExtension` (`dynwork-chats`, "DynWork Chats"), depends on `users`; registers the schema and migrations and
  mounts `chats_router` at `EXTENSION_BASE_PATH` (`/api/dynwork/chats/v1`) as a public router: it does its own auth.
  The state (`ChatsState`: store, token authority, device check, clock, model source, the registry of running replies)
  is built on core's first `router` call and kept in the extension value.
- `store::ChatStore`, an async trait with two implementations: `store::postgres::PgChatStore` (production) and
  `store::memory::MemoryChatStore` (the seam the suites use to run the HTTP layer without a database). Both pass
  the same contract suite in `tests/integration/dynwork-chats`.
- `ids`: `ThreadId` (`ses_` and a ULID), `MessageId` (`msg_` and a ULID), and `Cursor`, the opaque cursor of the
  changes feed (base64url of a `change_seq`, never parsed by clients). A reply's `MessageId` is derived from its
  prompt (`MessageId::derived_at`: the prompt's time and a hash of the thread and prompt ids in place of the random
  bits).
- `catalog` and `config`: the models a chat can use, and the extension's settings (see Models and Settings below);
  `retention`: the inactivity job.
- `reply`: the model behind `ReplyModel`, the runner, the event builders, the message documents, the registry of
  running replies and the recovery of a reply an earlier process left unfinished (see Replies below).

## Routes

All under `/api/dynwork/chats/v1`, under `/api/` for the relay's reason: core's session middleware treats every other
path as a tracked site page. The base path is the chats API itself because core nests each extension at its own base
path and the relay already holds `/api/dynwork`.

| Method and path | Headers | Body | Answer |
| --- | --- | --- | --- |
| `GET /models` | | | `200 {models: [model]}` |
| `GET /changes?since=<cursor>&limit=<n>` | | | `200 {threads: [thread], cursor, more}` |
| `PUT /threads/{id}` | `If-Match: <revision>` (absent = create) | `{title?, model?, archived?}` | `200 thread`, `ETag: "<revision>"` |
| `DELETE /threads/{id}` | `If-Match: <revision>` (required) | | `200 thread` (the tombstone) |
| `GET /threads/{id}/messages?before=<message id>&limit=<n>` | | | `200 {messages: [message], more, before}` |
| `POST /threads/{id}/prompt` | | `{messageID, parts: [{type: "text", text}], model?}` | `202 {assistantMessageID}` |
| `POST /threads/{id}/interrupt` | | | `204` |
| `GET /threads/{id}/log?after=<seq>&follow=1` | `Last-Event-ID: <seq>` (optional) | | `200 text/event-stream` |

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

### Models

`GET /models` answers `{models: [{id, name, providerID, provider: {id, name}, reasoning, tools}]}`, sorted by provider
and id. The phone reads `providerID`, the desktop's Chats view reads `provider`; `providerID` and `id` are what a
thread's `model` and a prompt's `model` take. It is `503 ai_unavailable` on an instance with no model service.

The list comes from the instance's own model catalog (the one `AiService::catalog` serves), never from a constant
(decision D3). A model is listed when the catalog advertises it and one of its enabled deployments has a platform
credential that is configured: a secret the instance holds (`SecretStore::key_names`) or a workload identity. The
provider is the first such deployment, which is the one `AiService` picks for `{providerID, id}`.

What core 0.81 offers as "the user's models" is narrower than the wording suggests, and this is the whole of it:
a gateway policy carries quotas, rate windows and safety scanners but no model list (only an API key can carry a
model allowlist), and `/v1/models` is itself not per user. What does vary is the credential. A reply runs on the
in-process `AiService`, which serves platform credentials and leaves member-paid ones (a personal key such as
Dynapps' `opencode-go`, the caller's own login) to the gateway, so those models are not listed: a reply to one would
fail. They join the list when replies go through the gateway (stage 2). Two users of one instance therefore see the
same list today; the code is ready for a per-user filter at the `ModelSource::list` seam when core has one.

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
| 400 | `bad_request` | a malformed body (not JSON, wrong types, a title over 512 characters, a model part empty or over 128 bytes, a create with `archived: true`; a prompt with no text, a part that is not text, or a malformed `messageID`), `If-Match`, `limit`, `after`, `follow`, `Last-Event-ID` or query string; a prompt with no model on the prompt, the thread or the instance |
| 400 | `invalid_id` | a thread id or a `before` message id that is not `ses_`/`msg_` and 8 to 40 letters or digits |
| 400 | `invalid_cursor` | a `since` this server did not issue |
| 401 | `unauthorized` | a missing, malformed, forged, expired or wrong-audience token; a token that is not a signed-in person's; a revoked or unknown phone; a deleted account |
| 404 | `not_found` | a thread that is not the caller's or does not exist; an unknown route |
| 409 | `revision_conflict` | a stale `If-Match`, a write to a tombstone, a create of an existing id; the body carries `current` |
| 409 | `id_taken` | a create of an id another user holds; a prompt whose `messageID` is already a message of the thread that is not a prompt |
| 409 | `busy` | a prompt while another reply of the thread runs |
| 413 | `too_large` | a body over 64 KiB |
| 428 | `precondition_required` | a `DELETE` without `If-Match` |
| 503 | `ai_unavailable` | a prompt on an instance with no model service |
| 500 | `internal` | the store or the device check failed |

## Replies

A prompt starts a reply that runs on the server, in-process, with the `AiService` core attaches to the request
(`Extension<Option<Arc<AiService>>>`), as the user (`Actor::user`), so the instance's quotas, policies and audit
(`ai_requests`) apply. Stage 1 streams through `AiService::generate_stream`, which yields text and usage only: no
reasoning, no tools. The call sits behind `reply::model::ReplyModel` (`GatewayModel`), so stage 2 swaps it for
`generate_events_stream` (decision D1) without touching the runner or the wire.

- **Starting.** The thread must be the caller's and live. The model is the prompt's, else the thread's, else the
  instance's default. The prompt is stored as the user message (`{id, type: "user", text, time, metadata:
  {assistantMessageID}}`), the history (the thread's last 200 messages, text only, oldest first, roles alternating)
  goes to the model, and an empty assistant message is stored before the answer `202 {assistantMessageID}`. The same
  `messageID` again, while its reply runs or after, answers the same reply and starts nothing; another prompt while a
  reply runs is `409 busy`.
- **Running.** One task per reply on the registry's `BackgroundTasks`, holding the thread's claim until it ends.
  Every event is written to `dynwork_chat_events` first and then published to the thread's in-memory broadcast.
  A follower going away never stops it.
- **Events.** OpenCode v2 envelopes: `session.created` (the thread's first event, before its first reply);
  `session.execution.started`, `session.step.started`, `session.text.started`, one `session.text.delta` per chunk the
  model emits, `session.text.ended` with the whole text; then `session.step.ended` (`finish`, `rawFinish`, `tokens`,
  `cost`), `session.usage.updated` (the thread's running token total) and `session.execution.succeeded`; or
  `session.execution.interrupted` (`reason: "user"`); or `session.execution.failed` (`error: {type, message}`).
  Each event carries exactly the fields the desktop fixtures carry for its type: `location` is `{directory: ""}` and
  `subpath` `""` (a cloud thread has no directory); `durable: {aggregateID, seq, version: 1}` on every type except
  `session.text.delta` and `session.usage.updated`, which the desktop sends without it. `id` is derived from the row
  (`evt_`, the time in hex, the `seq`), so a replay is byte for byte the live copy. `cost` is `0`: stage 1 has no
  pricing outside core's audit.
- **Ending.** The closing events are written, the reply is compacted into its assistant message (the whole OpenCode
  v2 document: `content: [{type: "text", text}]`, `finish`, `rawFinish`, `tokens`, `cost`, `time.completed`, and
  `error` when it did not succeed: `aborted` for an interrupt, the model's error for a failure), and only then are the
  closing events published, so a client that reloads the history on the terminal event finds the finished message.
- **Interrupt.** Cancels the running reply, waits up to 10 s for its last events and answers `204`, also when nothing
  runs. The partial text is kept: `session.text.ended` with it, then `session.execution.interrupted`.
- **Restart.** The registry is in process, so an unfinished assistant message nothing here has claimed was being
  written when the server stopped. It is settled on the first touch of its thread (a prompt, an `interrupt` or a
  `/log` reader): the missing `session.text.ended`, then `session.execution.failed` with error type `restarted`, and
  the compaction with its partial text.

### The `/log` stream

`text/event-stream`, one `data:` line of JSON (the envelope) per event, `id:` its `seq`, and a comment heartbeat
every 15 s. It replays the stored events after `after` or `Last-Event-ID` (the later of the two), then, with
`follow=1`, the live ones until the client goes away; `follow=0` (the default) closes after the replay. The live feed
is subscribed before the store is read, and an event at or below the last `seq` sent is dropped, so a resume has no
gap and no duplicate; a reader that falls behind the feed (or sees a `seq` jump) reads the store again. Events of
finished replies are pruned 24 h after their compaction, so a resume older than that starts from what remains (the
history route has the messages).

### One replica

The claims and the live broadcasts are in memory, like the relay's hub: the extension supports one server replica.
Behind two, a follower on one replica would not see a reply running on the other until it reconnects (the store
still has every event), and a reply running on one would look abandoned to the other.

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

## Settings and the inactivity job

The extension has an optional config file, `services/config/dynwork-chats.yaml` (config prefix `dynwork-chats`):

```yaml
inactive_days: 15
web_search_per_thread: 100
```

Both keys are optional and unknown keys are refused. Core validates the file at startup (`validate_config`), and
the job reads it on every run, so a change applies from the next run without a restart.

- `inactive_days`: threads with no activity for longer than this are deleted by the job. Unset (the default) keeps
  threads until a person deletes them. It must be at least 1, so a typo can never read as "delete everything idle".
- `web_search_per_thread`: the per-thread web search ceiling of stage 3. It is read (`ChatsConfig`) and nothing uses
  it yet.

The job is `dynwork_chats_retention` (cron `0 30 3 * * *`), registered with `submit_job!` and listed in
`services/scheduler/config.yaml`. Activity is the thread's `updated_at` (a message, the end of a reply, an edit by a
person) or any event of a reply written since, so a reply still streaming keeps its thread. An idle thread, archived
or not, is tombstoned through `ChatStore::delete_thread` at the revision the job read: the same tombstone a person's
`DELETE` writes, with the messages and events dropped, so clients drop it on their next `changes` call. A thread
edited between the read and the delete is left alone. The job never removes a row for good (a later purge is out of
scope), works in batches of 200 threads, at most 100 batches a run, and the next night's run takes what is left. The
store's `idle_threads` is the one read across users, with `prune_events`.

## Queries

`PgChatStore` uses the compile-time checked `sqlx::query!` family, so the 23 queries have entries in the workspace's
`.sqlx/` cache (`query-*.json`). After changing a query, regenerate them against a database that has the schema:
run `cargo check -p dynwork-chats` with `DATABASE_URL` set, `SQLX_OFFLINE=false` and `SQLX_OFFLINE_DIR` pointing at an
empty directory, and copy the resulting `query-*.json` files into `.sqlx/`. The Postgres suite then runs every query
against the real schema.

## Tests

- `tests/unit/dynwork-chats`: the config file and block, the model list over the committed catalog, ids, the cursor, error keys and statuses, the extension's registration, the memory
  store, `If-Match` and page sizes, the thread body, the wire shapes; the event envelope per type, derived ids, finish
  reasons, usage folding, the history sent to the model, the prompt body, the log query.
- `tests/integration/dynwork-chats`: the store contract, and the HTTP suites (auth, threads, the feed, history) driven
  in process through the router as core mounts it, beside a stand-in for the relay's `/api/dynwork` mount, with real
  RS256 desktop and mobile tokens. Each is stamped once onto the memory store and once onto Postgres (`db_or_skip!`
  throwaway database, dropped at the end of each test; with `CI=1` a missing database is a failure, not a skip). On
  Postgres the phone check runs against the relay's real device table. The reply suites (`http_replies`, `http_log`)
  run the same way with a scripted `ReplyModel` in place of the AI service, as do `http_models` and the store-level
  `retention` suite.
- `tests/integration/dynwork-chats/fixtures/cloud-reply.jsonl`: written by the recorder test on every run (memory
  store, fixed thread, prompt and clock, so the file only changes when the events do), in the line format of the
  `@dynwork/mobile-model` fixtures: one `meta` line and one `{"kind": "event", "event"}` line per `/log` event. The
  phone's and the desktop's shape checks read it.
