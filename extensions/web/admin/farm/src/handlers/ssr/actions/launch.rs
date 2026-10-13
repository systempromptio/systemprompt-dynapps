//! `POST /admin/farm/{farm}/agents/{agent}/runs` — queue a run from the launch
//! dialog as the signed-in user, then land on the run's own page.
//!
//! An empty or malformed body is a 422, never a 500: the form is read as raw
//! pairs so a missing field is a validation refusal, not an extractor fault.

use std::sync::Arc;

use axum::Extension;
use axum::extract::rejection::FormRejection;
use axum::extract::{Form, Path};
use axum::response::{IntoResponse, Response};
use systemprompt_web_admin_core::error::{AdminError, AdminHtmlResult};
use systemprompt_web_admin_core::ssr::flash::{FlashTone, redirect_with_flash};
use systemprompt_web_admin_core::ssr::page::Page;

use super::field;
use crate::error::FarmError;
use crate::handlers::ssr::farm_agent::{
    Launch, LaunchValues, PROMPT_MAX, TIMEOUT_RANGE, TURNS_RANGE, load_agent, model_choices,
    render_agent_page, require_console,
};
use crate::repositories::fixtures;
use crate::repositories::rows::{AgentRow, FarmRow};
use crate::services::runs::{LaunchRequest, launch};
use crate::state::FarmState;
use crate::types::{FarmAgentId, FarmId, TriggeredVia};

#[derive(Debug, Clone, PartialEq, Eq)]
struct CheckedLaunch {
    prompt: String,
    model: Option<String>,
    fixture: Option<String>,
    max_turns: u32,
    timeout_seconds: u64,
}

fn values_from(fields: &[(String, String)]) -> LaunchValues {
    let get = |name: &str| field(fields, name).unwrap_or_default().trim().to_owned();
    LaunchValues {
        prompt: field(fields, "prompt").unwrap_or_default().to_owned(),
        model: get("model"),
        fixture: get("fixture"),
        max_turns: get("max_turns"),
        timeout_seconds: get("timeout_seconds"),
    }
}

fn in_range<T: std::str::FromStr + PartialOrd + std::fmt::Display>(
    raw: &str,
    label: &str,
    (min, max): (T, T),
) -> Result<T, String> {
    let refused = || format!("{label} must be a whole number from {min} to {max}.");
    let Ok(value) = raw.parse::<T>() else {
        return Err(refused());
    };
    if value < min || value > max {
        return Err(refused());
    }
    Ok(value)
}

fn check_shape(values: &LaunchValues) -> Result<CheckedLaunch, String> {
    let prompt = values.prompt.trim();
    if prompt.is_empty() {
        return Err("Write a prompt for the agent.".to_owned());
    }
    if prompt.chars().count() > PROMPT_MAX {
        return Err(format!(
            "The prompt is longer than {PROMPT_MAX} characters."
        ));
    }
    let max_turns = in_range(&values.max_turns, "Max turns", TURNS_RANGE)?;
    let timeout_seconds = in_range(&values.timeout_seconds, "Timeout", TIMEOUT_RANGE)?;
    let optional = |v: &str| (!v.is_empty()).then(|| v.to_owned());
    Ok(CheckedLaunch {
        prompt: prompt.to_owned(),
        model: optional(&values.model),
        fixture: optional(&values.fixture),
        max_turns,
        timeout_seconds,
    })
}

async fn check_choices(
    state: &FarmState,
    farm: &FarmRow,
    agent: &AgentRow,
    checked: &CheckedLaunch,
) -> Result<Result<(), String>, AdminError> {
    if let Some(model) = &checked.model
        && !model_choices(farm, agent).contains(model)
    {
        return Ok(Err(format!("Model {model} is not offered for this agent.")));
    }
    if let Some(name) = &checked.fixture
        && fixtures::find_fixture(&state.pool, &farm.id, name)
            .await?
            .is_none()
    {
        return Ok(Err(format!("Fixture {name} is not synced for this farm.")));
    }
    Ok(Ok(()))
}

pub(crate) async fn launch_run_action(
    shell: Page,
    Extension(state): Extension<Arc<FarmState>>,
    Path((farm_id, agent_id)): Path<(String, String)>,
    submitted: Result<Form<Vec<(String, String)>>, FormRejection>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let fields = submitted.map(|Form(f)| f).unwrap_or_default();
    let values = values_from(&fields);
    let shape = check_shape(&values);
    let (farm_id, agent_id) = (FarmId::new(farm_id), FarmAgentId::new(agent_id));
    let Some((farm, agent)) = load_agent(&state, &farm_id, &agent_id).await? else {
        return Err(match shape {
            Err(reason) => AdminError::Unprocessable(reason),
            Ok(_) => AdminError::NotFound(format!("agent {}", agent_id.as_str())),
        }
        .into());
    };
    let refuse = |error: String, values: LaunchValues| Launch {
        values,
        open: true,
        error: Some(error),
    };
    let checked = match shape {
        Ok(checked) => checked,
        Err(reason) => {
            return render_agent_page(&shell, &state, &farm, &agent, refuse(reason, values)).await;
        },
    };
    if let Err(reason) = check_choices(&state, &farm, &agent, &checked).await? {
        return render_agent_page(&shell, &state, &farm, &agent, refuse(reason, values)).await;
    }
    let request = LaunchRequest {
        farm_id: farm.id.clone(),
        agent_id: agent.id.clone(),
        user_id: shell.user.user_id.clone(),
        triggered_via: TriggeredVia::Dashboard,
        prompt: Some(checked.prompt),
        case_prompt: None,
        fixture: checked.fixture,
        model: checked.model,
        max_turns: i32::try_from(checked.max_turns).ok(),
        max_budget_usd: None,
        timeout_seconds: i32::try_from(checked.timeout_seconds).ok(),
        eval_result_id: None,
    };
    match launch(&state, request).await {
        Ok(run) => Ok(redirect_with_flash(
            &format!("/admin/farm/runs/{}", run.id.as_str()),
            FlashTone::Ok,
            "Run queued",
        )
        .into_response()),
        Err(FarmError::Validation(reason) | FarmError::Config(reason)) => {
            render_agent_page(&shell, &state, &farm, &agent, refuse(reason, values)).await
        },
        Err(other) => Err(AdminError::from(other).into()),
    }
}
