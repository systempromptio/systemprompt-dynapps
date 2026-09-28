//! Generic entity-access HTTP handlers backing the unified `/admin/access`
//! matrix and per-entity inline panels (gateway routes, MCP servers, …).
//!
//! Wraps [`systemprompt_security::authz::AccessControlRepository`] with the
//! same endpoint shape the gateway-specific handlers use, but parameterized on
//! `entity_type`. Allowed values mirror the Postgres CHECK constraint on
//! `access_control_rules.entity_type`.
//!
//! A blank subject value is rejected before the subject is parsed, so the
//! error names the value field rather than the rule type. A shared band needs
//! a stated reason; a per-person override, and any removal, may go without.

mod bulk;
mod support;
mod template;
mod types;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::identifiers::RuleId;
use systemprompt_security::authz::{DASHBOARD_SOURCE, EntityRef, RuleType, UpsertRuleParams};

use systemprompt_web_admin_core::activity::RuleChange;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::repositories::gateway_catalog::registered_routes_from_services;
use systemprompt_web_admin_core::types::UserContext;

use support::{
    parse_access, parse_subject, record_change, repo, rule_subject, validate_entity_type,
};
use types::{
    DefaultIncludedBody, EntityAccessResponse, EntityDefaultResponse, RemoveRuleQuery,
    UpsertRuleBody, UpsertRuleResponse,
};

pub(crate) use bulk::{apply_template_handler, list_all_entity_access_handler};

fn require_non_blank(value: &str, field: &str) -> AdminResult<()> {
    if value.trim().is_empty() {
        return Err(AdminError::BadRequest(format!("{field} required")));
    }
    Ok(())
}

fn trimmed_reason(raw: Option<&str>) -> Option<&str> {
    raw.map(str::trim).filter(|r| !r.is_empty())
}

fn shared_band_lacks_reason(rule_type: &RuleType, justification: Option<&str>) -> bool {
    *rule_type != RuleType::USER && justification.is_none()
}

pub(crate) async fn list_entity_access_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    let entity = EntityRef::from_kind_and_id(kind, &entity_id)
        .map_err(|error| AdminError::Unprocessable(error.to_string()))?;
    let r = repo(&pool);
    let rules = r
        .list_rules_for_entity(entity.kind(), entity.id_str())
        .await
        .map_err(AdminError::internal)?;
    let default_included = r
        .get_entity(entity.kind(), entity.id_str())
        .await
        .map_err(AdminError::internal)?
        .is_some_and(|entity| entity.default_included);
    Ok(Json(EntityAccessResponse {
        entity_type: entity.kind().as_str().to_owned(),
        entity_id: entity.id_str().to_owned(),
        default_included,
        rules,
    })
    .into_response())
}

pub(crate) async fn upsert_entity_rule_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<UpsertRuleBody>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    require_non_blank(&body.rule_value, "rule_value")?;
    let (rule_type, rule_value) = parse_subject(&pool, &body.rule_type, &body.rule_value).await?;
    let access = parse_access(&body.access)
        .ok_or_else(|| AdminError::BadRequest("invalid access".to_owned()))?;
    let justification = trimmed_reason(body.justification.as_deref());
    if shared_band_lacks_reason(&rule_type, justification) {
        return Err(AdminError::BadRequest(format!(
            "a reason (justification) is required for the {rule_type} rule on '{rule_value}'"
        )));
    }
    let rule = repo(&pool)
        .upsert_rule(UpsertRuleParams {
            entity_type: kind,
            entity_id: &entity_id,
            rule_type: rule_type.clone(),
            rule_value: &rule_value,
            access,
            justification,
            source: DASHBOARD_SOURCE,
        })
        .await
        .map_err(AdminError::internal)?;
    if body
        .valid_until
        .is_some_and(|until| until <= chrono::Utc::now())
    {
        return Err(AdminError::BadRequest(
            "valid_until must be in the future".to_owned(),
        ));
    }
    systemprompt_web_admin_core::repositories::access_rules::validity::set_rule_validity(
        pool.as_ref(),
        rule.id.as_str(),
        body.valid_until,
    )
    .await?;
    record_change(
        &pool,
        &user_ctx.user_id,
        RuleChange {
            entity_type: kind.as_str(),
            entity_id: &entity_id,
            subject: &format!("{rule_type}:{rule_value}"),
            access: Some(&body.access),
            reason: justification,
        },
    )
    .await;
    Ok(Json(UpsertRuleResponse {
        rule,
        valid_until: body.valid_until,
    })
    .into_response())
}

pub(crate) async fn delete_entity_rule_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path((entity_type, entity_id, rule_id)): Path<(String, String, String)>,
    Query(query): Query<RemoveRuleQuery>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    let subject = rule_subject(&pool, kind, &entity_id, &rule_id).await;
    if !repo(&pool)
        .delete_rule(&RuleId::new(rule_id))
        .await
        .map_err(AdminError::internal)?
    {
        return Err(AdminError::NotFound("rule not found".to_owned()));
    }
    let reason = trimmed_reason(query.reason.as_deref());
    record_change(
        &pool,
        &user_ctx.user_id,
        RuleChange {
            entity_type: kind.as_str(),
            entity_id: &entity_id,
            subject: &subject,
            access: None,
            reason,
        },
    )
    .await;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub(crate) async fn set_entity_default_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<DefaultIncludedBody>,
) -> AdminResult<Response> {
    let kind = validate_entity_type(&entity_type)?;
    registered_routes_from_services()?.require(kind, &entity_id)?;
    let entity = EntityRef::from_kind_and_id(kind, &entity_id)
        .map_err(|error| AdminError::Unprocessable(error.to_string()))?;
    repo(&pool)
        .upsert_entity(
            entity.kind(),
            entity.id_str(),
            body.default_included,
            "admin:dashboard",
        )
        .await
        .map_err(AdminError::internal)?;
    Ok(Json(EntityDefaultResponse {
        entity_type: entity.kind().as_str().to_owned(),
        entity_id: entity.id_str().to_owned(),
        default_included: body.default_included,
    })
    .into_response())
}
