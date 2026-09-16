//! The impact page's query string and the two independent table pagers it
//! drives: one for the per-skill cohort rows, one for the invocation evidence.

use crate::handlers::ssr::list_view::Pagination;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ImpactQuery {
    pub(super) start: Option<DateTime<Utc>>,
    pub(super) end: Option<DateTime<Utc>>,
    pub(super) revision: Option<String>,
    pub(super) traffic_class: Option<String>,
    pub(super) skill: Option<String>,
    pub(super) page: Option<u32>,
    pub(super) evidence_page: Option<u32>,
}

// Why: the cohort and evidence tables page independently, so each owns its own
// parameter and each link carries the other's page plus every filter — a single
// `page` would reset the table the reader was not paging.
impl ImpactQuery {
    pub(super) fn url(&self, cohort: i64, evidence: i64) -> String {
        let mut pairs: Vec<(&str, String)> = Vec::new();
        for (key, value) in [
            ("start", self.start.map(|at| at.to_rfc3339())),
            ("end", self.end.map(|at| at.to_rfc3339())),
            ("skill", self.skill.clone()),
            ("revision", self.revision.clone()),
            ("traffic_class", self.traffic_class.clone()),
        ] {
            if let Some(value) = value {
                pairs.push((key, value));
            }
        }
        if cohort > 1 {
            pairs.push(("page", cohort.to_string()));
        }
        if evidence > 1 {
            pairs.push(("evidence_page", evidence.to_string()));
        }
        let query = pairs
            .into_iter()
            .map(|(key, value)| format!("{key}={}", urlencoding::encode(&value)))
            .collect::<Vec<_>>()
            .join("&");
        if query.is_empty() {
            "/admin/analysis/impact".to_owned()
        } else {
            format!("/admin/analysis/impact?{query}")
        }
    }
}

pub(super) const PAGE_SIZE: i64 = 50;

// Why: which table this page belongs to, carried explicitly so the prev/next
// links move one table and leave the other where the reader left it.
#[derive(Clone, Copy)]
pub(super) struct PageRef {
    pub(super) cohort: i64,
    pub(super) evidence: i64,
    pub(super) cohorts: bool,
}

pub(super) fn paginate(query: &ImpactQuery, at: PageRef, total: i64, shown: usize) -> Pagination {
    let current = if at.cohorts { at.cohort } else { at.evidence };
    let total_pages = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let offset = (current - 1) * PAGE_SIZE;
    let shown = i64::try_from(shown).unwrap_or(i64::MAX);
    let link = |target: i64| {
        if at.cohorts {
            query.url(target, at.evidence)
        } else {
            query.url(at.cohort, target)
        }
    };
    Pagination {
        current_page: current.min(total_pages),
        total_pages,
        first_row: if shown == 0 { 0 } else { offset + 1 },
        last_row: offset + shown,
        total_rows: total,
        noun: if at.cohorts {
            "cohort rows"
        } else {
            "invocations"
        },
        has_prev: current > 1,
        has_next: current < total_pages,
        prev_url: (current > 1).then(|| link(current - 1)),
        next_url: (current < total_pages).then(|| link(current + 1)),
    }
}

pub(super) fn clean(value: Option<&String>) -> Option<String> {
    value
        .filter(|value| !value.trim().is_empty() && value.len() <= 255)
        .cloned()
}
