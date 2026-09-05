//! Typed template contexts for the demo pages.

use serde::Serialize;

use super::view::{
    KpiView, MatrixView, McpToolStatView, ServerCardView, SkillTotalView, UserTotalView,
};
use crate::handlers::ssr::types::{ChartView, TabLinkView};

#[derive(Debug, Serialize)]
pub(super) struct DemoSkillsContext {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub kpis: Vec<KpiView>,
    pub chart: ChartView,
    pub tabs: Vec<TabLinkView>,
    pub is_by_skill: bool,
    pub is_by_user: bool,
    pub skills: Vec<SkillTotalView>,
    pub has_skills: bool,
    pub matrix: MatrixView,
    pub user_totals: Vec<UserTotalView>,
    pub has_user_totals: bool,
    pub attribution_note: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct DemoToolsContext {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub kpis: Vec<KpiView>,
    pub chart: ChartView,
    pub servers: Vec<ServerCardView>,
    pub has_servers: bool,
    pub tools: Vec<McpToolStatView>,
    pub has_tools: bool,
    pub matrix: MatrixView,
    pub attribution_note: &'static str,
}
