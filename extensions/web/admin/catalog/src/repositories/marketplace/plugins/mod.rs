//! Marketplace catalog and overview queries backed by `services/` YAML.
//!
//! The readers live in core's `catalog_index` (`plugin_catalog` walks the
//! filesystem for read-only skill / agent / plugin rows; `plugin_overview`
//! filters plugins by the caller's roles and counts them), because the
//! identity access-control matrix lists the same rows.

pub use systemprompt_web_admin_core::catalog_index::plugin_catalog::{
    list_agent_catalog, list_plugin_catalog, list_skill_catalog,
};
pub use systemprompt_web_admin_core::catalog_index::plugin_overview::{
    MarketplaceCounts, count_visible_items, list_plugins_for_roles, list_plugins_for_roles_full,
    list_plugins_for_user,
};
