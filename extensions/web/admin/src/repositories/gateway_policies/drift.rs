//! Declared policies against the rows: one difference per policy, and
//! which fields disagree.
//!
//! Comparison is on the normalised spec (monthly windows folded to their
//! sentinel), so the daily rewrite is invisible here; a console edit to a
//! ceiling or a scanner list is not. A row the file does not name is an
//! orphan whatever wrote it — the console is the only other writer, and
//! *Overwrite from code* deletes it exactly as the boot seed would.

use serde::Serialize;
use systemprompt::ai::{GatewayPolicyEntry, GatewayPolicySpec};

use super::declared::DeclaredPolicies;
use super::month_window::normalise_spec;
use super::rows::PolicyRow;

#[derive(Debug, Clone, Serialize)]
pub struct ChangedPolicy {
    pub name: String,
    pub fields: Vec<String>,
    pub in_code: String,
    pub in_db: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct PolicyDrift {
    pub missing_in_db: Vec<String>,
    pub only_in_db: Vec<String>,
    pub changed: Vec<ChangedPolicy>,
}

impl PolicyDrift {
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.missing_in_db.is_empty() && self.only_in_db.is_empty() && self.changed.is_empty()
    }
}

fn windows_summary(spec: &GatewayPolicySpec) -> String {
    if spec.quota_windows.is_empty() {
        return "no windows".to_owned();
    }
    spec.quota_windows
        .iter()
        .map(|w| {
            let mut parts = vec![format!("{}/{}s", w.subject, w.window_seconds)];
            if let Some(n) = w.max_requests {
                parts.push(format!("{n} req"));
            }
            if let Some(n) = w.max_cost_microdollars {
                parts.push(format!("${:.2}", n as f64 / 1_000_000.0));
            }
            parts.join(" ")
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[must_use]
pub fn summarise_policy(entry_enabled: bool, priority: i32, spec: &GatewayPolicySpec) -> String {
    format!(
        "{} · p{priority} · quota {:?} · {} · safety {:?} [{}] block {:?}/{:?}",
        if entry_enabled { "on" } else { "off" },
        spec.quota_mode,
        windows_summary(spec),
        spec.safety.mode,
        spec.safety.scanners.join(","),
        spec.safety.block_categories,
        spec.safety.block_response_categories,
    )
    .to_lowercase()
}

// Why: field-level so the row names what moved — "quota_windows" against
// "safety.block_categories" is the difference between a spend and a scanner
// decision, and the operator should not have to diff two JSON blobs.
fn changed_fields(declared: &GatewayPolicyEntry, row: &PolicyRow) -> Vec<String> {
    let mut out = Vec::new();
    if declared.enabled != row.enabled {
        out.push("enabled".to_owned());
    }
    if declared.priority != row.priority {
        out.push("priority".to_owned());
    }
    let db = normalise_spec(&row.spec);
    let d = &declared.spec;
    if d.quota_mode != db.quota_mode {
        out.push("quota_mode".to_owned());
    }
    if json(&d.quota_windows) != json(&db.quota_windows) {
        out.push("quota_windows".to_owned());
    }
    if d.safety.mode != db.safety.mode {
        out.push("safety.mode".to_owned());
    }
    if d.safety.scanners != db.safety.scanners {
        out.push("safety.scanners".to_owned());
    }
    if d.safety.block_categories != db.safety.block_categories {
        out.push("safety.block_categories".to_owned());
    }
    if d.safety.block_response_categories != db.safety.block_response_categories {
        out.push("safety.block_response_categories".to_owned());
    }
    if d.safety.history != db.safety.history {
        out.push("safety.history".to_owned());
    }
    if json(&d.safety.heuristic) != json(&db.safety.heuristic) {
        out.push("safety.heuristic".to_owned());
    }
    out
}

// JSON: a comparison key only — two spec sections are equal when their
// serialised forms are, and the value never leaves this function
fn json<T: Serialize>(v: &T) -> serde_json::Value {
    // Why: discard-ok: the spec types serialise infallibly; Null on both
    // sides compares equal, which is the harmless direction
    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
}

#[must_use]
pub fn compute_policy_drift(declared: &DeclaredPolicies, rows: &[PolicyRow]) -> PolicyDrift {
    let mut drift = PolicyDrift::default();
    for entry in &declared.entries {
        match rows.iter().find(|r| r.name == entry.name) {
            None => drift.missing_in_db.push(entry.name.clone()),
            Some(row) => {
                let fields = changed_fields(entry, row);
                if !fields.is_empty() {
                    drift.changed.push(ChangedPolicy {
                        name: entry.name.clone(),
                        fields,
                        in_code: summarise_policy(entry.enabled, entry.priority, &entry.spec),
                        in_db: summarise_policy(
                            row.enabled,
                            row.priority,
                            &normalise_spec(&row.spec),
                        ),
                    });
                }
            },
        }
    }
    for row in rows {
        if declared.find(&row.name).is_none() {
            drift.only_in_db.push(row.name.clone());
        }
    }
    drift
}
