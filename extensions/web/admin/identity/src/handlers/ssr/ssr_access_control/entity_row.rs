//! One governed entity's row in the entities table, mapped from its slice of
//! the rule ledger: labels, allow/deny chips, the plain-English resolution,
//! drift state and the band-ordered rule list.
//!
//! Only a gateway route is relabelled from the route catalog — its id says
//! nothing, while every other kind is addressed by a name an operator chose.

use chrono::Utc;
use systemprompt_security::authz::DASHBOARD_SOURCE;

use super::bands::{band_label, band_rank, chips, resolution};
use super::entities::SYNC_URL;
use super::view::{AcEntityView, AcRuleView};
use crate::handlers::ssr::people_view::expires_soon;
use systemprompt_web_admin_core::repositories::access_rules::drift::{DriftReport, EntityState};
use systemprompt_web_admin_core::repositories::access_rules::rules::LedgerRuleRow;
use systemprompt_web_admin_core::repositories::gateway_catalog::RouteLabels;
use systemprompt_web_admin_core::ssr::entity_kind::entity_kind_label;
use systemprompt_web_admin_core::types::access_control::AccessDecision;

fn rule_view(row: &LedgerRuleRow) -> AcRuleView {
    let dashboard = row.source == DASHBOARD_SOURCE;
    AcRuleView {
        band: row.rule_type.clone(),
        band_label: band_label(&row.rule_type),
        subject: row.rule_value.clone(),
        access: row.access.to_string(),
        access_tone: match row.access {
            AccessDecision::Allow => "ok",
            AccessDecision::Deny => "err",
        },
        source: if dashboard {
            "console".to_owned()
        } else {
            "code".to_owned()
        },
        source_tone: if dashboard { "warn" } else { "muted" },
        justification: row.justification.clone().unwrap_or_default(),
        expires_at: row.valid_until.map(|t| t.to_rfc3339()),
        expires_soon: expires_soon(row.valid_until, Utc::now()),
    }
}

fn label_of(kind: &str, id: &str, labels: &RouteLabels) -> (String, Option<String>, bool) {
    if kind != "gateway_route" {
        return (id.to_owned(), None, false);
    }
    labels.find(id).map_or_else(
        || (id.to_owned(), None, false),
        |l| (l.label.clone(), Some(l.subtitle()), true),
    )
}

pub(super) fn entity_view(
    rows: &[&LedgerRuleRow],
    drift: Option<&DriftReport>,
    labels: &RouteLabels,
) -> AcEntityView {
    let first = rows[0];
    let (entity_label, entity_sub, labelled) =
        label_of(&first.entity_type, &first.entity_id, labels);
    let allow = chips(rows, AccessDecision::Allow);
    let deny = chips(rows, AccessDecision::Deny);
    let open = first.default_included;
    let state = EntityState::for_entity(drift, &first.entity_type, &first.entity_id);
    let mut why: Vec<&str> = rows
        .iter()
        .filter_map(|r| r.justification.as_deref())
        .filter(|w| !w.trim().is_empty())
        .collect();
    why.sort_unstable();
    why.dedup();
    let mut rules: Vec<AcRuleView> = rows.iter().map(|r| rule_view(r)).collect();
    rules.sort_by(|a, b| {
        band_rank(&a.band)
            .cmp(&band_rank(&b.band))
            .then(a.subject.cmp(&b.subject))
    });
    AcEntityView {
        entity_type_label: entity_kind_label(&first.entity_type),
        entity_type: first.entity_type.clone(),
        entity_id: first.entity_id.clone(),
        entity_label,
        entity_sub,
        labelled,
        default_open: open,
        default_label: if open { "open" } else { "closed" },
        resolution: resolution(&allow, &deny, open),
        allow,
        deny,
        has_why: !why.is_empty(),
        why: why.join(" · "),
        state: state.label(),
        state_tone: state.tone(),
        in_sync: state == EntityState::InSync,
        sync_url: SYNC_URL.to_owned(),
        rule_count: rules.len(),
        expiring_soon: rules.iter().any(|r| r.expires_soon),
        rules,
    }
}
