//! Indexing the native Brain: the same process dynbrain runs, in Rust.
//!
//! Writing a variant queues it (`dynbrain_index_job`, one row per variant
//! content); nothing is indexed inside the request. A job of the scheduler
//! ([`job::DynbrainIndexJob`]) works the queue under a lease, so only one
//! replica does it at a time:
//!
//! 1. An article is eligible by its own mode or by the first rule that matches
//!    it, and denied when nothing says so ([`eligibility`]); an ineligible one
//!    has its chunks purged.
//! 2. The Markdown, with `asset://` references replaced by their alt text, is
//!    cut by the dynbrain chunker ([`chunker`], 300 words, 50 of overlap, with
//!    the heading path of each chunk).
//! 3. A chunk with no heading gets a one-sentence synopsis from a small model
//!    ([`context`]); one whose text did not change keeps the synopsis it had.
//! 4. Each chunk is embedded as its context prefix plus its text, in batches of
//!    50 that split in half when a batch is refused ([`bisect`]), and stored
//!    with the fingerprint (`provider:model:dims`) of what made the vector. The
//!    cost of each request is recorded.
//!
//! The models sit behind [`provider::Embedder`] and [`provider::Summarizer`];
//! [`core_ai`] implements them over core's `AiService`. Without an embedding
//! key the chunks wait, and the search later uses only its text arm.
//!
//! The job `dynbrain_upload_purge` ([`purge`]) deletes the uploads whose link
//! expired and that no article ever adopted, with their bytes.

pub mod bisect;
pub mod chunker;
pub mod context;
pub mod core_ai;
pub mod eligibility;
pub mod index;
pub mod job;
pub mod model;
pub mod provider;
pub mod purge;
pub mod run;
pub mod synopsis;
pub mod text;
