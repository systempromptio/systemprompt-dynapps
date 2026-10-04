//! Transaction-local row closure and schema evidence for deletion previews.
use super::browse::{Selection, predicate};
use super::catalog::{
    Column, DatabaseTrigger, ForeignKey, Table, list_columns, list_data_triggers, list_foreign_keys,
};
use super::clear_plan::{Effect, temp};
use super::{bad, quote_identifier};
use sqlx::{PgConnection, Postgres, QueryBuilder, Row};
use std::collections::BTreeMap;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub(super) async fn prepare_tables(
    conn: &mut PgConnection,
    tables: &BTreeMap<i64, Table>,
    keys: &[ForeignKey],
    lock: bool,
) -> AdminResult<(String, Vec<DatabaseTrigger>)> {
    let mut triggers = Vec::new();
    let mut schema = serde_json::to_string(keys).map_err(AdminError::internal)?;
    for table in tables.values() {
        if lock {
            sqlx::query(sqlx::AssertSqlSafe(format!(
                "LOCK TABLE {} IN SHARE ROW EXCLUSIVE MODE",
                table.sql()
            )))
            .execute(&mut *conn)
            .await?;
        }
        triggers.extend(list_data_triggers(conn, table).await?);
        schema.push_str(
            &serde_json::to_string(&list_columns(conn, table).await?)
                .map_err(AdminError::internal)?,
        );
        sqlx::query(sqlx::AssertSqlSafe(format!("CREATE TEMP TABLE sp_clear_{} (rel oid NOT NULL, tid tid NOT NULL, deleting boolean NOT NULL, PRIMARY KEY(rel,tid)) ON COMMIT DROP", table.oid)))
            .execute(&mut *conn).await?;
    }
    if lock {
        let locked_keys: Vec<_> = list_foreign_keys(conn)
            .await?
            .into_iter()
            .filter(|key| tables.contains_key(&key.parent))
            .collect();
        if locked_keys != keys {
            return Err(bad(
                "Database dependencies changed while locks were acquired. Create a fresh preview.",
            ));
        }
    }
    Ok((schema, triggers))
}

#[derive(Debug, Clone, Copy)]
pub(super) struct RootSelection<'a> {
    pub(super) table: &'a Table,
    pub(super) columns: &'a [Column],
    pub(super) selection: &'a Selection,
    pub(super) ticket: uuid::Uuid,
}

pub(super) async fn populate(
    conn: &mut PgConnection,
    root: RootSelection<'_>,
    tables: &BTreeMap<i64, Table>,
    keys: &[ForeignKey],
) -> AdminResult<()> {
    let mut q = QueryBuilder::<Postgres>::new(format!(
        "INSERT INTO {} SELECT t.tableoid,t.ctid,true FROM {} t WHERE",
        temp(root.table.oid),
        root.table.sql()
    ));
    predicate(&mut q, root.selection, root.columns, "t")?;
    // Why: Postgres stores the active ticket in a clearable table; excluding it
    // keeps the confirmation available until its transaction completes.
    if root.table.label() == "public.platform_data_clear_previews" {
        q.push(" AND t.id <> ").push_bind(root.ticket);
    }
    q.build().execute(&mut *conn).await?;
    loop {
        let mut changed = 0;
        for fk in keys {
            let child = &tables[&fk.child];
            let parent = &tables[&fk.parent];
            let joins = fk
                .child_columns
                .iter()
                .zip(&fk.parent_columns)
                .map(|(c, p)| format!("c.{} = p.{}", quote_identifier(c), quote_identifier(p)))
                .collect::<Vec<_>>()
                .join(" AND ");
            let deleting = !matches!(fk.action.as_str(), "n" | "d");
            let sql = format!("INSERT INTO {} AS existing SELECT DISTINCT c.tableoid,c.ctid,{deleting}
                FROM {} c JOIN {} p ON {joins} JOIN {} picked ON picked.rel=p.tableoid AND picked.tid=p.ctid AND picked.deleting
                ON CONFLICT(rel,tid) DO UPDATE SET deleting=true WHERE NOT existing.deleting AND EXCLUDED.deleting",
                temp(child.oid), child.sql(), parent.sql(), temp(parent.oid));
            changed += sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
                .execute(&mut *conn)
                .await?
                .rows_affected();
        }
        if changed == 0 {
            break;
        }
    }
    Ok(())
}

pub(super) async fn effects(
    conn: &mut PgConnection,
    tables: &BTreeMap<i64, Table>,
) -> AdminResult<Vec<Effect>> {
    let mut effects = Vec::new();
    for table in tables.values() {
        let sql = format!("SELECT COUNT(*) FILTER(WHERE x.deleting)::bigint AS deleted,
            COUNT(*) FILTER(WHERE NOT x.deleting)::bigint AS updated,
            COALESCE(bit_xor(hashtextextended(t.tableoid::text || ':' || t.ctid::text || ':' || row_to_json(t)::text || ':' || x.deleting::text,0)),0)::text AS fingerprint
            FROM {} t JOIN {} x ON x.rel=t.tableoid AND x.tid=t.ctid", table.sql(), temp(table.oid));
        let row = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
            .fetch_one(&mut *conn)
            .await?;
        effects.push(Effect {
            table: table.label(),
            deleted: row.try_get("deleted")?,
            updated: row.try_get("updated")?,
            fingerprint: row.try_get("fingerprint")?,
        });
    }
    Ok(effects)
}
