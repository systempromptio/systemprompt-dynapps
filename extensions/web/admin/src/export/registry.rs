//! Every exportable table, by id.

use super::datasets;
use super::model::DataSet;

// Why: a static slice rather than `inventory` — the set is closed, small, and
// a reader should be able to see every export the console offers in one
// place. Order is the order the dialog lists them in when a page offers more
// than one.
pub(crate) const ALL: &[&dyn DataSet] = &[
    &datasets::requests::Requests,
    &datasets::sessions::Sessions,
    &datasets::traces::Traces,
    &datasets::artifacts::Artifacts,
    &datasets::contexts::Contexts,
    &datasets::contexts::People,
    &datasets::history::History,
    &datasets::history::OrgHistory,
    &datasets::dashboard::ModelStats,
    &datasets::dashboard::SkillStats,
    &datasets::dashboard::ToolStats,
    &datasets::dashboard::ToolServers,
    &datasets::dashboard::SessionCosts,
    &datasets::dashboard_cost::ProviderCosts,
    &datasets::dashboard_cost::ProviderCostByDay,
    &datasets::dashboard_cost::ContainerUsage,
    &datasets::analysis_skills::RetainedSkills,
    &datasets::analysis_skills::SkillConversations,
    &datasets::analysis_conversations::ClassifiedConversations,
    &datasets::analysis_conversations::Breakdown,
    &datasets::portfolio::Resources,
    &datasets::versions::Marketplaces,
    &datasets::versions::Versions,
    &datasets::evaluation::Runs,
    &datasets::governance::Decisions,
    &datasets::governance::Findings,
    &datasets::governance::Secrets,
    &datasets::reports::CustomerUsers,
    &datasets::reports::CustomerProjects,
    &datasets::reports::CustomerModels,
    &datasets::reports::InternalProviders,
    &datasets::reports::InternalModels,
];

pub(crate) fn find(id: &str) -> Option<&'static dyn DataSet> {
    ALL.iter().copied().find(|d| d.id() == id)
}
