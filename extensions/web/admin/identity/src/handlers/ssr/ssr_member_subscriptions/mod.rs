//! `/admin/member-subscriptions` — which members hold a personal-subscription
//! key, for which provider, and when it was last verified and used.
//!
//! Admins only. The page lists metadata from core's credential store and
//! never a key: the repository does not select the sealed columns.

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::provider_credentials::{
    ProviderCredentialHolder, list_provider_credential_holders,
};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::format::relative_time;
use systemprompt_web_admin_core::ssr::helpers::render_typed_page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};

#[derive(Debug, Serialize)]
struct HolderView {
    user_url: String,
    who: String,
    provider: String,
    label: String,
    created: String,
    created_at: String,
    verified: String,
    last_used: String,
}

#[derive(Debug, Serialize)]
struct MemberSubscriptionsView {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    rows: Vec<HolderView>,
    count: usize,
}

fn holder_view(h: &ProviderCredentialHolder) -> HolderView {
    HolderView {
        user_url: format!("/admin/users/{}", urlencoding::encode(h.user_id.as_str())),
        who: h
            .display_name
            .clone()
            .or_else(|| h.email.clone())
            .unwrap_or_else(|| h.user_id.as_str().to_owned()),
        provider: h.credential.clone(),
        label: h.label.clone().unwrap_or_else(|| "—".to_owned()),
        created: relative_time(h.created_at),
        created_at: h.created_at.to_rfc3339(),
        verified: h
            .verified_at
            .map_or_else(|| "never".to_owned(), relative_time),
        last_used: h
            .last_used_at
            .map_or_else(|| "never".to_owned(), relative_time),
    }
}

pub(crate) async fn member_subscriptions_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let holders = list_provider_credential_holders(&pool).await?;
    let rows: Vec<HolderView> = holders.iter().map(holder_view).collect();
    let view = MemberSubscriptionsView {
        page: "member-subscriptions",
        title: "Member subscriptions",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("People & access", "/admin/users"),
            BreadcrumbView::current("Member subscriptions"),
        ],
        count: rows.len(),
        rows,
    };
    Ok(render_typed_page(
        &engine,
        "member-subscriptions",
        &view,
        &user_ctx,
        &mkt_ctx,
    ))
}
