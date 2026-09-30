//! The trace list as the export surface reads it.

use sqlx::PgPool;

use crate::repositories::traces::{TracePage, TraceSummary, list_traces};
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::repositories::scope::ScopeRequest;
use systemprompt_web_admin_core::types::UserContext;
use systemprompt_web_admin_core::util::time_range::TimeRange;

use super::{TraceListQuery, build_filter, sort_from_query};

pub(crate) async fn export_rows(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: TraceListQuery,
    range: TimeRange,
    limit: i64,
) -> AdminResult<(Vec<TraceSummary>, i64)> {
    let request =
        ScopeRequest::from_query(user_ctx, query.group.as_deref(), query.project.as_deref());
    let subjects = systemprompt_web_admin_core::repositories::scope::membership::get_subject_scope(
        pool, &request,
    )
    .await?;
    let filter = build_filter(&query, subjects.as_sql());
    let page = TracePage {
        sort: sort_from_query(&query),
        limit,
        offset: 0,
    };
    Ok(list_traces(pool, filter, range, page).await?)
}
