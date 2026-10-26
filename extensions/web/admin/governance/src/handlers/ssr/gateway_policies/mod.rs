//! `/admin/gateway/policies` — the gateway policy editor: quota windows,
//! safety scanners, block lists and the warn/enforce switch of each plane,
//! written straight to `ai_gateway_policies`.
//!
//! Core reads that table on every inference request (a sixty-second
//! cache), so a save here is live without a restart. A plain form, no
//! JavaScript in the path: every field posts as `(name, value)` pairs and
//! [`parse_policy_form`] turns them into the spec the file would declare.
//! The Sync tab shows what this editor changed as drift on the
//! `gateway_policies` plane until it is exported or overwritten.

mod view;

use std::sync::Arc;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use axum::Extension;
use axum::extract::{Form, Query, State};
use axum::http::HeaderMap;
use axum::response::{Redirect, Response};
use serde::Deserialize;
use sqlx::PgPool;

use self::view::{GatewayPoliciesPageData, InForce, new_policy_view, policy_view};
use crate::repositories::gateway_policies::form::parse_policy_form;
use crate::repositories::gateway_policies::rows::{
    PolicyWrite, delete_policy, list_policies, upsert_policy,
};
use crate::repositories::gateway_policies::unedited::carry_unedited_safety;
use systemprompt_web_admin_core::activity;
use systemprompt_web_admin_core::activity::NewActivity;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::repositories::sync::core_planes::CorePlanes;
use systemprompt_web_admin_core::repositories::sync::plane_ids::GATEWAY_POLICIES;
use systemprompt_web_admin_core::request::require_write_origin;
use systemprompt_web_admin_core::ssr::core_plane::{CorePlaneCardOpts, core_plane_card};
use systemprompt_web_admin_core::ssr::flash::{FlashTone, redirect_with_flash};
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::ssr::sync_plane::{is_sync_tab, sync_tab};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

pub(crate) const BASE_URL: &str = "/admin/gateway/policies";

#[derive(Debug, Default, Deserialize)]
pub(crate) struct PoliciesQuery {
    pub tab: Option<String>,
}

fn tabs(on_sync: bool) -> Vec<TabLinkView> {
    vec![
        TabLinkView {
            slug: "policies",
            label: "Policies",
            href: BASE_URL.to_owned(),
            is_active: !on_sync,
            count: None,
        },
        sync_tab(BASE_URL, on_sync),
    ]
}

fn require_console(page: &Page) -> Result<(), AdminError> {
    if page.user.is_console {
        return Ok(());
    }
    Err(AdminError::Forbidden("Admin access required.".to_owned()))
}

fn require_admin_write(
    page: &Page,
    boot: &ExtensionBoot,
    headers: &HeaderMap,
) -> Result<(), AdminError> {
    if !page.user.is_admin {
        return Err(AdminError::Forbidden(
            "Editing gateway policies needs an administrator.".to_owned(),
        ));
    }
    require_write_origin(boot, headers)
}

fn winners(
    rows: &[crate::repositories::gateway_policies::rows::PolicyRow],
) -> (Option<&str>, Option<&str>) {
    let mut windows = None;
    let mut safety = None;
    for r in rows.iter().filter(|r| r.enabled) {
        if !r.spec.quota_windows.is_empty() {
            windows = Some(r.name.as_str());
        }
        let s = &r.spec.safety;
        if !s.scanners.is_empty()
            || !s.block_categories.is_empty()
            || !s.block_response_categories.is_empty()
            || s.mode.is_warn()
        {
            safety = Some(r.name.as_str());
        }
    }
    (windows, safety)
}

