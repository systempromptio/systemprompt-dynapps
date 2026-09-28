//! Every exportable table, by id.

use super::datasets;
use super::model::DataSet;

// Why: a closed static slice, not `inventory` registration, so every export
// is visible in one place; order is the dialog's listing order.
pub(crate) const ALL: &[&dyn DataSet] = &[
    &datasets::requests::Requests,
    &datasets::sessions::Sessions,
    &datasets::traces::Traces,
    &datasets::tools::Tools,
    &datasets::tools::ToolsBreakdown,
    &datasets::tools::Artifacts,
    &datasets::tools::ArtifactsBreakdown,
    &datasets::contexts::Contexts,
    &datasets::contexts::People,
    &datasets::people::Users,
    &datasets::people::Groups,
    &datasets::projects::Projects,
    &datasets::history::History,
    &datasets::history::OrgHistory,
    &datasets::dashboard::ModelStats,
    &datasets::dashboard::SkillStats,
    &datasets::dashboard::ToolStats,
    &datasets::dashboard::ToolServers,
    &datasets::mcp::McpServers,
    &datasets::mcp::McpCalls,
    &datasets::mcp::McpTools,
    &datasets::dashboard::SessionCosts,
    &datasets::dashboard_cost::ProviderCosts,
    &datasets::dashboard_cost::ProviderCostByDay,
    &datasets::dashboard_cost::ContainerUsage,
    &datasets::analysis_skills::Skills,
    &datasets::analysis_skill_conversations::SkillConversations,
    &datasets::analysis_conversations::ClassifiedConversations,
    &datasets::analysis_conversations::Breakdown,
    &datasets::analysis_conversation::ConversationTurns,
    &datasets::analysis_skill_runs::SkillRuns,
    &datasets::analysis_skill_runs::KitReleaseImpact,
    &datasets::versions::Marketplaces,
    &datasets::versions::Versions,
    &datasets::plugin_eval::PluginEval,
    &datasets::plugin_eval::PluginEvalTools,
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
