//! A generated report as the Evaluation pages show it: the model's themes
//! and recommendations with every cited id turned into a console link. Only
//! ids the digest contained survive parsing, so every link here resolves.

use serde::Serialize;
use systemprompt::identifiers::{ContextId, MarketplaceId, SessionId};

use crate::handlers::ssr::analysis_urls::{
    ANALYSIS_CONVERSATIONS_URL, ANALYSIS_EVALUATION_URL, analysis_skill_url,
    analysis_version_url,
};
use crate::handlers::ssr::entity_urls::{context_detail_url, session_detail_url};
use crate::handlers::ssr::format::{format_cost, local_time, relative_time};
use crate::repositories::evaluation::reports::{EvaluationReportRow, ReportScope};
use crate::services::evaluation::report::schema::{
    ReportEntity, ReportFindings, ReportRecommendation, ReportTheme,
};

#[derive(Debug, Serialize)]
pub(crate) struct EntityLinkView {
    pub kind: &'static str,
    pub label: String,
    pub href: String,
    pub note: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ThemeView {
    pub index: usize,
    pub kind_label: &'static str,
    pub severity: String,
    pub severity_tone: &'static str,
    pub title: String,
    pub detail: String,
    pub metric_display: Option<String>,
    pub evidence: Vec<EntityLinkView>,
}

#[derive(Debug, Serialize)]
pub(crate) struct RecommendationView {
    pub action: String,
    pub rationale: String,
    pub priority: String,
    pub priority_label: &'static str,
    pub priority_tone: &'static str,
    pub theme_title: Option<String>,
    pub targets: Vec<EntityLinkView>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ReportView {
    pub id: String,
    pub href: String,
    pub scope_label: String,
    pub headline: String,
    pub assessment: String,
    pub assessment_tone: &'static str,
    pub generated_display: String,
    pub generated_relative: String,
    pub window_display: String,
    pub model: String,
    pub trigger: String,
    pub source_run_count: i64,
    pub cost: String,
    pub version_short: Option<String>,
    pub themes: Vec<ThemeView>,
    pub recommendations: Vec<RecommendationView>,
    pub has_findings: bool,
}

const fn kind_label(kind: &str) -> &'static str {
    match kind.as_bytes() {
        b"slow_skill" => "Slow skill",
        b"costly_skill" => "Costly skill",
        b"low_quality_skill" => "Low quality",
        b"intent_mismatch" => "Intent missed",
        b"high_token_category" => "Token-heavy category",
        b"version_regression" => "Version regressed",
        b"version_improvement" => "Version improved",
        b"failing_judge" => "Judge failing",
        _ => "Observation",
    }
}

const fn severity_tone(severity: &str) -> &'static str {
    match severity.as_bytes() {
        b"critical" => "err",
        b"warning" => "warn",
        _ => "info",
    }
}

const fn priority(priority: &str) -> (&'static str, &'static str) {
    match priority.as_bytes() {
        b"now" => ("Now", "err"),
        b"later" => ("Later", "muted"),
        _ => ("This week", "warn"),
    }
}

const fn assessment_tone(assessment: &str) -> &'static str {
    match assessment.as_bytes() {
        b"healthy" => "ok",
        b"degraded" => "err",
        _ => "warn",
    }
}

fn metric_display(theme: &ReportTheme) -> Option<String> {
    let m = &theme.metric;
    if m.criterion == "none" {
        return None;
    }
    let unit = if m.criterion == "tokens" { "" } else { "/100" };
    Some(format!(
        "{}: {:.0}{unit} vs {:.0}{unit} over {} sessions",
        m.criterion, m.value, m.baseline, m.sessions
    ))
}

// Why: a version is cited by its hash alone; the digest the report was
// written from says which marketplace it belongs to.
fn version_marketplace(row: &EvaluationReportRow, hash: &str) -> Option<MarketplaceId> {
    if let ReportScope::Marketplace(id) = &row.scope {
        return Some(MarketplaceId::new(id));
    }
    row.inputs
        .get("versions")?
        .as_array()?
        .iter()
        .find(|v| v.get("version").and_then(|h| h.as_str()) == Some(hash))
        .and_then(|v| v.get("marketplace_id").and_then(|m| m.as_str()))
        .map(MarketplaceId::new)
}

