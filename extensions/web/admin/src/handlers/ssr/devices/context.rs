//! Template context for `/admin/devices`.
//!
//! Four row types, one page. They are separate structs rather than one
//! flattened row because the four tables share only a person: a bridge has a
//! heartbeat and no state, a token has a state and no heartbeat, and a
//! rendering that pretended otherwise would have to invent a value for half
//! its own columns.

use serde::Serialize;

use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};

#[derive(Debug, Serialize)]
pub(super) struct DevicesPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) tabs: Vec<TabLinkView>,
    pub(super) tab: &'static str,
    pub(super) stats: FleetStatsView,
    pub(super) versions: Vec<VersionBarView>,
    pub(super) has_versions: bool,
    pub(super) filters: Vec<DeviceChipView>,
    pub(super) sort_headers: Vec<DeviceSortHeaderView>,
    pub(super) sessions: Vec<BridgeSessionRowView>,
    pub(super) pats: Vec<ApiKeyRowView>,
    pub(super) certs: Vec<CertRowView>,
    pub(super) links: Vec<LinkRowView>,
    pub(super) count_label: String,
    pub(super) has_rows: bool,
    pub(super) empty_message: &'static str,
    pub(super) pagination: Pagination,
    // Why: the revoke buttons, not the page. A project manager reads the
    // fleet; only the admin roles may disable a credential, and the API
    // refuses them anyway — this stops the console offering a button that
    // would come back 403.
    pub(super) can_manage: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct FleetStatsView {
    pub(super) bridges_active: String,
    pub(super) bridges_stale: String,
    pub(super) bridges_total_sub: String,
    pub(super) stale_tone: &'static str,
    pub(super) stale_url: String,
    pub(super) stale_active: bool,
    pub(super) versions: String,
    pub(super) versions_sub: String,
    pub(super) pats_active: String,
    pub(super) pats_sub: String,
    pub(super) certs_active: String,
    pub(super) certs_sub: String,
    pub(super) links_pending: String,
    pub(super) links_sub: String,
    pub(super) links_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct VersionBarView {
    pub(super) label: String,
    pub(super) devices: i64,
    // Why: a percentage rather than a pixel width, so the bar is drawn from
    // the same number the label states and cannot drift from it.
    pub(super) pct: i64,
    pub(super) is_latest: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct DeviceChipView {
    pub(super) label: &'static str,
    pub(super) url: String,
    pub(super) active: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct DeviceSortHeaderView {
    pub(super) label: &'static str,
    pub(super) class: &'static str,
    pub(super) hint: &'static str,
    pub(super) url: String,
    pub(super) active: bool,
    pub(super) aria_sort: &'static str,
    pub(super) indicator: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct BridgeSessionRowView {
    pub(super) session_id: String,
    pub(super) hostname: String,
    pub(super) os: String,
    pub(super) version: String,
    pub(super) user_name: String,
    pub(super) user_email: String,
    pub(super) user_url: String,
    pub(super) started_display: String,
    pub(super) heartbeat_display: String,
    pub(super) forwarded_display: String,
    pub(super) tokens_display: String,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ApiKeyRowView {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) key_prefix: String,
    pub(super) user_name: String,
    pub(super) user_url: String,
    pub(super) created_display: String,
    pub(super) used_display: String,
    pub(super) expires_display: String,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
    pub(super) can_revoke: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct CertRowView {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) fingerprint: String,
    pub(super) user_name: String,
    pub(super) user_url: String,
    pub(super) enrolled_display: String,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
    pub(super) can_revoke: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct LinkRowView {
    pub(super) user_name: String,
    pub(super) user_email: String,
    pub(super) user_url: String,
    pub(super) created_display: String,
    pub(super) expires_display: String,
    pub(super) status_label: &'static str,
    pub(super) status_tone: &'static str,
}
