//! The "Who gets this" panel every catalog detail page carries.
//!
//! It shows who reaches the entity, the rules that decide it, whether code
//! and this database agree about it, and why one named person does or does
//! not reach it.
//!
//! One builder, one partial (`components/entity-access`) and one script
//! (`pages/admin-entity-access.js`), so a marketplace, a plugin, a skill and
//! an MCP server answer the question in the same words and edit it the same
//! way. Every read is best-effort: a part that cannot load leaves its own
//! empty state instead of failing the page it sits on, and the panel names
//! each such part in its own `unavailable` list.

pub mod reach;
pub mod rules;
pub mod view;
pub mod why;

use std::collections::HashMap;

use sqlx::PgPool;

pub use view::EntityAccessView;
use view::{OptionView, SubjectOptions};

use crate::error::AdminResult;
use crate::repositories::access_rules::rules::{list_entity_defaults, list_ledger_rules};
use crate::repositories::scope::containers::{list_group_summaries, list_project_summaries};
use crate::repositories::sync::access_control::{declared_now, drift_now};
use crate::repositories::sync::attention::{ReviewedEntity, reviews_for};
use crate::ssr::faults::Faults;
use crate::ssr::pickable_users::list_pickable_users;
use crate::types::Role;

pub const SYNC_URL: &str = crate::repositories::sync::access_control::REVIEW_URL;

#[must_use]
pub fn entity_access_url(entity_type: &str, entity_id: &str) -> Option<String> {
    let base = match entity_type {
        "marketplace" => "/admin/marketplaces",
        "plugin" => "/admin/plugins",
        "skill" => "/admin/skills",
        "mcp_server" => "/admin/mcp",
        "gateway_route" => "/admin/gateway/routes",
        _ => return None,
    };
    Some(format!("{base}/{}#access", urlencoding::encode(entity_id)))
}

#[derive(Debug, Clone, Copy)]
pub struct PanelRequest<'a> {
    pub boot: &'a crate::extension_boot::ExtensionBoot,
    pub entity_type: &'a str,
    pub entity_id: &'a str,
    pub page_url: &'a str,
    pub why: Option<&'a str>,
    pub can_write: bool,
    pub note: Option<&'static str>,
}

async fn subject_options(pool: &PgPool, faults: &mut Faults) -> SubjectOptions {
    let groups = faults.take("groups", list_group_summaries(pool).await);
    let projects = faults.take("projects", list_project_summaries(pool).await);
    SubjectOptions {
        roles: Role::ALL
            .iter()
            .map(|r| OptionView {
                value: r.as_str().to_owned(),
                label: r.label().to_owned(),
            })
            .collect(),
        groups: groups
            .into_iter()
            .map(|g| OptionView {
                value: g.id.as_str().to_owned(),
                label: g.name,
            })
            .collect(),
        projects: projects
            .into_iter()
            .map(|p| OptionView {
                value: p.id.as_str().to_owned(),
                label: p.name,
            })
            .collect(),
    }
}

fn subject_names(options: &SubjectOptions) -> HashMap<(String, String), String> {
    let mut names = HashMap::new();
    for (band, list) in [
        ("role", &options.roles),
        ("group", &options.groups),
        ("project", &options.projects),
    ] {
        for o in list {
            names.insert((band.to_owned(), o.value.clone()), o.label.clone());
        }
    }
    names
}

async fn review_for(
    pool: &PgPool,
    services: &crate::extension_services::ExtensionServices,
    boot: &crate::extension_boot::ExtensionBoot,
    key: &str,
) -> AdminResult<Option<ReviewedEntity>> {
    let declared = declared_now(&services.config(), boot.services_root()).await?;
    let drift = drift_now(pool, &declared).await?;
    let split = reviews_for(pool, &drift).await?;
    Ok(split
        .pending
        .into_iter()
        .chain(split.kept)
        .find(|r| r.review.key == key))
}

pub async fn build_entity_panel(
    pool: &PgPool,
    services: &crate::extension_services::ExtensionServices,
    req: PanelRequest<'_>,
) -> EntityAccessView {
    let key = format!("{}/{}", req.entity_type, req.entity_id);
    let mut faults = Faults::new();
    let ledger = faults.take("access rules", list_ledger_rules(pool).await);
    let rows: Vec<_> = ledger
        .iter()
        .filter(|r| r.entity_type == req.entity_type && r.entity_id == req.entity_id)
        .collect();
    let open = faults
        .take("entity defaults", list_entity_defaults(pool).await)
        .into_iter()
        .any(|e| {
            e.entity_type == req.entity_type && e.entity_id == req.entity_id && e.default_included
        });
    let subjects = subject_options(pool, &mut faults).await;
    let entity = (req.entity_type, req.entity_id);
    let mut reaches = reach::resolved_reach(pool, services, entity, &subjects, &mut faults).await;
    reaches.extend(reach::ruled_reach(&rows));
    let (review, drift_unreadable) = match review_for(pool, services, req.boot, &key).await {
        Ok(review) => (review, None),
        Err(e) => (None, Some(e.to_string())),
    };
    let why = match req.why.map(str::trim).filter(|w| !w.is_empty()) {
        Some(q) => Some(why::why(pool, services, q, entity, &mut faults).await),
        None => None,
    };
    let people = faults.take("people", list_pickable_users(pool, None).await);
    EntityAccessView {
        entity_type: req.entity_type.to_owned(),
        entity_id: req.entity_id.to_owned(),
        default_decision: if open { "open" } else { "closed" },
        headline: reach::headline(open, &reaches),
        nobody: !open && reaches.is_empty(),
        reaches,
        bands: rules::band_rules(&rows, &subject_names(&subjects)),
        rule_count: rows.len(),
        not_live: review.as_ref().is_some_and(|r| r.review.not_live),
        review,
        drift_unreadable,
        subjects,
        people,
        can_write: req.can_write,
        note: req.note,
        why,
        page_url: req.page_url.to_owned(),
        sync_url: SYNC_URL,
        key,
        unavailable: faults.into_vec(),
    }
}
