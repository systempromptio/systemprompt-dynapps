//! The kind-wide entity-access endpoints: the matrix read across every entity
//! of one kind, and a template that applies one subject's rule to all of them.
//!
//! Entity ids come from configuration — the profile's dispatchable routes
//! (`gateway_route`) or `services/mcp/*.yaml` (`mcp_server`) — never from the
//! database, so an entity with no rules yet still appears.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::governance::authz::{Access, AccessRule, EntityRef};

use systemprompt_web_admin_core::error::{AdminError, AdminResult};

use super::support::{collect_entity_ids, parse_subject, repo, validate_entity_type};
use super::template::{Tally, TemplateSubject, clear_subject_rules, upsert_subject_rule};
use super::types::{
    AllAccessQuery, ApplyTemplateBody, ApplyTemplateResponse, EntityAccessEntry,
    ListAllEntityAccessResponse,
};
use super::{require_non_blank, shared_band_lacks_reason, trimmed_reason};

pub(crate) async fn list_all_entity_access_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AllAccessQuery>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&query.entity_type)?;
    let entity_ids = collect_entity_ids(&query.entity_type)?;
    let r = repo(&pool);
    let bulk = r
        .list_rules_bulk(kind, &entity_ids)
        .await
        .map_err(AdminError::internal)?;
    let mut entries: Vec<EntityAccessEntry> = Vec::with_capacity(entity_ids.len());
    for eid in &entity_ids {
        let entity = EntityRef::from_kind_and_id(kind, eid).map_err(AdminError::internal)?;
        let default_included = r
            .get_entity(entity.kind(), entity.id_str())
            .await
            .inspect_err(
                |e| tracing::warn!(error = %e, eid = %eid, "entity_access: get_entity failed"),
            )
            .ok()
            .flatten()
            .is_some_and(|e| e.default_included);
        let rules: Vec<AccessRule> = bulk.get(eid).cloned().unwrap_or_default();
        entries.push(EntityAccessEntry {
            entity_id: entity.id_str().to_owned(),
            default_included,
            rules,
        });
    }
    Ok(Json(ListAllEntityAccessResponse {
        entity_type: query.entity_type,
        entities: entries,
    })
    .into_response())
}

pub(crate) async fn apply_template_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<ApplyTemplateBody>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&body.entity_type)?;
    require_non_blank(&body.subject_value, "subject_value")?;
    let (rule_type, rule_value) =
        parse_subject(&pool, &body.subject_type, &body.subject_value).await?;
    if !["allow", "deny", "clear"].contains(&body.action.as_str()) {
        return Err(AdminError::BadRequest(
            "action must be allow|deny|clear".to_owned(),
        ));
    }
    let justification = trimmed_reason(body.justification.as_deref());
    if body.action != "clear" && shared_band_lacks_reason(&rule_type, justification) {
        return Err(AdminError::BadRequest(
            "a reason (justification) is required when granting or denying a shared band"
                .to_owned(),
        ));
    }

    let entity_ids = collect_entity_ids(&body.entity_type)?;
    let r = repo(&pool);
    let subject = TemplateSubject {
        kind,
        rule_type: &rule_type,
        rule_value: &rule_value,
        justification,
    };
    let mut tally = Tally::default();

    for eid in &entity_ids {
        if body.action == "clear" {
            tally += clear_subject_rules(&r, &subject, eid).await;
        } else {
            let access = if body.action == "deny" {
                Access::Deny
            } else {
                Access::Allow
            };
            tally += upsert_subject_rule(&r, &subject, eid, access).await;
        }
    }

    Ok(Json(ApplyTemplateResponse {
        applied: tally.applied,
        failed: tally.failed,
        entity_count: entity_ids.len(),
    })
    .into_response())
}
