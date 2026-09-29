//! `/admin/connect` — the one onboarding page: connect a client — Claude
//! Code, Claude Desktop or `OpenCode` — with a single-use connect code, then
//! the phases that follow, with completion derived from what reaches the
//! account.

use axum::extract::Extension;
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Serialize;

use systemprompt_web_admin_catalog::services::bridge_profile;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::helpers::render_typed_page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};

use super::ssr_setup::{SetupPhase, setup_phases};

// Why: core's `RoutingDecision::is_static_asset_path` has no archive extension
// but whitelists the `/files` prefix, so `/downloads` would not serve. Names
// match `scripts/package-bridge-linux.sh`.
const DOWNLOAD_BASE_URL: &str = "/files/downloads";

#[derive(Debug, Serialize)]
struct ConnectPageView {
    page: &'static str,
    title: &'static str,
    user_email: String,
    gateway: Option<String>,
    gateway_url: String,
    download_base_url: &'static str,
    bridge_connect_available: bool,
    phases: Vec<SetupPhase>,
    complete_count: usize,
    phase_count: usize,
    breadcrumbs: Vec<BreadcrumbView>,
}

pub(crate) async fn connect_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    headers: HeaderMap,
) -> AdminHtmlResult<Response> {
    let (_, gateway) = bridge_profile::read_config_strings();
    let phases = setup_phases(&mkt_ctx);
    let view = ConnectPageView {
        page: "connect",
        title: "Connect a client",
        user_email: user_ctx.email.as_str().to_owned(),
        gateway_url: gateway.clone().unwrap_or_else(|| request_origin(&headers)),
        download_base_url: DOWNLOAD_BASE_URL,
        bridge_connect_available: gateway.is_some(),
        gateway,
        complete_count: phases.iter().filter(|p| p.complete).count(),
        phase_count: phases.len(),
        phases,
        breadcrumbs: vec![
            BreadcrumbView::link("Account", "/admin/profile"),
            BreadcrumbView::current("Connect"),
        ],
    };
    Ok(render_typed_page(
        &engine, "connect", &view, &user_ctx, &mkt_ctx,
    ))
}

fn request_origin(headers: &HeaderMap) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:8080");
    format!("{scheme}://{host}")
}
