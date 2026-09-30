//! Shaping the loaded tab data into the detail page's context.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::{GroupId, ProjectId};

pub(super) use super::access_view::access_tab;

use crate::repositories::users::enrolment::UserDeviceRow;
use crate::repositories::users::sessions::SigninSessionRow;
use systemprompt_web_admin_core::repositories::bridge::presence::bridge_presence;
use systemprompt_web_admin_core::repositories::scope::defaults::ScopeDefaults;
use systemprompt_web_admin_core::ssr::history_rows::{HistoryRowView, HistoryView, row_view};
use systemprompt_web_admin_core::ssr::list_view::{PageWindow, Pagination};

use super::context::{
    DeviceRowView, DevicesTabView, IdentityTabView, MembershipTabView, SalesforceIdentityView,
    ScopeDefaultOptionView, UserConversationsTabView, UserSessionRowView, UserSessionsTabView,
};
use super::load::{IdentityData, MembershipData, UserConversationsData};
use systemprompt_web_admin_core::types::UserContext;
use systemprompt_web_admin_platform::connector_oauth::Provider;

pub(super) use super::usage::usage_tab;
use systemprompt_web_admin_core::ssr::list_view::paginate_prefixed;

const SLACK_ISSUER: &str = "https://slack.com";

pub(super) fn export(
    user_id: &UserId,
    viewer: &UserContext,
) -> Option<systemprompt_web_admin_core::export::view::ExportView> {
    viewer.is_console.then(|| {
        let query = systemprompt_web_admin_core::export::view::query_string(&[(
            "user_id",
            Some(user_id.as_str()),
        )]);
        systemprompt_web_admin_core::export::view::ExportView::new(
            &["requests", "sessions", "conversations"],
            &query,
        )
        .with_transcripts(
            systemprompt_web_admin_core::export::transcripts::TranscriptSource::Sessions,
        )
    })
}

pub(super) fn identity_tab(
    detail: &systemprompt_web_admin_core::types::UserDetail,
    data: &IdentityData,
    role_choices: Vec<crate::handlers::ssr::types::RoleChoiceView>,
) -> IdentityTabView {
    let primary = data.identities.first();
    IdentityTabView {
        display_name: detail.display_name.clone().unwrap_or_default(),
        email: detail
            .email
            .as_ref()
            .map(|e| e.as_str().to_owned())
            .unwrap_or_default(),
        is_active: detail.is_active,
        created_at: stamp(Some(detail.created_at)),
        role_choices,
        roles_valid_until_day: data
            .manual_roles_valid_until
            .map(|t| t.format("%Y-%m-%d").to_string())
            .unwrap_or_default(),
        has_adfs_groups: !data.adfs_groups.is_empty(),
        adfs_groups: data.adfs_groups.clone(),
        idp_issuer: primary.map(|i| i.issuer.clone()).unwrap_or_default(),
        external_sub: primary.map(|i| i.external_sub.clone()).unwrap_or_default(),
        linked_at: primary.map_or_else(String::new, |i| stamp(Some(i.linked_at))),
        slack_user_id: data
            .identities
            .iter()
            .find(|i| i.issuer == SLACK_ISSUER)
            .map(|i| i.external_sub.clone())
            .unwrap_or_default(),
        salesforce_identities: data
            .salesforce_identities
            .iter()
            .map(|identity| SalesforceIdentityView {
                label: Provider::try_from(identity.provider.clone())
                    .map_or_else(|_| identity.provider.clone(), |p| p.display_name()),
                provider: identity.provider.clone(),
                sf_username: identity.sf_username.clone(),
            })
            .collect(),
        share_token_version: data.share_token_version,
    }
}

pub(super) fn membership_tab(data: MembershipData) -> MembershipTabView {
    let defaults = data.defaults.unwrap_or_else(|| ScopeDefaults {
        primary_group_id: None,
        primary_project_id: None,
        source: "auto".to_owned(),
    });
    MembershipTabView {
        primary_group_options: scope_options(
            &data.groups,
            defaults.primary_group_id.as_ref().map(GroupId::as_str),
            "No primary group",
        ),
        primary_project_options: scope_options(
            &data.projects,
            defaults.primary_project_id.as_ref().map(ProjectId::as_str),
            "No primary project",
        ),
        scope_source_is_manual: defaults.source == "manual",
        scope_source: defaults.source,
        group_choices: data.groups,
        project_choices: data.projects,
    }
}

