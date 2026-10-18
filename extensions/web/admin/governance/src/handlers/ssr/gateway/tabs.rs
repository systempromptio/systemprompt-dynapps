//! The `/admin/gateway` tab strip: four views of the routing table, its
//! Sync tab (the `gateway_routes` plane), and Policies, which lives on its
//! own page.

use systemprompt_web_admin_core::ssr::sync_plane::sync_tab;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

pub(super) const BASE_URL: &str = "/admin/gateway";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GatewayTab {
    Overview,
    Providers,
    Routes,
    Settings,
    Sync,
}

impl GatewayTab {
    pub(super) fn parse(tab: Option<&str>) -> Self {
        match tab {
            Some("providers") => Self::Providers,
            Some("routes" | "resolved") => Self::Routes,
            Some("settings") => Self::Settings,
            Some(t) if systemprompt_web_admin_core::ssr::sync_plane::is_sync_tab(Some(t)) => {
                Self::Sync
            },
            _ => Self::Overview,
        }
    }
}

pub(super) fn view_tabs(active: GatewayTab, routes: usize, providers: usize) -> Vec<TabLinkView> {
    let link = |slug, label, href: &str, tab: Option<GatewayTab>, count| TabLinkView {
        slug,
        label,
        href: href.to_owned(),
        is_active: tab == Some(active),
        count,
    };
    vec![
        link(
            "overview",
            "Overview",
            BASE_URL,
            Some(GatewayTab::Overview),
            None,
        ),
        link(
            "providers",
            "Providers",
            "/admin/gateway?tab=providers",
            Some(GatewayTab::Providers),
            Some(i64::try_from(providers).unwrap_or(i64::MAX)),
        ),
        link(
            "routes",
            "Routes",
            "/admin/gateway?tab=routes",
            Some(GatewayTab::Routes),
            Some(i64::try_from(routes).unwrap_or(i64::MAX)),
        ),
        link(
            "settings",
            "Settings",
            "/admin/gateway?tab=settings",
            Some(GatewayTab::Settings),
            None,
        ),
        sync_tab(BASE_URL, active == GatewayTab::Sync),
        link(
            "policies",
            "Policies",
            "/admin/gateway/policies",
            None,
            None,
        ),
    ]
}
