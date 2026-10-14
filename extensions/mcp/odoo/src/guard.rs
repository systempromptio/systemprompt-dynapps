//! The tool-layer guardrail on destructive and erasure tools.
//!
//! Governance is the enforced gate on who may call a destructive tool at all;
//! this is the second gate, on *which records* such a call may touch and what
//! it must carry. Three rules, applied by every guarded handler before it
//! writes:
//!
//! - a non-empty `reason` ([`require_reason`]);
//! - while demo scope is on ([`demo_scope_enabled`], `ODOO_DEMO_SCOPE`, default
//!   on), the record must carry a tag or category named `demo-*`
//!   ([`check_demo_scope`]) — so a demo agent cannot reach real customer data
//!   whatever its prompt says;
//! - the record is read first, and the result says plainly that the change
//!   cannot be undone ([`irreversible_notice`]).

use rmcp::ErrorData as McpError;

pub const DEMO_SCOPE_ENV: &str = "ODOO_DEMO_SCOPE";
pub const DEMO_TAG_PREFIX: &str = "demo-";

#[must_use]
pub fn parse_demo_scope(value: Option<&str>) -> bool {
    !value.is_some_and(|v| {
        matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "false" | "0" | "no" | "off"
        )
    })
}

#[must_use]
pub fn demo_scope_enabled() -> bool {
    parse_demo_scope(std::env::var(DEMO_SCOPE_ENV).ok().as_deref())
}

pub fn require_reason(tool: &str, reason: &str) -> Result<String, McpError> {
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(McpError::invalid_params(
            format!("{tool} is irreversible and needs a non-empty `reason` saying why."),
            None,
        ));
    }
    Ok(reason.to_owned())
}

#[must_use]
pub fn carries_demo_tag(tags: &[String]) -> bool {
    tags.iter()
        .any(|tag| tag.trim().to_ascii_lowercase().starts_with(DEMO_TAG_PREFIX))
}

/// The record this guard is judging, named the way the refusal reports it.
#[derive(Debug, Clone, Copy)]
pub struct Target<'a> {
    pub label: &'a str,
    pub id: i64,
    pub tag_kind: &'a str,
}

pub fn check_demo_scope(
    enabled: bool,
    target: Target<'_>,
    tags: &[String],
) -> Result<(), McpError> {
    if !enabled || carries_demo_tag(tags) {
        return Ok(());
    }
    let held = if tags.is_empty() {
        "none".to_owned()
    } else {
        tags.join(", ")
    };
    Err(McpError::invalid_request(
        format!(
            "Refused: {} {} carries no `{DEMO_TAG_PREFIX}*` {} (it has: {held}). Demo scope \
             ({DEMO_SCOPE_ENV}) is on, so destructive and erasure tools only touch demo records.",
            target.label, target.id, target.tag_kind
        ),
        None,
    ))
}

#[must_use]
pub fn irreversible_notice(action: &str, reason: &str) -> String {
    format!("IRREVERSIBLE: {action}. It cannot be undone. Reason recorded: \"{reason}\".")
}
