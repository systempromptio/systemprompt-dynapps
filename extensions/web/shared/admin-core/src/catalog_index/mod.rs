//! The marketplace catalog as every domain reads it: marketplace manifests
//! and who may see them, the plugin → skill/agent/MCP maps, and the
//! inventory index the analysis pages resolve names against.

pub mod inventory_index;
pub mod manifests;
pub mod manifests_access;
pub mod marketplaces;
pub mod plugin_loader;
pub mod plugin_maps;
pub mod plugin_resolvers;
