//! Shaping the loaded tab data into the detail page's context.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::repositories::scope::defaults::ScopeDefaults;
use crate::repositories::users::enrolment::{UserCommitRow, UserDeviceRow};
use crate::repositories::users::sessions::SigninSessionRow;

use super::context::{
    CommitRowView, ConversationRowView, DeviceRowView, DevicesTabView, IdentityTabView,
    MembershipTabView, ScopeDefaultOptionView, UsageTabView, UserModelRowView, UserSessionRowView,
    UserSessionsTabView,
};
use super::load::{IdentityData, MembershipData, UsageData};

const SLACK_ISSUER: &str = "https://slack.com";

pub(super) fn identity_tab(
    detail: &crate::types::UserDetail,
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
        salesforce_username: data.salesforce_username.clone().unwrap_or_default(),
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
            defaults.primary_group_id.as_deref(),
            "No primary group",
        ),
        primary_project_options: scope_options(
            &data.projects,
            defaults.primary_project_id.as_deref(),
            "No primary project",
        ),
        scope_source_is_manual: defaults.source == "manual",
        scope_source: defaults.source,
        group_choices: data.groups,
        project_choices: data.projects,
    }
}

// Why: only a container this person actually belongs to may be their primary
// one — attributing their spend to a group they are not in would put a cost on
// a page whose member list cannot explain it.
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
    let active_count = rows.iter().filter(|r| r.revoked_at.is_none()).count();
    DevicesTabView {
        count: rows.len(),
        active_count,
        has_rows: !rows.is_empty(),
        rows: rows.into_iter().map(device_row).collect(),
    }
}

fn device_row(row: UserDeviceRow) -> DeviceRowView {
    let revoked = row.revoked_at.is_some();
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
        status_label: if revoked { "Revoked" } else { "Active" },
        status_tone: if revoked { "muted" } else { "ok" },
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
    let live_count = rows.iter().filter(|r| r.revoked_at.is_none()).count();
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

fn sessions_pagination(user_id: &UserId, window: PageWindow) -> Pagination {
    let page = window.index;
    let prefix = format!(
        "/admin/users/{}?tab=sessions&",
        urlencoding::encode(user_id.as_str())
    );
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page - 1));
    let next_url = (page + 1 < window.total_pages).then(|| format!("{prefix}page={}", page + 1));
    let (first_row, last_row) = window.bounds();
    Pagination {
        current_page: page + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun: window.noun,
        has_prev: prev_url.is_some(),
        has_next: next_url.is_some(),
        prev_url,
        next_url,
    }
}

pub(super) fn usage_tab(data: UsageData, user_id: &UserId) -> UsageTabView {
    let encoded = urlencoding::encode(user_id.as_str()).into_owned();
    UsageTabView {
        window_label: "Gateway traffic through /v1/messages, all time.".to_owned(),
        requests: data.summary.requests,
        conversations: data.summary.conversations,
        failed: data.summary.failed,
        tokens_display: format_token_total(data.summary.tokens),
        cost_display: format_cost(data.summary.cost_microdollars),
        first_request_at: stamp(data.summary.first_request_at),
        last_request_at: stamp(data.summary.last_request_at),
        has_models: !data.models.is_empty(),
        models: data
            .models
            .into_iter()
            .map(|share| UserModelRowView {
                model: share.model,
                requests: share.requests,
                tokens_display: format_token_total(share.tokens),
                cost_display: format_cost(share.cost_microdollars),
            })
            .collect(),
        has_conversations: !data.conversations.is_empty(),
        conversations_rows: data
            .conversations
            .iter()
            .map(|c| ConversationRowView {
                title: c.title.clone(),
                short_id: c.context_id_short.clone(),
                url: c.context_url.clone(),
                model: c.model.clone().unwrap_or_else(|| "—".to_owned()),
                requests: c.request_count,
                messages: c.message_count,
                errors: c.error_count,
                has_errors: c.has_errors,
                cost_display: c.cost_display.clone(),
                last_seen: c.last_seen_local.clone().unwrap_or_default(),
            })
            .collect(),
        conversations_url: format!("/admin/contexts?user_id={encoded}"),
        log_url: format!("/admin/requests?user_id={encoded}"),
        has_commits: !data.commits.is_empty(),
        commits: data.commits.into_iter().map(commit_row).collect(),
    }
}

fn commit_row(row: UserCommitRow) -> CommitRowView {
    CommitRowView {
        short_hash: row.commit_hash.chars().take(8).collect(),
        message: row.message.lines().next().unwrap_or_default().to_owned(),
        branch: row.branch.unwrap_or_default(),
        files_changed: row.files_changed,
        insertions: row.insertions,
        deletions: row.deletions,
        committed_at: stamp(Some(row.committed_at)),
    }
}

// Why: one date format across every tab. An absent timestamp renders as an
// em dash, never as the epoch or as "now".
pub(super) fn stamp(value: Option<DateTime<Utc>>) -> String {
    value.map_or_else(
        || "—".to_owned(),
        |t| t.format("%Y-%m-%d %H:%M").to_string(),
    )
}
