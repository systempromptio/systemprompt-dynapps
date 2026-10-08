//! Bounded catalog-driven reads and typed selection predicates shared by
//! browsing and clearing. Cursors contain only primary-key values.
use super::catalog::{Column, Table, get_table, list_columns};
use super::{bad, quote_identifier, read_only, transaction};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{PgConnection, PgPool, Postgres, QueryBuilder, Row};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Filter {
    pub column: String,
    pub value: String,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Selection {
    pub table: String,
    pub filters: Vec<Filter>,
    pub time_column: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

pub(crate) fn predicate(
    q: &mut QueryBuilder<Postgres>,
    selection: &Selection,
    columns: &[Column],
    alias: &str,
) -> AdminResult<()> {
    if selection.filters.len() > 8 {
        return Err(bad("Use at most eight filters."));
    }
    q.push(" TRUE ");
    for filter in &selection.filters {
        let column = columns
            .iter()
            .find(|c| c.name == filter.column && c.scalar)
            .ok_or_else(|| bad("Equality filters require a scalar column from this table."))?;
        if filter.value.len() > 4096 {
            return Err(bad("Filter values are limited to 4 KiB."));
        }
        q.push(format!(
            " AND {alias}.{} = CAST(",
            quote_identifier(&column.name)
        ))
        .push_bind(filter.value.clone())
        .push(format!(" AS {})", column.type_sql));
    }
    if selection.from.is_some() || selection.until.is_some() {
        let column = columns
            .iter()
            .find(|c| {
                Some(&c.name) == selection.time_column.as_ref()
                    && c.type_sql.starts_with("timestamp")
            })
            .ok_or_else(|| bad("Select a timestamp column for time filters."))?;
        let name = format!("{alias}.{}", quote_identifier(&column.name));
        if let Some(from) = selection.from {
            q.push(format!(" AND {name} >= ")).push_bind(from);
        }
        if let Some(until) = selection.until {
            q.push(format!(" AND {name} <= ")).push_bind(until);
        }
        if selection
            .from
            .zip(selection.until)
            .is_some_and(|(a, b)| a > b)
        {
            return Err(bad("The start must precede the end."));
        }
    }
    Ok(())
}

pub(crate) fn primary_columns(columns: &[Column]) -> Vec<&Column> {
    let mut keys: Vec<_> = columns.iter().filter(|c| c.primary > 0).collect();
    keys.sort_by_key(|c| c.primary);
    keys
}

// JSON: Cursors carry catalog-selected primary-key columns.
fn cursor_value(cursor: &str) -> AdminResult<serde_json::Value> {
    if cursor.len() > 65_536 {
        return Err(bad("Invalid cursor."));
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_error| bad("Invalid cursor."))?;
    serde_json::from_slice(&bytes).map_err(|_error| bad("Invalid cursor."))
}

// JSON: The selected application table defines the row shape at runtime.
#[derive(Debug)]
pub struct BrowsePage {
    pub table: Table,
    pub columns: Vec<Column>,
    pub rows: Vec<serde_json::Value>,
    pub row_keys: Vec<String>,
    pub next: Option<String>,
    pub limited: bool,
}

// JSON: Cursor fields are primary keys selected from the runtime table catalog.
fn append_page_cursor(
    q: &mut QueryBuilder<Postgres>,
    table: &Table,
    order: &[&str],
    cursor: Option<serde_json::Value>,
    logs: bool,
) -> AdminResult<()> {
    if let Some(value) = cursor {
        if !order.iter().all(|k| value.get(k).is_some()) {
            return Err(bad("Cursor does not match this table."));
        }
        let left = order
            .iter()
            .map(|k| format!("t.{}", quote_identifier(k)))
            .collect::<Vec<_>>()
            .join(",");
        let right = order
            .iter()
            .map(|k| format!("c.{}", quote_identifier(k)))
            .collect::<Vec<_>>()
            .join(",");
        q.push(format!(
            " AND ({left}) {} (SELECT {right} FROM jsonb_populate_record(NULL::{}, ",
            if logs { "<" } else { ">" },
            table.sql()
        ))
        .push_bind(value)
        .push(") c)");
    }
    if !order.is_empty() {
        q.push(" ORDER BY ").push(
            order
                .iter()
                .map(|k| {
                    format!(
                        "t.{} {}",
                        quote_identifier(k),
                        if logs { "DESC" } else { "ASC" }
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    Ok(())
}

pub async fn get_table_page(
    pool: &PgPool,
    selection: &Selection,
    cursor: Option<&str>,
) -> AdminResult<BrowsePage> {
    let mut tx = transaction(pool, "3s").await?;
    read_only(&mut tx).await?;
    let table = get_table(&mut tx, &selection.table).await?;
    let columns = list_columns(&mut tx, &table).await?;
    let keys = primary_columns(&columns);
    let logs = table.schema == "public" && table.name == "logs";
    let order: Vec<_> = if logs {
        vec!["timestamp", "id"]
    } else {
        keys.iter().map(|c| c.name.as_str()).collect()
    };
    let cursor = cursor.map(cursor_value).transpose()?;
    if cursor.is_some() && order.is_empty() {
        return Err(bad(
            "This table has no primary key; pagination is unavailable.",
        ));
    }
    let projection = super::projection::fields(&columns, (16_384 / columns.len().max(1)).min(1000));
    let key_sql = order
        .iter()
        .map(|name| format!("{}, to_jsonb(t.{})", literal(name), quote_identifier(name)))
        .collect::<Vec<_>>()
        .join(",");
    let mut q = QueryBuilder::new(format!(
        "SELECT ({projection}) AS body, jsonb_build_object({key_sql}) AS key FROM {} t WHERE",
        table.sql()
    ));
    predicate(&mut q, selection, &columns, "t")?;
    append_page_cursor(&mut q, &table, &order, cursor, logs)?;
    q.push(" LIMIT 51");
    let mut result = BrowsePage {
        table: table.clone(),
        columns: columns.clone(),
        rows: Vec::new(),
        row_keys: Vec::new(),
        next: None,
        limited: order.is_empty(),
    };
    let mut bytes = 0;
    {
        let mut stream = q.build().fetch(&mut *tx);
        while let Some(row) = stream.try_next().await.map_err(super::read_error)? {
            // JSON: Database projections and keysets have runtime-selected columns.
            let body: serde_json::Value = row.try_get("body")?;
            let key: serde_json::Value = row.try_get("key")?;
            if result.rows.len() == 50 || bytes + body.to_string().len() > 1_048_576 {
                result.limited = true;
                if !order.is_empty() {
                    result.next = result.row_keys.last().cloned();
                }
                break;
            }
            bytes += body.to_string().len();
            result.rows.push(body);
            result
                .row_keys
                .push(URL_SAFE_NO_PAD.encode(key.to_string()));
        }
    }
    tx.rollback().await?;
    Ok(result)
}

// JSON: Detail fields follow the selected table catalog, not a fixed schema.
pub async fn find_row_detail(
    pool: &PgPool,
    name: &str,
    key: &str,
) -> Result<Option<serde_json::Value>, AdminError> {
    let mut tx = transaction(pool, "3s").await?;
    read_only(&mut tx).await?;
    let table = get_table(&mut tx, name).await?;
    let columns = list_columns(&mut tx, &table).await?;
    let keys = primary_columns(&columns);
    let value = cursor_value(key)?;
    if keys.is_empty() || !keys.iter().all(|c| value.get(&c.name).is_some()) {
        return Err(bad("A primary-key row identifier is required."));
    }
    // Why: Limit the aggregate response to 1 MiB even for very wide tables.
    let per_field = (1_048_576 / columns.len().max(1)).min(65_536);
    let projection = super::projection::fields(&columns, per_field);
    let joins = keys
        .iter()
        .map(|c| {
            format!(
                "t.{} = k.{}",
                quote_identifier(&c.name),
                quote_identifier(&c.name)
            )
        })
        .collect::<Vec<_>>()
        .join(" AND ");
    let sql = format!(
        "SELECT ({projection}) AS body FROM {} t, jsonb_populate_record(NULL::{}, $1) k WHERE {joins} LIMIT 1",
        table.sql(),
        table.sql()
    );
    let row = sqlx::query(sqlx::AssertSqlSafe(sql.as_str()))
        .bind(value)
        .fetch_optional(&mut *tx)
        .await
        .map_err(super::read_error)?;
    let result = row.map(|r| r.try_get("body")).transpose()?;
    tx.rollback().await?;
    Ok(result)
}

pub(crate) fn literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(crate) async fn resolve_selection(
    conn: &mut PgConnection,
    selection: &Selection,
) -> AdminResult<(Table, Vec<Column>)> {
    let table = get_table(conn, &selection.table).await?;
    let columns = list_columns(conn, &table).await?;
    // Why: Validate even when no rows match.
    predicate(&mut QueryBuilder::new(""), selection, &columns, "t")?;
    Ok((table, columns))
}
