//! Cohort shaping for the impact page: production rows grouped per skill, the
//! per-invocation evidence view, and the coverage shares the KPI strip reads.

use crate::repositories::analysis::version_impact;
use serde::Serialize;
use std::collections::BTreeMap;

// Why: a cohort is only comparable against another cohort of the same skill,
// so one skill's days and revisions are grouped together rather than left in a
// flat mixture a reader has to sort by eye.
#[derive(Serialize)]
pub(super) struct SkillCohort {
    pub(super) skill: String,
    pub(super) invocations: i64,
    pub(super) requests: i64,
    pub(super) rows: Vec<CohortRow>,
}

#[derive(Serialize)]
pub(super) struct CohortRow {
    pub(super) row: version_impact::VersionImpactRow,
    pub(super) revision: Option<String>,
    pub(super) cost: String,
    pub(super) latency: String,
    pub(super) quality: String,
}

#[derive(Serialize)]
pub(super) struct InvocationView {
    pub(super) row: version_impact::InvocationDrilldown,
    pub(super) revision: Option<String>,
    pub(super) attribution_tone: &'static str,
    pub(super) requests: String,
}

// Why: one invocation's evidence row, with its ids shortened for the table and
// its attribution turned into a badge tone, so the template carries no logic.
pub(super) fn evidence_views(
    rows: Vec<version_impact::InvocationDrilldown>,
) -> Vec<InvocationView> {
    rows.into_iter()
        .map(|row| InvocationView {
            revision: row.revision_id.as_deref().map(short),
            attribution_tone: if row.attribution_status == "verified" {
                "ok"
            } else {
                "warn"
            },
            requests: row
                .request_ids
                .iter()
                .map(String::as_str)
                .map(short)
                .collect::<Vec<_>>()
                .join(", "),
            row,
        })
        .collect()
}

pub(super) fn group_by_skill(rows: Vec<version_impact::VersionImpactRow>) -> Vec<SkillCohort> {
    let mut index_of: BTreeMap<String, usize> = BTreeMap::new();
    let mut cohorts: Vec<SkillCohort> = Vec::new();
    for row in rows {
        let index = *index_of.entry(row.skill.clone()).or_insert_with(|| {
            cohorts.push(SkillCohort {
                skill: row.skill.clone(),
                invocations: 0,
                requests: 0,
                rows: Vec::new(),
            });
            cohorts.len() - 1
        });
        let Some(cohort) = cohorts.get_mut(index) else {
            continue;
        };
        cohort.invocations = cohort.invocations.saturating_add(row.invocations);
        cohort.requests = cohort.requests.saturating_add(row.requests);
        cohort.rows.push(CohortRow {
            revision: row.revision_id.as_deref().map(short),
            cost: super::dollars(row.related_conversation_cost_microdollars),
            latency: row
                .average_latency_ms
                .map_or_else(missing, |value| format!("{value:.0} ms")),
            quality: row
                .average_quality_score
                .map_or_else(missing, |value| format!("{value:.2}")),
            row,
        });
    }
    cohorts
}

fn missing() -> String {
    "\u{2014}".to_owned()
}

pub(super) fn short(value: &str) -> String {
    value.chars().take(8).collect()
}

pub(super) fn share(part: i64, whole: i64) -> String {
    if whole <= 0 {
        return "no denominator".to_owned();
    }
    let pct = (part as f64 / whole as f64) * 100.0;
    format!("{pct:.1}% of {whole}")
}
