//! The part of the user detail page every tab shares: title, breadcrumbs, tab
//! bar, account header and KPI strip, with every tab panel left empty for
//! `fill_active_tab` to populate.
//!
//! The primary group and project tiles show what the next request will be
//! stamped with, so a person with no primary reads as unattributed however
//! many groups they are in.

use systemprompt::identifiers::UserId;

use systemprompt_web_admin_core::repositories::scope::defaults::ScopeDefaults;
use systemprompt_web_admin_core::repositories::scope::membership::UNATTRIBUTED;
use systemprompt_web_admin_core::ssr::format::{format_cost, format_token_total};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::types::{UserContext, UserDetail};

use super::super::BASE_URL;
use super::context::{DetailKpiView, UserDetailContext, UserHeaderView};
use super::load::Headline;
use super::{tab_href, tab_links, view};

pub(super) struct BaseInputs<'a> {
    pub(super) user_id: &'a UserId,
    pub(super) detail: &'a UserDetail,
    pub(super) headline: Headline,
    pub(super) tab: &'static str,
    pub(super) viewer: &'a UserContext,
}

pub(super) fn base_context(inputs: BaseInputs<'_>) -> UserDetailContext {
    let BaseInputs {
        user_id,
        detail,
        headline,
        tab,
        viewer,
    } = inputs;
    UserDetailContext {
        page: "user-detail",
        title: display_name(detail),
        breadcrumbs: vec![
            BreadcrumbView::link("Users", BASE_URL),
            BreadcrumbView::current(display_name(detail)),
        ],
        tabs: tab_links(user_id, tab),
        tab,
        header: header_view(detail, headline.defaults.as_ref()),
        kpis: kpi_view(user_id, detail, &headline),
        can_write: viewer.is_admin,
        is_self: viewer.user_id.as_str() == user_id.as_str(),
        identity: None,
        conversations: None,
        membership: None,
        access: None,
        devices: None,
        sessions: None,
        usage: None,
        export: view::export(user_id, viewer),
    }
}

fn kpi_view(user_id: &UserId, detail: &UserDetail, headline: &Headline) -> DetailKpiView {
    let profile = headline.profile.as_ref();
    DetailKpiView {
        requests_display: format_token_total(headline.summary.requests),
        cost_display: format_cost(headline.summary.cost_microdollars),
        tokens_display: format_token_total(headline.summary.tokens),
        sessions: i64::try_from(detail.sessions.len()).unwrap_or(0),
        devices: 0,
        groups: profile.map_or(0, |p| p.group_ids.len()),
        projects: profile.map_or(0, |p| p.project_ids.len()),
        membership_url: tab_href(user_id, "membership"),
    }
}

fn display_name(detail: &UserDetail) -> String {
    detail
        .display_name
        .clone()
        .unwrap_or_else(|| detail.user_id.as_str().to_owned())
}

fn initials(name: &str) -> String {
    name.split(|c: char| c.is_whitespace() || c == '-' || c == '.' || c == '@')
        .filter(|p| !p.is_empty())
        .take(2)
        .filter_map(|p| p.chars().next())
        .flat_map(char::to_uppercase)
        .collect()
}

fn primary_or_unattributed(primary: Option<String>) -> String {
    primary.unwrap_or_else(|| UNATTRIBUTED.to_owned())
}

fn header_view(detail: &UserDetail, defaults: Option<&ScopeDefaults>) -> UserHeaderView {
    let name = display_name(detail);
    UserHeaderView {
        initials: initials(&name),
        email: detail
            .email
            .as_ref()
            .map(|e| e.as_str().to_owned())
            .unwrap_or_default(),
        has_roles: !detail.roles.is_empty(),
        roles: detail.roles.clone(),
        status_label: if detail.is_active {
            "Active"
        } else {
            "Suspended"
        },
        status_tone: if detail.is_active { "ok" } else { "muted" },
        is_active: detail.is_active,
        created_at: view::stamp(Some(detail.created_at)),
        last_active: view::stamp(detail.last_active),
        primary_group: primary_or_unattributed(
            defaults.and_then(|d| d.primary_group_id.as_ref().map(ToString::to_string)),
        ),
        primary_project: primary_or_unattributed(
            defaults.and_then(|d| d.primary_project_id.as_ref().map(ToString::to_string)),
        ),
        name,
        user_id: detail.user_id.clone(),
    }
}
