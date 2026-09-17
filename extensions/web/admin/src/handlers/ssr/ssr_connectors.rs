//! SSR page for a user's connectors — one card per provider with its status,
//! account and actions. The first paint is server-rendered from the same
//! snapshot the browser then polls; `connectors.js` re-renders in place.

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::AdminHtmlResult;
use crate::handlers::ssr::format::relative_time;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::services::connector_accounts::{Connection, get_connections};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

const ATTENTION: [&str; 3] = [
    "reconnect_required",
    "verification_required",
    "temporarily_unavailable",
];

#[derive(Debug, Default, Serialize)]
struct ConnectorsSummary {
    configured: usize,
    connected: usize,
    needs_attention: usize,
    not_connected: usize,
}

#[derive(Debug, Serialize)]
struct ConnectorCardView {
    provider: String,
    display_name: String,
    initial: String,
    status: String,
    status_label: &'static str,
    tone: &'static str,
    account_name: Option<String>,
    resource_host: Option<String>,
    verified_relative: Option<String>,
    note: Option<String>,
    can_test: bool,
}

#[derive(Debug, Serialize)]
struct ConnectorsPageView {
    page: &'static str,
    title: &'static str,
    user_email: String,
    user_id: UserId,
    summary: ConnectorsSummary,
    connectors: Vec<ConnectorCardView>,
    connectors_count: usize,
    has_connectors: bool,
    breadcrumbs: Vec<BreadcrumbView>,
}

fn status_label(status: &str) -> &'static str {
    match status {
        "connected" => "Connected",
        "reconnect_required" => "Reconnect required",
        "verification_required" => "Verification required",
        "temporarily_unavailable" => "Temporarily unavailable",
        "not_provisioned" => "Not provisioned",
        "not_configured" => "Not configured",
        "no_auth_required" => "No authorization required",
        _ => "Not connected",
    }
}

fn tone(status: &str) -> &'static str {
    match status {
        "connected" => "ok",
        "reconnect_required" | "verification_required" => "warn",
        "temporarily_unavailable" => "err",
        _ => "muted",
    }
}

// Why: the order a person should act in — broken first, then never
// connected, then healthy, then the ones nothing can be done about.
fn rank(status: &str) -> u8 {
    match status {
        s if ATTENTION.contains(&s) => 0,
        "not_connected" => 1,
        "connected" => 2,
        _ => 3,
    }
}

fn host(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .unwrap_or_else(|| url.to_owned())
}

fn note(c: &Connection) -> Option<String> {
    if !c.entitled {
        return Some(if c.provider.starts_with("salesforce") {
            "Your account is not entitled to this Salesforce org".to_owned()
        } else {
            "An active account is required to connect providers".to_owned()
        });
    }
    match (c.status.as_str(), c.error_code.as_deref()) {
        ("not_provisioned", _) => {
            Some("Waiting for a Salesforce administrator to provision this org".to_owned())
        },
        (_, Some("sandbox_refreshed")) => {
            Some("This org was refreshed by an administrator — reconnect to continue".to_owned())
        },
        (_, Some("verification_failed")) => {
            Some("Authorization saved, but the last verification failed".to_owned())
        },
        (_, Some("grant_rejected")) => Some("The provider rejected the saved grant".to_owned()),
        (_, Some("configuration_changed")) => {
            Some("The connector configuration changed — reconnect to authorize again".to_owned())
        },
        (_, Some("provider_unavailable")) => {
            Some("The provider did not answer the last check".to_owned())
        },
        (_, Some("provider_permission_denied")) => {
            Some("The provider refused the permissions this connector needs".to_owned())
        },
        _ => None,
    }
}

fn card(c: &Connection) -> ConnectorCardView {
    let verified_relative = c
        .verified_at
        .as_deref()
        .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
        .map(|t| relative_time(t.with_timezone(&Utc)));
    ConnectorCardView {
        provider: c.provider.clone(),
        display_name: c.display_name.clone(),
        initial: c.display_name.chars().take(1).collect(),
        status: c.status.clone(),
        status_label: status_label(&c.status),
        tone: tone(&c.status),
        account_name: c.account_name.clone(),
        resource_host: c.resource_name.as_deref().map(host),
        verified_relative,
        note: note(c),
        can_test: c.actions.iter().any(|a| a == "test"),
    }
}

fn summarise(connections: &[Connection]) -> ConnectorsSummary {
    let mut summary = ConnectorsSummary::default();
    for c in connections
        .iter()
        .filter(|c| c.configured && c.requires_auth)
    {
        summary.configured += 1;
        match c.status.as_str() {
            "connected" => summary.connected += 1,
            s if ATTENTION.contains(&s) => summary.needs_attention += 1,
            "not_connected" => summary.not_connected += 1,
            _ => {},
        }
    }
    summary
}

pub(crate) async fn connectors_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    let snapshot = get_connections(&pool, &user_ctx.user_id).await?;
    let mut connections = snapshot.connections;
    connections.sort_by_key(|c| rank(&c.status));
    let connectors: Vec<_> = connections.iter().map(card).collect();
    let view = ConnectorsPageView {
        page: "connectors",
        title: "Connectors",
        user_email: user_ctx.email.as_str().to_owned(),
        user_id: user_ctx.user_id.clone(),
        summary: summarise(&connections),
        connectors_count: connectors.len(),
        has_connectors: !connectors.is_empty(),
        connectors,
        breadcrumbs: vec![
            BreadcrumbView::link("Account", "/admin/profile"),
            BreadcrumbView::current("Connectors"),
        ],
    };
    Ok(render_typed_page(
        &engine,
        "connectors",
        &view,
        &user_ctx,
        &mkt_ctx,
    ))
}
