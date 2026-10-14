//! Output contracts for the analytics tools: dashboard figures reduced to
//! what a person or a project may see, with costs in USD.

use schemars::JsonSchema;
use serde::Serialize;
use systemprompt::mcp::McpOutputSchema;

macro_rules! farm_output {
    ($ty:ty, $name:literal) => {
        impl McpOutputSchema for $ty {
            fn artifact_type() -> &'static str {
                $name
            }
        }
    };
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ProviderCostOutput {
    pub provider: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AnalyticsCostsOutput {
    pub scope: String,
    pub window: String,
    pub total_requests: i64,
    pub error_count: i64,
    pub total_cost_usd: f64,
    pub total_tokens: i64,
    pub output_tokens: i64,
    pub prev_total_cost_usd: f64,
    pub by_provider: Vec<ProviderCostOutput>,
}
farm_output!(AnalyticsCostsOutput, "agent_farm_analytics_costs");

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ModelBreakdownOutput {
    pub model: String,
    pub requests: i64,
    pub error_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AnalyticsRequestsOutput {
    pub scope: String,
    pub window: String,
    pub total: i64,
    pub failed: i64,
    pub rejected: i64,
    pub denied: i64,
    pub tool_calls: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub by_model: Vec<ModelBreakdownOutput>,
}
farm_output!(AnalyticsRequestsOutput, "agent_farm_analytics_requests");

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SkillStatOutput {
    pub skill: String,
    pub invocations: i64,
    pub distinct_users: i64,
    pub conversations: i64,
    pub requests: i64,
    pub conversation_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AnalyticsSkillsOutput {
    pub scope: String,
    pub window: String,
    pub skills: Vec<SkillStatOutput>,
    pub returned: usize,
    pub total: i64,
    pub hint: String,
}
farm_output!(AnalyticsSkillsOutput, "agent_farm_analytics_skills");

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ToolStatOutput {
    pub server: String,
    pub tool: String,
    pub executions: i64,
    pub succeeded: i64,
    pub failed: i64,
    pub pending: i64,
    pub p50_ms: Option<f64>,
    pub p95_ms: Option<f64>,
    pub distinct_users: i64,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct AnalyticsToolsOutput {
    pub scope: String,
    pub window: String,
    pub tools: Vec<ToolStatOutput>,
    pub returned: usize,
    pub total: i64,
}
farm_output!(AnalyticsToolsOutput, "agent_farm_analytics_tools");

pub(crate) fn usd(microdollars: i64) -> f64 {
    microdollars as f64 / 1_000_000.0
}
