//! The `gateway_routes` plane's drift table: one row per route that is added
//! in code, removed from code, changed, or reordered, labelled with what each
//! sync mode would do to it.

use crate::repositories::gateway_routes::drift::RouteDrift;
use systemprompt_web_admin_core::repositories::sync::plane::{DriftRow, resolution};

fn row(kind: &'static str, id: &str, in_code: String, in_db: String, dashboard: bool) -> DriftRow {
    let (kind_label, kind_tone, insert_applies, overwrite_effect) = match (kind, dashboard) {
        ("missing", _) => ("Added in code", "ok", true, "inserted"),
        ("orphan", true) => ("Written in console", "warn", false, "deleted"),
        ("orphan", false) => ("Removed from code", "err", false, "deleted"),
        ("reordered", _) => ("Order changed in code", "warn", false, "reordered"),
        (_, true) => ("Edited in console", "warn", false, "updated"),
        (_, false) => ("Changed in code", "warn", false, "updated"),
    };
    let (resolve, resolve_tone) = resolution(insert_applies, true, overwrite_effect);
    DriftRow {
        kind,
        kind_label,
        kind_tone,
        entity_type: "gateway_route".to_owned(),
        entity_type_label: "gateway route".to_owned(),
        entity_id: id.to_owned(),
        band: String::new(),
        band_label: "route",
        subject: id.to_owned(),
        in_code,
        in_db,
        origin: match kind {
            "missing" => "",
            _ if dashboard => "console",
            _ => "code",
        },
        origin_tone: if dashboard { "info" } else { "muted" },
        governed: true,
        insert_applies,
        overwrite_applies: true,
        overwrite_effect,
        resolve,
        resolve_tone,
    }
}

pub(super) fn rows(drift: &RouteDrift) -> Vec<DriftRow> {
    let mut out = Vec::new();
    out.extend(
        drift
            .missing_in_db
            .iter()
            .map(|id| row("missing", id, "declared".to_owned(), "—".to_owned(), false)),
    );
    out.extend(drift.only_in_db.iter().map(|o| {
        row(
            "orphan",
            &o.id,
            "—".to_owned(),
            o.summary.clone(),
            o.dashboard,
        )
    }));
    out.extend(drift.changed.iter().map(|c| {
        let mut r = row(
            "changed",
            &c.id,
            c.in_code.clone(),
            c.in_db.clone(),
            c.dashboard,
        );
        r.subject = c.fields.join(", ");
        r
    }));
    if let Some((code, db)) = &drift.reordered {
        out.push(row(
            "reordered",
            "order",
            code.join(" › "),
            db.join(" › "),
            true,
        ));
    }
    out
}
