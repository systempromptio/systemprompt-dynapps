//! How one marketplace row of a person's access matrix reads on the Access
//! tab: whether it grants access, the status label, the reason behind it and
//! the badge tone.
//!
//! The status labels are compared by the templates, so they are page contract.

use systemprompt_web_admin_core::repositories::entitlement::MatrixRow;

const NOT_ASSIGNED: &str = "Not assigned";

pub(super) fn grants_access(row: &MatrixRow) -> bool {
    matches!(row.effective.as_str(), "allow" | "warn")
}

pub(super) fn workspace_status(row: &MatrixRow) -> &'static str {
    match (row.effective.as_str(), row.source.layer.as_str()) {
        ("allow", _) => "Allowed",
        ("deny", "user" | "role" | "group" | "project") => "Explicitly denied",
        ("deny", "default") if row.source.detail.contains("not assigned") => NOT_ASSIGNED,
        ("deny", _) => "Access denied",
        ("warn", _) => "Allowed with warning",
        _ => "Approval pending",
    }
}

fn rule_name(detail: &str) -> &str {
    detail
        .strip_suffix(" allow")
        .or_else(|| detail.strip_suffix(" deny"))
        .unwrap_or(detail)
}

pub(super) fn workspace_reason(row: &MatrixRow, allowed: bool, status: &str) -> String {
    match row.source.layer.as_str() {
        "user" => "A rule for this person".to_owned(),
        "default" if allowed => "Included by default".to_owned(),
        "default" if status == NOT_ASSIGNED => "No matching grant for this workspace".to_owned(),
        "group" | "role" | "project" => format!(
            "{} through {}",
            if allowed { "Allowed" } else { "Denied" },
            rule_name(&row.source.detail).replace(':', " ")
        ),
        _ => row.source.detail.clone(),
    }
}

pub(super) fn workspace_tone(row: &MatrixRow, allowed: bool, status: &str) -> &'static str {
    if row.effective == "warn" {
        "warn"
    } else if allowed {
        "ok"
    } else if status == NOT_ASSIGNED {
        "muted"
    } else {
        "warn"
    }
}
