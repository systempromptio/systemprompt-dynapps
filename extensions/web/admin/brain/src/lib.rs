//! Brain domain of the admin dashboard: the instance's whole Brain, for
//! console admins.
//!
//! It shows what the Brain view of the desktop app shows (the tree of spaces
//! and articles, a document in its language, its access) and the technical
//! layer the desktop app leaves out: the chunks of each document, the ingest
//! queue, a search laboratory that explains every score, and usage. It is
//! read-only except three actions: reindex, retry, and the access of an
//! article. The content is never edited here.
//!
//! The pages read the Brain in the same process through
//! [`dynbrain::console::Console`], not through its REST API. The tree, the
//! document, the files and the laboratory's answer context go through the
//! Brain's operations as the signed-in admin, so they read the same on a
//! native and on a proxy instance. Chunks, the queue, the counters, usage and
//! the laboratory read the native index, and on a proxy instance they say the
//! chunks live at the source and query nothing.
//!
//! "Run as" searches with another person's permissions without using their
//! token, and every laboratory run leaves a row in `dynbrain_admin_audit`.
//!
//! - [`source`] — where the pages get the Brain, and who may enter.
//! - [`read`] — the Brain's operations as typed reads.
//! - [`tree`] — the collapsible tree of spaces and articles.
//! - [`overview`], [`spaces`], [`article`], [`chunks`], [`access`], [`lab`],
//!   [`ingest`], [`usage`] — the pages.
//! - [`actions`] — Reindex and Retry.
//! - [`routes`] — the SSR routes the admin router merges.

pub mod access;
pub mod actions;
pub mod article;
pub mod chunks;
pub mod ingest;
pub mod lab;
pub mod overview;
pub mod read;
pub mod routes;
pub mod source;
pub mod spaces;
pub mod tree;
pub mod usage;
pub mod views;
