//! Route fragments the composition root merges.
//!
//! The dashboard pages and their polled fragments (SSR router, `/admin`),
//! the console JSON API (`/api/public/admin`), and the public run API
//! (`/api/public/agent-farm`). Paths are relative to the mount point the
//! root nests them under.

pub mod api;
pub mod farm_fragments;
pub mod farm_ssr;
pub mod public;
