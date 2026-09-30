//! Loading a tab as people, and shaping each person for the template.
//!
//! This file holds the shared shaping and the bridge tab; the three
//! credential tabs are in `credentials.rs` and use the same helpers.
//! Every loader reads twice: a page of people with their totals, then the
//! items belonging to the people on that page, stitched back under them in
//! order. A read that fails renders an empty tab and is listed on the page
//! as unavailable rather than becoming an error page: the KPI strip above it
//! comes from different statements and is still worth showing, and a fleet
//! page that refuses to render because one of four tables was slow is worse
//! than one that says which tab could not be read.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::devices::sessions::{
    self, BridgeHostRow, BridgeUserSessionsRow, SessionQuery,
};
use systemprompt_web_admin_core::repositories::bridge::presence::bridge_presence;
use systemprompt_web_admin_core::ssr::faults::Faults;
use systemprompt_web_admin_core::ssr::format::{format_token_total, short_num};

use super::context::{BridgeHostRowView, CellView, UserGroupView};

pub(super) const DATE: &str = "sp-table__cell--date";
pub(super) const NUM: &str = "sp-table__cell--num sp-u-num";

pub(super) fn stamp(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M").to_string()
}

pub(super) fn maybe_stamp(value: Option<DateTime<Utc>>) -> String {
    value.map_or_else(|| "—".to_owned(), stamp)
}

pub(super) const fn cell(value: String, class: &'static str) -> CellView {
    CellView { value, class }
}

pub(super) fn plural(n: i64, noun: &str) -> String {
    if n == 1 {
        format!("1 {noun}")
    } else {
        format!("{n} {noun}s")
    }
}

pub(super) fn ids(users: impl Iterator<Item = UserId>) -> Vec<String> {
    users.map(|id| id.as_str().to_owned()).collect()
}

pub(super) fn group(user_id: &UserId, user_name: &str, tab: &str) -> UserGroupView {
    let id = user_id.as_str();
    UserGroupView {
        user_name: user_name.to_owned(),
        user_url: format!("/admin/users/{}", urlencoding::encode(id)),
        detail_id: format!("devices-{tab}-{}", urlencoding::encode(id)),
        count_label: String::new(),
        summary: String::new(),
        cells: Vec::new(),
        status_label: "",
        status_tone: "",
        sessions: Vec::new(),
        pats: Vec::new(),
        certs: Vec::new(),
        links: Vec::new(),
    }
}

pub(super) fn credential_summary(total: i64, active: i64) -> String {
    let inactive = total - active;
    if inactive == 0 {
        "all active".to_owned()
    } else if active == 0 {
        "none active".to_owned()
    } else {
        format!("{active} active · {inactive} not")
    }
}

pub(super) const fn credential_state(revoked: bool, expired: bool) -> (&'static str, &'static str) {
    if revoked {
        ("Revoked", "muted")
    } else if expired {
        ("Expired", "warn")
    } else {
        ("Active", "ok")
    }
}

pub(super) const fn rollup_state(active: i64, any_revoked: bool) -> (&'static str, &'static str) {
    if active > 0 {
        ("Active", "ok")
    } else if any_revoked {
        ("Revoked", "muted")
    } else {
        ("Expired", "warn")
    }
}

pub(super) async fn load_sessions(
    pool: &PgPool,
    query: SessionQuery<'_>,
    faults: &mut Faults,
) -> (Vec<UserGroupView>, i64) {
    let (users, total) = faults.take(
        "bridge users",
        sessions::list_bridge_users_paged(pool, query).await,
    );
    let user_ids = ids(users.iter().map(|u| u.user_id.clone()));
    let hosts = faults.take(
        "bridge hosts",
        sessions::list_bridge_hosts_for_users(pool, &user_ids, query.stale_only).await,
    );
    let groups = users
        .iter()
        .map(|user| session_group(user, &hosts))
        .collect();
    (groups, total)
}

fn session_group(user: &BridgeUserSessionsRow, hosts: &[BridgeHostRow]) -> UserGroupView {
    let mut view = group(&user.user_id, &user.user_name, "bridges");
    view.sessions = hosts
        .iter()
        .filter(|h| h.user_id == user.user_id)
        .map(host_row)
        .collect();
    view.count_label = plural(user.hosts, "host");
    view.summary = view
        .sessions
        .iter()
        .map(|h| h.hostname.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    view.cells = vec![
        cell(user.latest_version.clone(), "sp-col-version sp-u-mono"),
        cell(stamp(user.first_started_at), DATE),
        cell(stamp(user.last_heartbeat_at), DATE),
        cell(short_num(user.forwarded_total), NUM),
        cell(format_token_total(user.tokens_total), NUM),
    ];
    let presence = bridge_presence(Utc::now(), user.last_heartbeat_at);
    (view.status_label, view.status_tone) = (presence.label(), presence.tone());
    view
}

fn host_row(row: &BridgeHostRow) -> BridgeHostRowView {
    BridgeHostRowView {
        hostname: row.hostname.clone(),
        os: row.os.clone(),
        version: row.bridge_version.clone(),
        started_display: stamp(row.started_at),
        heartbeat_display: stamp(row.last_heartbeat_at),
        forwarded_display: short_num(row.forwarded_total),
        tokens_display: format_token_total(row.tokens_total),
        sessions_display: (row.session_count > 1).then(|| plural(row.session_count, "session")),
        status_label: bridge_presence(Utc::now(), row.last_heartbeat_at).label(),
        status_tone: bridge_presence(Utc::now(), row.last_heartbeat_at).tone(),
    }
}
