//! Catalog domain of the admin dashboard: what is installable on this
//! instance and how it is versioned, published and retained.
//!
//! - [`repositories`] — the `services/` YAML behind the marketplace, plugin,
//!   skill, agent and MCP pages, the MCP runtime tables, the retention ledger,
//!   and the per-version rollups.
//! - [`routes`] — the route fragments the admin composition root merges: the
//!   catalog and lifecycle pages, the versions Analysis pages, the managed
//!   authoring API, the plugin and agent listings, and manifest sharing.
//! - [`test_support`] — the pure helpers the test workspace exercises.
//!
//! The MCP, versions and plugin-evaluation export datasets register with the
//! core export registry from here, so a binary that serves exports must link
//! this crate.

mod export;
pub(crate) mod handlers;
pub mod repositories;
pub mod routes;

pub mod test_support {
    pub use crate::handlers::catalog::mcp::status_of;
    pub use crate::handlers::catalog::sorting::{
        direction, matches, preserved_search, sort_headers,
    };
    pub use crate::handlers::ssr::analysis::marketplace_versions::diff::{
        Change, ManifestDiff, SkillChange, diff as manifest_diff,
    };
}
