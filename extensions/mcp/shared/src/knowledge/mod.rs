//! Knowledge-bank types shared below the entry tier.
//!
//! The knowledge-bank MCP server owns ingestion and retrieval, but its
//! configuration model, error type, calling principal and the durable Drive
//! inventory store are also read by the scheduler's daily Drive sync job in
//! `extensions/web/jobs`. That crate sits in the app tier and may not depend
//! on an MCP server binary, so the pieces both need live here and the server
//! re-exports them at its original paths.

pub mod config;
pub mod drive;
pub mod error;
pub mod principal;
