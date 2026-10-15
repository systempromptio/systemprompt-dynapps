//! Input contracts for the agent-farm tools. Flat structs only; see the
//! module head of `typed` for why no field is an `Option`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

fn farm() -> String {
    "farm".to_owned()
}

fn mine() -> String {
    "mine".to_owned()
}

fn seven_days() -> String {
    "7d".to_owned()
}

const fn fifty() -> u16 {
    50
}

const fn minus_one() -> i64 {
    -1
}

const fn two_hundred() -> u16 {
    200
}

/// Queue one headless run and return at once.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunStartInput {
    #[schemars(description = "What launches the run: `farm` (the only origin on this instance).")]
    #[serde(default = "farm")]
    pub origin: String,
    #[schemars(description = "Farm id, as `catalog_list` or the farm dashboard shows it.")]
    pub farm: String,
    #[schemars(description = "Agent id within the farm.")]
    pub agent: String,
    #[schemars(description = "Prompt for the agent. Empty uses the agent's entry prompt.")]
    #[serde(default)]
    pub prompt: String,
    #[schemars(
        description = "Eval case id from the agent's suite. Set, the run uses the case's prompt and fixture and is evaluated when it finishes (assertions, then the judge for a rubric case); the result names the eval run. Empty launches a plain run."
    )]
    #[serde(default)]
    pub case: String,
    #[schemars(description = "Turn ceiling. 0 means the agent's or farm's default.")]
    #[serde(default)]
    pub max_turns: u32,
    #[schemars(description = "Spend ceiling in USD. 0 means the agent's or farm's default.")]
    #[serde(default)]
    pub max_budget_usd: f64,
    #[schemars(description = "Wall-clock ceiling in seconds. 0 means the farm's default.")]
    #[serde(default)]
    pub timeout_seconds: u32,
    #[schemars(
        description = "Project id the run is attributed to. A farm run always belongs to its farm's project; naming a different project, or one you do not belong to, is refused. Empty means the farm's own project."
    )]
    #[serde(default)]
    pub project: String,
    #[schemars(
        description = "Client slug the run is for (e.g. `dynapps`). Empty means the farm's own client."
    )]
    #[serde(default)]
    pub client: String,
}

/// One run by id.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunIdInput {
    #[schemars(description = "Agent run id (UUID), as `run_start` or `run_list` returned it.")]
    pub run_id: String,
}

/// A page of a run's event stream.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunEventsInput {
    #[schemars(description = "Agent run id (UUID).")]
    pub run_id: String,
    #[schemars(
        description = "Return events with `seq` strictly greater than this. Sequence numbers start at 0, so -1 (the default) starts at the beginning."
    )]
    #[serde(default = "minus_one")]
    pub after_seq: i64,
    #[schemars(description = "Events per page. 1-500; defaults to 200.")]
    #[serde(default = "two_hundred")]
    pub limit: u16,
}

/// Runs the caller may see, newest first.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunListInput {
    #[schemars(description = "Origin kind filter, e.g. `farm`. Empty means every origin.")]
    #[serde(default)]
    pub origin: String,
    #[schemars(
        description = "Status filter: `queued`, `preparing`, `running`, `succeeded`, `failed`, `cancelled`, `timed_out`. Empty means every status."
    )]
    #[serde(default)]
    pub state: String,
    #[schemars(
        description = "`mine` (runs you launched or that ran as you), `project` (runs of every project you belong to), or `all` (administrators only). Defaults to `mine`."
    )]
    #[serde(default = "mine")]
    pub scope: String,
    #[schemars(description = "Rows to return. 1-200; defaults to 50.")]
    #[serde(default = "fifty")]
    pub limit: u16,
}

/// Run rollups over a window.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunStatsInput {
    #[schemars(
        description = "`mine`, `project`, `farm` (one farm; needs `farm`), or `all` (administrators only). Defaults to `mine`."
    )]
    #[serde(default = "mine")]
    pub scope: String,
    #[schemars(description = "`24h`, `7d` or `30d`. Defaults to `7d`.")]
    #[serde(default = "seven_days")]
    pub window: String,
    #[schemars(description = "Farm id, required when `scope` is `farm`.")]
    #[serde(default)]
    pub farm: String,
}

/// Start an eval suite: one run per case, scored by deterministic assertions.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FarmEvalsRunInput {
    #[schemars(description = "Farm id.")]
    pub farm: String,
    #[schemars(description = "Eval suite id within the farm.")]
    pub suite: String,
    #[schemars(description = "Comma-separated case ids to run. Empty runs every case.")]
    #[serde(default)]
    pub case_filter: String,
}

/// The latest decided assertion result for every case.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FarmEvalResultsInput {
    #[schemars(description = "Farm id. Empty means every farm you may use.")]
    #[serde(default)]
    pub farm: String,
    #[schemars(description = "Suite id. Empty means every suite.")]
    #[serde(default)]
    pub suite: String,
    #[schemars(
        description = "History window: `<n>d`, `<n>h` or an RFC 3339 time. Set, `history` carries per agent per day eval runs, pass rate, mean judge score and cost since then. Empty returns no history."
    )]
    #[serde(default)]
    pub since: String,
}

/// Usage over a window, for you or for one of your projects.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalyticsInput {
    #[schemars(description = "`24h`, `7d` or `30d`. Defaults to `7d`.")]
    #[serde(default = "seven_days")]
    pub window: String,
    #[schemars(
        description = "`mine` (your own usage), `project` (one project you belong to; set `project` when you belong to several), or `all` (administrators only). Defaults to `mine`."
    )]
    #[serde(default = "mine")]
    pub scope: String,
    #[schemars(description = "Project id for `scope: project`. Empty picks your only project.")]
    #[serde(default)]
    pub project: String,
}

/// What the caller's access grants.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CatalogListInput {
    #[schemars(
        description = "Limit to one section: `marketplace`, `plugin`, `skill` or `mcp_server`. Empty returns all four."
    )]
    #[serde(default)]
    pub section: String,
}

/// Per-client and per-agent farm outcomes with recent runs, for the dashboard.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FarmDashboardInput {
    #[schemars(
        description = "`mine` (runs you launched) or `all` (administrators only). Empty picks `all` for an administrator and `mine` otherwise."
    )]
    #[serde(default)]
    pub scope: String,
    #[schemars(description = "`24h`, `7d` or `30d`. Defaults to `7d`.")]
    #[serde(default = "seven_days")]
    pub window: String,
    #[schemars(
        description = "Client id (its slug) to narrow to one client; empty for every client."
    )]
    #[serde(default)]
    pub client: String,
    #[schemars(description = "Recent runs to return. 1-200; defaults to 50.")]
    #[serde(default = "fifty")]
    pub limit: u16,
}
