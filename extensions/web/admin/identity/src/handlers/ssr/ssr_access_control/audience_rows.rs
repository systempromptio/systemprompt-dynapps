//! The audience grid's cells and rows: the decision vocabulary a cell
//! carries (decision, glyph, tone and the short band word), and the
//! resolver's per-subject matrices regrouped into one row per entity, with
//! the column and row groupings the template draws.

use std::collections::HashMap;

use super::audience::audience_url;
use super::view::{
    AcAudienceCellView, AcAudienceColumnGroupView, AcAudienceColumnView, AcAudienceLegendView,
    AcAudienceRowGroupView, AcAudienceRowView,
};
use systemprompt_web_admin_core::repositories::entitlement::{MatrixSection, SectionInput};
use systemprompt_web_admin_core::ssr::entity_kind::{
    entity_kind_label, entity_kind_plural, entity_kind_rank,
};

pub(super) fn short_layer(layer: &str, effective: &str) -> String {
    match layer {
        "user" => "person".to_owned(),
        "default" | "default_included" if effective == "allow" => "open".to_owned(),
        "default" | "default_included" => "closed".to_owned(),
        other => other.replace('_', " "),
    }
}

pub(super) fn rule_type_of(short: &str) -> String {
    match short {
        "person" => "user".to_owned(),
        "open" | "closed" => "default".to_owned(),
        other => other.replace(' ', "_"),
    }
}

pub(super) fn decision_of(effective: &str) -> (&'static str, &'static str, &'static str) {
    match effective {
        "allow" => ("allow", "✓", "ok"),
        "warn" => ("warn", "⚠", "warn"),
        "pending" => ("pending", "◌", "info"),
        _ => ("deny", "✕", "err"),
    }
}

pub(super) fn legend() -> Vec<AcAudienceLegendView> {
    [
        ("allow", "Allowed"),
        ("deny", "Denied"),
        ("warn", "Warn"),
        ("pending", "Pending approval"),
    ]
    .into_iter()
    .map(|(decision, label)| {
        let (decision, glyph, tone) = decision_of(decision);
        AcAudienceLegendView {
            decision,
            glyph,
            tone,
            label,
        }
    })
    .collect()
}

fn route_pattern_line(entity_type: &str, description: Option<&str>) -> Option<String> {
    (entity_type == "gateway_route")
        .then_some(description)
        .flatten()
        .map(str::to_owned)
}

pub(super) fn resolved_rows(
    resolved: &[Vec<MatrixSection>],
    columns: &[AcAudienceColumnView],
    sections: &[SectionInput],
) -> Vec<AcAudienceRowView> {
    let mut cells_by_entity: HashMap<(String, String), Vec<AcAudienceCellView>> = HashMap::new();
    for (matrix, col) in resolved.iter().zip(columns) {
        for section in matrix {
            for row in &section.rows {
                let (decision, glyph, tone) = decision_of(&row.effective);
                cells_by_entity
                    .entry((section.entity_type.clone(), row.entity_id.clone()))
                    .or_default()
                    .push(AcAudienceCellView {
                        decision,
                        glyph,
                        tone,
                        layer: short_layer(&row.source.layer, &row.effective),
                        detail: row.source.detail.clone(),
                        subject_id: col.id.clone(),
                    });
            }
        }
    }
    let mut rows = Vec::new();
    for (entity_type, _, entities) in sections {
        for (entity_id, name, description) in entities {
            let Some(cells) = cells_by_entity.remove(&(entity_type.clone(), entity_id.clone()))
            else {
                continue;
            };
            let entity_ref = format!("{entity_type}/{entity_id}");
            rows.push(AcAudienceRowView {
                entity_type_label: entity_kind_label(entity_type),
                entity_type: entity_type.clone(),
                entity_id: entity_id.clone(),
                entity_name: name.clone(),
                entity_sub: route_pattern_line(entity_type, description.as_deref()),
                drill_url: audience_url(&[("entity", &entity_ref)]),
                allowed: 0,
                denied: 0,
                cells,
            });
        }
    }
    rows
}

pub(super) fn column_groups(columns: &[AcAudienceColumnView]) -> Vec<AcAudienceColumnGroupView> {
    let mut groups: Vec<AcAudienceColumnGroupView> = Vec::new();
    for col in columns {
        match groups.last_mut() {
            Some(g) if g.kind == col.kind => g.span += 1,
            _ => groups.push(AcAudienceColumnGroupView {
                kind: col.kind,
                kind_label: col.kind_label,
                span: 1,
            }),
        }
    }
    groups
}

pub(super) fn row_groups(rows: Vec<AcAudienceRowView>) -> Vec<AcAudienceRowGroupView> {
    let mut groups: Vec<AcAudienceRowGroupView> = Vec::new();
    for row in rows {
        match groups.iter_mut().find(|g| g.kind == row.entity_type) {
            Some(g) => g.rows.push(row),
            None => groups.push(AcAudienceRowGroupView {
                kind_label: entity_kind_plural(&row.entity_type),
                kind: row.entity_type.clone(),
                count: 0,
                rows: vec![row],
            }),
        }
    }
    groups.sort_by_key(|g| entity_kind_rank(&g.kind));
    for g in &mut groups {
        g.rows.sort_by(|a, b| {
            a.entity_name
                .to_lowercase()
                .cmp(&b.entity_name.to_lowercase())
        });
        g.count = g.rows.len();
    }
    groups
}
