//! Explicit POST actions, gated before any database access.
use super::view::DataPage;
use super::{render, require_admin};
use crate::repositories::data_console::browse::Selection;
use crate::repositories::data_console::{clear, query};
use axum::extract::{Extension, Form, State};
use axum::http::HeaderMap;
use axum::response::Response;
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_admin_core::request::require_write_origin;
use systemprompt_web_admin_core::ssr::page::Page;

#[derive(Debug, Deserialize)]
pub(crate) struct QueryForm {
    query: String,
}
#[derive(Debug, Deserialize)]
pub(crate) struct PreviewForm {
    selection: String,
    mode: String,
    source: String,
}
#[derive(Debug, Deserialize)]
pub(crate) struct ConfirmForm {
    token: String,
    confirmation: String,
    acknowledge: Option<String>,
    source: String,
}

pub(crate) async fn run_query(
    shell: Page,
    Extension(boot): Extension<ExtensionBoot>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Form(form): Form<QueryForm>,
) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    require_write_origin(&boot, &headers)?;
    let mut data = DataPage::new(false);
    data.query = form.query.clone();
    match query::get_query_result(&pool, &form.query).await {
        Ok(result) => {
            data.notice = Some(format!(
                "Query completed: {} rows displayed.",
                result.rows.len()
            ));
            data.query_rows = result.rows;
            data.limited = result.limited;
        },
        Err(error) => {
            tracing::warn!(%error,"Platform SQL query failed");
            data.notice = Some(user_error(&error));
        },
    }
    Ok(render(&shell, &data))
}

pub(crate) async fn clear_preview(
    shell: Page,
    Extension(boot): Extension<ExtensionBoot>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Form(form): Form<PreviewForm>,
) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    require_write_origin(&boot, &headers)?;
    if form.selection.len() > 16_384 {
        return Err(AdminError::BadRequest("Selection is too large.".into()).into());
    }
    let mut selection: Selection = serde_json::from_str(&form.selection)
        .map_err(|_error| AdminError::BadRequest("Invalid clearing selection.".into()))?;
    let logs = form.source == "logs";
    if logs && selection.table != "public.logs" {
        return Err(AdminError::BadRequest("Logs clearing must target public.logs.".into()).into());
    }
    if form.mode == "all" {
        selection.filters.clear();
        selection.from = None;
        if logs {
            selection.time_column = Some("timestamp".into());
            selection.until = Some(Utc::now());
        } else {
            selection.time_column = None;
            selection.until = None;
        }
    } else if form.mode != "matching" {
        return Err(AdminError::BadRequest("Unknown clearing mode.".into()).into());
    }
    let mut data = DataPage::new(logs);
    data.selection(&selection).map_err(AdminError::internal)?;
    match clear::insert_clear_preview(&pool, shell.user.user_id.as_str(), &selection).await {
        Ok(preview) => data.preview = Some(preview),
        Err(error) => {
            tracing::warn!(%error,"Platform clearing preview failed");
            data.notice = Some(user_error(&error));
        },
    }
    Ok(render(&shell, &data))
}

pub(crate) async fn clear_confirm(
    shell: Page,
    Extension(boot): Extension<ExtensionBoot>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Form(form): Form<ConfirmForm>,
) -> AdminHtmlResult<Response> {
    require_admin(&shell)?;
    require_write_origin(&boot, &headers)?;
    if form.acknowledge.as_deref() != Some("yes") {
        return Err(AdminError::BadRequest(
            "Acknowledge irreversible deletion and related-table effects.".into(),
        )
        .into());
    }
    let mut data = DataPage::new(form.source == "logs");
    match clear::delete_confirmed_rows(
        &pool,
        shell.user.user_id.as_str(),
        &form.token,
        &form.confirmation,
    )
    .await
    {
        Ok(result) => {
            data.notice = Some("Clearing completed. The table below records affected rows; a deletion receipt was saved.".into());
            data.cleared = Some(result);
        },
        Err(error) => {
            tracing::warn!(%error,"Platform clearing failed");
            data.notice = Some(user_error(&error));
        },
    }
    Ok(render(&shell, &data))
}
fn user_error(error: &AdminError) -> String {
    match error {
        AdminError::BadRequest(message) | AdminError::Conflict(message) => message.clone(),
        AdminError::Database(sqlx::Error::Database(error)) => format!("Database refused the operation ({}): {}. No clearing changes were committed.", error.code().unwrap_or_default(), error.message()),
        _ => "The operation could not complete within its query/lock limits or database constraints. No clearing changes were committed. Narrow the selection and try again.".into(),
    }
}
