//! Version-impact cohorts with explicit attribution and accounting coverage.

use super::impact_cohorts::{InvocationView, SkillCohort, evidence_views, group_by_skill, share};
use super::impact_paging::{ImpactQuery, PAGE_SIZE, PageRef, clean, paginate};
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::list_view::Pagination;
use crate::repositories::analysis::version_impact::{self, VersionImpactFilter};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, Form, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::BTreeMap;
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
        || query.traffic_class.as_deref().is_some_and(|value| {
            !matches!(
                value,
                "production" | "fixture" | "live_evaluation" | "suggestion" | "judge"
            )
        })
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FailureCaptureForm {
    invocation_id: String,
    prompt: String,
    expected_behavior: String,
    assertions: String,
}

pub(crate) async fn capture_failure(
    Extension(user): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Form(form): Form<FailureCaptureForm>,
) -> Result<Response, AdminError> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required".to_owned(),
        ));
    }
    crate::handlers::evaluation_experiments::require_write_origin(&headers)?;
    let prompt = sanitize(&form.prompt, 16_000)?;
    let expected_behavior = lines(&form.expected_behavior, 20)?;
    let assertions = form
        .assertions
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if form.invocation_id.trim().is_empty()
        || form.invocation_id.len() > 255
        || assertions.is_empty()
        || assertions.len() > 20
        || assertions.iter().any(|value| value.len() > 100)
    {
        return Err(AdminError::BadRequest(
            "Invalid reviewed failure capture".to_owned(),
        ));
    }
    let mut tx = pool.begin().await?;
    let invocation = sqlx::query!("SELECT skill,session_id,attribution_status FROM analysis_skill_version_events WHERE invocation_id=$1 AND user_id=$2 AND traffic_class='production'",
        &form.invocation_id, user.user_id.as_str()).fetch_optional(&mut *tx).await?
        .ok_or_else(|| AdminError::BadRequest("Only an owned production invocation can be captured".to_owned()))?;
    let content = systemprompt::evaluation::experiments::resources::ResourceContent::Case(
        systemprompt::evaluation::experiments::resources::CaseContent {
            prompt,
            expected_behavior,
            fixtures: BTreeMap::new(),
            partition: systemprompt::evaluation::experiments::resources::Partition::Development,
            assertions,
        },
    );
    content.validate().map_err(AdminError::from)?;
    let digest = systemprompt::evaluation::experiments::content_digest(&content)
        .map_err(AdminError::from)?;
    let revision = systemprompt::identifiers::EvalRevisionId::generate();
    let key = format!("production-failure-{}", form.invocation_id);
    let revision_id = sqlx::query_scalar!("INSERT INTO eval_resource_revisions(id,owner_id,resource_kind,resource_key,digest,content) VALUES($1,$2,'case',$3,$4,$5) ON CONFLICT(owner_id,resource_kind,resource_key,digest) DO UPDATE SET digest=EXCLUDED.digest RETURNING id",
        revision.as_str(), user.user_id.as_str(), &key, &digest, serde_json::to_value(&content).map_err(AdminError::internal)?).fetch_one(&mut *tx).await?;
    let evidence = serde_json::json!({"skill":invocation.skill,"session_id":invocation.session_id,"attribution_status":invocation.attribution_status,"sanitized":true});
    sqlx::query!("INSERT INTO reviewed_production_failures(id,owner_id,invocation_id,reviewer_id,sanitized_evidence,development_case_revision_id) VALUES($1,$2,$3,$2,$4,$5) ON CONFLICT(owner_id,invocation_id) DO NOTHING",
        format!("rpf_{}", uuid::Uuid::new_v4()), user.user_id.as_str(), &form.invocation_id, evidence, &revision_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Redirect::to("/admin/analysis/impact").into_response())
}

fn lines(value: &str, maximum: usize) -> Result<Vec<String>, AdminError> {
    let values = value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| sanitize(line, 2_000))
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() || values.len() > maximum {
        return Err(AdminError::BadRequest(
            "Expected bounded nonempty behavior lines".to_owned(),
        ));
    }
    Ok(values)
}

fn sanitize(value: &str, maximum: usize) -> Result<String, AdminError> {
    if value.trim().is_empty()
        || value.len() > maximum
        || value
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\t'))
    {
        return Err(AdminError::BadRequest(
            "Captured evidence is empty or excessive".to_owned(),
        ));
    }
    Ok(value
        .split_whitespace()
        .map(|token| {
            let lower = token.to_ascii_lowercase();
            if token.contains('@')
                || lower.starts_with("sk-")
                || lower.starts_with("spexec_")
                || lower.starts_with("bearer")
            {
                "[REDACTED]"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" "))
}
