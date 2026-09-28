//! The shared core of the admin dashboard: everything two or more admin
//! domains read, and nothing that belongs to only one of them.
//!
//! - [`error`], [`types`], [`util`], [`numeric`] — the error sink, wire and
//!   view types, and pure helpers every admin crate agrees on.
//! - [`templates`], [`assets`], [`ssr`] — the Handlebars engine, the admin
//!   asset list, and the server-rendered page kit (shell, formatting, shared
//!   view-models, chart builders).
//! - [`auth`], [`middleware`], [`request`], [`state`] — caller identity, the
//!   role gates, request helpers, and the managed-repository state.
//! - [`authz`] — the subject-dimension ladder, the per-database registry of
//!   subject-attribute providers, and the providers whose data lives here.
//! - [`connections`], [`marketplace_filter`] — the connection model and its
//!   registered snapshot source, and the marketplace filter that gates on it.
//! - [`activity`], [`event_hub`] — the activity log and the live event fan-out.
//! - [`repositories`] — data access shared across domains: scope, principal,
//!   entitlement, access rules, the sync engine, bridge credentials, the
//!   gateway route catalog, conversations and usage ingestion.
//! - [`catalog_index`], [`crypto`], [`export`] — the marketplace catalog index,
//!   secret envelope encryption, and the export data model.
//!
//! Crates that depend on this one path-qualify; nothing is re-exported from
//! the crate root.

pub mod activity;
pub mod assets;
pub mod auth;
pub mod authz;
pub mod catalog_index;
pub mod connections;
pub mod crypto;
pub mod error;
pub mod event_hub;
pub mod export;
pub mod marketplace_filter;
pub mod middleware;
pub mod numeric;
pub mod repositories;
pub mod request;
pub mod ssr;
pub mod state;
pub mod templates;
pub mod types;
pub mod util;
