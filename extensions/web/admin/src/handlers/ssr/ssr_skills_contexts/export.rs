//! Activity by person as the export surface reads it.

use crate::error::AdminResult;
use crate::export::datasets::requests::time_range;
use crate::export::model::ExportContext;
use crate::repositories;
use crate::repositories::analytics::conversation_rows::{
    ConversationPage, ConversationPageMode, ConversationPageResult, load_conversation_page,
};
use crate::repositories::scope::ScopeRequest;

use super::{ContextsListQuery, parse_inputs};

// Why: this page's `since` labels reach past the live window contract (90d,
// all); the export offers the nearest window it can serve.
pub(super) fn export_preset(since: Option<&str>) -> &'static str {
    match since {
        Some("24h") => "24h",
        Some("7d") => "7d",
        _ => "30d",
    }
}

// Why: the export reads the same filtered aggregate the page shows, in either
// view — one row per conversation, or one per person.
pub(crate) async fn export_rows(
    ctx: &ExportContext<'_>,
    mode: ConversationPageMode,
) -> AdminResult<ConversationPageResult> {
    let params: ContextsListQuery = ctx.query()?;
    let range = time_range(ctx)?;
    let request =
        ScopeRequest::from_query(ctx.user, params.group.as_deref(), params.project.as_deref());
    let scope = repositories::scope::membership::get_subject_scope(ctx.pool, &request).await?;
    let mut inputs = parse_inputs(&params, &scope);
    inputs.filter.since = Some(range.from);
    inputs.filter.until = Some(range.to);
    let page = ConversationPage {
        sort: inputs.sort,
        descending: inputs.descending,
        limit: ctx.limit,
        offset: 0,
    };
    Ok(load_conversation_page(ctx.pool, &inputs.filter, page, mode).await?)
}
