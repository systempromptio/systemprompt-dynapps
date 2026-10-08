//! The marketplace catalog as every domain reads it.
//!
//! Marketplace manifests and who may see them, the plugin → skill/agent/MCP
//! maps, the inventory index the analysis pages resolve names against, and
//! the `services/` readers for MCP servers, plugins, agents and skills that
//! the catalog pages and the access-control matrix both list.

pub mod agents;
pub mod inventory_index;
pub mod manifests;
pub mod manifests_access;
pub mod marketplaces;
pub mod mcp_servers;
pub mod plugin_catalog;
pub mod plugin_loader;
pub mod plugin_maps;
pub mod plugin_overview;
pub mod plugin_resolvers;
