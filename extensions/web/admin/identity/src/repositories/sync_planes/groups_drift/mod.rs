//! `groups.yaml` against the `groups`, `projects` and mapping tables — the
//! comparison, with no database in it.
//!
//! The loader's asymmetry is the drift's vocabulary. Groups and projects
//! are only ever upserted, so a set the file no longer names is reported
//! and *kept* by every mode — deleting one is a dashboard act with the
//! member list in view. Mappings are reconciled, so a `yaml`-sourced
//! mapping the file dropped is deleted by *Overwrite*, while a mapping the
//! console wrote is kept. The derived `unassigned` group is system-owned
//! and never compared.

mod compare;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::repositories::config::groups_yaml_types::{GroupsDoc, MemberSetDef};
use systemprompt_web_admin_core::repositories::sync::plane::DriftRow;

/// A group or project as the database holds it.
#[derive(Debug, Clone, Serialize)]
pub struct MemberSetRow {
    pub kind: String,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub source: String,
    pub is_system: bool,
}

/// One AD-group mapping as the database holds it.
#[derive(Debug, Clone, Serialize)]
pub struct MappingRow {
    pub kind: String,
    pub ad_group: String,
    pub set_id: String,
    pub source: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct GroupsDrift {
    pub missing_sets: usize,
    pub changed_sets: usize,
    pub orphan_sets: usize,
    pub missing_mappings: usize,
    pub orphan_yaml_mappings: usize,
    pub orphan_console_mappings: usize,
    pub rows: Vec<DriftRow>,
}

impl GroupsDrift {
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.rows.is_empty()
    }
}

fn declared_sets(doc: &GroupsDoc) -> impl Iterator<Item = (&'static str, &MemberSetDef)> {
    doc.groups
        .iter()
        .map(|d| ("group", d))
        .chain(doc.projects.iter().map(|d| ("project", d)))
}

#[must_use]
pub fn declared_hash(doc: &GroupsDoc) -> String {
    let mut hasher = Sha256::new();
    for (kind, def) in declared_sets(doc) {
        hasher.update(
            format!(
                "{kind}\t{}\t{}\t{}\n",
                def.id,
                def.name,
                def.description.as_deref().unwrap_or("")
            )
            .as_bytes(),
        );
        let mut ad = def.ad_groups.clone();
        ad.sort();
        for a in ad {
            hasher.update(format!("{kind}-mapping\t{}\t{a}\n", def.id).as_bytes());
        }
    }
    format!("{:x}", hasher.finalize())
}

#[must_use]
pub fn declared_count(doc: &GroupsDoc) -> usize {
    declared_sets(doc).map(|(_, d)| 1 + d.ad_groups.len()).sum()
}

#[must_use]
pub fn compute_groups_drift(
    doc: &GroupsDoc,
    sets: &[MemberSetRow],
    mappings: &[MappingRow],
) -> GroupsDrift {
    let mut drift = GroupsDrift::default();
    compare::compare_sets(doc, sets, &mut drift);
    compare::compare_mappings(doc, mappings, &mut drift);
    drift.rows.sort_by(|a, b| {
        (&a.entity_type, &a.entity_id, &a.subject).cmp(&(&b.entity_type, &b.entity_id, &b.subject))
    });
    drift
}
