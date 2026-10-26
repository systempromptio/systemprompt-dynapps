//! Data assembly for the marketplace catalog pages.
//!
//! The audience matrix asks the resolver one question per (subject,
//! marketplace) pair, where the subject is a group or a role rather than a
//! person. Every row goes through the same
//! [`resolve_subject_matrix`](systemprompt_web_admin_core::repositories::entitlement::resolve_subject_matrix)
//! the per-user matrix uses, so a cell here and a decision at the enforcement
//! point cannot disagree.

use std::collections::HashMap;
use std::path::Path;
use systemprompt::identifiers::MarketplaceId;
use systemprompt_web_shared::GroupId;

use sqlx::PgPool;

use systemprompt_web_admin_core::catalog_index::manifests::MarketplaceConfigSummary;
use systemprompt_web_admin_core::repositories::entitlement::{
    MatrixSection, MatrixSubject, group_subject, resolve_subject_matrices, role_subject,
};

use super::view::{AudienceCellView, AudienceColumnView, AudienceMatrixView, AudienceRowView};

pub(crate) const MARKETPLACE_ENTITY: &str = "marketplace";

pub(super) async fn load_manifests(
    pool: &PgPool,
    services_path: &Path,
) -> Vec<MarketplaceConfigSummary> {
    let mut manifests =
        systemprompt_web_admin_core::catalog_index::manifests::list_marketplace_configs(
            services_path,
        )
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to load marketplace manifests");
            Vec::new()
        });
    systemprompt_web_admin_core::catalog_index::manifests_access::attach_access(
        pool,
        &mut manifests,
    )
    .await;
    manifests
}

fn section_input(
    manifests: &[MarketplaceConfigSummary],
) -> Vec<systemprompt_web_admin_core::repositories::entitlement::SectionInput> {
    let rows = manifests
        .iter()
        .map(|m| (m.id.as_str().to_owned(), m.name.clone(), None))
        .collect();
    vec![(
        MARKETPLACE_ENTITY.to_owned(),
        "Marketplaces".to_owned(),
        rows,
    )]
}

fn cells_of(sections: Vec<MatrixSection>) -> Vec<AudienceCellView> {
    sections
        .into_iter()
        .next()
        .map_or_else(Vec::new, |section| {
            section
                .rows
                .into_iter()
                .map(|row| AudienceCellView {
                    marketplace_id: MarketplaceId::new(row.entity_id),
                    is_allow: row.effective == "allow",
                    effective: row.effective,
                    layer: row.source.layer,
                    detail: row.source.detail,
                })
                .collect()
        })
}

pub(super) async fn audience_matrix(
    pool: &PgPool,
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    manifests: &[MarketplaceConfigSummary],
    roles: &[String],
) -> AudienceMatrixView {
    let groups =
        systemprompt_web_admin_core::repositories::scope::containers::list_group_summaries(pool)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "audience matrix: group listing failed"))
            .unwrap_or_default();

    let mut subjects: Vec<MatrixSubject> = Vec::with_capacity(groups.len() + roles.len());
    let mut labels: Vec<(String, &'static str)> = Vec::with_capacity(subjects.capacity());
    for group in &groups {
        subjects.push(group_subject(group.id.as_str()));
        labels.push((group.name.clone(), "group"));
    }
    for role in roles {
        subjects.push(role_subject(role));
        labels.push((role.clone(), "role"));
    }

    let resolved = resolve_subject_matrices(pool, services, &subjects, &section_input(manifests))
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "audience matrix failed to resolve"))
        .unwrap_or_default();

    let mut resolved = resolved.into_iter();
    let rows: Vec<AudienceRowView> = subjects
        .iter()
        .zip(labels)
        .map(|(subject, (label, kind))| AudienceRowView {
            subject: subject.id.as_str().to_owned(),
            label,
            kind,
            cells: resolved.next().map_or_else(Vec::new, cells_of),
        })
        .collect();

    AudienceMatrixView {
        columns: manifests
            .iter()
            .map(|m| AudienceColumnView {
                id: m.id.clone(),
                name: m.name.clone(),
            })
            .collect(),
        has_rows: !rows.is_empty(),
        rows,
    }
}

pub(super) async fn group_grants(pool: &PgPool) -> HashMap<MarketplaceId, Vec<GroupId>> {
    let rules = systemprompt_web_admin_core::repositories::entitlement::list_all_rules(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "marketplaces: rule listing failed"))
        .unwrap_or_default();
    let mut out: HashMap<MarketplaceId, Vec<GroupId>> = HashMap::new();
    for rule in rules {
        if rule.entity_type == MARKETPLACE_ENTITY
            && rule.rule_type.as_str() == "group"
            && rule.access.to_string() == "allow"
        {
            out.entry(MarketplaceId::new(rule.entity_id))
                .or_default()
                .push(GroupId::new(rule.rule_value));
        }
    }
    for ids in out.values_mut() {
        ids.sort();
    }
    out
}
