//! Typed template context for the artifacts list (`artifacts.hbs`).

use serde::Serialize;

use crate::handlers::ssr::format::short_num;
use crate::handlers::ssr::list_view::{
    AnnotatedOption, Chip, Pagination, Preserved, ScopeFilterView, TimeRangeContext,
};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analytics::artifacts::ArtifactStats;

use super::rows::ArtifactRowView;

#[derive(Debug, Serialize)]
pub(super) struct ArtifactsPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) time_range: TimeRangeContext,
    pub(super) filter_ribbon: ArtifactFilterRibbon,
    pub(super) scope_filter: ScopeFilterView,
    pub(super) stats: ArtifactStatsView,
    pub(super) artifacts: Vec<ArtifactRowView>,
    pub(super) has_artifacts: bool,
    pub(super) total_count: i64,
    pub(super) count_label: String,
    pub(super) pagination: Pagination,
    pub(super) structured_only: bool,
    pub(super) all_toggle_url: String,
    pub(super) errors_only: bool,
    pub(super) errors_toggle_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ArtifactFilterRibbon {
    pub(super) base_url: &'static str,
    pub(super) preserved: Vec<Preserved>,
    pub(super) options: ArtifactFilterOptionsView,
    pub(super) chips: Vec<Chip>,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct ArtifactFilterOptionsView {
    pub(super) users: Vec<AnnotatedOption>,
}

#[derive(Debug, Serialize)]
pub(super) struct ArtifactStatsView {
    pub(super) artifacts: i64,
    pub(super) structured: i64,
    pub(super) errors: i64,
    pub(super) ui: i64,
    pub(super) inferred: i64,
    pub(super) with_findings: i64,
    pub(super) bytes_display: String,
    pub(super) tools: i64,
    pub(super) sessions: i64,
}

pub(super) fn stats_view(stats: &ArtifactStats) -> ArtifactStatsView {
    ArtifactStatsView {
        artifacts: stats.artifacts,
        structured: stats.structured,
        errors: stats.errors,
        ui: stats.ui,
        inferred: stats.inferred,
        with_findings: stats.with_findings,
        bytes_display: format_bytes(stats.bytes),
        tools: stats.tools,
        sessions: stats.sessions,
    }
}

pub(super) fn format_bytes(bytes: i64) -> String {
    if bytes <= 0 {
        return "\u{2014}".to_owned();
    }
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let kib = bytes / 1024;
    if kib < 1024 {
        return format!("{kib} KiB");
    }
    format!("{} MiB", short_num(kib / 1024))
}