fn link(row: &EvaluationReportRow, entity: &ReportEntity) -> EntityLinkView {
    let (kind, label, href) = match entity.kind.as_str() {
        "skill" => ("skill", entity.id.clone(), analysis_skill_url(&entity.id)),
        "conversation" => (
            "conversation",
            entity.id.chars().take(8).collect(),
            context_detail_url(&ContextId::new(&entity.id)),
        ),
        "session" => (
            "session",
            entity.id.chars().take(8).collect(),
            format!("{}#evaluation", session_detail_url(&SessionId::new(&entity.id))),
        ),
        "version" => (
            "version",
            entity.id.chars().take(12).collect(),
            version_marketplace(row, &entity.id).map_or_else(
                || ANALYSIS_EVALUATION_URL.to_owned(),
                |m| analysis_version_url(&m, Some(&entity.id)),
            ),
        ),
        _ => (
            "category",
            entity.id.clone(),
            format!(
                "{ANALYSIS_CONVERSATIONS_URL}?category={}",
                urlencoding::encode(&entity.id)
            ),
        ),
    };
    EntityLinkView {
        kind,
        label,
        href,
        note: entity.note.clone(),
    }
}

fn theme_view(row: &EvaluationReportRow, index: usize, theme: &ReportTheme) -> ThemeView {
    ThemeView {
        index,
        kind_label: kind_label(&theme.kind),
        severity: theme.severity.clone(),
        severity_tone: severity_tone(&theme.severity),
        title: theme.title.clone(),
        detail: theme.detail.clone(),
        metric_display: metric_display(theme),
        evidence: theme.evidence.iter().map(|e| link(row, e)).collect(),
    }
}

fn recommendation_view(
    row: &EvaluationReportRow,
    findings: &ReportFindings,
    rec: &ReportRecommendation,
) -> RecommendationView {
    let (priority_label, priority_tone) = priority(&rec.priority);
    RecommendationView {
        action: rec.action.clone(),
        rationale: rec.rationale.clone(),
        priority: rec.priority.clone(),
        priority_label,
        priority_tone,
        theme_title: usize::try_from(rec.theme_index)
            .ok()
            .and_then(|i| findings.themes.get(i))
            .map(|t| t.title.clone()),
        targets: rec.targets.iter().map(|t| link(row, t)).collect(),
    }
}

impl ReportView {
    pub(crate) fn of(row: &EvaluationReportRow) -> Self {
        let findings: Option<ReportFindings> = row
            .findings
            .clone()
            .and_then(|f| serde_json::from_value(f).ok());
        let (headline, assessment, themes, recommendations) = match &findings {
            Some(f) => (
                f.headline.clone(),
                f.overall_assessment.clone(),
                f.themes
                    .iter()
                    .enumerate()
                    .map(|(i, t)| theme_view(row, i, t))
                    .collect(),
                f.recommendations
                    .iter()
                    .map(|r| recommendation_view(row, f, r))
                    .collect(),
            ),
            None => (
                "No findings recorded".to_owned(),
                "watch".to_owned(),
                Vec::new(),
                Vec::new(),
            ),
        };
        Self {
            id: row.id.clone(),
            href: format!("{ANALYSIS_EVALUATION_URL}/report/{}", row.id),
            scope_label: match &row.scope {
                ReportScope::Global => "Whole instance".to_owned(),
                ReportScope::Marketplace(id) => format!("Marketplace {id}"),
            },
            headline,
            assessment_tone: assessment_tone(&assessment),
            assessment,
            generated_display: row.generated_at.map(local_time).unwrap_or_default(),
            generated_relative: row.generated_at.map(relative_time).unwrap_or_default(),
            window_display: format!(
                "{} → {}",
                row.window_start.format("%b %-d"),
                row.window_end.format("%b %-d")
            ),
            model: row
                .model
                .clone()
                .map(|m| format!("{}/{m}", row.provider.clone().unwrap_or_default()))
                .unwrap_or_default(),
            trigger: row.trigger.clone(),
            source_run_count: row.source_run_count,
            cost: row.cost_microdollars.map_or_else(|| "—".to_owned(), format_cost),
            version_short: row
                .marketplace_hash
                .as_deref()
                .map(|h| h.chars().take(12).collect()),
            has_findings: findings.is_some(),
            themes,
            recommendations,
        }
    }
}
