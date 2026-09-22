//! Which conversations a JSON Lines export covers: a ticked selection, every
//! context of one session, or the set a list page is showing — resolved
//! through that page's own query type, so the file answers the question the
//! page was asked.

use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, SessionId};

use crate::error::{AdminError, AdminResult};
use crate::export::model::{ExportContext, Window};
use crate::export::window;
use crate::repositories::analytics::session_detail::list_session_contexts;
use crate::types::UserContext;
use crate::util::time_range::{TimeRange, TimeRangePreset};

// Why: a bundle carries every body of a conversation; five hundred of them is
// already a file in the tens of megabytes, and the page's filters narrow the
// set further before anyone reaches that.
pub(crate) const MAX_CONVERSATIONS: i64 = 500;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SelectionQuery {
    source: Option<String>,
    ids: Option<String>,
    #[serde(rename = "session_id")]
    session: Option<String>,
}

pub(crate) struct Selection {
    pub context_ids: Vec<ContextId>,
    pub total: i64,
}

impl Selection {
    pub(crate) fn capped(&self) -> bool {
        self.total > i64::try_from(self.context_ids.len()).unwrap_or(i64::MAX)
    }
}

pub(crate) async fn resolve_selection(
    pool: &PgPool,
    user: &UserContext,
    ctx: &ExportContext<'_>,
) -> AdminResult<Selection> {
    let query: SelectionQuery = ctx.query()?;
    if let Some(ids) = query.ids.as_deref().filter(|s| !s.trim().is_empty()) {
        return Ok(ticked(ids));
    }
    if let Some(session) = query.session.as_deref().filter(|s| !s.trim().is_empty()) {
        return session_contexts(pool, session).await;
    }
    let range = live_range(ctx)?;
    match query.source.as_deref().unwrap_or("sessions") {
        "sessions" => sessions_set(pool, user, ctx, range).await,
        "analysis" => analysis_set(pool, user, ctx, range).await,
        other => Err(AdminError::BadRequest(format!(
            "Unknown conversation source `{other}`; use sessions or analysis"
        ))),
    }
}

fn ticked(ids: &str) -> Selection {
    let context_ids: Vec<ContextId> = ids
        .split(',')
        .map(str::trim)
        .filter_map(|s| ContextId::try_new(s).ok())
        .take(usize::try_from(MAX_CONVERSATIONS).unwrap_or(usize::MAX))
        .collect();
    let total =
        i64::try_from(ids.split(',').filter(|s| !s.trim().is_empty()).count()).unwrap_or(i64::MAX);
    Selection { context_ids, total }
}

async fn session_contexts(pool: &PgPool, session: &str) -> AdminResult<Selection> {
    let session_id = SessionId::new(session.trim());
    let rows = list_session_contexts(pool, &session_id).await?;
    let total = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    Ok(Selection {
        context_ids: rows
            .into_iter()
            .map(|r| r.context_id)
            .take(usize::try_from(MAX_CONVERSATIONS).unwrap_or(usize::MAX))
            .collect(),
        total,
    })
}

fn live_range(ctx: &ExportContext<'_>) -> AdminResult<TimeRange> {
    let w = window::resolve(Window::Live, &ctx.query()?)?
        .ok_or_else(|| AdminError::BadRequest("This export needs a window".to_owned()))?;
    Ok(TimeRange {
        from: w.from,
        to: w.to,
        preset: TimeRangePreset::Custom,
        rejected_bounds: false,
    })
}

async fn sessions_set(
    pool: &PgPool,
    user: &UserContext,
    ctx: &ExportContext<'_>,
    range: TimeRange,
) -> AdminResult<Selection> {
    let (rows, total) = crate::handlers::ssr::ssr_sessions_list::export::export_rows(
        pool,
        user,
        ctx.query()?,
        range,
        MAX_CONVERSATIONS,
    )
    .await?;
    Ok(Selection {
        context_ids: rows.into_iter().map(|r| r.context_id).collect(),
        total,
    })
}

async fn analysis_set(
    pool: &PgPool,
    user: &UserContext,
    ctx: &ExportContext<'_>,
    range: TimeRange,
) -> AdminResult<Selection> {
    let result = crate::handlers::ssr::analysis::conversations::export::export_rows(
        pool,
        user,
        ctx.query()?,
        range,
        MAX_CONVERSATIONS,
    )
    .await?;
    Ok(Selection {
        total: result.totals.conversations,
        context_ids: result.rows.into_iter().map(|r| r.context_id).collect(),
    })
}
