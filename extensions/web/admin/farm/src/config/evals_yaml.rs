//! The `evals:` block of `web/config/agent-farms.yaml`: which provider and
//! model judge a rubric case. The judge is an ordinary gateway call through
//! the process's `AiService`, so the pair must name a provider and model the
//! AI catalog serves.

use serde::{Deserialize, Serialize};

use crate::error::{FarmError, FarmResult};

pub const DEFAULT_JUDGE_PROVIDER: &str = "anthropic";
pub const DEFAULT_JUDGE_MODEL: &str = "anthropic-claude-sonnet-5-5";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalsConfig {
    #[serde(default = "default_judge_provider")]
    pub judge_provider: String,
    #[serde(default = "default_judge_model")]
    pub judge_model: String,
}

impl Default for EvalsConfig {
    fn default() -> Self {
        Self {
            judge_provider: default_judge_provider(),
            judge_model: default_judge_model(),
        }
    }
}

impl EvalsConfig {
    pub fn validate(&self, file: &str) -> FarmResult<()> {
        if self.judge_provider.trim().is_empty() || self.judge_model.trim().is_empty() {
            return Err(FarmError::Config(format!(
                "{file}: evals.judge_provider and evals.judge_model must not be empty"
            )));
        }
        Ok(())
    }
}

fn default_judge_provider() -> String {
    DEFAULT_JUDGE_PROVIDER.to_owned()
}

fn default_judge_model() -> String {
    DEFAULT_JUDGE_MODEL.to_owned()
}
