//! The entities table: every governed entity as one row, read the way an
//! auditor asks the question — who reaches this, who is refused, why, and
//! does the code agree with the database.
//!
//! Built from the flat rule ledger by grouping on the entity, so the page
//! shows forty-odd facts instead of a hundred and fifty rows. The plain-
//! English resolution under each row is derived from the same bands the
//! resolver reads, in the resolver's own order.

use chrono::Utc;
use serde::Deserialize;
use systemprompt_security::authz::DASHBOARD_SOURCE;

use super::bands::{band_label, band_rank, chips, resolution};
use super::summary::{self, options};
use super::view::{AcEntitiesView, AcEntityView, AcRuleView};
use crate::handlers::ssr::people_view::expires_soon;
use crate::repositories::access_control::drift::DriftReport;
use crate::repositories::access_control::rules::LedgerRuleRow;
use crate::types::access_control::AccessDecision;

pub(super) const BASE_URL: &str = "/admin/access-control";
pub(super) const SYNC_URL: &str = "/admin/sync";
pub(super) const DOCS_URL: &str = "/documentation/access-control";

#[derive(Debug, Default, Deserialize)]
pub(crate) struct AcQuery {
    // Why: the editor's old deep link. A `?user=` on this page is answered by
    // a redirect to that person's Access tab, where the matrix now lives.
    pub user: Option<String>,
    pub entity_kind: Option<String>,
    pub band: Option<String>,
    pub state: Option<String>,
    // Why: `expiring` narrows to entities with a rule that lapses inside the
    // next week — the list an operator renews from.
    pub expiring: Option<String>,
    pub q: Option<String>,
}

impl AcQuery {
    fn pick(value: Option<&String>) -> Option<&str> {
        value.map(String::as_str).filter(|v| !v.is_empty())
    }

    pub(super) fn any_applied(&self) -> bool {
        Self::pick(self.entity_kind.as_ref()).is_some()
            || Self::pick(self.band.as_ref()).is_some()
            || Self::pick(self.state.as_ref()).is_some()
            || Self::pick(self.expiring.as_ref()).is_some()
            || Self::pick(self.q.as_ref()).is_some()
    }
}

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

fn entity_view(rows: &[&LedgerRuleRow], drift: Option<&DriftReport>) -> AcEntityView {
    let first = rows[0];
    let allow = chips(rows, AccessDecision::Allow);
    let deny = chips(rows, AccessDecision::Deny);
    let open = first.default_included;
    let drifted = drift.is_some_and(|d| d.touches(&first.entity_type, &first.entity_id));
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
        entity_type_label: first.entity_type.replace('_', " "),
        entity_type: first.entity_type.clone(),
        entity_id: first.entity_id.clone(),
        default_open: open,
        default_label: if open { "open" } else { "closed" },
        resolution: resolution(&allow, &deny, open),
        allow,
        deny,
        has_why: !why.is_empty(),
        why: why.join(" · "),
        state: if drift.is_none() {
            "Unknown"
        } else if drifted {
            "Drift"
        } else {
            "In sync"
        },
        state_tone: if drift.is_none() {
            "muted"
        } else if drifted {
            "warn"
        } else {
            "ok"
        },
        in_sync: drift.is_some() && !drifted,
        sync_url: format!(
            "{SYNC_URL}?entity={}",
            urlencoding::encode(&format!("{}/{}", first.entity_type, first.entity_id))
        ),
        rule_count: rules.len(),
        expiring_soon: rules.iter().any(|r| r.expires_soon),
        rules,
    }
}

fn matches(view: &AcEntityView, query: &AcQuery) -> bool {
    if let Some(kind) = AcQuery::pick(query.entity_kind.as_ref())
        && view.entity_type != kind
    {
        return false;
    }
    if let Some(band) = AcQuery::pick(query.band.as_ref())
        && !view.rules.iter().any(|r| r.band == band)
    {
        return false;
    }
    if let Some(state) = AcQuery::pick(query.state.as_ref()) {
        let want_sync = state == "in_sync";
        if view.in_sync != want_sync {
            return false;
        }
    }
    if AcQuery::pick(query.expiring.as_ref()).is_some() && !view.expiring_soon {
        return false;
    }
    if let Some(needle) = AcQuery::pick(query.q.as_ref()) {
        let needle = needle.to_lowercase();
        let hay = format!(
            "{} {} {}",
            view.entity_id,
            view.why,
            view.rules
                .iter()
                .map(|r| r.subject.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        )
        .to_lowercase();
        if !hay.contains(&needle) {
            return false;
        }
    }
    true
}

pub(super) struct EntitiesInput<'a> {
    pub rows: &'a [LedgerRuleRow],
    pub drift: Option<&'a DriftReport>,
    pub open_entities: i64,
    pub audiences: usize,
    pub capped: bool,
}

pub(super) fn build(input: &EntitiesInput<'_>, query: &AcQuery) -> AcEntitiesView {
    let mut grouped: Vec<Vec<&LedgerRuleRow>> = Vec::new();
    for row in input.rows {
        match grouped
            .iter_mut()
            .find(|g| g[0].entity_type == row.entity_type && g[0].entity_id == row.entity_id)
        {
            Some(g) => g.push(row),
            None => grouped.push(vec![row]),
        }
    }
    let all: Vec<AcEntityView> = grouped
        .iter()
        .map(|g| entity_view(g, input.drift))
        .collect();

    let kpis = summary::kpis(input, all.len());

    let mut kinds: Vec<String> = all.iter().map(|e| e.entity_type.clone()).collect();
    kinds.sort();
    kinds.dedup();
    let mut bands: Vec<String> = input.rows.iter().map(|r| r.rule_type.clone()).collect();
    bands.sort_by_key(|b| band_rank(b));
    bands.dedup();

    let rows: Vec<AcEntityView> = all.into_iter().filter(|e| matches(e, query)).collect();
    AcEntitiesView {
        total: rows.len(),
        rows,
        kpis,
        entity_options: options(
            kinds
                .into_iter()
                .map(|k| (k.clone(), k.replace('_', " ")))
                .collect(),
            "All entity kinds",
            AcQuery::pick(query.entity_kind.as_ref()),
        ),
        band_options: options(
            bands
                .into_iter()
                .map(|b| (b.clone(), band_label(&b).to_owned()))
                .collect(),
            "Any band",
            AcQuery::pick(query.band.as_ref()),
        ),
        state_options: options(
            vec![
                ("in_sync".to_owned(), "In sync with code".to_owned()),
                ("drift".to_owned(), "Differs from code".to_owned()),
            ],
            "Any state",
            AcQuery::pick(query.state.as_ref()),
        ),
        expiring_only: AcQuery::pick(query.expiring.as_ref()).is_some(),
        search: AcQuery::pick(query.q.as_ref())
            .unwrap_or_default()
            .to_owned(),
        filters_applied: query.any_applied(),
        clear_url: BASE_URL,
        capped: input.capped,
    }
}
