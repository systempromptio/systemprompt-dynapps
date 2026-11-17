//! Credentials: metadata only.
//!
//! The model (`models::CredentialOut`) has no
//! field for a value, the fingerprint or the free-form metadata, so none of
//! them can reach a page whatever farm-api sends; the template has no input.

use axum::Extension;
use axum::extract::Query;
use axum::response::Response;
use serde::Serialize;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::helpers::{Viewer, render_page_with_faults};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;

use crate::client::FarmApi;
use crate::handlers::{TenantQuery, require_console, scope};
use crate::models::{CredentialOut, TenantRef};
use crate::views::{TenantSelect, format_time, opt_text, row_tenant, tenant_label, tenant_select};

const URL: &str = "/admin/farm/credentials";

#[derive(Debug, Serialize)]
pub struct CredentialRowView {
    pub name: String,
    pub scope: String,
    pub scope_tone: &'static str,
    pub kind: String,
    pub tenant: String,
    pub description: String,
    pub expires: String,
    pub created: String,
    pub updated: String,
}

#[derive(Debug, Serialize)]
pub struct CredentialsPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub select: TenantSelect,
    pub rows: Vec<CredentialRowView>,
    pub has_rows: bool,
    pub loaded: bool,
    pub count_label: String,
}

pub fn credential_row(c: &CredentialOut, filter: &str) -> CredentialRowView {
    let scope = c.scope.clone().unwrap_or_default();
    CredentialRowView {
        name: c.name.clone(),
        scope_tone: if scope == "shared" { "accent" } else { "muted" },
        scope,
        kind: opt_text(c.kind.as_deref()),
        tenant: tenant_label(row_tenant(c.tenant.as_deref(), filter).as_deref()),
        description: opt_text(c.description.as_deref()),
        expires: format_time(c.expires_at.as_deref()),
        created: format_time(c.created_at.as_deref()),
        updated: format_time(c.updated_at.as_deref()),
    }
}

pub fn credentials_view(
    tenant: &str,
    tenants: &[TenantRef],
    credentials: Option<&[CredentialOut]>,
) -> CredentialsPageContext {
    let mut rows: Vec<CredentialRowView> = credentials
        .map(|l| l.iter().map(|c| credential_row(c, tenant)).collect())
        .unwrap_or_default();
    rows.sort_by(|a, b| (&a.tenant, &a.scope, &a.name).cmp(&(&b.tenant, &b.scope, &b.name)));
    let count_label = match rows.len() {
        1 => "1 credential".to_owned(),
        n => format!("{n} credentials"),
    };
    CredentialsPageContext {
        page: "farm-credentials",
        title: "Credentials",
        breadcrumbs: vec![
            BreadcrumbView::link("Farm", "/admin/farm"),
            BreadcrumbView::current("Credentials"),
        ],
        select: tenant_select(URL, tenants, tenant),
        has_rows: !rows.is_empty(),
        rows,
        loaded: credentials.is_some(),
        count_label,
    }
}

pub async fn credentials_page(
    shell: Page,
    Extension(api): Extension<FarmApi>,
    Query(query): Query<TenantQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let mut s = scope(&api, &query).await;
    let mut all: Option<Vec<CredentialOut>> = None;
    if s.ready {
        for kind in ["personal", "shared"] {
            match api.list_credentials(&s.tenant, kind).await {
                Ok(list) => all.get_or_insert_with(Vec::new).extend(list.items),
                Err(e) => s.faults.record("Credentials", &e),
            }
        }
    }
    let ctx = credentials_view(&s.tenant, &s.tenants, all.as_deref());
    Ok(render_page_with_faults(
        &shell.engine,
        "farm-credentials",
        &ctx,
        Viewer::new(&shell.user, &shell.marketplace),
        &s.faults,
    ))
}
