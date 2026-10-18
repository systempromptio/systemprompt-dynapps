//! Platform Logs and SQL: bounded reads with explicit, previewed deletions.
mod actions;
mod selection;
mod view;

use crate::repositories::data_console::browse::{self, Selection};
use crate::repositories::data_console::{catalog, transaction};
pub(crate) use actions::{clear_confirm, clear_preview, run_query};
use axum::extract::{Query, State};
use axum::response::Response;
use chrono::Utc;
use selection::{selection, url};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult, AdminResult};
use systemprompt_web_admin_core::ssr::page::Page;
use view::{Cell, DataPage, DetailField, Row};

#[derive(Debug, Default, Deserialize)]
pub(crate) struct BrowseQuery {
    table: Option<String>,
    cursor: Option<String>,
    key: Option<String>,
    from: Option<String>,
    until: Option<String>,
    time_column: Option<String>,
    column: Option<String>,
    value: Option<String>,
    level: Option<String>,
    module: Option<String>,
    #[serde(rename = "context_id")]
    context_filter: Option<String>,
    #[serde(rename = "session_id")]
    session_filter: Option<String>,
    trace_id: Option<String>,
}

fn require_admin(shell: &Page) -> AdminResult<()> {
    if shell.user.is_platform_admin {
        Ok(())
    } else {
        Err(AdminError::Forbidden(
            "Platform administrator access required.".into(),
        ))
    }
}
fn render(shell: &Page, data: &DataPage) -> Response {
    systemprompt_web_admin_core::ssr::helpers::render_typed_page(
        &shell.engine,
        "data-console",
        data,
        &shell.user,
        &shell.marketplace,
    )
}
async fn load_page(pool: &PgPool, query: &BrowseQuery, logs: bool) -> AdminResult<DataPage> {
    let mut data = DataPage::new(logs);
    let s = selection(query, logs)?;
    data.selection(&s).map_err(AdminError::internal)?;
    let mut tx = transaction(pool, "3s").await?;
    data.tables(catalog::list_tables(&mut tx).await?);
    tx.rollback().await?;
    if s.table.is_empty() {
        return Ok(data);
    }
    data.from = s.from.map_or_else(String::new, |d| d.to_rfc3339());
    data.until = s.until.map_or_else(String::new, |d| d.to_rfc3339());
    data.time_column = s.time_column.clone().unwrap_or_default();
    data.level = query.level.clone().unwrap_or_default();
    data.module = query.module.clone().unwrap_or_default();
    data.context_filter = query.context_filter.clone().unwrap_or_default();
    data.session_filter = query.session_filter.clone().unwrap_or_default();
    data.trace_id = query.trace_id.clone().unwrap_or_default();
    data.filter_column = query.column.clone().unwrap_or_default();
    data.filter_value = query.value.clone().unwrap_or_default();
    let mut refreshed = s.clone();
    if logs {
        let now = Utc::now();
        refreshed.from = Some(now - (s.until.unwrap_or(now) - s.from.unwrap_or(now)));
        refreshed.until = Some(now);
    }
    data.refresh_url = url(&refreshed, logs, None);
    if let Some(key) = &query.key {
        let detail = browse::find_row_detail(pool, &s.table, key)
            .await?
            .ok_or_else(|| AdminError::NotFound("This row no longer exists.".into()))?;
        if let Some(fields) = detail.as_object() {
            data.detail = fields
                .iter()
                .map(|(name, value)| {
                    let value = value
                        .as_str()
                        .map_or_else(|| value.to_string(), String::from);
                    let limited = value.ends_with(" [display truncated]");
                    DetailField {
                        name: name.clone(),
                        value,
                        limited,
                    }
                })
                .collect();
        }
        return Ok(data);
    }
    let page = browse::get_table_page(pool, &s, query.cursor.as_deref()).await?;
    data.no_primary = !page.columns.iter().any(|c| c.primary > 0);
    data.limited = page.limited;
    let names: Vec<_> = if logs {
        [
            "timestamp",
            "level",
            "module",
            "message",
            "instance_id",
            "context_id",
            "session_id",
            "trace_id",
        ]
        .iter()
        .map(|s| (*s).into())
        .collect()
    } else {
        page.columns.iter().map(|c| c.name.clone()).collect()
    };
    data.rows = table_rows(&page, &names, &s, logs, data.no_primary);
    data.columns = names;
    data.next_url = page.next.as_deref().map(|c| url(&s, logs, Some(c)));
    Ok(data)
}

fn table_rows(
    page: &browse::BrowsePage,
    names: &[String],
    selection: &Selection,
    logs: bool,
    no_primary: bool,
) -> Vec<Row> {
    page.rows
        .iter()
        .zip(&page.row_keys)
        .map(|(row, key)| Row {
            cells: names
                .iter()
                .map(|name| {
                    // JSON: Rows have catalog-selected, variable column names.
                    let value = row
                        .get(name)
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("NULL")
                        .to_owned();
                    let href = if value == "NULL" {
                        None
                    } else {
                        match name.as_str() {
                            "context_id" => {
                                Some(format!("/admin/contexts/{}", urlencoding::encode(&value)))
                            },
                            "trace_id" => {
                                Some(format!("/admin/traces/{}", urlencoding::encode(&value)))
                            },
                            _ => None,
                        }
                    };
                    Cell {
                        limited: value.ends_with(" [display truncated]"),
                        value,
                        href,
                    }
                })
                .collect(),
            detail: if no_primary {
                None
            } else {
                Some(format!(
                    "{}&key={}",
                    url(selection, logs, None),
                    urlencoding::encode(key)
                ))
            },
        })
        .collect()
}

pub(crate) async fn logs_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<BrowseQuery>,
) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    let data = load_page(&pool, &query, true).await?;
    Ok(render(&shell, &data))
}
pub(crate) async fn sql_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<BrowseQuery>,
) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    let data = load_page(&pool, &query, false).await?;
    Ok(render(&shell, &data))
}
