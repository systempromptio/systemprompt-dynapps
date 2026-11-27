//! The admin's Brain section against Postgres, in a throwaway database with
//! the full extension schema, driven in process through the routes the admin
//! mounts and the real Handlebars templates:
//!
//! - access: every page answers `200` to an admin and `403` to anyone else, and
//!   so do the three writes
//! - the tree shows every space and article, an article opens in the language
//!   asked for, and an unknown one is `404`
//! - the technical tabs: chunks with their heading path, text-search
//!   configuration, embedding state and prefix; the ingest steps and the last
//!   error; the global queue and its filter
//! - Reindex queues the document and Retry requeues a failed job, each with an
//!   audit row; neither works on what is not there
//! - access: the article's own and effective grants, and the form that adds,
//!   changes and removes one
//! - the search lab: the rank and score of each arm, what deduplication drops,
//!   the answer context, "Run as" with another person's permissions and one
//!   audit row for every run; the `explain` order is the order of `search`
//! - usage: the documents returned and cited, and the searches that found
//!   nothing, as counts
//! - a proxy instance: the tree and the document read through the operations,
//!   and the technical pages say the index lives at the source while the tables
//!   they would read are gone

#[cfg(test)]
mod access;
#[cfg(test)]
mod env;
#[cfg(test)]
mod lab;
#[cfg(test)]
mod pages;
#[cfg(test)]
mod proxy;
#[cfg(test)]
mod technical;
#[cfg(test)]
mod usage;
