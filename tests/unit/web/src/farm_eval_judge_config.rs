//! The `evals:` block of agent-farms.yaml: the judge's provider and model
//! default to Anthropic's Sonnet, can be overridden, may not be empty, and
//! an unknown key is a load error. Every trigger builds its judge from it,
//! and without inference there is no judge at all.

use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::config::{
    DEFAULT_JUDGE_MODEL, DEFAULT_JUDGE_PROVIDER, EvalsConfig, FARMS_FILE, parse_farms_doc,
};
use systemprompt_web_admin_farm::services::evals::gateway_judge;
use systemprompt_web_admin_farm::state::{JudgeSource, read_evals_config};

const FARM: &str = "farms:
  - id: triage
    name: Triage
    marketplace: dynapps
    source:
      github: dynapps/agent-farm
";

#[test]
fn no_evals_block_means_the_default_judge() {
    let doc = parse_farms_doc(FARM).expect("parses");
    doc.validate().expect("valid");
    assert_eq!(doc.evals, None);
    let config = EvalsConfig::default();
    assert_eq!(config.judge_provider, DEFAULT_JUDGE_PROVIDER);
    assert_eq!(config.judge_model, DEFAULT_JUDGE_MODEL);
    assert_eq!(DEFAULT_JUDGE_MODEL, "anthropic-claude-sonnet-5-5");
}

#[test]
fn evals_block_names_the_judge_model() {
    let text = format!("{FARM}evals:\n  judge_provider: gemini\n  judge_model: gemini-3.8-flash\n");
    let doc = parse_farms_doc(&text).expect("parses");
    doc.validate().expect("valid");
    let evals = doc.evals.expect("evals block");
    assert_eq!(evals.judge_provider, "gemini");
    assert_eq!(evals.judge_model, "gemini-3.8-flash");
}

#[test]
fn a_partial_evals_block_keeps_the_other_default() {
    let text = format!("{FARM}evals:\n  judge_model: claude-opus-5-5\n");
    let evals = parse_farms_doc(&text)
        .expect("parses")
        .evals
        .expect("evals");
    assert_eq!(evals.judge_provider, DEFAULT_JUDGE_PROVIDER);
    assert_eq!(evals.judge_model, "claude-opus-5-5");
}

#[test]
fn an_empty_judge_model_is_refused() {
    let text = format!("{FARM}evals:\n  judge_model: \"  \"\n");
    let doc = parse_farms_doc(&text).expect("parses");
    let err = doc.validate().expect_err("empty judge model must fail");
    assert!(err.to_string().contains("judge_model"), "{err}");
}

#[test]
fn an_unknown_evals_key_is_a_load_error() {
    let text = format!("{FARM}evals:\n  judge: claude-sonnet-5-5\n");
    assert!(parse_farms_doc(&text).is_err());
}

#[test]
fn the_shipped_farms_file_judges_with_sonnet() {
    let root = crate::support::repo_root().join("services");
    let config = read_evals_config(&root);
    assert_eq!(config.judge_provider, "anthropic");
    assert_eq!(config.judge_model, "claude-sonnet-5-5");
}

#[test]
fn a_missing_farms_file_reads_as_the_defaults() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(!dir.path().join(FARMS_FILE).exists());
    assert_eq!(read_evals_config(dir.path()), EvalsConfig::default());
}

#[test]
fn without_inference_there_is_no_judge() {
    let user = UserId::new("judge-test-user");
    assert!(gateway_judge(None, &user, &EvalsConfig::default()).is_none());
    assert!(JudgeSource::default().judge_for(None, &user).is_none());
}
