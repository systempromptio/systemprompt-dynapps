//! Declared shape of `services/evaluation/config.yaml`.
//!
//! Every struct rejects unknown keys: a misspelled threshold would otherwise
//! be ignored and the judge would run against defaults nobody wrote down.
//! The file names four criteria by fixed name; each is either judged by a
//! model or scored deterministically from the session's own metrics with the
//! model asked only for the rationale.

use std::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const CONFIG_FILE: &str = "evaluation/config.yaml";
pub const CONFIG_DIR: &str = "evaluation";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationDoc {
    pub evaluation: EvaluationConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationConfig {
    pub enabled: bool,
    pub session_close: SessionCloseConfig,
    pub worker: WorkerConfig,
    pub judge_defaults: JudgeRef,
    pub report: ReportConfig,
    pub criteria: Vec<CriterionConfig>,
}

/// The zoomed-out report: a model reads the judge's aggregate figures and
/// worst and best sessions for a scope and writes themes and actions.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportConfig {
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
    pub instructions: PathBuf,
    // Why: the window the report reads; the staleness rule decides when a
    // scope is re-read — after this many new judge runs, or this many hours
    // with at least one new run.
    pub window_days: u32,
    pub min_new_runs: u32,
    pub max_age_hours: u32,
    pub input_token_budget: u32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SessionCloseConfig {
    pub quiet_minutes: u32,
    pub min_user_prompts: u32,
    pub lookback_hours: u32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerConfig {
    pub batch_size: u32,
    pub lease_seconds: u32,
    pub max_attempts: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct JudgeRef {
    pub provider: String,
    pub model: String,
    pub max_output_tokens: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriterionConfig {
    pub name: CriterionName,
    pub scoring: Scoring,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub max_output_tokens: Option<u32>,
    pub instructions: PathBuf,
    pub inputs: Vec<InputSlice>,
    #[serde(default)]
    pub metric: Option<MetricThresholds>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CriterionName {
    Intent,
    Quality,
    Speed,
    Cost,
}

impl CriterionName {
    pub const ALL: [Self; 4] = [Self::Intent, Self::Quality, Self::Speed, Self::Cost];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Intent => "intent",
            Self::Quality => "quality",
            Self::Speed => "speed",
            Self::Cost => "cost",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|name| name.as_str() == value)
    }
}

impl fmt::Display for CriterionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scoring {
    Judge,
    Metric,
}

impl Scoring {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Judge => "judge",
            Self::Metric => "metric",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InputSlice {
    UserMessages,
    LastAssistantMessage,
    TranscriptTail,
    Skills,
    SessionMetadata,
    Activity,
    Metrics,
}

impl InputSlice {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserMessages => "user_messages",
            Self::LastAssistantMessage => "last_assistant_message",
            Self::TranscriptTail => "transcript_tail",
            Self::Skills => "skills",
            Self::SessionMetadata => "session_metadata",
            Self::Activity => "activity",
            Self::Metrics => "metrics",
        }
    }
}

/// Thresholds for the two deterministic criteria. Which variant a criterion
/// may carry is checked in `validate`, not by serde, so the error names the
/// criterion rather than an untagged-enum mismatch.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields, untagged)]
pub enum MetricThresholds {
    Speed {
        target_minutes_per_prompt: f64,
        target_latency_ms: i64,
    },
    Cost {
        budget_microdollars_per_prompt: i64,
        expected_cache_read_ratio: f64,
    },
}

impl MetricThresholds {
    #[must_use]
    pub fn describe(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::Speed {
                target_minutes_per_prompt,
                target_latency_ms,
            } => vec![
                (
                    "target minutes per prompt",
                    format!("{target_minutes_per_prompt:.1}"),
                ),
                ("target latency", format!("{target_latency_ms} ms")),
            ],
            Self::Cost {
                budget_microdollars_per_prompt,
                expected_cache_read_ratio,
            } => vec![
                (
                    "budget per prompt",
                    format!(
                        "${:.4}",
                        *budget_microdollars_per_prompt as f64 / 1_000_000.0
                    ),
                ),
                (
                    "expected cache-read ratio",
                    format!("{expected_cache_read_ratio:.2}"),
                ),
            ],
        }
    }
}

/// One criterion with every default resolved and its instructions read.
#[derive(Debug, Clone, Serialize)]
pub struct LoadedCriterion {
    pub name: CriterionName,
    pub scoring: Scoring,
    pub provider: String,
    pub model: String,
    pub max_output_tokens: u32,
    pub instructions_path: String,
    pub instructions_markdown: String,
    pub inputs: Vec<InputSlice>,
    pub metric: Option<MetricThresholds>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadedReport {
    pub provider: String,
    pub model: String,
    pub max_output_tokens: u32,
    pub instructions_path: String,
    pub instructions_markdown: String,
    pub window_days: u32,
    pub min_new_runs: u32,
    pub max_age_hours: u32,
    pub input_token_budget: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoadedEvaluation {
    pub enabled: bool,
    pub session_close: SessionCloseConfig,
    pub worker: WorkerConfig,
    pub report: LoadedReport,
    pub criteria: Vec<LoadedCriterion>,
    // Why: sha256 over the YAML and every instruction file, stamped on each run
    // so a score is always traceable to the instructions that produced it.
    pub config_hash: String,
}

impl LoadedEvaluation {
    #[must_use]
    pub fn criterion(&self, name: CriterionName) -> Option<&LoadedCriterion> {
        self.criteria.iter().find(|c| c.name == name)
    }
}
