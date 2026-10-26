//! Sidebar marketplace-count context injected into every admin page.
//!
//! `marketplace_context_middleware` attaches a [`MarketplaceContext`] to each
//! request so the layout can show plugin / skill / MCP counts. The counts are
//! resolved from `services/` on disk and cached process-wide for
//! `MARKETPLACE_CACHE_TTL` to keep the filesystem walk off the hot path.
//!
//! A session refused for its own account state keeps that status; only a
//! failure to evaluate entitlement for a real account is `Unavailable`.

use axum::Extension;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;
use systemprompt::identifiers::UserId;
use systemprompt::prelude::PgPool;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::extension_boot::BootServices;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};

pub(super) async fn marketplace_context_middleware(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    BootServices { boot, services }: BootServices,
    mut request: Request,
    next: Next,
) -> AdminResult<Response> {
    let result = async {
        let path = systemprompt_web_admin_core::request::get_services_path(&boot);
        let plugins = systemprompt_web_admin_catalog::repositories::marketplace::plugins::list_plugins_for_user(
            &pool,
            &services,
            &path,
            &user_ctx.user_id,
        )
        .await?;
        Ok::<_, AdminError>(
            systemprompt_web_admin_catalog::repositories::marketplace::plugins::count_visible_items(&plugins),
        )
    }
    .await;
    let counts = match result {
        Ok(counts) => counts,
        Err(error) => return Err(classify_catalog_failure(error, &user_ctx.user_id)),
    };
    let site_url = boot
        .config()
        .api_external_url
        .trim_end_matches('/')
        .to_owned();

    let ctx = MarketplaceContext {
        user_id: user_ctx.user_id.clone(),
        site_url,
        total_plugins: counts.total_plugins,
        total_skills: counts.total_skills,
        agents_count: counts.agents_count,
        mcp_count: counts.mcp_count,
        rank_level: 1,
        rank_name: String::from("Beginner"),
        rank_tier: systemprompt_web_shared::RankTier::Bronze,
        total_xp: 0,
        xp_progress_pct: 0.0,
        has_completed_onboarding: true,
        current_streak: 0,
        longest_streak: 0,
        next_rank_name: String::from("Apprentice"),
        xp_to_next_rank: 100,
    };

    request.extensions_mut().insert(ctx);
    Ok(next.run(request).await)
}

fn classify_catalog_failure(error: AdminError, user_id: &UserId) -> AdminError {
    match error {
        caller_refusal @ (AdminError::Unauthorized(_) | AdminError::Forbidden(_)) => {
            tracing::warn!(error = %caller_refusal, user_id = %user_id, "catalog authorization refused");
            caller_refusal
        },
        error => {
            tracing::error!(%error, user_id = %user_id, "catalog authorization unavailable");
            AdminError::Unavailable("Catalog authorization unavailable".into())
        },
    }
}
