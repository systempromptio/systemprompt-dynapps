//! The digest a report is written from: every figure the model sees, as one
//! JSON document sized to a token budget, plus the set of ids it contains so
//! the parser can refuse any the model invents.
//!
//! The rationales of the worst and best sessions are the largest item and
//! the most useful one, so they are added last, worst first, until the
//! budget is spent.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{MarketplaceId, SessionId};

use super::schema::KnownIds;
use crate::repositories::evaluation::report_inputs::{
    ReportCategoryFigure, ReportOutlierSession, ReportScopeTotals, ReportSkillFigure, ReportWindow,
    get_report_scope_totals, list_report_category_figures, list_report_outlier_sessions,
    list_report_skill_figures,
};
use crate::repositories::evaluation::reports::ReportScope;
use crate::repositories::evaluation::version_scores::{
    VersionScoreRow, list_marketplace_score_rollups, list_version_scores,
};

const SKILL_LIMIT: i64 = 40;
const OUTLIERS_PER_SIDE: i64 = 6;
const RATIONALE_CHARS: usize = 600;
// Why: the same rough ratio the classifier budgets its transcript by.
const CHARS_PER_TOKEN: usize = 4;

#[derive(Debug, Serialize)]
pub struct ReportDigest {
    pub scope: String,
    pub window_start: String,
    pub window_end: String,
    pub window_days: i64,
    pub totals: ReportScopeTotals,
    pub skills: Vec<ReportSkillFigure>,
    pub versions: Vec<VersionDigest>,
    pub categories: Vec<ReportCategoryFigure>,
    pub sessions: Vec<SessionDigest>,
    // Why: how many outliers the budget left out, so the model knows the
    // list is a sample.
    pub sessions_omitted: usize,
}

#[derive(Debug, Serialize)]
pub struct VersionDigest {
    pub marketplace_id: MarketplaceId,
    pub version: String,
    pub is_current: bool,
    pub sessions: i64,
    pub scored_sessions: i64,
    pub intent: Option<f64>,
    pub quality: Option<f64>,
    pub speed: Option<f64>,
    pub cost: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct SessionDigest {
    pub session_id: SessionId,
    pub conversation_id: Option<String>,
    pub side: &'static str,
    pub title: Option<String>,
    pub category: Option<String>,
    pub skills: Vec<String>,
    pub intent: Option<i16>,
    pub quality: Option<i16>,
    pub speed: Option<i16>,
    pub cost: Option<i16>,
    pub quality_rationale: Option<String>,
    pub evidence_quote: Option<String>,
    pub intent_rationale: Option<String>,
}

#[derive(Debug)]
pub struct Assembled {
    pub digest: ReportDigest,
    pub known: KnownIds,
    pub json: String,
}

fn clip(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

fn session_digest(s: ReportOutlierSession) -> SessionDigest {
    SessionDigest {
        session_id: s.session_id,
        conversation_id: s.context_id.map(|c| c.as_str().to_owned()),
        side: if s.is_worst { "worst" } else { "best" },
        title: s.title.map(|t| clip(&t, 120)),
        category: s.category,
        skills: s.skills,
        intent: s.intent,
        quality: s.quality,
        speed: s.speed,
        cost: s.cost,
        quality_rationale: s.quality_rationale.map(|r| clip(&r, RATIONALE_CHARS)),
        evidence_quote: s.evidence_quote.map(|q| clip(&q, 300)),
        intent_rationale: s.intent_rationale.map(|r| clip(&r, RATIONALE_CHARS / 2)),
    }
}

async fn list_versions(
    pool: &PgPool,
    scope: &ReportScope,
) -> Result<Vec<VersionDigest>, sqlx::Error> {
    let rows: Vec<VersionScoreRow> = match scope {
        ReportScope::Marketplace(id) => list_version_scores(pool, &MarketplaceId::new(id)).await?,
        ReportScope::Global => {
            let mut all = Vec::new();
            for rollup in list_marketplace_score_rollups(pool).await? {
                all.extend(list_version_scores(pool, &rollup.marketplace_id).await?);
            }
            all
        },
    };
    Ok(rows
        .into_iter()
        .map(|v| VersionDigest {
            marketplace_id: v.marketplace_id,
            version: v.content_hash,
            is_current: v.is_current,
            sessions: v.sessions,
            scored_sessions: v.scored_sessions,
            intent: v.intent,
            quality: v.quality,
            speed: v.speed,
            cost: v.cost,
        })
        .collect())
}

// Why: serialise once per candidate size; the digest is small enough that
// re-serialising while trimming sessions costs nothing worth a smarter
// budget.
fn fit(mut digest: ReportDigest, budget_tokens: usize) -> (ReportDigest, String) {
    let budget = budget_tokens.saturating_mul(CHARS_PER_TOKEN);
    loop {
        // Why: discard-ok: an in-memory struct always serialises
        let json = serde_json::to_string(&digest).unwrap_or_default();
        if json.len() <= budget || digest.sessions.is_empty() {
            return (digest, json);
        }
        digest.sessions.pop();
        digest.sessions_omitted += 1;
    }
}

pub async fn assemble_digest(
    pool: &PgPool,
    scope: &ReportScope,
    window: ReportWindow,
    budget_tokens: usize,
) -> Result<Assembled, sqlx::Error> {
    let totals = get_report_scope_totals(pool, scope, window).await?;
    let skills = list_report_skill_figures(pool, scope, window, SKILL_LIMIT).await?;
    let versions = list_versions(pool, scope).await?;
    let categories = list_report_category_figures(pool, scope, window).await?;
    let outliers = list_report_outlier_sessions(pool, scope, window, OUTLIERS_PER_SIDE).await?;

    let mut known = KnownIds::default();
    known.skills.extend(skills.iter().map(|s| s.skill.clone()));
    known
        .versions
        .extend(versions.iter().map(|v| v.version.clone()));
    known
        .categories
        .extend(categories.iter().map(|c| c.category.clone()));
    let sessions: Vec<SessionDigest> = outliers.into_iter().map(session_digest).collect();
    known
        .sessions
        .extend(sessions.iter().map(|s| s.session_id.as_str().to_owned()));
    known
        .conversations
        .extend(sessions.iter().filter_map(|s| s.conversation_id.clone()));

    let digest = ReportDigest {
        scope: match scope {
            ReportScope::Global => "the whole instance".to_owned(),
            ReportScope::Marketplace(id) => format!("marketplace {id}"),
        },
        window_start: window.start.to_rfc3339(),
        window_end: window.end.to_rfc3339(),
        window_days: (window.end - window.start).num_days().max(1),
        totals,
        skills,
        versions,
        categories,
        sessions,
        sessions_omitted: 0,
    };
    let (digest, json) = fit(digest, budget_tokens);
    Ok(Assembled {
        digest,
        known,
        json,
    })
}
