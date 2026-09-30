//! The conversation set as the export surface reads it.

use sqlx::PgPool;
use systemprompt_web_shared::{GroupId, ProjectId};

use systemprompt_web_admin_core::error::AdminResult;

use crate::repositories::analysis::conversations::{
    ConversationAnalysisFilter, ConversationAnalysisPage, ConversationAnalysisResult,
    load_conversation_analysis_page,
};
use systemprompt_web_admin_core::repositories::scope::ScopeRequest;
use systemprompt_web_admin_core::types::UserContext;
use systemprompt_web_admin_core::util::time_range::TimeRange;

use super::query::ConversationAnalysisQuery;

pub(crate) async fn export_rows(
    pool: &PgPool,
    user_ctx: &UserContext,
    params: ConversationAnalysisQuery,
    range: TimeRange,
    limit: i64,
) -> AdminResult<ConversationAnalysisResult> {
    let request =
        ScopeRequest::from_query(user_ctx, params.group.as_deref(), params.project.as_deref());
    let scope = systemprompt_web_admin_core::repositories::scope::membership::get_subject_scope(
        pool, &request,
    )
    .await?;
    let ticked_only = params.context_ids().is_some();
    let filter = ConversationAnalysisFilter {
        subject_ids: scope.as_sql().map(<[String]>::to_vec),
        user_id: params.user_id(),
        category: params.category(),
        outcome: params.outcome(),
        skill: params.skill(),
        free_text: params.free_text(),
        since: (!ticked_only).then_some(range.from),
        until: (!ticked_only).then_some(range.to),
        judged: params.judged(),
        model: params.model(),
        client_kind: params.client(),
        group_id: request.group.as_deref().map(GroupId::new),
        project_id: request.project.as_deref().map(ProjectId::new),
        flag: params.flag(),
        context_ids: params.context_ids(),
        include_without_turns: params.show_all() || ticked_only,
    };
    Ok(load_conversation_analysis_page(
        pool,
        &filter,
        ConversationAnalysisPage {
            sort: params.sort(),
            descending: params.descending(),
            limit,
            offset: 0,
            breakdown: params.breakdown(),
        },
    )
    .await?)
}
