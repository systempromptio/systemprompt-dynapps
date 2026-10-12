//! The launch dialog's values and view. Values stay raw strings so a refused
//! submission renders back exactly what was typed, beside the reason.

use serde::Serialize;

use crate::repositories::rows::{AgentRow, FarmRow, FixtureRow};

pub(crate) const PROMPT_MAX: usize = 8000;
pub(crate) const TURNS_RANGE: (u32, u32) = (1, 200);
pub(crate) const TIMEOUT_RANGE: (u64, u64) = (30, 7200);
const MODEL_ALIASES: [&str; 3] = ["sonnet", "opus", "haiku"];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LaunchValues {
    pub prompt: String,
    pub model: String,
    pub fixture: String,
    pub max_turns: String,
    pub timeout_seconds: String,
}

impl LaunchValues {
    pub(crate) fn defaults(farm: &FarmRow, agent: &AgentRow) -> Self {
        Self {
            prompt: agent.entry_prompt.clone(),
            model: String::new(),
            fixture: agent.fixture.clone().unwrap_or_default(),
            max_turns: agent.max_turns.unwrap_or(farm.max_turns).to_string(),
            timeout_seconds: farm.timeout_seconds.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ChoiceView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LaunchFormView {
    pub action_url: String,
    pub close_url: String,
    pub open: bool,
    pub error: Option<String>,
    pub prompt: String,
    pub prompt_len: usize,
    pub prompt_max: usize,
    pub models: Vec<ChoiceView>,
    pub fixtures: Vec<ChoiceView>,
    pub max_turns: String,
    pub turns_min: u32,
    pub turns_max: u32,
    pub timeout_seconds: String,
    pub timeout_min: u64,
    pub timeout_max: u64,
}

pub(crate) fn model_choices(farm: &FarmRow, agent: &AgentRow) -> Vec<String> {
    let mut models: Vec<String> = Vec::new();
    let declared = [agent.model.as_deref(), farm.model.as_deref()];
    for model in declared.into_iter().flatten().chain(MODEL_ALIASES) {
        if !model.is_empty() && !models.iter().any(|m| m == model) {
            models.push(model.to_owned());
        }
    }
    models
}

fn default_model_label(farm: &FarmRow, agent: &AgentRow) -> String {
    agent
        .model
        .as_deref()
        .or(farm.model.as_deref())
        .map_or_else(
            || "Agent default".to_owned(),
            |model| format!("Agent default ({model})"),
        )
}

fn choice(value: &str, label: String, current: &str) -> ChoiceView {
    ChoiceView {
        value: value.to_owned(),
        label,
        selected: value == current,
    }
}

pub(crate) struct FormInputs<'a> {
    pub farm: &'a FarmRow,
    pub agent: &'a AgentRow,
    pub fixtures: &'a [FixtureRow],
    pub values: &'a LaunchValues,
    pub open: bool,
    pub error: Option<String>,
}

pub(crate) fn launch_form_view(input: &FormInputs<'_>) -> LaunchFormView {
    let (farm, agent, values) = (input.farm, input.agent, input.values);
    let page_url = format!(
        "/admin/farm/{}/agents/{}",
        farm.id.as_str(),
        agent.id.as_str()
    );
    let mut models = vec![choice("", default_model_label(farm, agent), &values.model)];
    models.extend(
        model_choices(farm, agent)
            .iter()
            .map(|m| choice(m, m.clone(), &values.model)),
    );
    let mut fixtures = vec![choice("", "No fixture".to_owned(), &values.fixture)];
    fixtures.extend(input.fixtures.iter().map(|f| {
        choice(
            &f.name,
            format!("{} ({} files)", f.name, f.file_count),
            &values.fixture,
        )
    }));
    LaunchFormView {
        action_url: format!("{page_url}/runs"),
        close_url: page_url,
        open: input.open,
        error: input.error.clone(),
        prompt: values.prompt.clone(),
        prompt_len: values.prompt.chars().count(),
        prompt_max: PROMPT_MAX,
        models,
        fixtures,
        max_turns: values.max_turns.clone(),
        turns_min: TURNS_RANGE.0,
        turns_max: TURNS_RANGE.1,
        timeout_seconds: values.timeout_seconds.clone(),
        timeout_min: TIMEOUT_RANGE.0,
        timeout_max: TIMEOUT_RANGE.1,
    }
}
