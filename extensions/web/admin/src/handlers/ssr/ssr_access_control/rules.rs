//! The rule ledger the access-control page renders above its editor.
//!
//! The editor answers "what may this person reach"; the ledger answers the
//! question an auditor asks instead — what rules exist at all, at which
//! subject band, and which of them this instance's database holds without the
//! YAML in the source repository declaring them. That last column is the one
//! an operator cannot get anywhere else: a rule outside the declarative set
//! disappears the next time the estate is rebuilt from source.

use serde::{Deserialize, Serialize};

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::repositories::access_control::rules::LedgerRuleRow;
use crate::repositories::access_control::yaml_declared::DeclaredRules;

pub(super) const BASE_URL: &str = "/admin/access-control";
pub(super) const PAGE_SIZE: i64 = 50;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RulesQuery {
    pub subject_kind: Option<String>,
    pub entity_kind: Option<String>,
    pub source: Option<String>,
    pub q: Option<String>,
    pub page: Option<i64>,
    pub sort: Option<String>,
    pub dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcRuleView {
    pub entity_type: String,
    pub entity_type_label: String,
    pub entity_id: String,
    pub subject_kind: String,
    pub subject: String,
    pub access: String,
    pub access_tone: &'static str,
    pub source: &'static str,
    pub source_tone: &'static str,
    pub default_label: &'static str,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcSortHeaderView {
    pub label: &'static str,
    pub class: &'static str,
    pub hint: &'static str,
    pub url: String,
    pub active: bool,
    pub aria_sort: &'static str,
    pub indicator: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcKpiView {
    pub label: &'static str,
    pub value: String,
    pub sub: String,
    pub tone: &'static str,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct AcRulesView {
    pub rows: Vec<AcRuleView>,
    pub total: i64,
    pub pagination: Option<Pagination>,
    pub kpis: Vec<AcKpiView>,
    pub sort_headers: Vec<AcSortHeaderView>,
    pub subject_options: Vec<AcOptionView>,
    pub entity_options: Vec<AcOptionView>,
    pub source_options: Vec<AcOptionView>,
    pub search: String,
    pub filters_applied: bool,
    pub clear_url: &'static str,
    pub capped: bool,
}

const YAML: &str = "YAML";
const MANUAL: &str = "This instance";

fn field<'a>(value: Option<&'a String>) -> Option<&'a str> {
    value.map(String::as_str).filter(|v| !v.is_empty())
}

impl RulesQuery {
    fn selected(&self, name: &str) -> Option<&str> {
        match name {
            "subject_kind" => field(self.subject_kind.as_ref()),
            "entity_kind" => field(self.entity_kind.as_ref()),
            "source" => field(self.source.as_ref()),
            "q" => field(self.q.as_ref()),
            _ => None,
        }
    }

    fn sort_key(&self) -> &str {
        match self.sort.as_deref() {
            Some(key @ ("entity" | "subject_kind" | "subject" | "access" | "source")) => key,
            _ => "entity_kind",
        }
    }

    fn descending(&self) -> bool {
        self.dir.as_deref() == Some("desc")
    }

    fn any_applied(&self) -> bool {
        ["subject_kind", "entity_kind", "source", "q"]
            .iter()
            .any(|f| self.selected(f).is_some())
    }

    fn url_with(&self, overrides: &[(&str, &str)]) -> String {
        let mut parts: Vec<String> = Vec::new();
        for name in [
            "subject_kind",
            "entity_kind",
            "source",
            "q",
            "sort",
            "dir",
            "page",
        ] {
            let value = match overrides.iter().find(|(k, _)| *k == name) {
                Some((_, v)) => (*v).to_owned(),
                None => match name {
                    "sort" => self.sort_key().to_owned(),
                    "dir" => self.dir.clone().unwrap_or_default(),
                    "page" => String::new(),
                    other => self.selected(other).unwrap_or_default().to_owned(),
                },
            };
            if value.is_empty() || (name == "sort" && value == "entity_kind") {
                continue;
            }
            parts.push(format!("{name}={}", urlencoding::encode(&value)));
        }
        if parts.is_empty() {
            return BASE_URL.to_owned();
        }
        format!("{BASE_URL}?{}", parts.join("&"))
    }
}

