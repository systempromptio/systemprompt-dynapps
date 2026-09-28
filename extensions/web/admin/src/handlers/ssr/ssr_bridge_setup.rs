//! SSR page walking a user through bridge installation.

use axum::extract::Extension;
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Serialize;

use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};

use systemprompt_web_admin_core::ssr::helpers::render_typed_page;

// Why: core's `RoutingDecision::is_static_asset_path` has no archive extension
// but whitelists the `/files` prefix, so `/downloads` would not serve. Names
// match `scripts/package-bridge-linux.sh` and `admin-bridge-setup.js`.
const DOWNLOAD_BASE_URL: &str = "/files/downloads";

#[derive(Debug, Serialize)]
struct SetupPageData {
    gateway_url: String,
    user_email: String,
    download_base_url: &'static str,
}

pub(crate) async fn bridge_setup_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    headers: HeaderMap,
) -> AdminHtmlResult<Response> {
    let data = SetupPageData {
        gateway_url: derive_gateway_url(&headers),
        user_email: user_ctx.email.to_string(),
        download_base_url: DOWNLOAD_BASE_URL,
    };
    Ok(render_typed_page(
        &engine,
        "bridge-setup",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn derive_gateway_url(headers: &HeaderMap) -> String {
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
