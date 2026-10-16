//! `/admin/governance?tab=sync` — the `governance` plane's card:
//! `services/governance/config.yaml` against the staged `governance_chain`.
//!
//! The other three tabs share a time window, a scope and a KPI strip the
//! plane has no use for, so this tab renders its own template with the same
//! strip of tab links and nothing else above the card.

use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;

use super::{BASE_URL, GovernanceTab};
use crate::repositories::sync_planes::governance::PLANE_ID;
use systemprompt_web_admin_core::error::AdminHtmlResult;
use systemprompt_web_admin_core::ssr::sync_plane::{PlaneCardView, plane_card_by_id, sync_tab};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;
use systemprompt_web_admin_core::templates::AdminTemplateEngine;
use systemprompt_web_admin_core::types::{MarketplaceContext, UserContext};

#[derive(Debug, Serialize)]
struct GovernanceSyncPage {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    tabs: Vec<TabLinkView>,
    sync: Option<PlaneCardView>,
    can_write: bool,
}

pub(super) fn sync_link(is_active: bool) -> TabLinkView {
    sync_tab(BASE_URL, is_active)
}

fn tabs() -> Vec<TabLinkView> {
    let mut out: Vec<TabLinkView> = [
        GovernanceTab::Decisions,
        GovernanceTab::Safety,
        GovernanceTab::Hooks,
    ]
    .into_iter()
    .map(|tab| TabLinkView {
        slug: tab.as_str(),
        label: tab.label(),
        href: format!("{BASE_URL}?tab={}", tab.as_str()),
        is_active: false,
        count: None,
    })
    .collect();
    out.push(sync_link(true));
    out
}

pub(super) async fn sync_page(
    pool: &PgPool,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
    engine: &AdminTemplateEngine,
) -> AdminHtmlResult<Response> {
    let page = GovernanceSyncPage {
        page: "governance-sync",
        title: "Governance",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Governance", BASE_URL),
            BreadcrumbView::current("Sync"),
        ],
        tabs: tabs(),
        sync: plane_card_by_id(pool, PLANE_ID, "").await?,
        can_write: user_ctx.is_admin,
    };
    Ok(
        systemprompt_web_admin_core::ssr::helpers::render_typed_page(
            engine,
            "governance-sync",
            &page,
            user_ctx,
            mkt_ctx,
        ),
    )
}
