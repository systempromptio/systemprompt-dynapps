//! The judge's response shape: the JSON schema handed to the provider API and
//! the parser that normalises what comes back. The API enforces the schema, so
//! parsing only clamps every list and number into the bounds the table
//! expects; an unknown category or outcome is a parse error the caller
//! retries. `Classification::unreadable` is what an unreadable conversation is
//! recorded as, so it is not retried forever.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    Development,
    BusinessAnalysis,
    Operations,
    AdminConfig,
    WritingComms,
    ResearchLearning,
    Other,
}

impl Category {
    pub const ALL: [Self; 7] = [
        Self::Development,
        Self::BusinessAnalysis,
        Self::Operations,
        Self::AdminConfig,
        Self::WritingComms,
        Self::ResearchLearning,
        Self::Other,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::BusinessAnalysis => "business-analysis",
            Self::Operations => "operations",
            Self::AdminConfig => "admin-config",
            Self::WritingComms => "writing-comms",
            Self::ResearchLearning => "research-learning",
            Self::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Outcome {
    Achieved,
    Partial,
    Abandoned,
    Unclear,
}

impl Outcome {
    pub const ALL: [Self; 4] = [
        Self::Achieved,
        Self::Partial,
        Self::Abandoned,
        Self::Unclear,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Achieved => "achieved",
            Self::Partial => "partial",
            Self::Abandoned => "abandoned",
            Self::Unclear => "unclear",
        }
    }
}

/// The judge's verdict after normalisation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Classification {
    pub title: String,
    pub category: Category,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub outcome: Outcome,
    #[serde(default)]
    pub skills_observed: Vec<String>,
    #[serde(default)]
    pub confidence: f32,
    pub completion: u8,
    #[serde(default)]
    pub completion_rationale: String,
}

impl Classification {
    #[must_use]
    pub fn unreadable() -> Self {
        Self {
            title: "Unreadable conversation".to_owned(),
            category: Category::Other,
            summary: "No readable transcript was retained for this conversation.".to_owned(),
            tags: Vec::new(),
            outcome: Outcome::Unclear,
            skills_observed: Vec::new(),
            confidence: 0.0,
            completion: 0,
            completion_rationale: "No transcript to judge.".to_owned(),
        }
    }
}

const MAX_TAGS: usize = 8;
const MAX_SKILLS: usize = 20;
const MAX_SUMMARY_CHARS: usize = 600;
const MAX_TITLE_CHARS: usize = 80;
const MAX_RATIONALE_CHARS: usize = 400;

// Why: Gemini's responseSchema is an OpenAPI subset (enums and `required`
// only); core's sanitizer strips anything richer.
// JSON: outgoing fixed-shape
#[must_use]
pub fn classification_schema() -> serde_json::Value {
    let categories: Vec<&str> = Category::ALL.iter().map(|c| c.as_str()).collect();
    let outcomes: Vec<&str> = Outcome::ALL.iter().map(|o| o.as_str()).collect();
    serde_json::json!({
        "type": "object",
        "properties": {
            "title": {"type": "string"},
            "category": {"type": "string", "enum": categories},
            "summary": {"type": "string"},
            "tags": {"type": "array", "items": {"type": "string"}},
            "outcome": {"type": "string", "enum": outcomes},
            "skills_observed": {"type": "array", "items": {"type": "string"}},
            "confidence": {"type": "number"},
            "completion": {"type": "integer"},
            "completion_rationale": {"type": "string"}
        },
        "required": ["title", "category", "summary", "tags", "outcome", "skills_observed",
                     "confidence", "completion", "completion_rationale"]
    })
}

pub fn parse_classification(content: &str) -> Result<Classification, serde_json::Error> {
    let mut parsed: Classification = serde_json::from_str(content)?;
    parsed.summary = parsed
        .summary
        .trim()
        .chars()
        .take(MAX_SUMMARY_CHARS)
        .collect();
    parsed.title = clip(&parsed.title, MAX_TITLE_CHARS);
    parsed.completion_rationale = clip(&parsed.completion_rationale, MAX_RATIONALE_CHARS);
    parsed.completion = parsed.completion.min(100);
    parsed.tags = dedupe_lower(parsed.tags, MAX_TAGS);
    parsed.skills_observed = dedupe_lower(parsed.skills_observed, MAX_SKILLS);
    parsed.confidence = parsed.confidence.clamp(0.0, 1.0);
    Ok(parsed)
}

fn clip(value: &str, max_chars: usize) -> String {
    value.trim().chars().take(max_chars).collect()
}

fn dedupe_lower(values: Vec<String>, cap: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for value in values {
        let v = value.trim().to_lowercase();
        if v.is_empty() || out.contains(&v) {
            continue;
        }
        out.push(v);
        if out.len() == cap {
            break;
        }
    }
    out
}
