//! `evals/<agent_id>.yaml`: an agent's eval suite. The assertion set is
//! closed; an unknown type or key is an error, never ignored.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::validate::{
    definition_error, is_case_id, is_identifier, is_safe_relative_path, require,
};
use crate::error::{FarmError, FarmResult};

pub const DEFAULT_JUDGE_THRESHOLD: u8 = 70;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Assertion {
    FileExists {
        path: String,
    },
    OutputContains {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        text: String,
    },
    Regex {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        path: Option<String>,
        pattern: String,
    },
    NoToolOutsideAllowlist {},
    MaxTurns {
        value: u32,
    },
    MaxCostUsd {
        value: f64,
    },
    SkillInvoked {
        skill: String,
    },
}

impl Assertion {
    fn path(&self) -> Option<&str> {
        match self {
            Self::FileExists { path } => Some(path),
            Self::OutputContains { path, .. } | Self::Regex { path, .. } => path.as_deref(),
            _ => None,
        }
    }

    fn validate(&self, at: &str) -> FarmResult<()> {
        if let Some(path) = self.path() {
            require(
                is_safe_relative_path(path),
                at,
                format!("assertion path {path:?} must be relative without `..`"),
            )?;
        }
        match self {
            Self::OutputContains { text, .. } => {
                require(!text.is_empty(), at, "output_contains.text is empty")
            },
            Self::Regex { pattern, .. } => {
                regex::Regex::new(pattern)
                    .map(drop)
                    .map_err(|source| FarmError::InvalidRegex {
                        at: at.to_owned(),
                        pattern: pattern.clone(),
                        source,
                    })
            },
            Self::MaxCostUsd { value } => require(
                value.is_finite() && *value >= 0.0,
                at,
                "max_cost_usd.value must be a non-negative number",
            ),
            Self::SkillInvoked { skill } => {
                require(!skill.trim().is_empty(), at, "skill_invoked.skill is empty")
            },
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvalCaseDefinition {
    pub id: String,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub fixture: Option<String>,
    #[serde(default)]
    pub timeout_secs: Option<u32>,
    pub assertions: Vec<Assertion>,
    #[serde(default)]
    pub judge_rubric: Option<String>,
    #[serde(default)]
    pub judge_threshold: Option<u8>,
}

impl EvalCaseDefinition {
    #[must_use]
    pub fn effective_judge_threshold(&self) -> Option<u8> {
        self.judge_rubric
            .as_ref()
            .map(|_| self.judge_threshold.unwrap_or(DEFAULT_JUDGE_THRESHOLD))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct SuiteFile {
    schema: u32,
    agent: String,
    cases: Vec<EvalCaseDefinition>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EvalSuiteDefinition {
    pub agent: String,
    pub cases: Vec<EvalCaseDefinition>,
    pub source_path: String,
}

fn validate_case(path: &str, case: &EvalCaseDefinition) -> FarmResult<()> {
    let at = format!("{path}: case {:?}", case.id);
    require(
        is_case_id(&case.id),
        path,
        format!("case id {:?} does not match ^[a-z0-9_]+$", case.id),
    )?;
    if let Some(fixture) = &case.fixture {
        require(
            is_identifier(fixture),
            &at,
            format!("fixture {fixture:?} is not a valid fixture name"),
        )?;
    }
    require(
        case.timeout_secs != Some(0),
        &at,
        "timeout_secs must be > 0",
    )?;
    require(
        case.judge_threshold.is_none_or(|t| t <= 100),
        &at,
        "judge_threshold must be within 0..=100",
    )?;
    case.assertions.iter().try_for_each(|a| a.validate(&at))
}

pub fn parse_eval_suite(path: &str, text: &str) -> FarmResult<EvalSuiteDefinition> {
    let file: SuiteFile = serde_yaml::from_str(text).map_err(|e| definition_error(path, e))?;
    require(
        file.schema == 1,
        path,
        format!("schema {} is not 1", file.schema),
    )?;
    require(
        is_identifier(&file.agent),
        path,
        format!("agent {:?} is not a valid agent id", file.agent),
    )?;
    require(
        !file.cases.is_empty(),
        path,
        "cases must be a non-empty list",
    )?;
    let mut seen = BTreeSet::new();
    for case in &file.cases {
        validate_case(path, case)?;
        require(
            seen.insert(case.id.as_str()),
            path,
            format!("case id {:?} appears twice", case.id),
        )?;
    }
    Ok(EvalSuiteDefinition {
        agent: file.agent,
        cases: file.cases,
        source_path: path.to_owned(),
    })
}