pub(crate) async fn gateway_policies_page(
    page: Page,
    Extension(core_planes): Extension<CorePlanes>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PoliciesQuery>,
) -> AdminHtmlResult<Response> {
    require_console(&page)?;
    let on_sync = is_sync_tab(query.tab.as_deref());
    let sync = if on_sync {
        core_plane_card(
            &pool,
            &core_planes,
            GATEWAY_POLICIES,
            &CorePlaneCardOpts {
                entity_filter: "",
                show_export: true,
            },
        )
        .await?
    } else {
        None
    };
    let rows = list_policies(&pool).await.map_err(AdminError::Database)?;
    let today = chrono::Utc::now().date_naive();
    let (win_w, win_s) = winners(&rows);
    let policies = rows
        .iter()
        .map(|r| {
            policy_view(
                r,
                today,
                InForce {
                    windows: win_w == Some(r.name.as_str()),
                    safety: win_s == Some(r.name.as_str()),
                },
            )
        })
        .collect();
    let data = GatewayPoliciesPageData {
        page: "gateway-policies",
        title: "Gateway policies",
        tabs: tabs(on_sync),
        sync,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Gateway", "/admin/gateway"),
            BreadcrumbView::current("Policies"),
        ],
        can_write: page.user.is_admin,
        policies,
        new_policy: new_policy_view(),
        quotas_url: "/admin/governance/quotas",
        docs_url: "/documentation/enterprise-cost-management",
    };
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            &page.engine,
            "gateway-policies",
            &data,
            &page.user,
            &page.marketplace,
        ),
    )
}

fn saved(name: &str) -> Redirect {
    redirect_with_flash(
        BASE_URL,
        FlashTone::Ok,
        &format!(
            "Policy '{name}' is written. The gateway reads it on the next request; a cached policy lasts at most sixty seconds."
        ),
    )
}

fn not_saved(reason: &str) -> Redirect {
    redirect_with_flash(BASE_URL, FlashTone::Err, &format!("Not saved: {reason}"))
}

pub(crate) async fn save_gateway_policy(
    page: Page,
    State(pool): State<Arc<PgPool>>,
    Extension(boot): Extension<ExtensionBoot>,
    headers: HeaderMap,
    Form(fields): Form<Vec<(String, String)>>,
) -> AdminHtmlResult<Redirect> {
    require_admin_write(&page, &boot, &headers)?;
    let mut parsed = match parse_policy_form(&fields) {
        Ok(p) => p,
        Err(e) => return Ok(not_saved(&e.to_string())),
    };
    let stored = list_policies(&pool).await.map_err(AdminError::Database)?;
    if let Some(row) = stored.iter().find(|row| row.name == parsed.name) {
        carry_unedited_safety(&mut parsed.spec.safety, &row.spec.safety);
        parsed.spec.rate_windows.clone_from(&row.spec.rate_windows);
    }
    upsert_policy(
        &pool,
        &PolicyWrite {
            name: &parsed.name,
            spec: &parsed.spec,
            enabled: parsed.enabled,
            priority: parsed.priority,
        },
    )
    .await?;
    activity::record(
        &pool,
        NewActivity::gateway_policy_saved(&page.user.user_id, &parsed.name, false),
    )
    .await;
    Ok(saved(&parsed.name))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DeleteForm {
    name: String,
}

pub(crate) async fn delete_gateway_policy(
    page: Page,
    State(pool): State<Arc<PgPool>>,
    Extension(boot): Extension<ExtensionBoot>,
    headers: HeaderMap,
    Form(form): Form<DeleteForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin_write(&page, &boot, &headers)?;
    let name = form.name.trim();
    if name.is_empty() {
        return Ok(not_saved("a policy name is required to delete"));
    }
    let deleted = delete_policy(&pool, name)
        .await
        .map_err(AdminError::Database)?;
    if deleted == 0 {
        return Ok(not_saved(&format!("no policy named '{name}'")));
    }
    activity::record(
        &pool,
        NewActivity::gateway_policy_saved(&page.user.user_id, name, true),
    )
    .await;
    Ok(redirect_with_flash(
        BASE_URL,
        FlashTone::Ok,
        &format!("Policy '{name}' deleted. The gateway stops applying it within a minute."),
    ))
}
