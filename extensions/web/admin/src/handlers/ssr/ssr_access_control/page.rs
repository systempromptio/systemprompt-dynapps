//! The page-level context for `/admin/access-control`: the drift banner and
//! the struct the `access-control` template renders, assembled from the
//! per-tab views in [`super::view`]. `can_write` is the MANAGE tier; a project
//! manager reads every tab and changes nothing.

use serde::Serialize;

use super::entities::{DOCS_URL, SYNC_URL};
use super::person::PersonCheckView;
use super::view::{AcEntitiesView, AcStatsView, AudienceGridView};
use super::{AcTab, TabBody, tabs};
use systemprompt_web_admin_core::repositories::access_rules::drift::{DriftCounts, DriftReport};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::tabs::TabLinkView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DriftBannerView {
    pub counts: DriftCounts,
    pub url: &'static str,
    pub declared: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct AccessControlPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub stats: AcStatsView,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub tabs: Vec<TabLinkView>,
    pub on_rules: bool,
    pub on_audience: bool,
    pub on_person: bool,
    pub docs_url: &'static str,
    pub sync_url: &'static str,
    pub drift: Option<DriftBannerView>,
    pub declared_unreadable: Option<String>,
    pub entities: AcEntitiesView,
    pub audience: AudienceGridView,
    pub person: Option<PersonCheckView>,
}

pub(super) struct AcPageParts {
    pub tab: AcTab,
    pub can_write: bool,
    pub stats: AcStatsView,
    pub drift: Option<DriftBannerView>,
    pub declared_unreadable: Option<String>,
    pub entities: AcEntitiesView,
    pub body: TabBody,
}

pub(super) fn drift_banner(
    drift: Option<&DriftReport>,
    declared: usize,
) -> Option<DriftBannerView> {
    drift.filter(|d| !d.is_clean()).map(|d| DriftBannerView {
        counts: d.counts(),
        url: SYNC_URL,
        declared,
    })
}

pub(super) fn page_data(parts: AcPageParts) -> AccessControlPageData {
    let tab = parts.tab;
    AccessControlPageData {
        page: "access-control",
        title: "Access control",
        can_write: parts.can_write,
        stats: parts.stats,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("People & access", "/admin/users"),
            BreadcrumbView::current("Access control"),
        ],
        tabs: tabs(tab),
        on_rules: tab == AcTab::Rules,
        on_audience: tab == AcTab::Audience,
        on_person: tab == AcTab::Person,
        docs_url: DOCS_URL,
        sync_url: SYNC_URL,
        drift: parts.drift,
        declared_unreadable: parts.declared_unreadable,
        entities: parts.entities,
        audience: parts.body.audience,
        person: parts.body.person,
    }
}
