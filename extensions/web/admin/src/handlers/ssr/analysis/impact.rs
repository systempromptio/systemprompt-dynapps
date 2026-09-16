//! Version-impact cohorts with explicit attribution and accounting coverage.

use super::impact_cohorts::{InvocationView, SkillCohort, evidence_views, group_by_skill, share};
use super::impact_paging::{ImpactQuery, PAGE_SIZE, PageRef, clean, paginate};
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::Pagination;
use crate::repositories::analysis::version_impact::{self, VersionImpactFilter};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, Query, State};
use axum::response::Response;
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Serialize)]
struct ImpactContext {
    page: &'static str,
    title: &'static str,
    query: ImpactQuery,
    summary: version_impact::VersionImpactSummary,
    attributed_share: String,
    accounted_share: String,
    deduplicated_cost: String,
    cohorts: Vec<SkillCohort>,
    cohort_pagination: Pagination,
    invocations: Vec<InvocationView>,
    evidence_pagination: Pagination,
}

// Why: a cohort wider than a year is a report, not a page, and an unknown
// traffic class would silently return everything rather than nothing.
fn validated_window(query: &ImpactQuery) -> Result<(DateTime<Utc>, DateTime<Utc>), AdminError> {
    let end = query.end.unwrap_or_else(Utc::now);
    let start = query.start.unwrap_or(end - Duration::days(30));
    if start >= end
        || end - start > Duration::days(366)
        || query
            .traffic_class
            .as_deref()
            .is_some_and(|value| !matches!(value, "production" | "fixture"))
    {
        return Err(AdminError::BadRequest(
            "Invalid version-impact cohort".to_owned(),
        ));
    }
    Ok((start, end))
}

struct ImpactParts {
    query: ImpactQuery,
    at: PageRef,
    summary: version_impact::VersionImpactSummary,
    cohorts: Vec<SkillCohort>,
    cohort_total: i64,
    cohort_shown: usize,
    invocations: Vec<InvocationView>,
    evidence_total: i64,
}

impl From<ImpactParts> for ImpactContext {
    fn from(parts: ImpactParts) -> Self {
        let summary = parts.summary;
        Self {
            page: "analysis-impact",
            title: "Impact",
            cohort_pagination: paginate(
                &parts.query,
                PageRef {
                    cohorts: true,
                    ..parts.at
                },
                parts.cohort_total,
                parts.cohort_shown,
            ),
            evidence_pagination: paginate(
                &parts.query,
                PageRef {
                    cohorts: false,
                    ..parts.at
                },
                parts.evidence_total,
                parts.invocations.len(),
            ),
            query: parts.query,
            attributed_share: share(summary.attributed, summary.invocations),
            accounted_share: share(summary.accounted_requests, summary.distinct_requests),
            deduplicated_cost: super::dollars(summary.deduplicated_request_cost_microdollars),
            summary,
            cohorts: parts.cohorts,
            invocations: parts.invocations,
        }
    }
}

pub(crate) async fn page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(mut query): Query<ImpactQuery>,
) -> AdminHtmlResult<Response> {
    if !user.is_console {
        return Err(AdminError::Forbidden("Console access required".to_owned()).into());
    }
    let (start, end) = validated_window(&query)?;
    query.start = Some(start);
    query.end = Some(end);
    let at = PageRef {
        cohort: i64::from(query.page.unwrap_or(1).max(1)),
        evidence: i64::from(query.evidence_page.unwrap_or(1).max(1)),
        cohorts: true,
    };
    let cohort_filter = VersionImpactFilter {
        start,
        end,
        revision: clean(query.revision.as_ref()),
        traffic_class: clean(query.traffic_class.as_ref()),
        skill: clean(query.skill.as_ref()),
        limit: PAGE_SIZE,
        offset: (at.cohort - 1) * PAGE_SIZE,
    };
    let evidence_filter = VersionImpactFilter {
        offset: (at.evidence - 1) * PAGE_SIZE,
        ..cohort_filter.clone()
    };
    // Why: the three reads are independent and each pays the same per-event
    // receipt resolution, so running them in sequence made the page cost their
    // sum rather than their maximum.
    let (cohort_rows, evidence, summary) = tokio::try_join!(
        version_impact::list_version_impact_paged(&pool, &cohort_filter),
        version_impact::list_invocation_drilldowns_paged(&pool, &evidence_filter),
        version_impact::get_version_impact_summary(&pool, &cohort_filter),
    )?;
    let (rows, cohort_total) = cohort_rows;
    let (evidence, evidence_total) = evidence;
    let cohort_shown = rows.len();
    let parts = ImpactParts {
        query,
        at,
        summary,
        cohorts: group_by_skill(rows),
        cohort_total,
        cohort_shown,
        invocations: evidence_views(evidence),
        evidence_total,
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-impact",
        &ImpactContext::from(parts),
        &user,
        &marketplace,
    ))
}
