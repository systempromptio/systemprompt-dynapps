//! The audit guard's declarative schema is the one definition, and every
//! audit table the matrix names carries its triggers.
//!
//! Every install runs `schema/51_audit_integrity.sql`; there is no migration
//! chain to drift from it.

use crate::support::repo_root;

const SCHEMA: &str = "extensions/web/schema/51_audit_integrity.sql";
const GUARDED: &[(&str, &[&str])] = &[
    (
        "ai_requests",
        &["audit_update", "audit_delete", "audit_truncate"],
    ),
    (
        "ai_safety_findings",
        &["audit_update", "audit_delete", "audit_truncate"],
    ),
    (
        "ai_request_attributions",
        &["audit_update", "audit_delete", "audit_truncate"],
    ),
    ("governance_decisions", &["audit_delete", "audit_truncate"]),
    ("mcp_tool_executions", &["audit_delete", "audit_truncate"]),
    ("plugin_usage_events", &["audit_delete", "audit_truncate"]),
    (
        "secret_audit_log",
        &["audit_update", "audit_delete", "audit_truncate"],
    ),
    (
        "platform_data_clear_audit",
        &["audit_update", "audit_delete", "audit_truncate"],
    ),
];

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn the_chain_is_sealed_by_the_job_not_an_insert_trigger() {
    let schema = read(SCHEMA);
    for gone in [
        "audit_chain_link_ai_requests",
        "audit_chain_link_ai_safety_findings",
    ] {
        assert!(!schema.contains(gone), "schema/51 still declares {gone}");
    }
    for table in ["ai_requests", "ai_safety_findings"] {
        assert!(
            !schema.contains(&format!("TRIGGER {table}_audit_chain\n")),
            "{table} still links rows on insert"
        );
    }
    for needed in [
        "CREATE OR REPLACE FUNCTION audit_chain_seal(p_limit INTEGER)",
        "pg_try_advisory_xact_lock(hashtext('audit_chain_seal:chain'))",
        "unsealed_rows BIGINT NOT NULL DEFAULT 0",
        "CREATE OR REPLACE FUNCTION audit_chain_append(",
    ] {
        assert!(schema.contains(needed), "schema/51 is missing {needed}");
    }
}

#[test]
fn every_guarded_table_has_its_triggers() {
    let schema = read(SCHEMA);
    for (table, triggers) in GUARDED {
        for trigger in *triggers {
            let declaration = format!("CREATE OR REPLACE TRIGGER {table}_{trigger}\n");
            assert!(
                schema.contains(&declaration),
                "{table} is missing {trigger}"
            );
            let target = format!(" ON {table} ");
            let after = &schema[schema.find(&declaration).unwrap_or(0)..];
            assert!(
                after
                    .lines()
                    .nth(1)
                    .is_some_and(|line| line.contains(&target)),
                "{table}_{trigger} is not declared on {table}"
            );
        }
    }
}

#[test]
fn guard_defaults_to_warn_with_the_chain_on() {
    let schema = read(SCHEMA);
    assert!(schema.contains("mode TEXT NOT NULL DEFAULT 'warn'"));
    assert!(schema.contains("chain_enabled BOOLEAN NOT NULL DEFAULT TRUE"));
    assert!(
        schema
            .contains("COALESCE((SELECT mode FROM audit_guard_settings WHERE singleton), 'warn')")
    );
}
