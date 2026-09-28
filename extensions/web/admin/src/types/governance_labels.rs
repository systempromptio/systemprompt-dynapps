//! How a governance decision names itself: which plane a policy belongs to, and
//! what a row was actually about.
//!
//! Both were derivations inside the governance page's view module, where the
//! tests workspace could not reach them — and both had failed silently for
//! months precisely because nothing could assert on them. They are pure string
//! functions over values the database hands back, so they live here, beside the
//! other display types, and the unit suite pins them.
//!
//! An unrecognised policy names itself rather than rendering a dash, so a new
//! producer is visible. `tool_name` is overloaded — the gateway's `user_prompt`
//! sentinel, an MCP tool, or an authz entity id whose kind is in
//! `evaluated_rules` — so the target label says which.

#[must_use]
pub fn plane_of(policy: &str) -> String {
    let known = match policy {
        "agent_scope" => "chain · 1 scope",
        "secret_scan" => "chain · 2 secret",
        "tool_blocklist" => "chain · 3 blocklist",
        "rate_limit" => "chain · 4 rate",
        "default_allow" => "chain · pass",
        "governance_allow" => "chain · allow rule",
        "governance_disabled" => "chain · off",
        "quota" => "gateway · quota",
        "authentication" => "gateway · authentication",
        "authz" => "authz · extension",
        "authz_extension_hook" => "authz · hook",
        "authz_rule_based" => "authz · rules",
        "authz_default_deny" => "authz · default deny",
        "authz_unrestricted" => "authz · unrestricted",
        "authz_hook_fault" => "authz · fault",
        other => return other.to_owned(),
    };
    known.to_owned()
}

#[must_use]
pub fn target_label(tool_name: &str, entity_type: Option<&str>) -> String {
    if let Some(kind) = entity_type.filter(|k| !k.is_empty()) {
        return format!("{kind} {tool_name}");
    }
    if tool_name == "user_prompt" {
        return "prompt".to_owned();
    }
    if tool_name.is_empty() {
        return "\u{2014}".to_owned();
    }
    format!("tool {tool_name}")
}
