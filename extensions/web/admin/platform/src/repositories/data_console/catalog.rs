//! Application table/column and foreign-key discovery; no user-supplied SQL
//! identifier is used before resolving it against this catalog.
use super::{bad, quote_identifier};
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, Row};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Table {
    pub oid: i64,
    pub schema: String,
    pub name: String,
}
impl Table {
    #[must_use]
    pub fn label(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
    pub(crate) fn sql(&self) -> String {
        format!(
            "{}.{}",
            quote_identifier(&self.schema),
            quote_identifier(&self.name)
        )
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub type_sql: String,
    pub scalar: bool,
    pub primary: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForeignKey {
    pub name: String,
    pub child: i64,
    pub parent: i64,
    pub child_columns: Vec<String>,
    pub parent_columns: Vec<String>,
    pub action: String,
    pub deferrable: bool,
}

pub async fn list_tables(conn: &mut PgConnection) -> Result<Vec<Table>, AdminError> {
    let rows = sqlx::query(
        "SELECT c.oid::bigint AS oid, n.nspname AS schema, c.relname AS name
        FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE c.relkind IN ('r','p') AND NOT c.relispartition
          AND n.nspname <> 'information_schema' AND left(n.nspname,3) <> 'pg_'
        ORDER BY n.nspname, c.relname",
    )
    .fetch_all(conn)
    .await?;
    rows.into_iter()
        .map(|r| {
            Ok(Table {
                oid: r.try_get("oid")?,
                schema: r.try_get("schema")?,
                name: r.try_get("name")?,
            })
        })
        .collect()
}

pub async fn get_table(conn: &mut PgConnection, name: &str) -> AdminResult<Table> {
    list_tables(conn)
        .await?
        .into_iter()
        .find(|t| t.label() == name)
        .ok_or_else(|| bad("Select an application table from the catalog."))
}

pub async fn list_columns(
    conn: &mut PgConnection,
    table: &Table,
) -> Result<Vec<Column>, AdminError> {
    let rows = sqlx::query("SELECT a.attname AS name, format_type(a.atttypid,a.atttypmod) AS type_sql,
        t.typcategory NOT IN ('A','C','P','R') AND t.typname NOT IN ('json','jsonb','bytea') AS scalar,
        COALESCE(array_position(i.indkey::smallint[], a.attnum), -1)::int + 1 AS primary_position
        FROM pg_attribute a JOIN pg_type t ON t.oid = a.atttypid
        LEFT JOIN pg_index i ON i.indrelid = a.attrelid AND i.indisprimary
        WHERE a.attrelid = $1::bigint::oid AND a.attnum > 0 AND NOT a.attisdropped
        ORDER BY a.attnum").bind(table.oid).fetch_all(conn).await?;
    rows.into_iter()
        .map(|r| {
            Ok(Column {
                name: r.try_get("name")?,
                type_sql: r.try_get("type_sql")?,
                scalar: r.try_get("scalar")?,
                primary: r.try_get("primary_position")?,
            })
        })
        .collect()
}

pub async fn list_foreign_keys(conn: &mut PgConnection) -> Result<Vec<ForeignKey>, AdminError> {
    let rows = sqlx::query("SELECT c.conname AS name, c.conrelid::bigint AS child,
        c.confrelid::bigint AS parent, c.confdeltype::text AS action, c.condeferrable AS deferrable,
        ARRAY(SELECT a.attname::text FROM unnest(c.conkey) WITH ORDINALITY k(num,seq)
          JOIN pg_attribute a ON a.attrelid=c.conrelid AND a.attnum=k.num ORDER BY seq) AS child_columns,
        ARRAY(SELECT a.attname::text FROM unnest(c.confkey) WITH ORDINALITY k(num,seq)
          JOIN pg_attribute a ON a.attrelid=c.confrelid AND a.attnum=k.num ORDER BY seq) AS parent_columns
        FROM pg_constraint c WHERE c.contype='f' AND c.conparentid=0 ORDER BY c.oid")
        .fetch_all(conn).await?;
    rows.into_iter()
        .map(|r| {
            Ok(ForeignKey {
                name: r.try_get("name")?,
                child: r.try_get("child")?,
                parent: r.try_get("parent")?,
                child_columns: r.try_get("child_columns")?,
                parent_columns: r.try_get("parent_columns")?,
                action: r.try_get("action")?,
                deferrable: r.try_get("deferrable")?,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatabaseTrigger {
    pub table: String,
    pub name: String,
    pub definition: String,
}

pub async fn list_data_triggers(
    conn: &mut PgConnection,
    table: &Table,
) -> Result<Vec<DatabaseTrigger>, AdminError> {
    let rows = sqlx::query("SELECT tgname::text AS name, pg_get_triggerdef(oid) AS definition FROM pg_trigger WHERE tgrelid=$1::bigint::oid AND NOT tgisinternal AND tgenabled <> 'D' AND (tgtype::int & 24) <> 0 ORDER BY tgname")
        .bind(table.oid).fetch_all(conn).await?;
    rows.into_iter()
        .map(|row| {
            Ok(DatabaseTrigger {
                table: table.label(),
                name: row.try_get("name")?,
                definition: row.try_get("definition")?,
            })
        })
        .collect()
}
