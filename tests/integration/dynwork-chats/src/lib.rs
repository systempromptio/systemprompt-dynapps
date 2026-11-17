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

#[cfg(test)]
mod contract;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod http_auth;
#[cfg(test)]
mod http_changes;
#[cfg(test)]
mod http_messages;
#[cfg(test)]
mod http_support;
#[cfg(test)]
mod http_threads;
