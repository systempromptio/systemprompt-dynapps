//! Dependency closure for deletion previews. Physical tuple identities are
//! used only inside one transaction; receipts carry counts and fingerprints,
//! never CTIDs that could later point at unrelated rows.
use super::browse::{Selection, resolve_selection};
use super::catalog::{DatabaseTrigger, ForeignKey, Table, list_foreign_keys, list_tables};
use super::{bad, clear_materialize};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgConnection;
use std::collections::{BTreeMap, BTreeSet};
use systemprompt_web_admin_core::error::AdminResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Effect {
    pub table: String,
    pub deleted: i64,
    pub updated: i64,
    pub fingerprint: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClearPlan {
    pub effects: Vec<Effect>,
    pub schema_hash: String,
    pub triggers: Vec<DatabaseTrigger>,
}
pub(crate) struct PreparedPlan {
    pub receipt: ClearPlan,
    pub tables: BTreeMap<i64, Table>,
    pub order: Vec<i64>,
}

pub(crate) fn temp(oid: i64) -> String {
    format!("pg_temp.sp_clear_{oid}")
}

fn reachable(
    root: i64,
    keys: &[ForeignKey],
    tables: &BTreeMap<i64, Table>,
) -> AdminResult<BTreeSet<i64>> {
    let mut out = BTreeSet::from([root]);
    loop {
        let mut changed = false;
        for fk in keys {
            if !out.contains(&fk.parent) {
                continue;
            }
            if !tables.contains_key(&fk.child) {
                return Err(bad(format!(
                    "Constraint {} reaches a table outside the application catalog.",
                    fk.name
                )));
            }
            changed |= out.insert(fk.child);
        }
        if !changed {
            break;
        }
    }
    Ok(out)
}

fn delete_order(nodes: &BTreeSet<i64>, keys: &[ForeignKey]) -> AdminResult<Vec<i64>> {
    fn visit(
        node: i64,
        keys: &[ForeignKey],
        done: &mut BTreeSet<i64>,
        active: &mut BTreeSet<i64>,
        out: &mut Vec<i64>,
    ) -> AdminResult<()> {
        if done.contains(&node) {
            return Ok(());
        }
        if !active.insert(node) {
            return Ok(());
        }
        for fk in keys
            .iter()
            .filter(|f| f.parent == node && !matches!(f.action.as_str(), "n" | "d"))
        {
            if fk.child == node {
                continue;
            }
            if active.contains(&fk.child) && !fk.deferrable {
                return Err(bad(format!(
                    "Cannot safely clear non-deferrable dependency cycle at constraint {}.",
                    fk.name
                )));
            }
            visit(fk.child, keys, done, active, out)?;
        }
        active.remove(&node);
        done.insert(node);
        out.push(node);
        Ok(())
    }
    // Why: Every edge participating in a cycle must be deferrable, not just the
    // back edge.
    for edge in keys
        .iter()
        .filter(|f| !f.deferrable && f.parent != f.child && !matches!(f.action.as_str(), "n" | "d"))
    {
        let mut seen = BTreeSet::new();
        let mut stack = vec![edge.child];
        while let Some(n) = stack.pop() {
            if n == edge.parent {
                return Err(bad(format!(
                    "Non-deferrable dependency cycle at constraint {}.",
                    edge.name
                )));
            }
            if seen.insert(n) {
                stack.extend(
                    keys.iter()
                        .filter(|f| f.parent == n && !matches!(f.action.as_str(), "n" | "d"))
                        .map(|f| f.child),
                );
            }
        }
    }
    let mut done = BTreeSet::new();
    let mut out = Vec::new();
    for n in nodes {
        visit(*n, keys, &mut done, &mut BTreeSet::new(), &mut out)?;
    }
    Ok(out)
}

pub(crate) async fn prepare(
    conn: &mut PgConnection,
    selection: &Selection,
    lock: bool,
    ticket: uuid::Uuid,
) -> AdminResult<PreparedPlan> {
    let (root, columns) = resolve_selection(conn, selection).await?;
    let all: BTreeMap<_, _> = list_tables(conn)
        .await?
        .into_iter()
        .map(|t| (t.oid, t))
        .collect();
    let keys = list_foreign_keys(conn).await?;
    let nodes = reachable(root.oid, &keys, &all)?;
    let keys: Vec<_> = keys
        .into_iter()
        .filter(|f| nodes.contains(&f.parent))
        .collect();
    for fk in keys
        .iter()
        .filter(|f| matches!(f.action.as_str(), "n" | "d"))
    {
        if keys.iter().any(|other| {
            other.parent == fk.child
                && other
                    .parent_columns
                    .iter()
                    .any(|c| fk.child_columns.contains(c))
        }) {
            return Err(bad(format!(
                "Constraint {} updates a referenced key; narrow the selection or clear its dependent table first.",
                fk.name
            )));
        }
    }
    let order = delete_order(&nodes, &keys)?;
    let tables: BTreeMap<_, _> = all
        .into_iter()
        .filter(|(oid, _)| nodes.contains(oid))
        .collect();
    let (mut schema, triggers) =
        clear_materialize::prepare_tables(conn, &tables, &keys, lock).await?;
    clear_materialize::populate(
        conn,
        clear_materialize::RootSelection {
            table: &root,
            columns: &columns,
            selection,
            ticket,
        },
        &tables,
        &keys,
    )
    .await?;
    let effects = clear_materialize::effects(conn, &tables).await?;
    schema.push_str(
        &serde_json::to_string(&triggers)
            .map_err(systemprompt_web_admin_core::error::AdminError::internal)?,
    );
    Ok(PreparedPlan {
        receipt: ClearPlan {
            effects,
            schema_hash: format!("{:x}", Sha256::digest(schema.as_bytes())),
            triggers,
        },
        tables,
        order,
    })
}
