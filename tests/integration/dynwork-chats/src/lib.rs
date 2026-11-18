//! The `dynwork-chats` store contract and its HTTP API.
//!
//! The store contract runs against both stores: the in-memory one, and the
//! Postgres one against a throwaway database with the full extension schema.
//!
//! - threads: create, read, rename with a revision bump, `If-Match` conflicts,
//!   archive, tombstones
//! - the changes feed: order, tombstones, the cursor, paging
//! - messages: per-thread `seq`, idempotent appends, history pages
//! - events: per-thread `seq`, resuming after a `seq`
//! - ending a reply: compaction into its message, pruning, a `seq` that never
//!   repeats
//! - isolation between users
//!
//! The HTTP suites drive `/api/dynwork/chats/v1` in process, mounted as core
//! mounts it, over both stores:
//!
//! - auth: the desktop and the mobile JWT map to one user, a revoked phone and
//!   every other credential are `401`, and nothing is read before the token
//! - threads: create, `If-Match` edits and `409` with the current row, `428` on
//!   a bare delete, tombstones, `404` across users, bad requests
//! - the changes feed: the full list, the cursor round trip, paging
//! - message history: newest first, `before` and `more`
//! - replies: a prompt answers `202` with its reply, the same `messageID` twice
//!   is one reply, `409` while one runs, `404` across users, `503` without a
//!   model service, a model error, `interrupt` keeping the partial text, the
//!   history sent to the model, compaction into the messages, a reply an
//!   earlier process left unfinished
//! - the models list: the shape the phone and the desktop read, a phone, auth,
//!   `503` without a model service, a role the access rules deny losing a
//!   model, and the token's client reaching the rules
//! - model admission: a prompt or a thread write naming a model the instance's
//!   rules do not allow the caller is `403 model_not_allowed` and changes
//!   nothing; the thread's own model is judged when the prompt names none
//! - the inactivity sweep: past the window or inside it, a message, an edit or
//!   a streaming reply as activity, batches, every user's threads, tombstones
//!   in the feed
//! - the change hint: every kind of thread change, a reply starting and ending,
//!   and the inactivity sweep send one `chats` hint to the right user, a
//!   refused write sends none, and the relay's `NOTIFY` carries it
//! - the `/log` stream: replay then follow, a reader joining mid-reply, `after`
//!   and `Last-Event-ID`, `follow=0`, a follower going away
//! - the recorder: a cloud reply's events written to
//!   `fixtures/cloud-reply.jsonl` for the phone's shape check

#[cfg(test)]
mod contract;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod hints;
#[cfg(test)]
mod http_admission;
#[cfg(test)]
mod http_auth;
#[cfg(test)]
mod http_changes;
#[cfg(test)]
mod http_log;
#[cfg(test)]
mod http_messages;
#[cfg(test)]
mod http_models;
#[cfg(test)]
mod http_replies;
#[cfg(test)]
mod http_support;
#[cfg(test)]
mod http_threads;
#[cfg(test)]
mod models_support;
#[cfg(test)]
mod recorder;
#[cfg(test)]
mod reply_support;
#[cfg(test)]
mod retention;
