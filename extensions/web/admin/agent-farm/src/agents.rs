//! Agents: the catalogue farm-api offers a tenant, and one agent's
//! configuration. Credentials, MCP servers and environment variables appear by
//! name only (farm-api sends no values); a personal agent's instructions are
//! private to its owner and are not shown.

use axum::Extension;
use axum::extract::{Path, Query};
use axum::response::Response;
use serde::Serialize;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

pub use crate::agents_detail::{AgentDetailView, NamePair, agent_detail, limit_rows};
use crate::client::FarmApi;
use crate::handlers::{TenantQuery, require_console, scope};
use crate::models::{AgentOut, TenantRef};
use crate::views::{TenantSelect, href, opt_text, row_tenant, tenant_label, tenant_select};

const LIST_URL: &str = "/admin/farm/agents";

#[derive(Debug, Serialize)]
pub struct AgentRowView {
    pub slug: String,
    pub href: String,
    pub tenant: String,
    pub scope: String,
    pub scope_tone: &'static str,
    pub description: String,
    pub plugin_agent: String,
    pub model: String,
    pub harness: String,
    pub workspace: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AgentsPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub select: TenantSelect,
    pub rows: Vec<AgentRowView>,
    pub has_rows: bool,
    pub loaded: bool,
    pub count_label: String,
}

#[derive(Debug, Serialize)]
pub struct AgentPageContext {
    pub page: &'static str,
    pub title: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub back_url: String,
    pub agent: Option<AgentDetailView>,
}

pub fn scope_tone(scope: &str) -> &'static str {
    match scope {
        "platform" => "info",
        "team" => "accent",
        "personal" => "muted",
        _ => "warn",
    }
}

fn list_href(tenant: &str) -> String {
    href(LIST_URL, &[("tenant", tenant)])
}

pub fn agent_href(slug: &str, tenant: Option<&str>) -> String {
    href(
        &format!("{LIST_URL}/{}", urlencoding::encode(slug)),
        &[("tenant", tenant.unwrap_or_default())],
    )
}

pub const fn yes_no(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "yes",
        Some(false) => "no",
        None => "—",
    }
}

pub fn agent_row(agent: &AgentOut, filter: &str) -> AgentRowView {
    let scope = agent.scope.clone().unwrap_or_default();
    let own = row_tenant(agent.tenant.as_deref(), filter);
    AgentRowView {
        href: agent_href(&agent.slug, own.as_deref()),
        tenant: tenant_label(own.as_deref()),
        slug: agent.slug.clone(),
        scope_tone: scope_tone(&scope),
        scope,
        description: opt_text(agent.description.as_deref()),
        plugin_agent: opt_text(agent.plugin_agent.as_deref()),
        model: opt_text(agent.model_alias.as_deref()),
        harness: opt_text(agent.harness.as_deref()),
        workspace: yes_no(agent.requires_workspace),
    }
}

pub fn agents_view(
    tenant: &str,
    tenants: &[TenantRef],
    agents: Option<&[AgentOut]>,
) -> AgentsPageContext {
    let rows: Vec<AgentRowView> = agents
        .map(|a| a.iter().map(|x| agent_row(x, tenant)).collect())
        .unwrap_or_default();
    let count_label = match rows.len() {
        1 => "1 agent".to_owned(),
        n => format!("{n} agents"),
    };
    AgentsPageContext {
        page: "farm-agents",
        title: "Agents",
        breadcrumbs: vec![
            BreadcrumbView::link("Farm", "/admin/farm"),
            BreadcrumbView::current("Agents"),
        ],
        select: tenant_select(LIST_URL, tenants, tenant),
        has_rows: !rows.is_empty(),
        rows,
        loaded: agents.is_some(),
        count_label,
    }
}

pub fn agent_view(slug: &str, tenant: &str, agent: Option<&AgentOut>) -> AgentPageContext {
    let back_url = list_href(tenant);
    AgentPageContext {
        page: "farm-agent",
        title: format!("Agent {slug}"),
        breadcrumbs: vec![
            BreadcrumbView::link("Farm", "/admin/farm"),
            BreadcrumbView::link("Agents", back_url.clone()),
            BreadcrumbView::current(slug.to_owned()),
        ],
        back_url,
        agent: agent.map(agent_detail),
    }
}

pub async fn agents_page(
    shell: Page,
    Extension(api): Extension<FarmApi>,
    Query(query): Query<TenantQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let mut s = scope(&api, &query).await;
    let agents = if s.ready {
        match api.list_agents(&s.tenant).await {
            Ok(list) => Some(list.items),
            Err(e) => {
                s.faults.record("Agents", &e);
                None
            },
        }
    } else {
        None
    };
    let ctx = agents_view(&s.tenant, &s.tenants, agents.as_deref());
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-agents",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &s.faults,
    ))
}

pub async fn agent_page(
    shell: Page,
    Extension(api): Extension<FarmApi>,
    Path(slug): Path<String>,
    Query(query): Query<TenantQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let mut s = scope(&api, &query).await;
    let tenant = s.concrete().to_owned();
    let agent = if s.ready {
        match api.get_agent(&tenant, &slug).await {
            Ok(agent) => Some(agent),
            Err(e) => {
                s.faults.record("Agent", &e);
                None
            },
        }
    } else {
        None
    };
    let ctx = agent_view(&slug, &tenant, agent.as_ref());
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-agent",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &s.faults,
    ))
}
