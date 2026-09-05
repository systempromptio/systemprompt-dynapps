//! Assembling the group listing: the rows, their usage, and the marketplaces
//! each group reaches.
//!
//! Entitlement is resolved through the same resolver the enforcement point
//! uses, against a synthetic subject holding one group membership — so the
//! chips on this page say what a member would actually reach rather than what
//! a rule table happens to mention.

use std::collections::HashMap;

use sqlx::PgPool;

use crate::repositories;
use crate::services::marketplaces::load_marketplaces;

use super::super::types::{GroupRowView, MemberSetChipView};
use super::UNASSIGNED_GROUP;

pub(super) async fn load_group_rows(pool: &PgPool) -> Vec<GroupRowView> {
    let summaries = repositories::groups::crud::list_group_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "group summaries failed"))
        .unwrap_or_default();

    let marketplaces = load_group_marketplaces(pool, &summaries).await;

    summaries
        .into_iter()
        .map(|g| GroupRowView {
            href: format!("/admin/access/groups/{}", g.id),
            is_unassigned: g.id == UNASSIGNED_GROUP,
            marketplaces: marketplaces.get(&g.id).cloned().unwrap_or_default(),
            project_count: g.project_count,
            member_count: g.member_count,
            active_members_30d: g.active_members_30d,
            requests_30d: g.requests_30d,
            cost_30d_microdollars: g.cost_30d_microdollars,
            id: g.id,
            name: g.name,
            description: g.description,
        })
        .collect()
}

// Why: Every group's allowed marketplaces, resolved one subject at a time.
//
// The catalog is read once and shared: it comes off disk, and re-reading it
// per group would turn a page render into one filesystem walk per row.
async fn load_group_marketplaces(
    pool: &PgPool,
    summaries: &[crate::types::groups::GroupSummary],
) -> HashMap<String, Vec<MemberSetChipView>> {
    let section: Vec<(String, String, Option<String>)> = load_marketplaces()
        .into_iter()
        .map(|m| (m.id.to_string(), m.name, None))
        .collect();
    if section.is_empty() {
        return HashMap::new();
    }

    let mut out = HashMap::with_capacity(summaries.len());
    for group in summaries {
        let subject = repositories::users::access_control::group_subject(&group.id);
        let sections = vec![(
            "marketplace".to_owned(),
            "Marketplaces".to_owned(),
            section.clone(),
        )];
        let resolved = repositories::users::access_control::resolve_subject_matrix(
            pool, &subject, sections,
        )
        .await
        .inspect_err(
            |e| tracing::warn!(error = %e, group = %group.id, "group marketplace resolve failed"),
        )
        .unwrap_or_default();
        let chips: Vec<MemberSetChipView> = resolved
            .into_iter()
            .flat_map(|s| s.rows)
            .filter(|r| r.effective == "allow")
            .map(|r| MemberSetChipView {
                id: r.entity_id,
                label: r.entity_name,
            })
            .collect();
        out.insert(group.id.clone(), chips);
    }
    out
}
