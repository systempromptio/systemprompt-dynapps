//! Reads and validates `services/evaluation/`.
//!
//! Validation is against the live provider catalog, so a criterion cannot
//! name a model the gateway would refuse at dispatch: the boot fails instead,
//! with the criterion and the model in the message.

use std::fmt;
use std::path::Path;

use sha2::{Digest, Sha256};
use systemprompt::models::services::{AiConfig, ProviderRegistry};
use systemprompt_web_shared::error::MarketplaceError;

use super::types::{
    CONFIG_DIR, CONFIG_FILE, CriterionConfig, CriterionName, EvaluationDoc, JudgeRef,
    LoadedCriterion, LoadedEvaluation, LoadedReport, MetricThresholds, Scoring,
};

#[derive(Debug)]
pub struct EvaluationConfigError(String);

impl fmt::Display for EvaluationConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EvaluationConfigError {}

fn invalid(message: impl Into<String>) -> MarketplaceError {
    MarketplaceError::config_file(CONFIG_FILE, EvaluationConfigError(message.into()))
}

// Why: `None` when the file is absent — the engine is simply not declared.
pub fn load_evaluation(
    services_path: &Path,
    providers: &ProviderRegistry,
    ai: &AiConfig,
) -> Result<Option<LoadedEvaluation>, MarketplaceError> {
    let path = services_path.join(CONFIG_FILE);
    if !path.exists() {
        return Ok(None);
    }
    let yaml = std::fs::read_to_string(&path)
        .map_err(|e| MarketplaceError::config_file(CONFIG_FILE, e))?;
    let doc: EvaluationDoc =
        serde_yaml::from_str(&yaml).map_err(|e| MarketplaceError::config_file(CONFIG_FILE, e))?;
    validate_shape(&doc)?;

    let mut hasher = Sha256::new();
    hasher.update(yaml.as_bytes());
    let dir = services_path.join(CONFIG_DIR);
    let mut criteria = Vec::with_capacity(doc.evaluation.criteria.len());
    for criterion in &doc.evaluation.criteria {
        let loaded = resolve(&doc, criterion, &dir, providers, ai)?;
        hasher.update(loaded.name.as_str().as_bytes());
        hasher.update(loaded.instructions_markdown.as_bytes());
        criteria.push(loaded);
    }
    let report = resolve_report(&doc, &dir, providers, ai)?;
    hasher.update(report.instructions_markdown.as_bytes());
    Ok(Some(LoadedEvaluation {
        enabled: doc.evaluation.enabled,
        session_close: doc.evaluation.session_close,
        worker: doc.evaluation.worker,
        report,
        criteria,
        config_hash: format!("{:x}", hasher.finalize()),
    }))
}

fn resolve_report(
    doc: &EvaluationDoc,
    dir: &Path,
    providers: &ProviderRegistry,
    ai: &AiConfig,
) -> Result<LoadedReport, MarketplaceError> {
    let cfg = &doc.evaluation.report;
    let defaults = &doc.evaluation.judge_defaults;
    let judge = JudgeRef {
        provider: cfg
            .provider
            .clone()
            .unwrap_or_else(|| defaults.provider.clone()),
        model: cfg.model.clone().unwrap_or_else(|| defaults.model.clone()),
        max_output_tokens: cfg.max_output_tokens.unwrap_or(defaults.max_output_tokens),
    };
    check_model("the report", &judge, providers, ai)?;
    let instructions_markdown = read_instructions("the report", &cfg.instructions, dir)?;
    Ok(LoadedReport {
        provider: judge.provider,
        model: judge.model,
        max_output_tokens: judge.max_output_tokens,
        instructions_path: format!("{CONFIG_DIR}/{}", cfg.instructions.display()),
        instructions_markdown,
        window_days: cfg.window_days,
        min_new_runs: cfg.min_new_runs,
        max_age_hours: cfg.max_age_hours,
        input_token_budget: cfg.input_token_budget,
    })
}

// Why: a model the gateway would refuse at dispatch fails the boot instead,
// with the caller's name and the model in the message.
fn check_model(
    name: &str,
    judge: &JudgeRef,
    providers: &ProviderRegistry,
    ai: &AiConfig,
) -> Result<(), MarketplaceError> {
    let (provider, model) = (&judge.provider, &judge.model);
    let entry = providers.find_provider(provider).ok_or_else(|| {
        invalid(format!(
            "{name} names provider '{provider}', which is not in services/ai/providers.yaml"
        ))
    })?;
    if !ai.providers.get(provider).is_some_and(|p| p.enabled) {
        return Err(invalid(format!(
            "{name} names provider '{provider}', which is not enabled in services/ai/config.yaml"
        )));
    }
    let catalog = entry.find_model(model).ok_or_else(|| {
        invalid(format!(
            "{name} names model '{model}', which provider '{provider}' does not list"
        ))
    })?;
    if !catalog.capabilities.structured_output {
        return Err(invalid(format!(
            "{name} needs structured output, which '{provider}/{model}' does not declare"
        )));
    }
    if catalog.limits.max_output_tokens > 0
        && judge.max_output_tokens > catalog.limits.max_output_tokens
    {
        return Err(invalid(format!(
            "{name} asks for {} output tokens; '{model}' allows {}",
            judge.max_output_tokens, catalog.limits.max_output_tokens
        )));
    }
    Ok(())
}

