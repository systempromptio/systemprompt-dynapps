//! The set and mapping comparisons, and the one row shape both report in:
//! what each kind of difference is called, its tone, and what each sync
//! mode would do about it.

use super::{GroupsDrift, MappingRow, MemberSetRow, declared_sets};
use crate::repositories::config::groups_yaml_types::GroupsDoc;
use systemprompt_web_admin_core::repositories::sync::plane::{DriftRow, resolution};

fn describe(name: &str, description: Option<&str>) -> String {
    match description {
        Some(d) if !d.trim().is_empty() => format!("{name} — {}", d.trim()),
        _ => name.to_owned(),
    }
}

struct Line<'a> {
    kind: &'static str,
    entity_kind: &'a str,
    entity_id: &'a str,
    subject: String,
    in_code: String,
    in_db: String,
    origin: &'static str,
    overwrite_applies: bool,
    overwrite_effect: &'static str,
}

fn row(line: Line<'_>) -> DriftRow {
    let (kind_label, kind_tone) = match (line.kind, line.origin) {
        ("missing", _) => ("Added in code", "ok"),
        ("orphan", "console") => ("Written in console", "warn"),
        ("orphan", _) => ("Removed from code", "err"),
        (_, "console") => ("Edited in console", "warn"),
        _ => ("Changed in code", "warn"),
    };
    let (resolve, resolve_tone) = match (line.kind, line.overwrite_applies) {
        ("missing", _) => resolution(true, true, line.overwrite_effect),
        ("orphan", false) if line.origin == "console" => ("Export to keep".to_owned(), "info"),
        _ => resolution(false, line.overwrite_applies, line.overwrite_effect),
    };
    DriftRow {
        kind: line.kind,
        kind_label,
        kind_tone,
        entity_type: line.entity_kind.to_owned(),
        entity_type_label: line.entity_kind.to_owned(),
        entity_id: line.entity_id.to_owned(),
        band: String::new(),
        band_label: if line.subject.is_empty() {
            "definition"
        } else {
            "AD group"
        },
        subject: line.subject,
        in_code: line.in_code,
        in_db: line.in_db,
        origin: line.origin,
        origin_tone: if line.origin == "console" {
            "info"
        } else {
            "muted"
        },
        governed: true,
        insert_applies: line.kind == "missing",
        overwrite_applies: line.overwrite_applies,
        overwrite_effect: line.overwrite_effect,
        resolve,
        resolve_tone,
    }
}

pub(super) fn compare_sets(doc: &GroupsDoc, sets: &[MemberSetRow], drift: &mut GroupsDrift) {
    for (kind, def) in declared_sets(doc) {
        let in_code = describe(&def.name, def.description.as_deref());
        match sets.iter().find(|s| s.kind == kind && s.id == def.id) {
            None => {
                drift.missing_sets += 1;
                drift.rows.push(row(Line {
                    kind: "missing",
                    entity_kind: kind,
                    entity_id: &def.id,
                    subject: String::new(),
                    in_code,
                    in_db: "—".to_owned(),
                    origin: "",
                    overwrite_applies: true,
                    overwrite_effect: "inserted",
                }));
            },
            Some(s) => {
                let in_db = describe(&s.name, s.description.as_deref());
                if in_db != in_code {
                    drift.changed_sets += 1;
                    let origin = if s.source == "dashboard" {
                        "console"
                    } else {
                        "code"
                    };
                    drift.rows.push(row(Line {
                        kind: "changed",
                        entity_kind: kind,
                        entity_id: &def.id,
                        subject: String::new(),
                        in_code,
                        in_db,
                        origin,
                        overwrite_applies: true,
                        overwrite_effect: "updated",
                    }));
                }
            },
        }
    }
    for s in sets.iter().filter(|s| !s.is_system && s.source != "system") {
        let declared = declared_sets(doc).any(|(k, d)| k == s.kind && d.id == s.id);
        if declared {
            continue;
        }
        drift.orphan_sets += 1;
        let origin = if s.source == "dashboard" {
            "console"
        } else {
            "code"
        };
        drift.rows.push(row(Line {
            kind: "orphan",
            entity_kind: &s.kind,
            entity_id: &s.id,
            subject: String::new(),
            in_code: "—".to_owned(),
            in_db: describe(&s.name, s.description.as_deref()),
            origin,
            overwrite_applies: false,
            overwrite_effect: "kept (a group is deleted from its own page, never by sync)",
        }));
    }
}

pub(super) fn compare_mappings(doc: &GroupsDoc, mappings: &[MappingRow], drift: &mut GroupsDrift) {
    for (kind, def) in declared_sets(doc) {
        for ad in &def.ad_groups {
            let present = mappings
                .iter()
                .any(|m| m.kind == kind && m.set_id == def.id && &m.ad_group == ad);
            if !present {
                drift.missing_mappings += 1;
                drift.rows.push(row(Line {
                    kind: "missing",
                    entity_kind: kind,
                    entity_id: &def.id,
                    subject: ad.clone(),
                    in_code: "mapped".to_owned(),
                    in_db: "—".to_owned(),
                    origin: "",
                    overwrite_applies: true,
                    overwrite_effect: "inserted",
                }));
            }
        }
    }
    for m in mappings {
        let declared = declared_sets(doc)
            .any(|(k, d)| k == m.kind && d.id == m.set_id && d.ad_groups.contains(&m.ad_group));
        if declared {
            continue;
        }
        let console = m.source == "dashboard";
        if console {
            drift.orphan_console_mappings += 1;
        } else {
            drift.orphan_yaml_mappings += 1;
        }
        drift.rows.push(row(Line {
            kind: "orphan",
            entity_kind: &m.kind,
            entity_id: &m.set_id,
            subject: m.ad_group.clone(),
            in_code: "—".to_owned(),
            in_db: "mapped".to_owned(),
            origin: if console { "console" } else { "code" },
            overwrite_applies: !console,
            overwrite_effect: if console {
                "kept (written from the console)"
            } else {
                "deleted"
            },
        }));
    }
}
