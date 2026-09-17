//! The JSON a report must come back as, and the parser that clamps it and
//! drops every id the model was not shown — which is what lets the page
//! turn evidence into links without trusting the model.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const MAX_THEMES: usize = 8;
pub const MAX_RECOMMENDATIONS: usize = 8;
pub const MAX_EVIDENCE: usize = 6;
const MAX_TEXT: usize = 1200;
const MAX_TITLE: usize = 160;

pub const THEME_KINDS: [&str; 9] = [
    "slow_skill",
    "costly_skill",
    "low_quality_skill",
    "intent_mismatch",
    "high_token_category",
    "version_regression",
    "version_improvement",
    "failing_judge",
    "other",
];
pub const ENTITY_KINDS: [&str; 5] = ["skill", "conversation", "session", "version", "category"];

// JSON: the response schema handed to the provider API — objects, enums,
// arrays and `required` only, the subset every provider's structured output
// accepts.
#[must_use]
pub fn report_findings_schema() -> Value {
    let entity = json!({
        "type": "object",
        "properties": {
            "kind": {"type": "string", "enum": ENTITY_KINDS},
            "id": {"type": "string"},
            "note": {"type": "string"}
        },
        "required": ["kind", "id", "note"]
    });
    json!({
        "type": "object",
        "properties": {
            "headline": {"type": "string"},
            "overall_assessment": {"type": "string", "enum": ["healthy", "watch", "degraded"]},
            "themes": {"type": "array", "items": {
                "type": "object",
                "properties": {
                    "kind": {"type": "string", "enum": THEME_KINDS},
                    "severity": {"type": "string", "enum": ["info", "warning", "critical"]},
                    "title": {"type": "string"},
                    "detail": {"type": "string"},
                    "metric": {"type": "object", "properties": {
                        "criterion": {"type": "string", "enum": ["intent", "quality", "speed", "cost", "tokens", "none"]},
                        "value": {"type": "number"},
                        "baseline": {"type": "number"},
                        "sessions": {"type": "integer"}
                    }, "required": ["criterion", "value", "baseline", "sessions"]},
                    "evidence": {"type": "array", "items": entity}
                },
                "required": ["kind", "severity", "title", "detail", "metric", "evidence"]
            }},
            "recommendations": {"type": "array", "items": {
                "type": "object",
                "properties": {
                    "action": {"type": "string"},
                    "rationale": {"type": "string"},
                    "priority": {"type": "string", "enum": ["now", "this_week", "later"]},
                    "theme_index": {"type": "integer"},
                    "targets": {"type": "array", "items": entity}
                },
                "required": ["action", "rationale", "priority", "theme_index", "targets"]
            }}
        },
        "required": ["headline", "overall_assessment", "themes", "recommendations"]
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFindings {
    pub headline: String,
    pub overall_assessment: String,
    #[serde(default)]
    pub themes: Vec<ReportTheme>,
    #[serde(default)]
    pub recommendations: Vec<ReportRecommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTheme {
    pub kind: String,
    pub severity: String,
    pub title: String,
    pub detail: String,
    pub metric: ReportMetric,
    #[serde(default)]
    pub evidence: Vec<ReportEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetric {
    pub criterion: String,
    pub value: f64,
    pub baseline: f64,
    pub sessions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecommendation {
    pub action: String,
    pub rationale: String,
    pub priority: String,
    pub theme_index: i64,
    #[serde(default)]
    pub targets: Vec<ReportEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEntity {
    pub kind: String,
    pub id: String,
    #[serde(default)]
    pub note: String,
}

/// The ids the digest showed the model, by kind; anything else is dropped.
#[derive(Debug, Default)]
pub struct KnownIds {
    pub skills: HashSet<String>,
    pub conversations: HashSet<String>,
    pub sessions: HashSet<String>,
    pub versions: HashSet<String>,
    pub categories: HashSet<String>,
}

impl KnownIds {
    fn knows(&self, entity: &ReportEntity) -> bool {
        match entity.kind.as_str() {
            "skill" => self.skills.contains(&entity.id),
            "conversation" => self.conversations.contains(&entity.id),
            "session" => self.sessions.contains(&entity.id),
            "version" => self.versions.contains(&entity.id),
            "category" => self.categories.contains(&entity.id),
            _ => false,
        }
    }
}

fn clip(value: &str, max: usize) -> String {
    value.trim().chars().take(max).collect()
}

fn keep_known(entities: Vec<ReportEntity>, known: &KnownIds, cap: usize) -> Vec<ReportEntity> {
    let mut out: Vec<ReportEntity> = Vec::new();
    for mut e in entities {
        e.id = e.id.trim().to_owned();
        if !known.knows(&e) || out.iter().any(|k| k.kind == e.kind && k.id == e.id) {
            continue;
        }
        e.note = clip(&e.note, 300);
        out.push(e);
        if out.len() == cap {
            break;
        }
    }
    out
}

// Why: clamps every list and string, drops unknown theme kinds and any
// evidence or target the model was not shown, and re-points a
// recommendation whose theme index is out of range at no theme (-1).
pub fn parse_findings(
    content: &str,
    known: &KnownIds,
) -> Result<ReportFindings, serde_json::Error> {
    let raw: ReportFindings = serde_json::from_str(content)?;
    let themes: Vec<ReportTheme> = raw
        .themes
        .into_iter()
        .filter(|t| THEME_KINDS.contains(&t.kind.as_str()))
        .take(MAX_THEMES)
        .map(|t| ReportTheme {
            kind: t.kind,
            severity: match t.severity.as_str() {
                "critical" => "critical".to_owned(),
                "warning" => "warning".to_owned(),
                _ => "info".to_owned(),
            },
            title: clip(&t.title, MAX_TITLE),
            detail: clip(&t.detail, MAX_TEXT),
            metric: ReportMetric {
                criterion: t.metric.criterion,
                value: t.metric.value,
                baseline: t.metric.baseline,
                sessions: t.metric.sessions.max(0),
            },
            evidence: keep_known(t.evidence, known, MAX_EVIDENCE),
        })
        .collect();
    let theme_count = i64::try_from(themes.len()).unwrap_or(0);
    let recommendations = raw
        .recommendations
        .into_iter()
        .take(MAX_RECOMMENDATIONS)
        .map(|r| ReportRecommendation {
            action: clip(&r.action, MAX_TEXT),
            rationale: clip(&r.rationale, MAX_TEXT),
            priority: match r.priority.as_str() {
                "now" => "now".to_owned(),
                "later" => "later".to_owned(),
                _ => "this_week".to_owned(),
            },
            theme_index: if (0..theme_count).contains(&r.theme_index) {
                r.theme_index
            } else {
                -1
            },
            targets: keep_known(r.targets, known, MAX_EVIDENCE),
        })
        .collect();
    Ok(ReportFindings {
        headline: clip(&raw.headline, MAX_TITLE),
        overall_assessment: match raw.overall_assessment.as_str() {
            "healthy" => "healthy".to_owned(),
            "degraded" => "degraded".to_owned(),
            _ => "watch".to_owned(),
        },
        themes,
        recommendations,
    })
}
