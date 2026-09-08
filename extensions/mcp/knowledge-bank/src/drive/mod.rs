//! Shared-drive ingestion independent of personal OAuth sessions.

pub mod aws;
pub mod config;
pub mod extract;
pub mod google;
pub mod office;
pub mod output;
pub mod store;
pub mod types;
pub mod worker;

pub mod status;

mod comments;
mod process;
