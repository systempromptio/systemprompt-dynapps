//! Read-only explanation of entitlement, delivered catalog content and
//! connection state. Client installation and execution are deliberately never
//! inferred from a heartbeat.

use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::marketplace::{AssembleRequest, ManifestService, MarketplaceCache};

use crate::repositories::users::queries;
use systemprompt_web_admin_core::connections::Connection;
use systemprompt_web_admin_core::connections::source::get_connections;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::marketplace_filter::TemplateMarketplaceFilter;
use systemprompt_web_admin_core::repositories::entitlement::{MatrixRow, UserMatrix};

use super::workspace_grant::{grants_access, workspace_reason, workspace_status, workspace_tone};

#[derive(Debug, Default, Serialize)]
pub(crate) struct AccessOverview {
    pub catalog_available: bool,
    pub allowed_count: Option<usize>,
    pub workspaces: Vec<WorkspaceView>,
    pub other_workspaces: Vec<WorkspaceView>,
    pub connections_available: bool,
    pub attention_count: Option<usize>,
    pub connections: Vec<ConnectionView>,
    pub device_activity: String,
    pub device_detail: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct WorkspaceView {
    pub id: String,
    pub name: String,
    pub status: String,
    pub tone: &'static str,
    pub reason: String,
    pub plugins: Vec<PluginView>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PluginView {
    pub name: String,
    pub skills: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConnectionView {
    pub name: String,
    pub permission: &'static str,
    pub status: &'static str,
    pub tone: &'static str,
    pub next_step: &'static str,
    pub verified_at: String,
}

impl AccessOverview {
    pub(super) fn set_permissions(&mut self, matrix: Option<&UserMatrix>) {
        let Some(matrix) = matrix else { return };
        let Some(section) = matrix
            .sections
            .iter()
            .find(|s| s.entity_type == "marketplace")
        else {
            if self.catalog_available {
                self.allowed_count = Some(0);
            }
            return;
        };
        let mut included = std::mem::take(&mut self.workspaces);
        for row in &section.rows {
            let plugins = take_plugins(&mut included, &row.entity_id);
            let workspace = workspace_view(row, plugins);
            if grants_access(row) {
                self.workspaces.push(workspace);
            } else {
                self.other_workspaces.push(workspace);
            }
        }
        if self.catalog_available {
            self.allowed_count = Some(self.workspaces.len());
        }
    }
}

fn take_plugins(included: &mut [WorkspaceView], workspace_id: &str) -> Vec<PluginView> {
    included
        .iter_mut()
        .find(|w| w.id == workspace_id)
        .map(|w| std::mem::take(&mut w.plugins))
        .unwrap_or_default()
}

fn workspace_view(row: &MatrixRow, plugins: Vec<PluginView>) -> WorkspaceView {
    let allowed = grants_access(row);
    let status = workspace_status(row);
    WorkspaceView {
        id: row.entity_id.clone(),
        name: row.entity_name.clone(),
        status: status.into(),
        tone: workspace_tone(row, allowed, status),
        reason: workspace_reason(row, allowed, status),
        plugins,
    }
}

pub(super) async fn load(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    boot: &systemprompt_web_admin_core::extension_boot::ExtensionBoot,
    pool: &PgPool,
    user_id: &UserId,
) -> AccessOverview {
    let (catalog, connections, runtime) = tokio::join!(
        included_content(pool, services, boot, user_id),
        get_connections(pool, services, user_id),
        queries::get_user_runtime_detail(pool, user_id),
    );
    let mut view = AccessOverview::default();
    match catalog {
        Ok(workspaces) => {
            view.catalog_available = true;
            view.workspaces = workspaces;
        },
        Err(e) => tracing::warn!(error = %e, "user access: included content unavailable"),
    }
    match connections {
        Ok(snapshot) => {
            view.connections_available = true;
            view.attention_count = Some(
                snapshot
                    .connections
                    .iter()
                    .filter(|c| c.entitled && needs_attention(c))
                    .count(),
            );
            view.connections = snapshot.connections.iter().map(connection_view).collect();
        },
        Err(e) => tracing::warn!(error = %e, "user access: connections unavailable"),
    }
    match runtime {
        Ok(runtime) => {
            view.device_activity = runtime.last_heartbeat_at.map_or_else(
                || "Not verified".to_owned(),
                |t| super::view::stamp(Some(t)),
            );
            view.device_detail = match runtime.last_hostname {
                Some(host) => format!(
                    "{} · {} · bridge {}",
                    host,
                    runtime.last_os.unwrap_or_default(),
                    runtime.last_bridge_version.unwrap_or_default()
                ),
                None => "No device has reported activity".to_owned(),
            };
        },
        Err(e) => {
            tracing::warn!(error = %e, "user access: device activity unavailable");
            "Unable to load".clone_into(&mut view.device_activity);
        },
    }
    view
}

async fn included_content(
    pool: &PgPool,
    view: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    boot: &systemprompt_web_admin_core::extension_boot::ExtensionBoot,
    user_id: &UserId,
) -> AdminResult<Vec<WorkspaceView>> {
    let services = view.config();
    let root = systemprompt_web_admin_core::request::get_services_path(boot);
    let url = boot.config().api_external_url.clone();
    let filter = TemplateMarketplaceFilter::from_pool(Arc::new(pool.clone()), view.clone());
    let cache = uncached_marketplace();
    let request = AssembleRequest {
        services: &services,
        services_root: &root,
        filter: &filter,
        user_id,
        cache: &cache,
    };
    let candidate = ManifestService::assemble_candidate(&request, &url)
        .await
        .map_err(|e| AdminError::upstream("marketplace manifest", e))?;
    let (entries, _) = candidate.into_manifest_parts();
    let mut workspaces = Vec::new();
    for marketplace in entries.marketplaces {
        let mut plugins = Vec::new();
        for id in marketplace.plugin_ids {
            let name = services
                .plugins
                .get(id.as_str())
                .map_or_else(|| id.to_string(), |p| p.name.clone());
            plugins.push(PluginView {
                name,
                skills: entries
                    .skills
                    .iter()
                    .filter(|s| s.plugins.contains(&id))
                    .count(),
            });
        }
        plugins.sort_by(|a, b| a.name.cmp(&b.name));
        workspaces.push(WorkspaceView {
            id: marketplace.id.to_string(),
            name: marketplace.name,
            status: String::new(),
            tone: "ok",
            reason: String::new(),
            plugins,
        });
    }
    Ok(workspaces)
}

// Why: systemprompt core keeps the marketplace cache on its application
// context, which the admin plane never receives; one render reloads it once.
fn uncached_marketplace() -> MarketplaceCache {
    MarketplaceCache::default()
}

fn needs_attention(connection: &Connection) -> bool {
    !connection.configured || connection.status != "connected" || connection.verified_at.is_none()
}

fn connection_view(c: &Connection) -> ConnectionView {
    let (status, tone, next_step) = if c.configured {
        match c.status.as_str() {
            "connected" if c.verified_at.is_some() => (
                "Connected",
                "ok",
                "Connection verified; individual tool requests can still fail.",
            ),
            "temporarily_unavailable" => (
                "Temporarily unavailable",
                "warn",
                "Retry the connection when the provider is available.",
            ),
            "connected" => (
                "Verification required",
                "warn",
                "The user must test this connection in the bridge app.",
            ),
            "not_connected" | "expired" | "revoked" | "reauth_required" | "reconnect_required" => (
                "Sign-in required",
                "warn",
                "The user must sign in to this connector in the bridge app.",
            ),
            _ => (
                "Verification required",
                "warn",
                "The user must review and test this connection in the bridge app.",
            ),
        }
    } else {
        (
            "Not configured",
            "muted",
            "An administrator must configure this connector.",
        )
    };
    ConnectionView {
        name: c.provider.clone(),
        permission: if c.entitled { "Allowed" } else { "Not allowed" },
        status,
        tone,
        next_step: if c.entitled {
            next_step
        } else {
            "Review this person's workspace and connector permissions."
        },
        verified_at: c
            .verified_at
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map_or_else(
                || "Not verified".to_owned(),
                |t| super::view::stamp(Some(t.with_timezone(&chrono::Utc))),
            ),
    }
}