fn read_instructions(
    name: &str,
    instructions: &Path,
    dir: &Path,
) -> Result<String, MarketplaceError> {
    let markdown = std::fs::read_to_string(dir.join(instructions)).map_err(|e| {
        MarketplaceError::config_file(format!("{CONFIG_DIR}/{}", instructions.display()), e)
    })?;
    if markdown.trim().is_empty() {
        return Err(invalid(format!(
            "{name} has empty instructions at {}",
            instructions.display()
        )));
    }
    Ok(markdown)
}

fn validate_shape(doc: &EvaluationDoc) -> Result<(), MarketplaceError> {
    let cfg = &doc.evaluation;
    if cfg.session_close.quiet_minutes == 0 {
        return Err(invalid("session_close.quiet_minutes must be at least 1"));
    }
    if cfg.session_close.lookback_hours == 0 {
        return Err(invalid("session_close.lookback_hours must be at least 1"));
    }
    if !(1..=256).contains(&cfg.worker.batch_size) {
        return Err(invalid("worker.batch_size must be between 1 and 256"));
    }
    if cfg.worker.lease_seconds == 0 || cfg.worker.max_attempts == 0 {
        return Err(invalid(
            "worker.lease_seconds and worker.max_attempts must be at least 1",
        ));
    }
    if !(1..=366).contains(&cfg.report.window_days) {
        return Err(invalid("report.window_days must be between 1 and 366"));
    }
    if cfg.report.min_new_runs == 0 || cfg.report.max_age_hours == 0 {
        return Err(invalid(
            "report.min_new_runs and report.max_age_hours must be at least 1",
        ));
    }
    if !(2_000..=400_000).contains(&cfg.report.input_token_budget) {
        return Err(invalid(
            "report.input_token_budget must be between 2000 and 400000",
        ));
    }
    for name in CriterionName::ALL {
        let count = cfg.criteria.iter().filter(|c| c.name == name).count();
        if count != 1 {
            return Err(invalid(format!(
                "criterion '{name}' must be declared exactly once, found {count}"
            )));
        }
    }
    for criterion in &cfg.criteria {
        validate_criterion(criterion)?;
    }
    Ok(())
}

fn validate_criterion(criterion: &CriterionConfig) -> Result<(), MarketplaceError> {
    let name = criterion.name;
    if criterion.inputs.is_empty() {
        return Err(invalid(format!("criterion '{name}' declares no inputs")));
    }
    match (criterion.scoring, &criterion.metric) {
        (Scoring::Judge, Some(_)) => Err(invalid(format!(
            "criterion '{name}' is judged by a model and may not carry metric thresholds"
        ))),
        (Scoring::Metric, None) => Err(invalid(format!(
            "criterion '{name}' is scored from metrics and needs a metric block"
        ))),
        (Scoring::Metric, Some(MetricThresholds::Speed { .. })) if name != CriterionName::Speed => {
            Err(invalid(format!(
                "criterion '{name}' carries speed thresholds; only 'speed' may"
            )))
        },
        (Scoring::Metric, Some(MetricThresholds::Cost { .. })) if name != CriterionName::Cost => {
            Err(invalid(format!(
                "criterion '{name}' carries cost thresholds; only 'cost' may"
            )))
        },
        (Scoring::Metric, Some(_))
            if !matches!(name, CriterionName::Speed | CriterionName::Cost) =>
        {
            Err(invalid(format!(
                "criterion '{name}' cannot be scored from metrics"
            )))
        },
        _ => Ok(()),
    }
}

fn resolve(
    doc: &EvaluationDoc,
    criterion: &CriterionConfig,
    dir: &Path,
    providers: &ProviderRegistry,
    ai: &AiConfig,
) -> Result<LoadedCriterion, MarketplaceError> {
    let defaults = &doc.evaluation.judge_defaults;
    let name = criterion.name;
    let provider = criterion
        .provider
        .clone()
        .unwrap_or_else(|| defaults.provider.clone());
    let model = criterion
        .model
        .clone()
        .unwrap_or_else(|| defaults.model.clone());
    let max_output_tokens = criterion
        .max_output_tokens
        .unwrap_or(defaults.max_output_tokens);

    let judge = JudgeRef {
        provider,
        model,
        max_output_tokens,
    };
    check_model(&format!("criterion '{name}'"), &judge, providers, ai)?;
    let instructions_markdown =
        read_instructions(&format!("criterion '{name}'"), &criterion.instructions, dir)?;
    let JudgeRef {
        provider,
        model,
        max_output_tokens,
    } = judge;

    Ok(LoadedCriterion {
        name,
        scoring: criterion.scoring,
        provider,
        model,
        max_output_tokens,
        instructions_path: criterion.instructions.display().to_string(),
        instructions_markdown,
        inputs: criterion.inputs.clone(),
        metric: criterion.metric,
    })
}
