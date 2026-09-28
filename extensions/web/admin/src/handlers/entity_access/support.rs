//! Validation, parsing, and entity-id collection helpers shared by the
//! entity-access handlers.
//!
//! The handlers in the parent module own the HTTP shape; this module owns the
//! small pure conversions (string -> typed enum) and the on-disk lookups that
//! feed the bulk/matrix endpoints.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::RoleId;
use systemprompt_security::authz::{Access, AccessControlRepository, EntityKind, RuleType};
use systemprompt_web_shared::{GroupId, ProjectId};

use crate::repositories;
use crate::repositories::mcp::mcp_servers;
use systemprompt_web_admin_core::activity::{NewActivity, RuleChange};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::{activity, request as shared};

pub(super) fn validate_entity_type(entity_type: &str) -> AdminResult<EntityKind> {
    use std::str::FromStr;
    EntityKind::from_str(entity_type)
        .map_err(|source| AdminError::invalid("invalid entity_type", source))
}

fn is_rule_key_safe_user_id(value: &str) -> bool {
    !value.is_empty() && !value.contains(':')
}

pub(super) fn repo(pool: &PgPool) -> AccessControlRepository {
    AccessControlRepository::from_pool(Arc::new(pool.clone()))
}

// Why: core's `SubjectRef` has no arm for extension dimensions, so `group` and
// `project` are minted with `RuleType::extension` and checked against their
// tables — a mistyped one would otherwise match nothing and look granted.
pub(super) async fn parse_subject(
    pool: &PgPool,
    rule_type: &str,
    rule_value: &str,
) -> AdminResult<(RuleType, String)> {
    let invalid = || {
        AdminError::BadRequest(
            "invalid rule_type (invalid subject_type): must be user, role, group or project"
                .to_owned(),
        )
    };
    match rule_type {
        "user" => {
            if !is_rule_key_safe_user_id(rule_value) {
                return Err(invalid());
            }
            Ok((RuleType::USER, rule_value.to_owned()))
        },
        "role" => {
            let role = RoleId::try_new(rule_value)
                .map_err(|source| AdminError::invalid("invalid rule_value", source))?;
            Ok((RuleType::ROLE, role.as_str().to_owned()))
        },
        "group" => {
            if repositories::groups::crud::find_group(pool, &GroupId::new(rule_value))
                .await?
                .is_none()
            {
                return Err(AdminError::BadRequest(format!("No group {rule_value}")));
            }
            Ok((RuleType::extension("group")?, rule_value.to_owned()))
        },
        "project" => {
            if repositories::projects::crud::find_project(pool, &ProjectId::new(rule_value))
                .await?
                .is_none()
            {
                return Err(AdminError::BadRequest(format!("No project {rule_value}")));
            }
            Ok((RuleType::extension("project")?, rule_value.to_owned()))
        },
        _ => Err(invalid()),
    }
}

pub(super) fn parse_access(s: &str) -> Option<Access> {
    match s {
        "allow" => Some(Access::Allow),
        "deny" => Some(Access::Deny),
        _ => None,
    }
}

pub(super) fn collect_entity_ids(entity_type: &str) -> AdminResult<Vec<String>> {
    match entity_type {
        "gateway_route" => Ok(
            systemprompt_web_admin_core::repositories::gateway_catalog::dispatchable_route_ids(
                systemprompt::loader::ServicesBootstrap::get()?,
            ),
        ),
        "mcp_server" => {
            let services_path = shared::get_services_path()?;
            let servers =
                mcp_servers::list_mcp_servers(&services_path).map_err(AdminError::internal)?;
            Ok(servers
                .into_iter()
                .map(|s| s.id.as_str().to_owned())
                .collect())
        },
        _ => Ok(Vec::new()),
    }
}

pub(super) async fn rule_subject(
    pool: &PgPool,
    kind: EntityKind,
    entity_id: &str,
    rule_id: &str,
) -> String {
    repo(pool)
        .list_rules_for_entity(kind, entity_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "entity access: rule lookup failed"))
        .unwrap_or_default()
        .into_iter()
        .find(|r| r.id.as_str() == rule_id)
        .map_or_else(
            || format!("rule {rule_id}"),
            |r| format!("{}:{}", r.rule_type, r.rule_value),
        )
}

pub(super) async fn record_change(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
    change: RuleChange<'_>,
) {
    activity::record(pool, NewActivity::access_rule_changed(user_id, change)).await;
}
