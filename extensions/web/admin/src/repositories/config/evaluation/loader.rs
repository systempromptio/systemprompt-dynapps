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
    CONFIG_DIR, CONFIG_FILE, CriterionConfig, CriterionName, EvaluationDoc, LoadedCriterion,
    LoadedEvaluation, MetricThresholds, Scoring,
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
    Ok(Some(LoadedEvaluation {
        enabled: doc.evaluation.enabled,
        session_close: doc.evaluation.session_close,
        worker: doc.evaluation.worker,
        criteria,
        config_hash: format!("{:x}", hasher.finalize()),
    }))
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

    let entry = providers.find_provider(&provider).ok_or_else(|| {
        invalid(format!(
            "criterion '{name}' names provider '{provider}', which is not in services/ai/providers.yaml"
        ))
    })?;
    if !ai.providers.get(&provider).is_some_and(|p| p.enabled) {
        return Err(invalid(format!(
            "criterion '{name}' names provider '{provider}', which is not enabled in services/ai/config.yaml"
        )));
    }
    let catalog = entry.find_model(&model).ok_or_else(|| {
        invalid(format!(
            "criterion '{name}' names model '{model}', which provider '{provider}' does not list"
        ))
    })?;
    if !catalog.capabilities.structured_output {
        return Err(invalid(format!(
            "criterion '{name}' needs structured output, which '{provider}/{model}' does not declare"
        )));
    }
    if catalog.limits.max_output_tokens > 0 && max_output_tokens > catalog.limits.max_output_tokens
    {
        return Err(invalid(format!(
            "criterion '{name}' asks for {max_output_tokens} output tokens; '{model}' allows {}",
            catalog.limits.max_output_tokens
        )));
    }

    let instructions_path = dir.join(&criterion.instructions);
    let instructions_markdown = std::fs::read_to_string(&instructions_path).map_err(|e| {
        MarketplaceError::config_file(
            format!("{CONFIG_DIR}/{}", criterion.instructions.display()),
            e,
        )
    })?;
    if instructions_markdown.trim().is_empty() {
        return Err(invalid(format!(
            "criterion '{name}' has empty instructions at {}",
            criterion.instructions.display()
        )));
    }

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
