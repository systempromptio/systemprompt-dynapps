//! Template contexts for the runs explorer (`farm-runs.hbs`) and the live
//! partial (`farm/runs-live`) that both the page and its fragment render, so
//! a refreshed row can never read differently from the first paint.

use serde::Serialize;
use systemprompt_web_admin_core::ssr::list_view::{
    AnnotatedOption, Chip, Pagination, Preserved, TimeRangeContext,
};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::table::SortHeaderView;

use crate::handlers::ssr::shared::RunRowView;

#[derive(Debug, Serialize)]
pub(super) struct RunsPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) time_range: TimeRangeContext,
    pub(super) filters: RunsFilterView,
    pub(super) total_count: i64,
    pub(super) fragment_url: &'static str,
    pub(super) live: RunsLiveView,
}

#[derive(Debug, Serialize)]
pub(super) struct RunsFilterView {
    pub(super) base_url: &'static str,
    pub(super) preserved: Vec<Preserved>,
    pub(super) farms: Vec<AnnotatedOption>,
    pub(super) agents: Vec<AnnotatedOption>,
    pub(super) users: Vec<AnnotatedOption>,
    pub(super) statuses: Vec<AnnotatedOption>,
    pub(super) triggers: Vec<AnnotatedOption>,
    pub(super) chips: Vec<Chip>,
    pub(super) clear_url: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct RunsLiveView {
    pub(super) live_active: bool,
    pub(super) kpis: Vec<RunKpiView>,
    pub(super) rows: Vec<RunRowView>,
    pub(super) has_rows: bool,
    pub(super) filtered: bool,
    pub(super) pagination: Pagination,
    pub(super) sort_headers: RunSortHeaders,
}

#[derive(Debug, Serialize)]
pub(super) struct RunKpiView {
    pub(super) key: &'static str,
    pub(super) label: &'static str,
    pub(super) value: String,
    pub(super) tone: Option<&'static str>,
    pub(super) href: Option<String>,
    pub(super) active: bool,
    pub(super) note: Option<String>,
    pub(super) hint: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub(super) struct RunSortHeaders {
    pub(super) started: SortHeaderView,
    pub(super) duration: SortHeaderView,
    pub(super) tool_calls: SortHeaderView,
    pub(super) cost: SortHeaderView,
}
