//! `services/evaluation/config.yaml` as shipped, and the loader's refusals.
//!
//! The shipped file must resolve every criterion against the shipped provider
//! catalog, or the boot that validates it would fail on the next deploy. The
//! refusals are the ones an editor is likely to trip: a misspelled key, a
//! missing criterion, thresholds on a judged criterion, a model the catalog
//! does not list.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use std::path::Path;

use systemprompt::models::services::{AiConfig, ProviderRegistry};
use systemprompt_web_admin::repositories::config::evaluation::{
    CriterionName, Scoring, load_evaluation,
};

use crate::support::repo_root;

fn yaml_node(path: &Path, key: &str) -> serde_yaml::Value {
    let yaml =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let root: serde_yaml::Value =
        serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));
    root.get(key)
        .unwrap_or_else(|| panic!("{} has no {key}: key", path.display()))
        .clone()
}

fn registry() -> ProviderRegistry {
    serde_yaml::from_value(yaml_node(
        &repo_root().join("services/ai/providers.yaml"),
        "providers",
    ))
    .expect("providers catalog parses")
}

fn ai_config() -> AiConfig {
    serde_yaml::from_value(yaml_node(
        &repo_root().join("services/ai/config.yaml"),
        "ai",
    ))
    .expect("ai config parses")
}

const SHIPPED: &str = include_str!("../../../../services/evaluation/config.yaml");

// A copy of the shipped tree with the YAML replaced, so a refusal can be
// provoked without touching the real file.
fn tree_with(yaml: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("evaluation");
    std::fs::create_dir_all(root.join("criteria")).expect("mkdir");
    for name in ["intent", "quality", "speed", "cost"] {
        std::fs::copy(
            repo_root().join(format!("services/evaluation/criteria/{name}.md")),
            root.join(format!("criteria/{name}.md")),
        )
        .expect("copy criterion");
    }
    std::fs::copy(
        repo_root().join("services/evaluation/report.md"),
        root.join("report.md"),
    )
    .expect("copy report instructions");
    std::fs::write(root.join("config.yaml"), yaml).expect("write yaml");
    dir
}

fn load(yaml: &str) -> Result<String, String> {
    let dir = tree_with(yaml);
    load_evaluation(dir.path(), &registry(), &ai_config())
        .map(|loaded| loaded.expect("file present").config_hash)
        .map_err(|e| e.to_string())
}

#[test]
fn shipped_config_resolves_against_the_shipped_catalog() {
    let loaded = load_evaluation(&repo_root().join("services"), &registry(), &ai_config())
        .expect("shipped evaluation config loads")
        .expect("shipped evaluation config exists");
    assert!(loaded.enabled);
    assert_eq!(loaded.criteria.len(), 4);
    for name in CriterionName::ALL {
        let criterion = loaded
            .criterion(name)
            .unwrap_or_else(|| panic!("criterion {name} declared"));
        assert!(
            !criterion.instructions_markdown.trim().is_empty(),
            "{name} has instructions"
        );
        let expected = if matches!(name, CriterionName::Speed | CriterionName::Cost) {
            Scoring::Metric
        } else {
            Scoring::Judge
        };
        assert_eq!(criterion.scoring, expected, "{name} scoring mode");
        assert_eq!(criterion.metric.is_some(), expected == Scoring::Metric);
    }
    assert_eq!(loaded.config_hash.len(), 64);
    assert!(!loaded.report.instructions_markdown.trim().is_empty());
    assert!(loaded.report.window_days >= 1);
}

#[test]
fn report_thresholds_are_validated() {
    let yaml = SHIPPED.replace("    min_new_runs: 20\n", "    min_new_runs: 0\n");
    let err = load(&yaml).expect_err("zero threshold refused");
    assert!(err.contains("report.min_new_runs"), "{err}");
}

#[test]
fn absent_file_is_not_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let loaded = load_evaluation(dir.path(), &registry(), &ai_config()).expect("loads");
    assert!(loaded.is_none());
}

#[test]
fn hash_is_stable_for_identical_input() {
    assert_eq!(load(SHIPPED), load(SHIPPED));
}

#[test]
fn hash_changes_with_the_instructions() {
    let dir = tree_with(SHIPPED);
    let before = load_evaluation(dir.path(), &registry(), &ai_config())
        .expect("loads")
        .expect("present")
        .config_hash;
    std::fs::write(
        dir.path().join("evaluation/criteria/quality.md"),
        "# Quality\n\nChanged.\n",
    )
    .expect("rewrite");
    let after = load_evaluation(dir.path(), &registry(), &ai_config())
        .expect("loads")
        .expect("present")
        .config_hash;
    assert_ne!(before, after);
}

#[test]
fn unknown_key_is_rejected() {
    let yaml = SHIPPED.replace("  enabled: true\n", "  enabled: true\n  enabld: false\n");
    let err = load(&yaml).expect_err("unknown key refused");
    assert!(err.contains("enabld"), "{err}");
}

#[test]
fn missing_criterion_is_rejected() {
    let start = SHIPPED.find("    - name: cost").expect("cost block");
    let yaml = &SHIPPED[..start];
    let err = load(yaml).expect_err("missing cost refused");
    assert!(err.contains("criterion 'cost'"), "{err}");
}

#[test]
fn thresholds_on_a_judged_criterion_are_rejected() {
    let yaml = SHIPPED.replace(
        "      inputs: [user_messages, last_assistant_message, skills, session_metadata]\n",
        "      inputs: [user_messages, last_assistant_message, skills, session_metadata]\n      metric:\n        target_minutes_per_prompt: 1.0\n        target_latency_ms: 1\n",
    );
    let err = load(&yaml).expect_err("metric on intent refused");
    assert!(err.contains("'intent'"), "{err}");
}

#[test]
fn unlisted_model_is_rejected() {
    let yaml = SHIPPED.replace("model: claude-sonnet-5", "model: claude-nonesuch-9");
    let err = load(&yaml).expect_err("unlisted model refused");
    assert!(err.contains("claude-nonesuch-9"), "{err}");
}

#[test]
fn unknown_provider_is_rejected() {
    let yaml = SHIPPED.replace("provider: anthropic", "provider: nonesuch");
    let err = load(&yaml).expect_err("unknown provider refused");
    assert!(err.contains("nonesuch"), "{err}");
}