fn source_of(row: &LedgerRuleRow, declared: &DeclaredRules) -> &'static str {
    if declared.declares(
        &row.entity_type,
        &row.entity_id,
        &row.rule_type,
        &row.rule_value,
    ) {
        YAML
    } else {
        MANUAL
    }
}

fn matches(row: &LedgerRuleRow, source: &str, query: &RulesQuery) -> bool {
    if let Some(kind) = query.selected("subject_kind")
        && row.rule_type != kind
    {
        return false;
    }
    if let Some(kind) = query.selected("entity_kind")
        && row.entity_type != kind
    {
        return false;
    }
    if let Some(want) = query.selected("source") {
        let is_yaml = source == YAML;
        if is_yaml != (want == "yaml") {
            return false;
        }
    }
    if let Some(needle) = query.selected("q") {
        let needle = needle.to_lowercase();
        let haystack = format!("{} {}", row.entity_id, row.rule_value).to_lowercase();
        if !haystack.contains(&needle) {
            return false;
        }
    }
    true
}

fn options(
    values: &[(String, String)],
    all_label: &str,
    selected: Option<&str>,
) -> Vec<AcOptionView> {
    let mut out = vec![AcOptionView {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(values.iter().map(|(value, label)| AcOptionView {
        selected: selected == Some(value.as_str()),
        value: value.clone(),
        label: label.clone(),
    }));
    out
}

fn distinct(rows: &[LedgerRuleRow], pick: fn(&LedgerRuleRow) -> &String) -> Vec<(String, String)> {
    let mut seen: Vec<String> = rows.iter().map(|r| pick(r).clone()).collect();
    seen.sort();
    seen.dedup();
    seen.into_iter()
        .map(|v| {
            let label = v.replace('_', " ");
            (v, label)
        })
        .collect()
}

fn sort_headers(query: &RulesQuery) -> Vec<AcSortHeaderView> {
    let current = query.sort_key();
    let descending = query.descending();
    [
        ("entity_kind", "Entity kind", "", "What the rule grants access to."),
        ("entity", "Entity", "", "The id of that entity in the catalog."),
        (
            "subject_kind",
            "Subject kind",
            "",
            "The band the rule is written at: role, group, project or link.",
        ),
        ("subject", "Subject", "", "The value at that band the rule names."),
        ("access", "Access", "", "A deny at any band beats every allow."),
        (
            "source",
            "Rule source",
            "",
            "Whether the YAML in the source repository declares this rule, or only this database holds it.",
        ),
    ]
    .into_iter()
    .map(|(key, label, class, hint)| {
        let active = current == key;
        let next = if active && !descending { "desc" } else { "" };
        AcSortHeaderView {
            label,
            class,
            hint,
            url: query.url_with(&[("sort", key), ("dir", next)]),
            active,
            aria_sort: match (active, descending) {
                (true, false) => "ascending",
                (true, true) => "descending",
                (false, _) => "none",
            },
            indicator: match (active, descending) {
                (true, false) => "▲",
                (true, true) => "▼",
                (false, _) => "",
            },
        }
    })
    .collect()
}

fn kpis(rows: &[LedgerRuleRow], sources: &[&str], open_entities: i64) -> Vec<AcKpiView> {
    let manual = sources.iter().filter(|s| **s == MANUAL).count();
    let denies = rows
        .iter()
        .filter(|r| matches!(r.access, crate::types::access_control::AccessDecision::Deny))
        .count();
    let mut kinds: Vec<&str> = rows.iter().map(|r| r.rule_type.as_str()).collect();
    kinds.sort_unstable();
    kinds.dedup();
    let mut entities: Vec<(&str, &str)> = rows
        .iter()
        .map(|r| (r.entity_type.as_str(), r.entity_id.as_str()))
        .collect();
    entities.sort_unstable();
    entities.dedup();
    vec![
        AcKpiView {
            label: "Rules",
            value: rows.len().to_string(),
            sub: format!("across {} entities", entities.len()),
            tone: "accent",
        },
        AcKpiView {
            label: "Subject bands in use",
            value: kinds.len().to_string(),
            sub: kinds.join(", "),
            tone: "accent",
        },
        AcKpiView {
            label: "Denies",
            value: denies.to_string(),
            sub: "a deny beats every allow".to_owned(),
            tone: if denies > 0 { "warn" } else { "ok" },
        },
        AcKpiView {
            label: "Only in this database",
            value: manual.to_string(),
            sub: "not declared by any YAML file".to_owned(),
            tone: if manual > 0 { "warn" } else { "ok" },
        },
        AcKpiView {
            label: "Open by default",
            value: open_entities.to_string(),
            sub: "entities everyone reaches unless denied".to_owned(),
            tone: if open_entities > 0 { "warn" } else { "ok" },
        },
    ]
}

/// Build the whole ledger view from one read plus the declarative set.
pub(super) fn build(
    rows: &[LedgerRuleRow],
    declared: &DeclaredRules,
    open_entities: i64,
    query: &RulesQuery,
    capped: bool,
) -> AcRulesView {
    let sources: Vec<&str> = rows.iter().map(|r| source_of(r, declared)).collect();
    let mut matching: Vec<(&LedgerRuleRow, &str)> = rows
        .iter()
        .zip(sources.iter().copied())
        .filter(|(row, source)| matches(row, source, query))
        .collect();

    match query.sort_key() {
        "entity" => matching.sort_by(|a, b| a.0.entity_id.cmp(&b.0.entity_id)),
        "subject_kind" => matching.sort_by(|a, b| a.0.rule_type.cmp(&b.0.rule_type)),
        "subject" => matching.sort_by(|a, b| a.0.rule_value.cmp(&b.0.rule_value)),
        "access" => matching.sort_by_key(|(row, _)| row.access.to_string()),
        "source" => matching.sort_by_key(|(_, source)| *source),
        _ => matching.sort_by(|a, b| a.0.entity_type.cmp(&b.0.entity_type)),
    }
    if query.descending() {
        matching.reverse();
    }

    let total = i64::try_from(matching.len()).unwrap_or(i64::MAX);
    let last_page = (total.max(1) - 1) / PAGE_SIZE;
    let index = query.page.unwrap_or(0).clamp(0, last_page);
    let start = usize::try_from(index * PAGE_SIZE).unwrap_or(0);
    let page: Vec<AcRuleView> = matching
        .iter()
        .skip(start)
        .take(usize::try_from(PAGE_SIZE).unwrap_or(50))
        .map(|(row, source)| AcRuleView {
            entity_type_label: row.entity_type.replace('_', " "),
            entity_type: row.entity_type.clone(),
            entity_id: row.entity_id.clone(),
            subject_kind: row.rule_type.clone(),
            subject: row.rule_value.clone(),
            access: row.access.to_string(),
            access_tone: match row.access {
                crate::types::access_control::AccessDecision::Allow => "ok",
                crate::types::access_control::AccessDecision::Deny => "err",
            },
            source,
            source_tone: if *source == YAML { "muted" } else { "warn" },
            default_label: if row.default_included {
                "open"
            } else {
                "closed"
            },
            justification: row.justification.clone().unwrap_or_default(),
        })
        .collect();

    let window = PageWindow::new(
        index,
        PAGE_SIZE,
        total,
        i64::try_from(page.len()).unwrap_or(0),
        "rules",
    );
    let (first_row, last_row) = window.bounds();
    let prev_url = (index > 0).then(|| query.url_with(&[("page", &(index - 1).to_string())]));
    let next_url = (index + 1 < window.total_pages)
        .then(|| query.url_with(&[("page", &(index + 1).to_string())]));

    AcRulesView {
        rows: page,
        total,
        pagination: Some(Pagination {
            current_page: index + 1,
            total_pages: window.total_pages,
            first_row,
            last_row,
            total_rows: total,
            noun: window.noun,
            has_prev: prev_url.is_some(),
            has_next: next_url.is_some(),
            prev_url,
            next_url,
        }),
        kpis: kpis(rows, &sources, open_entities),
        sort_headers: sort_headers(query),
        subject_options: options(
            &distinct(rows, |r| &r.rule_type),
            "All subject kinds",
            query.selected("subject_kind"),
        ),
        entity_options: options(
            &distinct(rows, |r| &r.entity_type),
            "All entity kinds",
            query.selected("entity_kind"),
        ),
        source_options: options(
            &[
                ("yaml".to_owned(), "Declared in YAML".to_owned()),
                ("manual".to_owned(), "Only in this database".to_owned()),
            ],
            "Any rule source",
            query.selected("source"),
        ),
        search: query.selected("q").unwrap_or_default().to_owned(),
        filters_applied: query.any_applied(),
        clear_url: BASE_URL,
        capped,
    }
}
