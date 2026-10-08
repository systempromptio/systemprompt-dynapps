//! View-model types for `/admin/mcp` and `/admin/mcp/{id}`.
//!
//! An MCP server has two halves that can disagree, and the pages exist to show
//! the disagreement: the declaration in `services/mcp/*.yaml`, and the runtime
//! record of connections and tool calls. A server declared but never connected
//! and a server serving traffic under a name nothing declares are both real
//! states, so `configured` and the runtime figures are separate fields.

use serde::Serialize;
use systemprompt::identifiers::{McpExecutionId, McpServerId};

use systemprompt_web_admin_core::export::view::ExportView;
use systemprompt_web_admin_core::ssr::list_view::{Pagination, TimeRangeContext};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::table::SortHeaderView;

use super::super::view::LinkedEntity;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct McpServerRow {
    pub id: String,
    pub description: String,
    pub detail_url: String,
    pub access_url: String,
    pub source_path: String,
    pub configured: bool,
    pub enabled: bool,
    pub status_label: &'static str,
    pub status_tone: &'static str,
    pub server_type: String,
    pub transport: String,
    pub auth_label: String,
    pub oauth_required: bool,
    pub alive: bool,
    pub connections: i64,
    pub connected_users: i64,
    pub last_seen_display: String,
    pub calls: i64,
    pub prior_calls: i64,
    pub succeeded: i64,
    pub failures: i64,
    pub timeouts: i64,
    pub errors: i64,
    pub error_rate_display: String,
    pub error_tone: &'static str,
    pub attested: i64,
    pub attested_display: String,
    pub avg_ms: Option<f64>,
    pub p95_ms: Option<f64>,
    pub avg_display: String,
    pub p95_display: String,
    pub distinct_users: i64,
    pub distinct_sessions: i64,
    pub distinct_tools: i64,
    pub payload_bytes: i64,
    pub payload_display: String,
    pub secret_redactions: i64,
    pub delta_display: String,
    pub delta_dir: &'static str,
    #[serde(skip)]
    pub last_call_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_call_display: String,
    pub plugin_count: usize,
    pub assignment_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct McpKpiView {
    pub label: &'static str,
    pub value: String,
    pub note: String,
    pub tone: &'static str,
    pub unit: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct McpPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub window_label: String,
    pub heartbeat_label: String,
    pub time_range: TimeRangeContext,
    pub window_from: String,
    pub window_to: String,
    pub export: ExportView,
    pub kpis: Vec<McpKpiView>,
    pub sort_headers: Vec<SortHeaderView>,
    pub servers: Vec<McpServerRow>,
    pub servers_count: usize,
    pub unconfigured_count: usize,
    pub builtin_calls: i64,
    pub access_control_url: &'static str,
    pub sort_key: String,
    pub sort_dir: String,
    pub search: String,
    pub preserved_query: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct McpToolRow {
    pub tool_name: String,
    pub calls: i64,
    pub failures: i64,
    pub error_rate_display: String,
    pub error_tone: &'static str,
    pub attested: i64,
    pub distinct_users: i64,
    pub avg_display: String,
    pub p95_display: String,
    pub max_display: String,
    pub payload_display: String,
    pub last_call_display: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct McpCallerRowView {
    pub caller: String,
    pub user_url: String,
    pub calls: i64,
    pub failures: i64,
    pub distinct_tools: i64,
    pub last_call_display: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct McpExecutionRowView {
    pub execution_id: McpExecutionId,
    pub short_id: String,
    pub tool_name: String,
    pub source: String,
    pub source_label: &'static str,
    pub source_tone: &'static str,
    pub status: String,
    pub status_tone: &'static str,
    pub started_display: String,
    pub duration_display: String,
    pub caller: String,
    pub user_url: String,
    pub session: String,
    pub session_url: Option<String>,
    pub context_url: Option<String>,
    pub trace_url: Option<String>,
    pub payload_display: String,
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct McpConnectionRowView {
    pub session: String,
    pub short_id: String,
    pub caller: String,
    pub user_url: String,
    pub kind: String,
    pub client_name: String,
    pub connected: bool,
    pub status_label: &'static str,
    pub status_tone: &'static str,
    pub last_seen_display: String,
    pub expires_display: String,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ConfigFactView {
    pub label: &'static str,
    pub value: String,
    pub mono: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct McpDetailData {
    pub page: &'static str,
    pub title: String,
    pub subtitle: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub id: McpServerId,
    pub configured: bool,
    pub enabled: bool,
    pub status_label: &'static str,
    pub status_tone: &'static str,
    pub window_label: String,
    pub time_range: TimeRangeContext,
    pub export: ExportView,
    pub kpis: Vec<McpKpiView>,
    pub tools: Vec<McpToolRow>,
    pub tools_count: usize,
    pub callers: Vec<McpCallerRowView>,
    pub callers_count: usize,
    pub executions: Vec<McpExecutionRowView>,
    pub executions_count: i64,
    pub pagination: Pagination,
    pub connections: Vec<McpConnectionRowView>,
    pub connections_count: usize,
    pub connected_count: usize,
    pub access: systemprompt_web_admin_core::ssr::entity_panel::EntityAccessView,
    pub config_facts: Vec<ConfigFactView>,
    pub oauth_scopes: Vec<String>,
    pub included_by: Vec<LinkedEntity>,
    pub included_by_count: usize,
    pub access_url: String,
}
