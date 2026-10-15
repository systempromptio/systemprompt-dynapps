//! `POST /admin/farm/evals/{suite}/run` — start an eval run over a suite as the
//! signed-in user and land on the eval run's page. The suite id alone does not
//! name its farm, so the form posts the farm id beside it. The run is judged
//! with the same configured judge as every other trigger.

use std::sync::Arc;

use axum::Extension;
use axum::extract::rejection::FormRejection;
use axum::extract::{Form, Path};
use axum::response::{IntoResponse, Response};
use systemprompt::ai::AiService;
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::flash::{FlashTone, redirect_with_flash};
use systemprompt_web_admin_core::ssr::page::Page;

use super::field;
use crate::error::FarmError;
use crate::handlers::ssr::farm_agent::require_console;
use crate::services::evals::{EvalLaunch, start_eval_run};
use crate::state::FarmState;
use crate::types::{EvalSuiteId, FarmId, TriggeredVia};

const EVALS_URL: &str = "/admin/farm/evals";

fn farm_of_suite(suite_id: &str, posted: Option<&str>) -> Option<FarmId> {
    posted
        .map(str::trim)
        .filter(|f| !f.is_empty())
        .or_else(|| suite_id.split_once('.').map(|(farm, _)| farm))
        .map(FarmId::new)
}

pub(crate) async fn run_now_action(
    shell: Page,
    Extension(state): Extension<Arc<FarmState>>,
    ai: Option<Extension<Option<Arc<AiService>>>>,
    Path(suite_id): Path<String>,
    form: Result<Form<Vec<(String, String)>>, FormRejection>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let fields = form.map(|Form(f)| f).unwrap_or_default();
    let Some(farm_id) = farm_of_suite(&suite_id, field(&fields, "farm_id")) else {
        return Err(AdminError::Unprocessable("farm_id is required".to_owned()).into());
    };
    let judge = state
        .judge
        .judge_for(ai.and_then(|Extension(ai)| ai), &shell.user.user_id);
    let started = start_eval_run(
        Arc::clone(&state),
        judge,
        EvalLaunch {
            farm_id,
            suite_id: EvalSuiteId::new(suite_id),
            user_id: shell.user.user_id.clone(),
            triggered_via: TriggeredVia::Console,
            case_filter: None,
        },
    )
    .await;
    match started {
        Ok(run) => Ok(redirect_with_flash(
            &format!("/admin/farm/evals/runs/{}", run.id.as_str()),
            FlashTone::Ok,
            "Eval run started",
        )
        .into_response()),
        Err(FarmError::NotFound(reason) | FarmError::Validation(reason)) => Ok(
            redirect_with_flash(EVALS_URL, FlashTone::Err, &format!("Not started: {reason}"))
                .into_response(),
        ),
        Err(other) => Err(AdminError::from(other).into()),
    }
}
