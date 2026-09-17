//! What a console-triggered evaluation needs from the request: who clicked,
//! the process's judge client, and the owner every judge call runs as.
//!
//! One extractor rather than three, so a trigger handler's arguments are the
//! trigger's own inputs; the same-origin check and the "is anything here
//! able to judge" question live with it, because every trigger asks both.

use std::sync::Arc;

use axum::Extension;
use axum::extract::FromRequestParts;
use axum::extract::rejection::ExtensionRejection;
use axum::http::HeaderMap;
use axum::http::request::Parts;
use systemprompt::ai::AiService;

use super::load_config;
use crate::error::AdminError;
use crate::evaluation::Judge;
use crate::handlers::shared::require_write_origin;
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;

#[derive(Debug, Clone)]
pub(crate) struct JudgeRequest {
    pub(crate) user: UserContext,
    managed: Arc<ManagedState>,
    ai: Option<Arc<AiService>>,
    headers: HeaderMap,
}

impl<S: Send + Sync> FromRequestParts<S> for JudgeRequest {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(user) = Extension::<UserContext>::from_request_parts(parts, state).await?;
        let Extension(managed) =
            Extension::<Arc<ManagedState>>::from_request_parts(parts, state).await?;
        let Extension(ai) =
            Extension::<Option<Arc<AiService>>>::from_request_parts(parts, state).await?;
        Ok(Self {
            user,
            managed,
            ai,
            headers: parts.headers.clone(),
        })
    }
}

impl JudgeRequest {
    pub(crate) fn require_console_write(&self) -> Result<(), AdminError> {
        if !self.user.is_console {
            return Err(AdminError::Forbidden("Console access required".to_owned()));
        }
        require_write_origin(&self.headers)
    }

    pub(crate) fn judge(&self) -> Result<Judge, AdminError> {
        let config = load_config()?
            .filter(|c| c.enabled)
            .ok_or_else(|| AdminError::BadRequest("Evaluation is not configured.".to_owned()))?;
        let ai = self.ai.clone().ok_or_else(|| {
            AdminError::BadRequest("Inference is not configured on this instance.".to_owned())
        })?;
        Ok(Judge::from_parts(
            ai,
            Arc::new(config),
            self.managed.owner.clone(),
        ))
    }
}

// Why: a config that cannot be read hides the buttons rather than failing
// the page that carries them; the run itself reports the real error.
pub(crate) fn evaluation_enabled() -> bool {
    match load_config() {
        Ok(config) => config.is_some_and(|c| c.enabled),
        Err(error) => {
            tracing::warn!(%error, "Evaluation config could not be read");
            false
        },
    }
}
