//! The audience grid: every entity down the side, every role, group and
//! project across the top, and in each cell what the resolver decides for a
//! subject that holds exactly that one thing — with the band that decided.
//!
//! It is the picture the rule list cannot give: who reaches what, at a
//! glance, straight from the enforcement path rather than from a reading of
//! the rules. Cells are resolved in one batch so the page reads the rule
//! table once, however many columns it draws.

use std::collections::HashMap;

use sqlx::PgPool;

use super::view::{AcAudienceCellView, AcAudienceColumnView, AcAudienceRowView, AudienceGridView};
use crate::handlers::ssr::entity_kind::entity_kind_label;
use crate::repositories;
use crate::repositories::users::access_control::{
    MatrixSubject, SectionInput, group_subject, project_subject, resolve_subject_matrices,
    role_subject,
};
use crate::types::Role;

pub(super) struct Audience {
    pub subjects: Vec<MatrixSubject>,
    pub columns: Vec<AcAudienceColumnView>,
}

// Why: roles first as the baseline every person carries, then groups (where
// entitlement is granted here), then projects (the narrowest band).
pub(super) async fn audience(pool: &PgPool) -> Audience {
    let mut subjects = Vec::new();
    let mut columns = Vec::new();
    for role in Role::ALL {
        subjects.push(role_subject(role.as_str()));
        columns.push(AcAudienceColumnView {
            subject: role.as_str().to_owned(),
            label: role.as_str().to_owned(),
            kind: "role",
        });
    }
    let groups = repositories::groups::crud::list_group_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: group listing failed"))
        .unwrap_or_default();
    for group in groups {
        subjects.push(group_subject(group.id.as_str()));
        columns.push(AcAudienceColumnView {
            subject: group.id.as_str().to_owned(),
            label: group.name,
            kind: "group",
        });
    }
    let projects = repositories::projects::crud::list_project_summaries(pool)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: project listing failed"))
        .unwrap_or_default();
    for project in projects {
        subjects.push(project_subject(project.id.as_str()));
        columns.push(AcAudienceColumnView {
            subject: project.id.as_str().to_owned(),
            label: project.name,
            kind: "project",
        });
    }
    Audience { subjects, columns }
}

fn short_layer(layer: &str) -> String {
    match layer {
        "user" => "person".to_owned(),
        "default" | "default_included" => "open".to_owned(),
        "salesforce" => "linked".to_owned(),
        other => other.replace('_', " "),
    }
}

pub(super) async fn grid(
    pool: &PgPool,
    audience: Audience,
    sections: &[SectionInput],
) -> AudienceGridView {
    if audience.subjects.is_empty() || sections.is_empty() {
        return AudienceGridView::default();
    }
    let resolved = resolve_subject_matrices(pool, &audience.subjects, sections)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "access-control: audience grid failed"))
        .unwrap_or_default();
    if resolved.len() != audience.subjects.len() {
        return AudienceGridView::default();
    }

    // Why: the resolver answers per subject; the grid reads per entity, so the
    // answers are transposed through one map keyed on the entity.
    let mut cells: HashMap<(String, String), Vec<AcAudienceCellView>> = HashMap::new();
    for matrix in &resolved {
        for section in matrix {
            for row in &section.rows {
                cells
                    .entry((section.entity_type.clone(), row.entity_id.clone()))
                    .or_default()
                    .push(AcAudienceCellView {
                        allow: row.effective == "allow",
                        effective: row.effective.clone(),
                        layer: short_layer(&row.source.layer),
                        detail: row.source.detail.clone(),
                    });
            }
        }
    }

    let mut rows = Vec::new();
    for (entity_type, _, entities) in sections {
        for (entity_id, name, description) in entities {
            let Some(cells) = cells.remove(&(entity_type.clone(), entity_id.clone())) else {
                continue;
            };
            rows.push(AcAudienceRowView {
                entity_type_label: entity_kind_label(entity_type),
                entity_type: entity_type.clone(),
                entity_id: entity_id.clone(),
                entity_name: name.clone(),
                // Why: the matrix section carries a route's "pattern →
                // provider" line as its description; other kinds' prose
                // descriptions are too long for a grid row.
                entity_sub: if entity_type == "gateway_route" {
                    description.clone()
                } else {
                    None
                },
                cells,
            });
        }
    }
    AudienceGridView {
        has_rows: !rows.is_empty(),
        columns: audience.columns,
        rows,
    }
}
