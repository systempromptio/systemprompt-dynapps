//! The typed tools: flat inputs, bounded outputs, one handler per tool.
//!
//! Every input is a flat struct of `String` (empty = unset) and numeric
//! fields with defaults, never `Option`, because schemars renders an `Option`
//! as a type list and Gemini refuses a function declaration that holds one.
//! Every handler runs over a [`ToolCtx`]: the resolved caller and the server
//! environment. Scope words are shared across tools — `mine` is the caller's
//! own runs, `project` the runs of every project they belong to, `all` the
//! instance, which only an administrator may ask for.

mod analytics;
mod analytics_scope;
mod catalog;
mod dashboard;
mod evals;
#[cfg(feature = "farm")]
mod farm_access;
mod inputs;
mod outputs;
mod outputs_analytics;
mod outputs_dashboard;
mod outputs_report;
mod runs_read;
mod runs_stats;
mod runs_status;
mod runs_write;

pub use analytics::{
    AnalyticsCostsHandler, AnalyticsRequestsHandler, AnalyticsSkillsHandler, AnalyticsToolsHandler,
};
pub use analytics_scope::analytics_scope;
pub use catalog::CatalogListHandler;
pub use dashboard::FarmDashboardHandler;
pub use evals::{FarmEvalResultsHandler, FarmEvalsRunHandler};
pub use inputs::{
    AnalyticsInput, CatalogListInput, FarmDashboardInput, FarmEvalResultsInput, FarmEvalsRunInput,
    RunEventsInput, RunIdInput, RunListInput, RunStartInput, RunStatsInput,
};
pub use outputs::{
    CatalogOutput, EvalResultsOutput, EvalRunOutput, RunCancelOutput, RunEventsOutput, RunExtras,
    RunFactsOutput, RunListOutput, RunStartOutput, RunStatsOutput, RunStatusOutput,
};
pub use outputs_analytics::{
    AnalyticsCostsOutput, AnalyticsRequestsOutput, AnalyticsSkillsOutput, AnalyticsToolsOutput,
};
pub use outputs_dashboard::FarmDashboardOutput;
pub use outputs_report::{FarmRunReportOutput, RunReportOutput};
pub use runs_read::{RunEventsHandler, RunFactsHandler, RunListHandler};
pub use runs_stats::{RunStatsHandler, facts_scope};
pub use runs_status::{FarmRunReportHandler, RunStatusHandler};
pub use runs_write::{RunCancelHandler, RunStartHandler};

use systemprompt::identifiers::AgentRunId;
use systemprompt_web_admin_core::util::time_range::{TimeRange, TimeRangePreset, preset_to_range};

pub use crate::error::{ToolError, ToolResult};
pub use crate::server::ToolCtx;

pub const MAX_EVENTS: u16 = 500;
pub const MAX_LIST: u16 = 200;

pub(crate) fn window(value: &str) -> ToolResult<TimeRange> {
    let value = if value.trim().is_empty() {
        "7d"
    } else {
        value.trim()
    };
    TimeRangePreset::parse(value)
        .filter(|preset| *preset != TimeRangePreset::Custom)
        .map(preset_to_range)
        .ok_or_else(|| {
            ToolError::Invalid(format!(
                "`window` must be one of 24h, 7d, 30d (got {value:?})"
            ))
        })
}

pub(crate) fn run_id(value: &str) -> ToolResult<AgentRunId> {
    Ok(AgentRunId::try_new(value.trim())?)
}

pub(crate) fn bounded(requested: u16, default: u16, max: u16) -> i64 {
    let value = if requested == 0 {
        default
    } else {
        requested.min(max)
    };
    i64::from(value)
}

pub(crate) fn non_empty(name: &str, value: &str) -> ToolResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ToolError::Invalid(format!("`{name}` is required")));
    }
    Ok(value.to_owned())
}
