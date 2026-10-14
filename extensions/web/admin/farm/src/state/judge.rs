//! What every eval trigger needs to build its judge: the `evals:` block of
//! the farms file and, in a process that is not an HTTP server (the MCP
//! binary), the `AiService` its own `AppContext` carries. Routers pass the
//! service core layers onto each request instead.

use std::path::Path;
use std::sync::Arc;

use systemprompt::ai::AiService;
use systemprompt::identifiers::UserId;

use crate::config::{EvalsConfig, FARMS_FILE, parse_farms_doc};
use crate::services::evals::{EvalJudge, gateway_judge};

#[derive(Clone, Default)]
pub struct JudgeSource {
    pub config: EvalsConfig,
    pub ai: Option<Arc<AiService>>,
}

impl std::fmt::Debug for JudgeSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JudgeSource")
            .field("config", &self.config)
            .field("ai", &self.ai.is_some())
            .finish()
    }
}

impl JudgeSource {
    #[must_use]
    pub fn judge_for(
        &self,
        request_ai: Option<Arc<AiService>>,
        user_id: &UserId,
    ) -> Option<Arc<dyn EvalJudge>> {
        gateway_judge(
            request_ai.or_else(|| self.ai.clone()),
            user_id,
            &self.config,
        )
    }
}

#[must_use]
pub fn read_evals_config(services_root: &Path) -> EvalsConfig {
    let path = services_root.join(FARMS_FILE);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return EvalsConfig::default();
    };
    match parse_farms_doc(&text) {
        Ok(doc) => doc.evals.unwrap_or_default(),
        Err(e) => {
            tracing::error!(error = %e, "farms file unreadable; the eval judge uses its defaults");
            EvalsConfig::default()
        },
    }
}
