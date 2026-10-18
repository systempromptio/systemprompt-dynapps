//! Inventory registries must advertise every implementation the crates
//! define; a `submit_*!` forgotten next to a new job or renderer shows up
//! here as a count mismatch (the drift that once hid `SecretMigrationJob`).

use std::collections::BTreeSet;

use systemprompt_web_extension::jobs::extension_jobs;
use systemprompt_web_extension::shared::registry;

#[test]
fn all_jobs_registered() {
    let names: BTreeSet<&'static str> = extension_jobs().iter().map(|j| j.name()).collect();
    let expected: BTreeSet<&'static str> = [
        "access_expiry",
        "audit_chain_seal",
        "audit_chain_verify",
        "blog_content_ingestion",
        "bundle_admin_css",
        "content_analytics_aggregation",
        "conversation_judge",
        "conversation_rollup",
        "copy_extension_assets",
        "farm_scheduled_run",
        "gateway_overhead_rollup",
        "governance_bootstrap",
        "llms_txt_generation",
        "plugin_usage_retention",
        "publish_pipeline",
        "retention_daily_report",
        "retention_export_monthly",
        "retention_export_weekly",
        "robots_txt_generation",
        "secret_migration",
        "secrets_refresh",
        "scope_defaults_recompute",
        "site_content_prerender",
        "site_render",
        "sitemap_generation",
        "usage_anomaly",
        "usage_daily_rollup",
    ]
    .into();
    assert_eq!(names, expected);
}

#[test]
fn stateless_provider_registries_are_complete() {
    assert_eq!(registry::component_renderers().len(), 9);
    // Docs only: the template also registers the blog list and post providers,
    // and Dynapps ships no blog.
    assert_eq!(registry::page_data_providers().len(), 1);
    assert_eq!(registry::content_data_providers().len(), 1);
    // org_url and release_version (site/src/extenders).
    assert_eq!(registry::template_data_extenders().len(), 2);
}

#[test]
fn jobs_with_node_local_effects_are_node_scoped() {
    use systemprompt::traits::JobScope;
    let scope = |name: &str| {
        extension_jobs()
            .into_iter()
            .find(|j| j.name() == name)
            .unwrap_or_else(|| panic!("{name} registered"))
            .scope()
    };
    assert_eq!(scope("site_render"), JobScope::Node, "web.dist is per node");
    assert_eq!(
        scope("secrets_refresh"),
        JobScope::Node,
        "secrets are per process"
    );
    assert_eq!(
        scope("blog_content_ingestion"),
        JobScope::Cluster,
        "ingestion writes the database once per tick"
    );
}
