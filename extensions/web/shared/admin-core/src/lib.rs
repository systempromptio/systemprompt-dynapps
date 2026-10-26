//! The shared core of the admin dashboard: everything two or more admin
//! domains read, and nothing that belongs to only one of them. Dependents
//! path-qualify; nothing is re-exported from the crate root.

pub mod activity;
pub mod archive_objects;
pub mod assets;
pub mod auth;
pub mod authz;
pub mod bridge_client;
pub mod catalog_index;
pub mod connections;
pub mod crypto;
pub mod error;
pub mod event_hub;
pub mod export;
pub mod extension_services;
pub mod marketplace_filter;
pub mod middleware;
pub mod numeric;
pub mod repositories;
pub mod request;
pub mod ssr;
pub mod state;
pub mod templates;
pub mod ttl_cache;
pub mod types;
pub mod util;
