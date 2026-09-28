//! Schema and migration inventory for the web extension.
//!
//! DDL lives in `schema/*.sql` and is embedded with `include_str!`; migrations
//! are discovered from `schema/migrations/` by `build.rs`. Nothing here is SQL
//! text.

use systemprompt::extension::prelude::{Migration, SchemaDefinition, extension_migrations};

pub(crate) const SCHEMA_PLUGIN_USAGE: &str = include_str!("../schema/05_plugin_usage.sql");
pub(crate) const SCHEMA_ANALYTICS: &str = include_str!("../schema/07_analytics.sql");
pub(crate) const SCHEMA_SECRETS: &str = include_str!("../schema/09_secrets.sql");
pub(crate) const SCHEMA_ADMIN_DASHBOARD: &str = include_str!("../schema/10_admin_dashboard.sql");
pub(crate) const SCHEMA_MANAGEMENT: &str = include_str!("../schema/12_management.sql");
pub(crate) const SCHEMA_WEB_SIDE_TABLES: &str = include_str!("../schema/13_web_side_tables.sql");
pub(crate) const SCHEMA_USAGE_METRICS: &str = include_str!("../schema/17_usage_metrics.sql");
pub(crate) const SCHEMA_SALESFORCE_IDENTITY: &str =
    include_str!("../schema/21_salesforce_identity.sql");
pub(crate) const SCHEMA_DEV_LOGIN_CODES: &str = include_str!("../schema/22_dev_login_codes.sql");
pub(crate) const SCHEMA_GROUPS_PROJECTS: &str = include_str!("../schema/23_groups_projects.sql");
pub(crate) const SCHEMA_SCOPE_DEFAULTS: &str = include_str!("../schema/24_scope_defaults.sql");

pub(crate) const SCHEMA_CONNECTOR_CREDENTIALS: &str =
    include_str!("../schema/25_connector_credentials.sql");

#[doc(hidden)]
pub fn schema_definitions() -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::sql_only(SCHEMA_PLUGIN_USAGE),
        SchemaDefinition::sql_only(SCHEMA_ANALYTICS),
        SchemaDefinition::sql_only(SCHEMA_SECRETS),
        SchemaDefinition::sql_only(SCHEMA_ADMIN_DASHBOARD),
        SchemaDefinition::sql_only(SCHEMA_MANAGEMENT),
        SchemaDefinition::sql_only(SCHEMA_WEB_SIDE_TABLES),
        SchemaDefinition::sql_only(SCHEMA_USAGE_METRICS),
        SchemaDefinition::sql_only(SCHEMA_SALESFORCE_IDENTITY),
        SchemaDefinition::sql_only(SCHEMA_DEV_LOGIN_CODES),
        SchemaDefinition::sql_only(SCHEMA_GROUPS_PROJECTS),
        SchemaDefinition::sql_only(SCHEMA_SCOPE_DEFAULTS),
        SchemaDefinition::sql_only(SCHEMA_CONNECTOR_CREDENTIALS),
        SchemaDefinition::sql_only(include_str!("../schema/26_connector_accounts.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/27_conversation_requests.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/28_ingestion_integrity.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/29_skill_version_impact.sql")),
        SchemaDefinition::sql_only(include_str!(
            "../schema/30_feedback_authenticated_events.sql"
        )),
        SchemaDefinition::sql_only(include_str!("../schema/31_skill_version_receipts.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/32_raw_retention.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/33_sync_state.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/34_service_sources.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/36_marketplace_versions.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/37_conversation_analyses.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/40_request_scopes.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/41_time_bound_access.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/42_gateway_routes.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/45_conversation_facts.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/46_tool_artifacts.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/47_user_last_seen.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/48_retention.sql")),
    ]
}

#[doc(hidden)]
pub fn migrations() -> Vec<Migration> {
    extension_migrations!()
}