fn scope_options(
    choices: &[crate::handlers::ssr::types::MembershipChoiceView],
    selected: Option<&str>,
    none_label: &str,
) -> Vec<ScopeDefaultOptionView> {
    let mut out = vec![ScopeDefaultOptionView {
        value: String::new(),
        label: none_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(
        choices
            .iter()
            .filter(|choice| choice.held)
            .map(|choice| ScopeDefaultOptionView {
                selected: selected == Some(choice.id.as_str()),
                value: choice.id.clone(),
                label: choice.name.clone(),
            }),
    );
    out
}

pub(super) fn devices_tab(rows: Vec<UserDeviceRow>) -> DevicesTabView {
    let now = Utc::now();
    let rows: Vec<DeviceRowView> = rows.into_iter().map(|r| device_row(r, now)).collect();
    let active_count = rows.iter().filter(|r| r.is_active).count();
    DevicesTabView {
        count: rows.len(),
        active_count,
        has_rows: !rows.is_empty(),
        rows,
    }
}

fn device_state(row: &UserDeviceRow, now: DateTime<Utc>) -> (&'static str, &'static str, bool) {
    if row.revoked_at.is_some() {
        return ("Revoked", "muted", false);
    }
    if row.kind != "bridge" {
        return ("Active", "ok", true);
    }
    row.last_seen_at.map_or(("Silent", "muted", false), |last| {
        let presence = bridge_presence(now, last);
        (presence.label(), presence.tone(), presence.is_online())
    })
}

fn device_row(row: UserDeviceRow, now: DateTime<Utc>) -> DeviceRowView {
    let revoked = row.revoked_at.is_some();
    let (status_label, status_tone, is_active) = device_state(&row, now);
    DeviceRowView {
        kind_label: match row.kind.as_str() {
            "bridge" => "Bridge",
            "cert" => "Certificate",
            _ => "Token",
        },
        label: row.label,
        detail: row.detail.unwrap_or_default(),
        created_at: stamp(row.created_at),
        last_seen: stamp(row.last_seen_at),
        status_label,
        status_tone,
        is_active,
        revocable: row.revocable && !revoked,
        kind: row.kind,
        id: row.id,
    }
}

pub(super) fn sessions_tab(
    rows: &[SigninSessionRow],
    user_id: &UserId,
    page: i64,
    page_size: i64,
) -> UserSessionsTabView {
    let now = Utc::now();
    let live_count = rows
        .iter()
        .filter(|r| r.revoked_at.is_none() && r.expires_at.is_none_or(|e| e >= now))
        .count();
    let total = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    let offset = usize::try_from(page * page_size).unwrap_or(0);
    let take = usize::try_from(page_size).unwrap_or(50);
    let slice: Vec<UserSessionRowView> = rows
        .iter()
        .skip(offset)
        .take(take)
        .map(session_row)
        .collect();
    let shown = i64::try_from(slice.len()).unwrap_or(0);
    let window = PageWindow::new(page, page_size, total, shown, "sessions");
    UserSessionsTabView {
        has_rows: !slice.is_empty(),
        pagination: sessions_pagination(user_id, window),
        rows: slice,
        live_count,
    }
}

fn session_row(row: &SigninSessionRow) -> UserSessionRowView {
    let revoked = row.revoked_at.is_some();
    let expired = row.expires_at.is_some_and(|e| e < Utc::now());
    let id = row.session_id.as_str();
    UserSessionRowView {
        session_id: row.session_id.clone(),
        short_id: id.chars().take(12).collect(),
        detail_url: format!("/admin/sessions/{}", urlencoding::encode(id)),
        source: row.session_source.clone().unwrap_or_else(|| "—".to_owned()),
        ip: row.ip_address.clone().unwrap_or_default(),
        user_agent: row.user_agent.clone().unwrap_or_default(),
        requests: row.request_count,
        started_at: stamp(Some(row.started_at)),
        last_activity: stamp(Some(row.last_activity_at)),
        status_label: if revoked {
            "Revoked"
        } else if expired {
            "Expired"
        } else {
            "Live"
        },
        status_tone: if revoked {
            "muted"
        } else if expired {
            "warn"
        } else {
            "ok"
        },
        revocable: !revoked,
    }
}

pub(super) fn conversations_tab(
    data: &UserConversationsData,
    user_id: &UserId,
    viewer: &UserContext,
    page: i64,
    page_size: i64,
) -> UserConversationsTabView {
    let rows: Vec<HistoryRowView> = data
        .items
        .iter()
        .map(|item| row_view(item, viewer, HistoryView::Org))
        .collect();
    let shown = i64::try_from(rows.len()).unwrap_or(0);
    let window = PageWindow::new(page, page_size, data.total, shown, "conversations");
    UserConversationsTabView {
        has_rows: !rows.is_empty(),
        pagination: tab_pagination(user_id, "conversations", window),
        rows,
    }
}

fn sessions_pagination(user_id: &UserId, window: PageWindow) -> Pagination {
    tab_pagination(user_id, "sessions", window)
}

fn tab_pagination(user_id: &UserId, tab: &str, window: PageWindow) -> Pagination {
    let prefix = format!(
        "/admin/users/{}?tab={tab}&",
        urlencoding::encode(user_id.as_str())
    );
    paginate_prefixed(window, &prefix)
}

pub(super) fn stamp(value: Option<DateTime<Utc>>) -> String {
    value.map_or_else(
        || "—".to_owned(),
        |t| t.format("%Y-%m-%d %H:%M").to_string(),
    )
}
