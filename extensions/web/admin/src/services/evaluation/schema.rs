//! The JSON the judge must return, as a schema the gateway enforces.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::repositories::evaluation::{CriterionName, Scoring};

// Why: judged criteria return a score; metric criteria have theirs fixed and
// return only the explanation. Intent also names the session.
#[must_use]
pub fn judge_output_schema(criterion: CriterionName, scoring: Scoring) -> Value {
    let mut properties = json!({
        "rationale": { "type": "string", "maxLength": 2000 },
        "evidence": {
            "type": "array",
            "maxItems": 8,
            "items": {
                "type": "object",
                "properties": {
                    "quote": { "type": "string", "maxLength": 400 },
                    "note": { "type": "string", "maxLength": 300 }
                },
                "required": ["quote", "note"],
                "additionalProperties": false
            }
        }
    });
    let mut required = vec!["rationale", "evidence"];
    if scoring == Scoring::Judge {
        properties["score"] = json!({ "type": "integer", "minimum": 0, "maximum": 100 });
        required.push("score");
    }
    if criterion == CriterionName::Intent {
        properties["title"] = json!({ "type": "string", "maxLength": 80 });
        required.push("title");
    }
    json!({
        "type": "object",
        "properties": properties,
        "required": required,
        "additionalProperties": false
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct JudgeOutput {
    #[serde(default)]
    pub score: Option<i64>,
    pub rationale: String,
    #[serde(default)]
    pub evidence: Vec<JudgeEvidence>,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct JudgeEvidence {
    pub quote: String,
    pub note: String,
}
