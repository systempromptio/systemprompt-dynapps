//! SSR page for a user's own profile.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::ContextId;

use crate::repositories;
use crate::services::bridge_profile::{self, BridgeProfilePageData};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult, AdminResult};
use systemprompt_web_admin_core::extension_boot::BootServices;
use systemprompt_web_admin_core::repositories::user_usage::{
    ConversationSummary, RecentConversation,
};
use systemprompt_web_admin_core::ssr::format::{format_cost, relative_time};
use systemprompt_web_admin_core::ssr::helpers::render_typed_page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};

// Why: the bridge GUI shares `BridgeProfilePageData` as a wire shape, so the
// console's extras wrap it here; fields may be added to it, never removed.
#[derive(Debug, Serialize)]
struct ProfilePageView {
    #[serde(flatten)]
    data: BridgeProfilePageData,
    breadcrumbs: Vec<BreadcrumbView>,
    // Why: handlebars-rust has no `.length`, so section counts are fields.
    models_count: usize,
    latest_conversation: Option<ProfileConversationView>,
    recent_conversations: Vec<ProfileConversationView>,
    recent_count: usize,
    has_recent: bool,
    side_call_note: Option<String>,
    settings: SettingsView,
    username: String,
}

#[derive(Debug, Clone, Serialize)]
struct SettingsView {
    display_name: Option<String>,
    avatar_url: Option<String>,
    timezone: String,
}

#[derive(Debug, Serialize)]
struct ProfileConversationView {
    conversation_title: String,
    url: String,
    context_id: ContextId,
    model: String,
    turns: i64,
    cost_display: String,
    last_relative: String,
    last_at: String,
}

fn conversation_view(c: &RecentConversation) -> ProfileConversationView {
    ProfileConversationView {
        conversation_title: c.title.clone(),
        url: format!(
            "/admin/history/conversations/{}",
            urlencoding::encode(c.context_id.as_str())
        ),
        context_id: c.context_id.clone(),
        model: c.model.clone().unwrap_or_else(|| "—".to_owned()),
        turns: c.turn_count,
        cost_display: format_cost(c.cost_microdollars),
        last_relative: relative_time(c.last_activity),
        last_at: c.last_activity.to_rfc3339(),
    }
}

fn side_call_note(summary: &ConversationSummary) -> Option<String> {
    (summary.side_call_count > 0).then(|| {
        format!(
            "{} side calls in the last {} days",
            summary.side_call_count, summary.window_days
        )
    })
}

pub(crate) async fn profile_page(
    BootServices { boot, services }: BootServices,
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    let settings = repositories::users::user_settings::find_user_settings(&pool, &user_ctx.user_id)
        .await?
        .map_or_else(
            || SettingsView {
                display_name: None,
                avatar_url: None,
                timezone: "UTC".to_owned(),
            },
            |s| SettingsView {
                display_name: s.display_name,
                avatar_url: s.avatar_url,
                timezone: s.timezone,
            },
        );
    let data = bridge_profile::build_bridge_profile_data(&services, &boot, pool, &user_ctx).await?;
    let conversations = &data.usage.conversations;
    let recent_conversations: Vec<_> = conversations.recent.iter().map(conversation_view).collect();
    let view = ProfilePageView {
        models_count: data.usage.top_models.len(),
        latest_conversation: conversations.latest.as_ref().map(conversation_view),
        recent_count: recent_conversations.len(),
        has_recent: !recent_conversations.is_empty(),
        side_call_note: side_call_note(conversations),
        recent_conversations,
        data,
        settings,
        username: user_ctx.username.clone(),
        breadcrumbs: vec![
            BreadcrumbView::link("Account", "/admin/profile"),
            BreadcrumbView::current("Profile"),
        ],
    };
    Ok(render_typed_page(
        &engine, "profile", &view, &user_ctx, &mkt_ctx,
    ))
}

pub(crate) async fn issue_bridge_code(
    Extension(user_ctx): Extension<UserContext>,
    Extension(boot): Extension<systemprompt_web_admin_core::extension_boot::ExtensionBoot>,
    State(pool): State<Arc<PgPool>>,
) -> AdminResult<Response> {
    let (_, gateway_url) =
        systemprompt_web_admin_core::bridge_client::read_config_strings(boot.config());
    let block = bridge_profile::issue_bridge_connect(&pool, &user_ctx, gateway_url.as_deref())
        .await
        .ok_or_else(|| {
            AdminError::Unavailable("Could not mint a connect code just now.".to_owned())
        })?;
    Ok(Json(block).into_response())
}
