//! Which marketplaces a caller *participates* in, and the people that
//! participation lets them see.
//!
//! Access to a marketplace is declared in `services/access-control/rules.yaml`
//! and enforced from `access_control_rules`. A person participates in a
//! marketplace when they reach it **through a group or project rule** — the
//! membership the directory placed them in. A marketplace reached only by its
//! `default: open` flag or by a role rule (the commons, the admin control
//! plane) is not participation: it reaches a whole band, and counting it would
//! let every signed-in user read everyone's analysis and sync the commons kit.
//!
//! Participation is what the scoped console runs on. A participant sees the
//! activity of everyone in the groups and projects named on their
//! marketplaces' allow rules — a developers-group member sees the `storefront`
//! project's people because both reach `dynapps-commons` — and may read
//! the versions and drive the code sync of exactly those marketplaces.
//! Console (admin) callers never resolve this; they see everything.

use std::collections::BTreeSet;

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{MarketplaceId, UserId};

use crate::repositories::entitlement::{SectionInput, resolve_user_matrix};

const MARKETPLACE_ENTITY: &str = "marketplace";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Participation {
    pub marketplaces: Vec<MarketplaceId>,
    pub groups: Vec<String>,
    pub projects: Vec<String>,
}

impl Participation {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.marketplaces.is_empty()
    }

    #[must_use]
    pub fn includes_marketplace(&self, id: &str) -> bool {
        self.marketplaces.iter().any(|m| m.as_str() == id)
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ContainerRule {
    entity_id: String,
    rule_type: String,
    rule_value: String,
}

pub async fn get_participation(
    pool: &PgPool,
    user_id: &UserId,
    group_ids: &[String],
    project_ids: &[String],
) -> Result<Participation, sqlx::Error> {
    if group_ids.is_empty() && project_ids.is_empty() {
        return Ok(Participation::default());
    }
    let rules = list_candidate_container_rules(pool, group_ids, project_ids).await?;
    if rules.is_empty() {
        return Ok(Participation::default());
    }

    let candidates: BTreeSet<String> = rules.iter().map(|r| r.entity_id.clone()).collect();
    let allowed = confirm_allowed(pool, user_id, &candidates).await?;

    let mut groups = BTreeSet::new();
    let mut projects = BTreeSet::new();
    for rule in rules.iter().filter(|r| allowed.contains(&r.entity_id)) {
        match rule.rule_type.as_str() {
            "group" => groups.insert(rule.rule_value.clone()),
            "project" => projects.insert(rule.rule_value.clone()),
            _ => false,
        };
    }

    Ok(Participation {
        marketplaces: allowed.into_iter().map(MarketplaceId::new).collect(),
        groups: groups.into_iter().collect(),
        projects: projects.into_iter().collect(),
    })
}

async fn list_candidate_container_rules(
    pool: &PgPool,
    group_ids: &[String],
    project_ids: &[String],
) -> Result<Vec<ContainerRule>, sqlx::Error> {
    sqlx::query_as!(
        ContainerRule,
        r#"SELECT r.entity_id AS "entity_id!", r.rule_type AS "rule_type!",
                  r.rule_value AS "rule_value!"
           FROM access_control_rules r
           WHERE r.entity_type = $1
             AND r.access = 'allow'
             AND r.rule_type IN ('group', 'project')
             AND r.entity_id IN (
                 SELECT m.entity_id FROM access_control_rules m
                 WHERE m.entity_type = $1
                   AND m.access = 'allow'
                   AND ((m.rule_type = 'group' AND m.rule_value = ANY($2))
                     OR (m.rule_type = 'project' AND m.rule_value = ANY($3))))
           ORDER BY r.entity_id, r.rule_type, r.rule_value"#,
        MARKETPLACE_ENTITY,
        group_ids,
        project_ids
    )
    .fetch_all(pool)
    .await
}

async fn confirm_allowed(
    pool: &PgPool,
    user_id: &UserId,
    candidates: &BTreeSet<String>,
) -> Result<BTreeSet<String>, sqlx::Error> {
    let section: SectionInput = (
        MARKETPLACE_ENTITY.to_owned(),
        "Marketplaces".to_owned(),
        candidates
            .iter()
            .map(|id| (id.clone(), id.clone(), None))
            .collect(),
    );
    let Some(matrix) = resolve_user_matrix(pool, user_id, vec![section]).await? else {
        return Ok(BTreeSet::new());
    };
    Ok(matrix
        .sections
        .into_iter()
        .flat_map(|s| s.rows)
        .filter(|row| row.effective == "allow")
        .map(|row| row.entity_id)
        .collect())
}
