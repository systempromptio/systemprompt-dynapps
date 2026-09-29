//! Data access owned by the catalog.
//!
//! It reads the `services/` YAML the marketplace,
//! plugin, skill, agent and MCP pages list, the MCP runtime tables, the
//! retention ledger, and the per-version rollups the versions pages read.
//!
//! Callers path-qualify (`repositories::mcp::runtime::…`); nothing is
//! re-exported, so the module path is the only name a symbol has.

pub mod agents;
pub mod analysis;
pub mod lifecycle;
pub mod marketplace;
pub mod mcp;
