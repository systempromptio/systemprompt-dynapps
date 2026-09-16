//! Code versus database: what `rules.yaml` declares that the database does
//! not hold, what the database holds that the file does not declare, and
//! where the two disagree about a row they both name.
//!
//! Pure over two snapshots. Boot computes it to warn; the access-control page
//! computes it for the banner; the sync page lists it and, inside its own
//! transaction, recomputes it before writing so a dashboard edit made a
//! second earlier is never deleted from a stale picture.
//!
//! The `user` band is invisible here by construction: per-person overrides
//! are the dashboard's alone and the file has no shape for them.

use serde::Serialize;
use systemprompt_security::authz::{Access, DASHBOARD_SOURCE};

use super::declared::{DeclaredEntity, DeclaredKey, DeclaredRule, DeclaredSet};

/// One non-user rule row as the database holds it.
#[derive(Debug, Clone, Serialize)]
pub struct BandRuleRow {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub rule_type: String,
    pub rule_value: String,
    pub access: Access,
    pub justification: Option<String>,
    pub source: String,
}

impl BandRuleRow {
    #[must_use]
    pub fn key(&self) -> DeclaredKey {
        DeclaredKey {
            entity_type: self.entity_type.clone(),
            entity_id: self.entity_id.clone(),
            rule_type: self.rule_type.clone(),
            rule_value: self.rule_value.clone(),
        }
    }
}

/// One `access_control_entities` row.
#[derive(Debug, Clone, Serialize)]
pub struct EntityDefaultRow {
    pub entity_type: String,
    pub entity_id: String,
    pub default_included: bool,
    pub source: String,
}

/// Where a database-only row came from, as far as its `source` column says.
///
/// `Dashboard` was written from the console and stamped as such; `Stale` is
/// stamped `yaml` but absent from the file, or written by the console
/// without a stamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrphanOrigin {
    Dashboard,
    Stale,
}

/// A database-only row. `governed` says whether the file speaks for this
/// entity; only governed orphans are deleted by an overwrite.
#[derive(Debug, Clone, Serialize)]
pub struct OrphanRule {
    pub row: BandRuleRow,
    pub origin: OrphanOrigin,
    pub governed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChangedRule {
    pub key: DeclaredKey,
    pub db_id: String,
    pub declared_access: Access,
    pub db_access: Access,
    pub declared_why: String,
    pub db_why: Option<String>,
    pub db_source: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DefaultDrift {
    pub entity_type: String,
    pub entity_id: String,
    pub declared_open: bool,
    pub db_open: bool,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct DriftReport {
    pub missing_in_db: Vec<DeclaredRule>,
    pub only_in_db: Vec<OrphanRule>,
    pub changed: Vec<ChangedRule>,
    pub default_changed: Vec<DefaultDrift>,
    pub entities_missing: Vec<DeclaredEntity>,
}

#[derive(Debug, Default, Clone, Copy, Serialize)]
pub struct DriftCounts {
    pub missing_in_db: usize,
    pub only_in_db: usize,
    pub only_in_db_dashboard: usize,
    pub only_in_db_stale: usize,
    pub only_in_db_governed: usize,
    pub changed: usize,
    pub default_changed: usize,
    pub entities_missing: usize,
    pub total: usize,
}

impl DriftReport {
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.counts().total == 0
    }

    #[must_use]
    pub fn counts(&self) -> DriftCounts {
        let dashboard = self
            .only_in_db
            .iter()
            .filter(|o| o.origin == OrphanOrigin::Dashboard)
            .count();
        let governed = self.only_in_db.iter().filter(|o| o.governed).count();
        let total = self.missing_in_db.len()
            + self.only_in_db.len()
            + self.changed.len()
            + self.default_changed.len()
            + self.entities_missing.len();
        DriftCounts {
            missing_in_db: self.missing_in_db.len(),
            only_in_db: self.only_in_db.len(),
            only_in_db_dashboard: dashboard,
            only_in_db_stale: self.only_in_db.len() - dashboard,
            only_in_db_governed: governed,
            changed: self.changed.len(),
            default_changed: self.default_changed.len(),
            entities_missing: self.entities_missing.len(),
            total,
        }
    }

    // Why: the per-row "State" column on the access-control page.
    #[must_use]
    pub fn touches(&self, entity_type: &str, entity_id: &str) -> bool {
        let same = |t: &str, i: &str| t == entity_type && i == entity_id;
        self.missing_in_db
            .iter()
            .any(|r| same(&r.key.entity_type, &r.key.entity_id))
            || self
                .only_in_db
                .iter()
                .any(|o| same(&o.row.entity_type, &o.row.entity_id))
            || self
                .changed
                .iter()
                .any(|c| same(&c.key.entity_type, &c.key.entity_id))
            || self
                .default_changed
                .iter()
                .any(|d| same(&d.entity_type, &d.entity_id))
            || self
                .entities_missing
                .iter()
                .any(|e| same(&e.entity_type, &e.entity_id))
    }
}

#[must_use]
pub fn compute_drift(
    declared: &DeclaredSet,
    db_rules: &[BandRuleRow],
    db_entities: &[EntityDefaultRow],
) -> DriftReport {
    let mut report = DriftReport::default();

    for row in db_rules {
        if row.rule_type == "user" {
            continue;
        }
        match declared.rules.get(&row.key()) {
            None => report.only_in_db.push(OrphanRule {
                origin: if row.source == DASHBOARD_SOURCE {
                    OrphanOrigin::Dashboard
                } else {
                    OrphanOrigin::Stale
                },
                governed: declared.governs(&row.entity_type, &row.entity_id),
                row: row.clone(),
            }),
            Some(want) => {
                let why_differs = row.justification.as_deref() != Some(want.justification.as_str());
                if want.access != row.access || why_differs {
                    report.changed.push(ChangedRule {
                        key: want.key.clone(),
                        db_id: row.id.clone(),
                        declared_access: want.access,
                        db_access: row.access,
                        declared_why: want.justification.clone(),
                        db_why: row.justification.clone(),
                        db_source: row.source.clone(),
                    });
                }
            },
        }
    }

    for (key, want) in &declared.rules {
        if !db_rules.iter().any(|r| r.key() == *key) {
            report.missing_in_db.push(want.clone());
        }
    }

    for entity in declared.entities.values() {
        match db_entities
            .iter()
            .find(|e| e.entity_type == entity.entity_type && e.entity_id == entity.entity_id)
        {
            None => report.entities_missing.push(entity.clone()),
            Some(row) if row.default_included != entity.default_included => {
                report.default_changed.push(DefaultDrift {
                    entity_type: entity.entity_type.clone(),
                    entity_id: entity.entity_id.clone(),
                    declared_open: entity.default_included,
                    db_open: row.default_included,
                });
            },
            Some(_) => {},
        }
    }

    report
}
