//! The digest an AI report is written from — `digest.sql` shaped into typed
//! rows. Its text rendering (`digest_text`) is deterministic for a given
//! digest, so a regenerated report answers the same evidence. Scope filters
//! stay raw text, not typed ids: a filter query's values are bound straight
//! into the digest SQL and echoed into the stored inputs.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Json;

/// What narrows the record for one report.
#[derive(Debug, Clone, Default)]
pub struct DigestScope {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub subject_ids: Option<Vec<String>>,
    pub marketplace_key: Option<String>,
    pub skill: Option<String>,
    pub user_key: Option<String>,
    pub model: Option<String>,
    pub client_kind: Option<String>,
    pub category: Option<String>,
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DigestTotals {
    pub conversations: i64,
    pub people: i64,
    pub turns: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub errors: i64,
    pub denied: i64,
    pub safety_findings: i64,
    pub artifacts: i64,
    pub tool_calls: i64,
    pub judged: i64,
    pub completion_avg: Option<f64>,
    pub achieved: i64,
    pub partial: i64,
    pub abandoned: i64,
    pub unclear: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestSkill {
    pub skill: String,
    pub invocations: i64,
    pub conversations: i64,
    pub cost_microdollars: i64,
    pub completion_avg: Option<f64>,
    pub errors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestModel {
    pub model: String,
    pub conversations: i64,
    pub requests: i64,
    pub cost_microdollars: i64,
    pub errors: i64,
    pub completion_avg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestPerson {
    #[serde(rename = "user_id")]
    pub user_key: String,
    pub display_name: Option<String>,
    pub conversations: i64,
    pub cost_microdollars: i64,
    pub completion_avg: Option<f64>,
    pub errors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestJudged {
    #[serde(rename = "context_id")]
    pub context_key: String,
    pub title: String,
    pub judge_title: Option<String>,
    pub completion: Option<i16>,
    pub outcome: Option<String>,
    pub category: Option<String>,
    pub cost_microdollars: i64,
    pub turns: i64,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestOutlier {
    #[serde(rename = "context_id")]
    pub context_key: String,
    pub title: String,
    pub cost_microdollars: i64,
    pub turns: i64,
    pub tokens: i64,
    pub completion: Option<i16>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestDenial {
    pub tool_name: String,
    pub denied: i64,
    pub people: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestIntent {
    pub category: String,
    pub conversations: i64,
    pub completion_avg: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestClient {
    pub client_kind: String,
    pub conversations: i64,
    pub cost_microdollars: i64,
}

/// Everything the model sees, and everything the report page shows as its
/// evidence base.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportDigest {
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub totals: DigestTotals,
    #[serde(default)]
    pub skills: Vec<DigestSkill>,
    #[serde(default)]
    pub models: Vec<DigestModel>,
    #[serde(default)]
    pub people: Vec<DigestPerson>,
    #[serde(default)]
    pub worst: Vec<DigestJudged>,
    #[serde(default)]
    pub best: Vec<DigestJudged>,
    #[serde(default)]
    pub outliers: Vec<DigestOutlier>,
    #[serde(default)]
    pub denials: Vec<DigestDenial>,
    #[serde(default)]
    pub intents: Vec<DigestIntent>,
    #[serde(default)]
    pub clients: Vec<DigestClient>,
}

pub async fn get_report_digest(
    pool: &PgPool,
    scope: &DigestScope,
) -> Result<ReportDigest, sqlx::Error> {
    let row = sqlx::query_file!(
        "src/repositories/analysis/reports/digest.sql",
        scope.window_start,
        scope.window_end,
        scope.subject_ids.as_deref(),
        scope.marketplace_key,
        scope.skill,
        scope.user_key,
        scope.model,
        scope.client_kind,
        scope.category,
        scope.outcome,
    )
    .fetch_one(pool)
    .await?;
    let Json(digest) = row.payload;
    Ok(digest)
}
