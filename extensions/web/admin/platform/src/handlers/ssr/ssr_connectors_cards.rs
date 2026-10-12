//! One connector card: what the provider is, what it unlocks, which plugins
//! carry it, the account behind it and the actions it offers. The card is
//! built once here for the first paint and mirrored by `connectors.js`; the
//! static "about" block (kind, blurb, plugins) only exists here, so the script
//! carries it across re-renders instead of rebuilding it.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::models::plugin::ComponentSource;

use systemprompt_web_admin_core::connections::Connection;
use systemprompt_web_admin_core::ssr::format::relative_time;

pub(super) const ATTENTION: [&str; 3] = [
    "reconnect_required",
    "verification_required",
    "temporarily_unavailable",
];

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ActionView {
    action: &'static str,
    label: &'static str,
    variant: &'static str,
}

#[derive(Debug, Serialize)]
pub struct ConnectorCardView {
    pub provider: String,
    pub display_name: String,
    pub glyph: &'static str,
    pub monogram: String,
    pub kind: &'static str,
    pub blurb: String,
    pub used_by: Vec<String>,
    pub status: String,
    pub status_label: &'static str,
    pub tone: &'static str,
    pub account_name: Option<String>,
    pub resource_host: Option<String>,
    pub verified_relative: Option<String>,
    pub has_facts: bool,
    pub note: Option<String>,
    pub actions: Vec<ActionView>,
    pub can_test: bool,
}

pub(super) fn status_label(status: &str) -> &'static str {
    match status {
        "connected" => "Connected",
        "reconnect_required" => "Reconnect required",
        "verification_required" => "Verification required",
        "temporarily_unavailable" => "Temporarily unavailable",
        "not_provisioned" => "Not provisioned",
        "not_configured" => "Not configured",
        "no_auth_required" => "Built in",
        _ => "Not connected",
    }
}

pub(super) fn tone(status: &str) -> &'static str {
    match status {
        "connected" => "ok",
        "reconnect_required" | "verification_required" => "warn",
        "temporarily_unavailable" => "err",
        _ => "muted",
    }
}

fn family(provider: &str) -> &'static str {
    match provider {
        "atlassian" => "atlassian",
        "google_workspace" => "google",
        "systemprompt" => "systemprompt",
        p if p.starts_with("salesforce") => "salesforce",
        _ => "generic",
    }
}

fn kind(glyph: &str) -> &'static str {
    match glyph {
        "atlassian" => "Atlassian Cloud",
        "google" => "Google Workspace",
        "systemprompt" => "Control plane",
        "salesforce" => "Salesforce org",
        _ => "MCP server",
    }
}

fn monogram(glyph: &str, name: &str) -> String {
    match glyph {
        "atlassian" => "At".into(),
        "google" => "G".into(),
        "systemprompt" => "Sp".into(),
        "salesforce" => "Sf".into(),
        _ => name.chars().take(2).collect(),
    }
}

fn blurb(glyph: &str, c: &Connection) -> String {
    match glyph {
        "atlassian" => "Jira issues and Confluence pages, read and written as you.".into(),
        "google" => {
            "Search across Drive, Gmail and Calendar — only what you can already open.".into()
        },
        "systemprompt" => {
            "The Dynapps control plane: governed tools, catalogue and deployments, as the admin you are signed in as.".into()
        },
        "salesforce" => format!(
            "Records, SOQL and metadata in {}, as the Salesforce user you sign in as.",
            c.display_name
        ),
        _ => "Authorize once; every client you connect uses the same account.".into(),
    }
}

fn used_by(provider: &str) -> Vec<String> {
    let Ok(services) = systemprompt::loader::ServicesBootstrap::get() else {
        return Vec::new();
    };
    let mut names: Vec<String> = services
        .plugins
        .values()
        .filter(|p| p.enabled)
        .filter(|p| {
            matches!(p.mcp_servers.source, ComponentSource::Explicit)
                && p.mcp_servers.include.iter().any(|id| id == provider)
        })
        .map(|p| p.name.clone())
        .collect();
    names.sort();
    names
}

fn host(url: &str) -> String {
    reqwest::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_owned))
        .unwrap_or_else(|| url.to_owned())
}

pub(super) fn note(c: &Connection) -> Option<String> {
    if !c.entitled {
        return Some(if c.provider.starts_with("salesforce") {
            "Your account is not entitled to this Salesforce org".to_owned()
        } else if c.session_attested {
            "Not open to your account".to_owned()
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
        (_, Some("provider_configuration")) => Some("The provider rejected the app configuration — ask the platform administrator to check its OAuth settings".into()),
        (_, Some("provider_invalid_response")) => Some("The provider returned an invalid authorization response — test the connection before using it".into()),
        (_, Some("refresh_outcome_unknown")) => Some("The refresh response was lost or unusable — reconnect before using this provider again".into()),
        (_, Some("provider_unavailable")) => {
            Some("The provider did not answer the last check".to_owned())
        },
        (_, Some("provider_permission_denied")) => {
            Some("The provider refused the permissions this connector needs".to_owned())
        },
        _ => None,
    }
}

fn actions(c: &Connection) -> Vec<ActionView> {
    let connected = c.status == "connected";
    let order = ["test", "connect", "reconnect", "disconnect", "manual_token"];
    order
        .iter()
        .filter(|a| c.actions.iter().any(|have| have == *a))
        .map(|a| match *a {
            "test" => ActionView {
                action: "test",
                label: "Test connection",
                variant: if connected { "primary" } else { "outline" },
            },
            "connect" => ActionView {
                action: "connect",
                label: "Connect",
                variant: "primary",
            },
            "reconnect" => ActionView {
                action: "reconnect",
                label: "Reconnect",
                variant: if connected { "outline" } else { "primary" },
            },
            "disconnect" => ActionView {
                action: "disconnect",
                label: "Disconnect",
                variant: "outline-danger",
            },
            _ => ActionView {
                action: "manual_token",
                label: "Use personal token",
                variant: "ghost",
            },
        })
        .collect()
}

pub fn card(c: &Connection) -> ConnectorCardView {
    let glyph = family(&c.provider);
    let verified_relative = c
        .verified_at
        .as_deref()
        .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
        .map(|t| relative_time(t.with_timezone(&Utc)));
    let resource_host = c.resource_name.as_deref().map(host);
    ConnectorCardView {
        provider: c.provider.clone(),
        display_name: c.display_name.clone(),
        glyph,
        monogram: monogram(glyph, &c.display_name),
        kind: kind(glyph),
        blurb: blurb(glyph, c),
        used_by: used_by(&c.provider),
        status: c.status.clone(),
        status_label: status_label(&c.status),
        tone: tone(&c.status),
        has_facts: c.account_name.is_some()
            || resource_host.is_some()
            || verified_relative.is_some(),
        account_name: c.account_name.clone(),
        resource_host,
        verified_relative,
        note: note(c),
        actions: actions(c),
        can_test: c.actions.iter().any(|a| a == "test"),
    }
}
