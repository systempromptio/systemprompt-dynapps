//! `dynwork-routines` against a throwaway database with the full extension
//! schema, driven through its router with real RS256 desktop tokens:
//!
//! - optimistic concurrency: a stale or absent `If-Match` answers `409` with
//!   the current row
//! - the change feed: nothing lost between pages while writers race, and
//!   tombstones in it
//! - user isolation: one person's routines are invisible to another
//! - run summaries only from the routine's owner device
//! - the change hint reaching the user's other desktops through the relay
//! - the page's writes: pause, resume and delete without `If-Match`

#[cfg(test)]
mod support;

#[cfg(test)]
mod concurrency;
#[cfg(test)]
mod feed;
#[cfg(test)]
mod hint;
#[cfg(test)]
mod isolation;
#[cfg(test)]
mod page_writes;
#[cfg(test)]
mod runs;
