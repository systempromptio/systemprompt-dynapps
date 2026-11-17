//! The `dynwork-chats` store contract, run against both stores: the in-memory
//! one the later suites use, and the Postgres one against a throwaway database
//! with the full extension schema.
//!
//! - threads: create, read, rename with a revision bump, `If-Match` conflicts,
//!   archive, tombstones
//! - the changes feed: order, tombstones, the cursor, paging
//! - messages: per-thread `seq`, idempotent appends, history pages
//! - events: per-thread `seq`, resuming after a `seq`
//! - ending a reply: compaction into its message, pruning, a `seq` that never
//!   repeats
//! - isolation between users

#[cfg(test)]
mod contract;
#[cfg(test)]
mod fixture;
