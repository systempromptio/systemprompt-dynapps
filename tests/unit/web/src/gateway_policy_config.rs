//! The deployed gateway policy, pinned. `services/gateway/policies.yaml` is
//! projected into `ai_gateway_policies` at every boot, so what it says is what
//! production enforces — and the difference between a category being *scanned*
//! and a category *blocking* is the difference between an audit row and a
//! customer's 400.
//!
//! `jailbreak` and `pii_credit_card` produce false positives on ordinary
//! developer traffic, so they are scanned and audited but never block, and
//! the scanners run in warn mode so the report shows what would have blocked.
//! These tests exist so that putting any category back into a live block list
//! is a deliberate, review-visible act rather than a one-word edit.

use systemprompt::gateway::{CalendarPeriod, GatewayPolicyConfig, SafetyHistoryMode};
use systemprompt::manifest::services::EnforcementMode;

use crate::support::repo_root;

fn config() -> GatewayPolicyConfig {
    let path = repo_root().join("services/gateway/policies.yaml");
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()))
}

fn default_quotas() -> systemprompt::gateway::GatewayPolicyEntry {
    config()
        .policies
        .into_iter()
        .find(|p| p.name == "default-quotas")
        .expect("the default-quotas policy")
}

#[test]
fn the_deployed_file_parses_and_validates() {
    let cfg = config();
    cfg.validate().expect("policies.yaml must validate");
    assert!(!cfg.policies.is_empty());
}

// Why: the block lists below are pinned unchanged while the plane is in warn
// mode, and that is deliberate. Under `mode: warn` they stop being the
// behaviour and become the hypothesis: every listed category still fires, is
// still persisted, and is recorded with `blocked = false`, which is what makes
// `infra logs governance report` able to say what enforcement would have cost.
// Emptying them to unblock traffic would have thrown that measurement away.
#[test]
fn the_scanners_are_in_warn_mode_and_block_nothing() {
    assert_eq!(default_quotas().spec.safety.mode, EnforcementMode::Warn);
}

// Why: enforced quotas refused active users with 429, so the windows only warn.
#[test]
fn the_quota_windows_are_warning_thresholds() {
    assert_eq!(default_quotas().spec.quota_mode, EnforcementMode::Warn);
}

#[test]
fn the_scanner_plane_and_governance_chain_stay_in_warn_mode() {
    let spec = default_quotas().spec;
    assert!(spec.safety.mode.is_warn(), "safety scanners");
    let governance = std::fs::read_to_string(repo_root().join("services/governance/config.yaml"))
        .expect("read the governance chain config");
    let doc: serde_yaml::Value = serde_yaml::from_str(&governance).expect("valid YAML");
    assert_eq!(
        doc["governance"]["mode"].as_str(),
        Some("warn"),
        "governance chain"
    );
}

#[test]
fn the_false_positive_prone_categories_never_block_a_request() {
    let safety = default_quotas().spec.safety;
    for category in ["jailbreak", "pii_credit_card"] {
        assert!(
            !safety.block_categories.contains(&category.to_owned()),
            "{category} denied real traffic in production — it is audit-only. \
             Re-adding it needs a narrower detector and a note in the file header."
        );
    }
}

// Nothing blocks in either direction. Scanning is untouched, so
// the audit trail is complete; under warn mode the lists below deny nothing and
// only decide what the warnings report counts as would-have-blocked.
#[test]
fn the_would_have_blocked_lists_carry_the_pre_warn_enforcement_set() {
    let safety = default_quotas().spec.safety;
    assert_eq!(safety.block_categories, vec!["pii_ssn".to_owned()]);
    assert_eq!(
        safety.block_response_categories,
        vec!["secret".to_owned(), "pii_ssn".to_owned()]
    );
}

// Every scanner still runs. Narrowing what blocks must never narrow what is
// scanned: `persist_findings` writes a finding to `ai_safety_findings`
// regardless of `block_categories`, so the audit trail is unaffected.
#[test]
fn all_three_scanners_are_still_enabled() {
    let safety = default_quotas().spec.safety;
    for scanner in ["heuristic", "secrets", "pii_extended"] {
        assert!(
            safety.scanners.contains(&scanner.to_owned()),
            "{scanner} must keep running — blocking is narrowed, auditing is not"
        );
    }
}

// The phrase list is pinned in YAML rather than inherited, so the live
// behaviour cannot change under us when core's builtin does.
#[test]
fn the_jailbreak_phrase_list_is_pinned_and_excludes_ordinary_english() {
    let safety = default_quotas().spec.safety;
    let phrases = safety
        .heuristic
        .phrases
        .expect("the phrase list is pinned explicitly, not inherited from core");
    assert!(!phrases.is_empty());
    assert!(
        !phrases.iter().any(|p| p == "you are now"),
        "'you are now' is ordinary English and appears in most agent system prompts"
    );
}

// Rescanning history re-walks the whole conversation every turn and re-persists
// findings already recorded when each turn was itself a request.
#[test]
fn request_history_is_not_rescanned() {
    assert_eq!(default_quotas().spec.safety.history, SafetyHistoryMode::Off);
}

// Ceilings sized for the pilot: about ten $0.50 farm runs an
// hour per account, $40 a day for the whole installation. Pinned so raising
// them is review-visible.
fn window(subject: &str, seconds: i32) -> Option<i64> {
    default_quotas()
        .spec
        .quota_windows
        .into_iter()
        .find(|w| w.subject == subject && w.calendar.is_none() && w.window_seconds == seconds)
        .and_then(|w| w.max_cost_microdollars)
}

fn calendar_window(subject: &str, period: CalendarPeriod) -> Option<i64> {
    default_quotas()
        .spec
        .quota_windows
        .into_iter()
        .find(|w| w.subject == subject && w.calendar == Some(period))
        .and_then(|w| w.max_cost_microdollars)
}

#[test]
fn the_cost_ceilings_are_pinned() {
    assert_eq!(window("user", 3_600), Some(5_000_000));
    assert_eq!(window("user", 86_400), Some(15_000_000));
    assert_eq!(window("organization", 86_400), Some(40_000_000));
    assert_eq!(
        calendar_window("organization", CalendarPeriod::Month),
        Some(150_000_000)
    );
}
