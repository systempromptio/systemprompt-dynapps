//! How many runs execute at once and the ceilings a run falls back to when
//! neither its agent nor its farm sets one.

use crate::config::RunnerConfig;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RunnerLimits {
    pub max_concurrent: usize,
    pub default_timeout_seconds: u64,
    pub default_max_turns: u32,
    pub default_max_budget_usd: f64,
}

impl Default for RunnerLimits {
    fn default() -> Self {
        Self {
            max_concurrent: 2,
            default_timeout_seconds: 1200,
            default_max_turns: 40,
            default_max_budget_usd: 5.0,
        }
    }
}

impl RunnerLimits {
    #[must_use]
    pub fn from_config(config: Option<&RunnerConfig>) -> Self {
        let mut limits = Self::default();
        if let Some(config) = config {
            limits.max_concurrent = config.max_concurrent.max(1);
        }
        limits
    }
}
