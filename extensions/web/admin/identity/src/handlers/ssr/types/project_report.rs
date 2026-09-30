//! Template context for `/admin/projects/{id}/report`: the one-page account
//! of a project that is printed or saved as PDF.
//!
//! Every figure is exclusive attribution, the same slice the listing and the
//! detail page show, so the report never disagrees with the console it was
//! opened from. The members table is the one member-attributed section and
//! the template labels it so.

use serde::Serialize;
use systemprompt_web_shared::ProjectId;

use super::{
    AttributionSourceRowView, MemberRowView, MemberSetChipView, ModelMixRowView, ProjectKpiView,
    ProjectSessionRowView, ProjectSkillRowView, ProjectToolRowView,
};
use systemprompt_web_admin_core::ssr::types::breadcrumb::BreadcrumbView;
use systemprompt_web_admin_core::ssr::types::svg_line::SvgLineChartView;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectAgentRowView {
    pub client_kind: String,
    pub label: String,
    pub requests: i64,
    pub users: i64,
    pub tokens_display: String,
    pub cost_display: String,
    pub models: i64,
    pub share_pct: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectArtifactRowView {
    pub server_name: String,
    pub artifact_type: String,
    pub artifacts: i64,
    pub errors: i64,
    pub users: i64,
    pub last_created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProjectReportFactView {
    pub label: &'static str,
    pub value: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ProjectReportPageData {
    pub page: &'static str,
    pub title: String,
    pub project_id: ProjectId,
    pub project_name: String,
    pub description: Option<String>,
    pub window_label: String,
    pub generated_at: String,
    pub generated_by: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub detail_href: String,
    pub print_on_load: bool,
    pub facts: Vec<ProjectReportFactView>,
    pub kpis: Vec<ProjectKpiView>,
    pub groups_represented: Vec<MemberSetChipView>,
    pub group_count: i64,
    pub members: Vec<MemberRowView>,
    pub member_count: i64,
    pub attribution: Vec<AttributionSourceRowView>,
    pub daily: SvgLineChartView,
    pub daily_cost: SvgLineChartView,
    pub models: Vec<ModelMixRowView>,
    pub agents: Vec<ProjectAgentRowView>,
    pub skills: Vec<ProjectSkillRowView>,
    pub tools: Vec<ProjectToolRowView>,
    pub artifacts: Vec<ProjectArtifactRowView>,
    pub artifact_total: i64,
    pub sessions: Vec<ProjectSessionRowView>,
    pub export: systemprompt_web_admin_core::export::view::ExportView,
}
