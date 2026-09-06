//! Loading the contexts list: one page of contexts under the caller's scope,
//! the by-user rollup beside it, and the facets the filter form offers.
//!
//! Every read degrades to an empty result rather than failing the page: a
//! listing that loses its model facet is still worth rendering, and the count
//! above the table is the paged query's own so the two cannot disagree.

use sqlx::PgPool;

use crate::repositories;
use crate::repositories::analytics::contexts_list;
use crate::repositories::analytics::contexts_list::ContextPage;
use crate::repositories::scope::SubjectScope;

use super::{ContextsPageInputs, PAGE_SIZE, view};

pub(super) struct ContextsPageData {
    pub(super) contexts: Vec<contexts_list::ContextListItem>,
    pub(super) total_contexts: i64,
    pub(super) user_summaries: Vec<contexts_list::ContextUserSummary>,
    pub(super) kpis: contexts_list::ContextListKpis,
    pub(super) models: Vec<String>,
    pub(super) users_for_filter: Vec<crate::types::UserSummary>,
}

pub(super) async fn load_page_data(
    pool: &PgPool,
    inputs: &ContextsPageInputs,
    user_scope: &SubjectScope,
) -> ContextsPageData {
    let filter = &inputs.filter;
    let context_page = ContextPage {
        sort: inputs.sort,
        limit: PAGE_SIZE,
        offset: view::page_offset(inputs.page),
    };
    let (contexts, total_contexts) =
        contexts_list::list_context_list_paged(pool, filter, context_page)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "list_context_list_paged failed");
                (Vec::new(), 0)
            });
    let user_summaries = contexts_list::list_context_user_summary(pool, filter)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "list_context_user_summary failed");
            Vec::new()
        });
    let kpis = contexts_list::get_context_list_kpis(pool, filter)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "get_context_list_kpis failed");
            contexts_list::ContextListKpis {
                total_contexts: 0,
                active_users: 0,
                total_requests: 0,
                total_messages: 0,
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_cost_microdollars: 0,
            }
        });
    let models = contexts_list::list_distinct_models(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "list_distinct_models failed");
            Vec::new()
        });
    let users_for_filter = repositories::users::queries::list_users(pool, user_scope)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "list_users failed in contexts page");
            Vec::new()
        });
    ContextsPageData {
        contexts,
        total_contexts,
        user_summaries,
        kpis,
        models,
        users_for_filter,
    }
}
