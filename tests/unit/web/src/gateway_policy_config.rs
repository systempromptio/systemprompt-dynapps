//! The deployed gateway policy, pinned. `services/gateway/policies.yaml` is
//! projected into `ai_gateway_policies` at every boot, so what it says is what
//! production enforces — and the difference between a category being *scanned*
//! and a category *blocking* is the difference between an audit row and a
//! customer's 400.
//!
//! On 2026-08-31 `jailbreak` and `pii_credit_card` blocked ordinary developer
//! traffic in production: core's Luhn check slid a 16-digit window over merged
//! digit runs, and its phrase list carried the literal "you are now". Both
//! detectors have since been narrowed, but they stay audit-only here. These
//! tests exist so that putting either back into `block_categories` is a
//! deliberate, review-visible act rather than a one-word edit.

use systemprompt::ai::{GatewayPolicyConfig, SafetyHistoryMode};

use crate::support::repo_root;

fn config() -> GatewayPolicyConfig {
    let path = repo_root().join("services/gateway/policies.yaml");
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()))
}

fn default_quotas() -> systemprompt::ai::GatewayPolicyEntry {
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

#[test]
fn ssn_is_the_only_request_side_block() {
    let safety = default_quotas().spec.safety;
    assert_eq!(safety.block_categories, vec!["pii_ssn".to_owned()]);
}

#[test]
fn secrets_and_ssns_still_block_on_the_way_out() {
    let safety = default_quotas().spec.safety;
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

// A test environment: the clamp keeps an accident cheap. It is a real ceiling,
// so it has to stay above the traffic a working day actually produces.
#[test]
fn the_daily_organization_clamp_is_two_dollars() {
    let window = default_quotas()
        .spec
        .quota_windows
        .into_iter()
        .find(|w| w.window_seconds == 86_400)
        .expect("a daily quota window");
    assert_eq!(window.max_cost_microdollars, Some(2_000_000));
}
