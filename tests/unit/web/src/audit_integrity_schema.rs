//! The audit guard's declarative schema and its migrations stay one definition,
//! and every audit table the matrix names carries its triggers.
//!
//! A fresh install runs `schema/51_audit_integrity.sql`; an upgraded one runs
//! migration 118, then 120 (`audit_chain_append`), then 121 (the sealer). If
//! the two drift, the guard an operator reads about in the migration header is
//! not the guard a new instance gets.

use crate::support::repo_root;

const SCHEMA: &str = "extensions/web/schema/51_audit_integrity.sql";
const SEALER: &str = "extensions/web/schema/migrations/121_audit_chain_sealer.sql";

const MIGRATIONS_NEWEST_FIRST: &[&str] = &[
    SEALER,
    "extensions/web/schema/migrations/120_audit_chain_append_no_loop.sql",
    "extensions/web/schema/migrations/118_audit_integrity.sql",
];

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

const PLPGSQL_END: &str = "END $$;";
const SQL_END: &str = "\n$$;";

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn declared_functions(sql: &str) -> Vec<String> {
    let head = "CREATE OR REPLACE FUNCTION ";
    sql.match_indices(head)
        .filter_map(|(at, _)| {
            let rest = &sql[at + head.len()..];
            rest.find('(').map(|end| rest[..end].to_owned())
        })
        .collect()
}

fn function_block<'a>(sql: &'a str, name: &str) -> Option<&'a str> {
    let start = sql.find(&format!("CREATE OR REPLACE FUNCTION {name}("))?;
    let rest = &sql[start..];
    let end = match (rest.find(PLPGSQL_END), rest.find(SQL_END)) {
        (Some(a), Some(b)) if b < a => b + SQL_END.len(),
        (Some(a), _) => a + PLPGSQL_END.len(),
        (None, Some(b)) => b + SQL_END.len(),
        (None, None) => panic!("{name} has no closing $$;"),
    };
    Some(&rest[..end])
}

fn cursor_table(sql: &str) -> Option<&str> {
    let start = sql.find("CREATE TABLE IF NOT EXISTS audit_chain_cursors (")?;
    sql[start..].split(");\n").next()
}

#[test]
fn migrations_carry_the_schema_ddl_verbatim() {
    let schema = read(SCHEMA);
    let migrations: Vec<String> = MIGRATIONS_NEWEST_FIRST.iter().map(|m| read(m)).collect();
    for name in declared_functions(&schema) {
        let newest = migrations
            .iter()
            .find_map(|sql| function_block(sql, &name))
            .unwrap_or_else(|| panic!("no migration declares {name}"));
        assert_eq!(
            Some(newest),
            function_block(&schema, &name),
            "the newest migration declaring {name} must carry schema/51's body verbatim"
        );
    }
    assert_eq!(
        cursor_table(&migrations[0]),
        cursor_table(&schema),
        "migration 121 must create audit_chain_cursors exactly as schema/51 does"
    );
}

#[test]
fn the_chain_is_sealed_by_the_job_not_an_insert_trigger() {
    let schema = read(SCHEMA);
    let sealer = read(SEALER);
    for gone in [
        "audit_chain_link_ai_requests",
        "audit_chain_link_ai_safety_findings",
    ] {
        assert!(!schema.contains(gone), "schema/51 still declares {gone}");
        assert!(
            sealer.contains(&format!("DROP FUNCTION IF EXISTS {gone}();")),
            "migration 121 must drop {gone}"
        );
    }
    for table in ["ai_requests", "ai_safety_findings"] {
        assert!(
            !schema.contains(&format!("TRIGGER {table}_audit_chain\n")),
            "{table} still links rows on insert"
        );
        assert!(
            sealer.contains(&format!(
                "DROP TRIGGER IF EXISTS {table}_audit_chain ON {table};"
            )),
            "migration 121 must drop {table}_audit_chain"
        );
    }
    for needed in [
        "CREATE OR REPLACE FUNCTION audit_chain_seal(p_limit INTEGER)",
        "pg_try_advisory_xact_lock(hashtext('audit_chain_seal'))",
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
