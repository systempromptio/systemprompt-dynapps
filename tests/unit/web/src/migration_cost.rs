//! Static gate: an astound migration that rewrites a hot table must state
//! what it measured, and a new row trigger may not fan out beyond its row.
//!
//! Migrations run before the HTTP listener binds, and every per-row trigger
//! fires for every row a migration touches, so a backfill's true cost is the
//! fan-out (a 3,644-row `UPDATE ai_requests` once took 27 minutes). The
//! detector is core's `systemprompt::database::audit_one`; this repo adds its
//! own hot tables. Both baselines may only shrink: a stale entry fails.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use systemprompt::database::{HOT_TABLES, audit_one};

use crate::support::repo_root;

// Row counts from the 2026-09-22 production analysis.
const ASTOUND_HOT_TABLES: &[&str] = &[
    "plugin_usage_events",
    "conversation_facts",
    "analysis_skill_events",
    "mcp_tool_executions",
    "governance_decisions",
];

const COST_BASELINE: &str = "tests/unit/web/migration_cost_baseline.txt";
const FANOUT_BASELINE: &str = "tests/unit/web/trigger_fanout_baseline.txt";

fn hot_tables() -> Vec<&'static str> {
    let mut all = HOT_TABLES.to_vec();
    all.extend_from_slice(ASTOUND_HOT_TABLES);
    all
}

fn baseline(relative: &str) -> BTreeSet<String> {
    let path = repo_root().join(relative);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} must exist: {e}", path.display()));
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

fn migration_files() -> Vec<PathBuf> {
    let dir = repo_root().join("extensions/web/schema/migrations");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{} must be readable: {e}", dir.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|e| e == "sql")
                && !path.to_string_lossy().ends_with(".down.sql")
        })
        .collect();
    assert!(
        !files.is_empty(),
        "no migrations found under {} — a skipped run must not look green",
        dir.display()
    );
    files.sort();
    files
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

#[test]
fn every_hot_table_rewrite_declares_its_measured_cost() {
    let allowed = baseline(COST_BASELINE);
    let hot = hot_tables();
    let mut undeclared = Vec::new();
    let mut malformed = Vec::new();
    let mut declared = 0usize;
    let mut grandfathered = BTreeSet::new();

    for path in migration_files() {
        let sql = std::fs::read_to_string(&path).expect("migration readable");
        let name = stem(&path);
        let Some(cost) = audit_one("web", &name, &sql, &hot) else {
            continue;
        };
        if let Some(reason) = cost.malformed {
            malformed.push(format!("{name}: {reason}"));
        } else if cost.declared.is_some() {
            declared += 1;
        } else if allowed.contains(&name) {
            grandfathered.insert(name);
        } else {
            undeclared.push(format!("{name}: {}", cost.statement_summary()));
        }
    }

    assert!(
        malformed.is_empty(),
        "migrations with an unparseable `-- @cost:` directive:\n  {}",
        malformed.join("\n  ")
    );
    assert!(
        undeclared.is_empty(),
        "these migrations rewrite a hot table without declaring what it costs:\n  {}\n\n\
         Measure it against a production-shaped copy, then add a leading line:\n  \
         -- @cost: rows=<written> measured=<wall clock, e.g. 2.0s> triggers=<suspended|live>\n\n\
         If the per-row fan-out is what makes it slow, suspend the triggers around the\n\
         statement (`ALTER TABLE t DISABLE TRIGGER x` / `ENABLE`) and say\n\
         `triggers=suspended`; if a projection must see the change, leave them live and\n\
         say so. The runner turns `measured` into the statement_timeout, so it is\n\
         load-bearing, not a comment.",
        undeclared.join("\n  ")
    );

    let stale: Vec<&String> = allowed.difference(&grandfathered).collect();
    assert!(
        stale.is_empty(),
        "{COST_BASELINE} names migrations that no longer need grandfathering:\n  {stale:?}\n\
         Delete those lines — the baseline may only shrink."
    );
    println!(
        "migration cost: {declared} declared, {} grandfathered",
        grandfathered.len()
    );
}

struct RowTrigger {
    file: String,
    trigger: String,
    function: String,
}

// Why: the declarative schema is the live definition; a migration's copy is
// already-applied history and would grow the baseline every upgrade.
fn schema_files() -> Vec<PathBuf> {
    let dir = repo_root().join("extensions/web/schema");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{} must be readable: {e}", dir.display()))
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|e| e == "sql"))
        .collect();
    assert!(!files.is_empty(), "no schema files under {}", dir.display());
    files.sort();
    files
}

