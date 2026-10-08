//! Aggregator for the bridge-style profile pane.
//!
//! Produces the same payload shape consumed by the bridge GUI's profile tab
//! so the SSR profile page and (future) `/v1/bridge/profile/usage` endpoint
//! render the same data from the same source.

mod assemble;

use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{MarketplaceId, TenantId, UserId};

use systemprompt_web_admin_core::bridge_client::{BRIDGE_BINARY, read_config_strings};
use systemprompt_web_admin_core::types::UserContext;

use assemble::{
    build_agents_block, build_bridge_profile_block, build_reachable_marketplaces, build_usage,
    load_usage_sections, read_tenant_id,
};

#[derive(Debug, Clone, Serialize)]
pub struct ProfileIdentity {
    pub email: String,
    pub display_name: Option<String>,
    pub user_id: UserId,
    pub tenant_id: Option<TenantId>,
    pub provider: Option<String>,
    pub roles: Vec<String>,
    pub jwt_issuer: Option<String>,
    pub gateway: Option<String>,
    pub is_admin: bool,
}

use systemprompt_web_admin_core::repositories::user_usage::{
    ConversationSummary, ModelShare, UsageWindow,
};

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProfileUsage {
    pub d1: UsageWindow,
    pub d7: UsageWindow,
    pub d30: UsageWindow,
    pub top_models: Vec<ModelShare>,
    pub conversations: ConversationSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeProfileBlock {
    pub inference_gateway_base_url: String,
    pub auth_scheme: String,
    pub models: Vec<String>,
    pub models_count: usize,
    pub organization_uuid: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileMarketplaceView {
    pub id: MarketplaceId,
    pub name: String,
    pub version: String,
    pub plugin_count: usize,
    pub layer: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentItem {
    pub id: String,
    pub display_name: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AgentsBlock {
    pub total: i64,
    pub enabled: i64,
    pub items: Vec<AgentItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeConnectBlock {
    pub code: String,
    pub expires_in_seconds: i64,
    pub gateway: String,
    pub install_command: String,
    pub login_command: String,
    pub opencode_install_command: String,
    pub opencode_login_command: String,
    pub desktop_windows_login_command: String,
    pub just_install_command: String,
    pub just_login_command: String,
}

// Why: `install.sh` enrols a host only if its binary already exists and rejects
// `--hosts all`, so each command names its host; an unenrolled OpenCode fails
// every prompt with `403 no loopback credential presented`.
fn install_command(gateway: &str, code: &str, host: &str) -> String {
    format!(
        "curl -fsSL {gateway}/files/downloads/install.sh | sh -s -- \
         --download-base {gateway}/files/downloads --code {code} --host {host}"
    )
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeProfilePageData {
    pub page: &'static str,
    pub title: &'static str,
    pub identity: ProfileIdentity,
    pub bridge_connect_available: bool,
    pub bridge_profile: Option<BridgeProfileBlock>,
    pub usage: ProfileUsage,
    pub agents: AgentsBlock,
    pub marketplaces: Vec<ProfileMarketplaceView>,
    pub marketplaces_count: usize,
}

// Why: a connect code is a bearer credential that redeems for a durable PAT
// signing in as its owner, so it is minted only on an explicit request and
// never rendered into page HTML.
pub async fn issue_bridge_connect(
    pool: &PgPool,
    user_ctx: &UserContext,
    gateway: Option<&str>,
) -> Option<BridgeConnectBlock> {
    let gateway = gateway?.to_owned();
    let issued = systemprompt_web_admin_core::repositories::bridge::issue_exchange_code(
        pool,
        &user_ctx.user_id,
    )
    .await
    .map_err(|e| {
        tracing::warn!(
            error = %e,
            "could not mint a bridge exchange code"
        );
    })
    .ok()?;

    let expires_in_seconds = (issued.expires_at - chrono::Utc::now())
        .num_seconds()
        .max(0);

    Some(BridgeConnectBlock {
        install_command: install_command(&gateway, &issued.code, "claude-code"),
        opencode_install_command: install_command(&gateway, &issued.code, "opencode"),
        opencode_login_command: format!("{BRIDGE_BINARY} install --host opencode"),
        login_command: format!(
            "{BRIDGE_BINARY} login --code {code} --gateway {gateway}",
            code = issued.code
        ),
        desktop_windows_login_command: format!(
            ".\\{BRIDGE_BINARY}-windows.exe login --code {code} --gateway {gateway}",
            code = issued.code
        ),
        just_install_command: format!("just claude {code} {gateway}", code = issued.code),
        just_login_command: format!("just connect {code} {gateway}", code = issued.code),
        code: issued.code,
        expires_in_seconds,
        gateway,
    })
}

pub async fn build_bridge_profile_data(
    pool: Arc<PgPool>,
    user_ctx: &UserContext,
) -> systemprompt_web_admin_core::error::AdminResult<BridgeProfilePageData> {
    let user_id = user_ctx.user_id.clone();

    let sections = load_usage_sections(&pool, &user_id).await;
    let display_name = sections
        .bridge_user
        .as_ref()
        .and_then(|u| u.display_name.clone());

    let federated_provider =
        systemprompt_web_admin_core::repositories::principal::identity::find_identity_envelope(
            &pool, &user_id,
        )
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "profile: identity provider lookup failed"))
        .ok()
        .flatten()
        .and_then(|env| env.idp_issuer.map(|_| "adfs".to_owned()));

    let (jwt_issuer, gateway_url) = read_config_strings();
    let bridge_profile = build_bridge_profile_block();
    let bridge_connect_available = gateway_url.is_some();

    let identity = ProfileIdentity {
        email: user_ctx.email.as_str().to_owned(),
        display_name,
        user_id: user_ctx.user_id.clone(),
        tenant_id: read_tenant_id(),
        provider: federated_provider,
        roles: user_ctx.roles.clone(),
        jwt_issuer,
        gateway: gateway_url,
        is_admin: user_ctx.is_admin,
    };

    let usage = build_usage(sections);
    let mut agents = build_agents_block();
    let access =
        systemprompt_web_admin_core::authz::catalog::CatalogAccess::load(&pool, &user_id).await?;
    let ids = agents
        .items
        .iter()
        .filter(|a| a.enabled)
        .map(|a| a.id.clone())
        .collect::<Vec<_>>();
    let allowed = access
        .allowed(systemprompt_security::authz::EntityKind::Agent, &ids)
        .await?;
    agents
        .items
        .retain(|a| a.enabled && allowed.contains(&a.id));
    agents.total = agents.items.len() as i64;
    agents.enabled = agents.total;
    let marketplaces =
        build_reachable_marketplaces(&pool, &user_id, user_ctx.roles.clone()).await?;

    Ok(BridgeProfilePageData {
        marketplaces_count: marketplaces.len(),
        marketplaces,
        page: "profile",
        title: "Profile",
        identity,
        bridge_connect_available,
        bridge_profile,
        usage,
        agents,
    })
}
