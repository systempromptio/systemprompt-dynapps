//! SSR page for a user's own profile.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::services::bridge_profile::{self, BridgeProfilePageData};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

// Why: the bridge payload is a wire shape shared with the bridge GUI, so the
// trail the console draws around it is added here rather than grown onto it.
#[derive(Debug, Serialize)]
struct ProfilePageView {
    #[serde(flatten)]
    data: BridgeProfilePageData,
    breadcrumbs: Vec<BreadcrumbView>,
    // Why: handlebars-rust has no `.length`, so every count a section header
    // shows has to be counted here rather than in the template.
    models_count: usize,
}

pub(crate) async fn profile_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    let data = bridge_profile::build_bridge_profile_data(pool, &user_ctx).await?;
    let view = ProfilePageView {
        models_count: data.usage.top_models.len(),
        data,
        breadcrumbs: vec![
            BreadcrumbView::link("Account", "/admin/profile"),
            BreadcrumbView::current("Profile"),
        ],
    };
    Ok(render_typed_page(
        &engine, "profile", &view, &user_ctx, &mkt_ctx,
    ))
}

// Why: scoped to the caller and nobody else — the code is minted for
// `user_ctx.user_id` from the validated session, so there is no target-user
// parameter to tamper with. Redeeming it yields a durable PAT signing in as
// that user, so it is issued only on an explicit request.
pub(crate) async fn issue_bridge_code(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> AdminResult<Response> {
    let (_, gateway_url) = bridge_profile::read_config_strings();
    let block = bridge_profile::issue_bridge_connect(&pool, &user_ctx, gateway_url.as_deref())
        .await
        // Why: Unavailable, not Internal — the usual cause is no configured
        // gateway URL, a deployment state rather than a server fault, and the
        // caller can sensibly retry once one is set.
        .ok_or_else(|| {
            AdminError::Unavailable("Could not mint a connect code just now.".to_owned())
        })?;
    Ok(Json(block).into_response())
}