// Why: `CREATE TRIGGER ... ON t` and `FOR EACH ROW` are split across lines
// in this schema, so match statements, not lines. Dollar-quoted bodies go
// first so their internal `;` cannot split a statement.
fn strip_dollar_quoted(sql: &str) -> String {
    let mut out = String::with_capacity(sql.len());
    let mut rest = sql;
    while let Some(open) = rest.find("$$") {
        out.push_str(&rest[..open]);
        let after = &rest[open + 2..];
        match after.find("$$") {
            Some(close) => rest = &after[close + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

fn row_triggers(sql: &str, file: &str) -> Vec<RowTrigger> {
    let stripped = strip_dollar_quoted(sql);
    let mut out = Vec::new();
    for statement in stripped.split(';') {
        let flat = statement.split_whitespace().collect::<Vec<_>>().join(" ");
        // This schema uses `CREATE OR REPLACE TRIGGER` too.
        let Some(start) = flat
            .find("CREATE TRIGGER ")
            .or_else(|| flat.find("CREATE OR REPLACE TRIGGER "))
        else {
            continue;
        };
        let flat = &flat[start..];
        if !flat.contains("FOR EACH ROW") {
            continue;
        }
        let words: Vec<&str> = flat.split_whitespace().collect();
        let Some(trigger) = words.get(if flat.starts_with("CREATE OR REPLACE") {
            4
        } else {
            2
        }) else {
            continue;
        };
        let Some(function) = flat
            .split("EXECUTE FUNCTION ")
            .nth(1)
            .and_then(|rest| rest.split('(').next())
        else {
            continue;
        };
        out.push(RowTrigger {
            file: file.to_owned(),
            trigger: (*trigger).to_owned(),
            function: function.trim().to_owned(),
        });
    }
    out
}

// Why: the trigger's `EXECUTE FUNCTION f()` also names the function, after
// the definition; matching bare `FUNCTION f(` lands there and reads every
// fanning-out function as clean.
fn function_body(sql: &str, name: &str) -> Option<String> {
    let start = [
        format!("CREATE OR REPLACE FUNCTION {name}("),
        format!("CREATE FUNCTION {name}("),
    ]
    .iter()
    .filter_map(|marker| sql.rfind(marker.as_str()))
    .max()?;
    // Why: plpgsql bodies here may end `$$ LANGUAGE plpgsql;`, so bound on the
    // `$$` pair; searching for `$$;` runs into the next function.
    let body = &sql[start..];
    let open = body.find("$$")?;
    let rest = &body[open + 2..];
    let close = rest.find("$$").unwrap_or(rest.len());
    Some(rest[..close].to_owned())
}

// Why: plpgsql is not SQL the parser accepts, so the body text is scanned.
fn loops(body: &str) -> bool {
    body.contains(" LOOP") || body.contains("\nLOOP")
}

fn declared_functions(sql: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for marker in ["CREATE OR REPLACE FUNCTION ", "CREATE FUNCTION "] {
        for piece in sql.split(marker).skip(1) {
            if let Some(name) = piece.split('(').next() {
                let name = name.trim();
                if !name.is_empty() && !name.contains(char::is_whitespace) {
                    out.insert(name.to_owned());
                }
            }
        }
    }
    out
}

// Why transitive: a trigger function with no loop of its own may call one
// that loops over the whole session; the cost lands on the firing row.
fn fans_out(sql: &str, entry: &str, declared: &BTreeSet<String>) -> bool {
    let mut seen = BTreeSet::new();
    let mut queue = vec![entry.to_owned()];
    while let Some(name) = queue.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(body) = function_body(sql, &name) else {
            continue;
        };
        if loops(&body) {
            return true;
        }
        for callee in declared {
            if body.contains(&format!("{callee}(")) && !seen.contains(callee) {
                queue.push(callee.clone());
            }
        }
    }
    false
}

#[test]
fn a_row_trigger_does_not_fan_out_beyond_its_row() {
    let allowed = baseline(FANOUT_BASELINE);
    let mut bodies = String::new();
    let mut triggers = Vec::new();
    for path in schema_files() {
        let sql = std::fs::read_to_string(&path).expect("schema readable");
        let file = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        triggers.extend(row_triggers(&sql, &file));
        bodies.push_str(&sql);
    }
    // Why: counting `FOR EACH ROW` directly fails a parser that silently stops
    // matching a statement shape.
    let declared_row_triggers = strip_dollar_quoted(&bodies).matches("FOR EACH ROW").count();
    assert_eq!(
        triggers.len(),
        declared_row_triggers,
        "parsed {} row triggers but the schema declares {declared_row_triggers} — \
         the trigger parser is missing a statement shape",
        triggers.len()
    );
    assert!(
        !triggers.is_empty(),
        "found no row triggers — the gate is not reading the schema"
    );

    let declared = declared_functions(&bodies);
    let mut offenders = Vec::new();
    let mut exempt = BTreeSet::new();
    for trigger in &triggers {
        if !fans_out(&bodies, &trigger.function, &declared) {
            continue;
        }
        if allowed.contains(&trigger.function) {
            exempt.insert(trigger.function.clone());
        } else {
            offenders.push(format!(
                "{} runs {} FOR EACH ROW, and {} loops",
                trigger.file, trigger.trigger, trigger.function
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "these row triggers fan out past the row that fired them:\n  {}\n\n\
         One row written becomes N statements, so a bulk write becomes O(N²) and a\n\
         migration that touches the table stops being a migration. Aggregate the work\n\
         with a statement-level trigger (`FOR EACH STATEMENT` with a transition table,\n\
         as core's reporting capture does) instead of looping per row.",
        offenders.join("\n  ")
    );

    let stale: Vec<&String> = allowed.difference(&exempt).collect();
    assert!(
        stale.is_empty(),
        "{FANOUT_BASELINE} names functions that no longer fan out:\n  {stale:?}\n\
         Delete those lines — this baseline may only shrink."
    );
    println!(
        "row triggers: {} checked, {} exempt",
        triggers.len(),
        exempt.len()
    );
}
