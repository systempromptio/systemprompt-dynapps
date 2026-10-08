//! The audit guard's declarative schema and its migration stay one definition,
//! and every audit table the matrix names carries its triggers.
//!
//! A fresh install runs `schema/51_audit_integrity.sql`; an upgraded one runs
//! migration 118 then 120, which replaces `audit_chain_append`. If the two
//! drift, the guard an operator reads about in the migration header is not the
//! guard a new instance gets.

use crate::support::repo_root;

const GUARDED: &[(&str, &[&str])] = &[
    (
        "ai_requests",
        &[
            "audit_update",
            "audit_delete",
            "audit_truncate",
            "audit_chain",
        ],
    ),
    (
        "ai_safety_findings",
        &[
            "audit_update",
            "audit_delete",
            "audit_truncate",
            "audit_chain",
        ],
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

fn function_block<'a>(sql: &'a str, name: &str) -> &'a str {
    let start = sql
        .find(&format!("CREATE OR REPLACE FUNCTION {name}("))
        .unwrap_or_else(|| panic!("{name} is not declared"));
    let end = sql[start..].find("END $$;").map_or_else(
        || panic!("{name} has no closing END $$;"),
        |offset| start + offset + "END $$;".len(),
    );
    &sql[start..end]
}

#[test]
fn migrations_carry_the_schema_ddl_verbatim() {
    let schema = read("extensions/web/schema/51_audit_integrity.sql");
    let original = read("extensions/web/schema/migrations/118_audit_integrity.sql");
    let replacement = read("extensions/web/schema/migrations/120_audit_chain_append_no_loop.sql");
    let upgraded = original.replace(
        function_block(&original, "audit_chain_append"),
        function_block(&replacement, "audit_chain_append"),
    );
    assert!(
        upgraded.ends_with(&schema),
        "migration 118 with 120's audit_chain_append must end with schema/51_audit_integrity.sql unchanged"
    );
}

#[test]
fn every_guarded_table_has_its_triggers() {
    let schema = read("extensions/web/schema/51_audit_integrity.sql");
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
    let schema = read("extensions/web/schema/51_audit_integrity.sql");
    assert!(schema.contains("mode TEXT NOT NULL DEFAULT 'warn'"));
    assert!(schema.contains("chain_enabled BOOLEAN NOT NULL DEFAULT TRUE"));
    assert!(
        schema
            .contains("COALESCE((SELECT mode FROM audit_guard_settings WHERE singleton), 'warn')")
    );
}
