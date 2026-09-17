//! The judge prompts: the system prompt carries the criterion's own
//! instructions after a fixed preamble, and the user prompt lays the input
//! blocks out in the order the criterion declared them.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_web_admin::evaluation::prompt::{
    JUDGE_PREAMBLE, build_system_prompt, build_user_prompt,
};
use systemprompt_web_admin::evaluation::{JudgeOutput, judge_output_schema};
use systemprompt_web_admin::repositories::config::evaluation::{
    CriterionName, InputSlice, LoadedCriterion, MetricThresholds, Scoring,
};
use systemprompt_web_admin::repositories::evaluation::inputs::{AssembledInputs, InputBlock};

fn criterion(name: CriterionName, scoring: Scoring) -> LoadedCriterion {
    LoadedCriterion {
        name,
        scoring,
        provider: "anthropic".to_owned(),
        model: "claude-sonnet-5".to_owned(),
        max_output_tokens: 1024,
        instructions_path: "criteria/x.md".to_owned(),
        instructions_markdown: "# Rubric\n\nScore it.\n".to_owned(),
        inputs: vec![InputSlice::UserMessages, InputSlice::Metrics],
        metric: (scoring == Scoring::Metric).then_some(MetricThresholds::Speed {
            target_minutes_per_prompt: 4.0,
            target_latency_ms: 20_000,
        }),
    }
}

#[test]
fn system_prompt_is_preamble_then_rubric() {
    let prompt = build_system_prompt(&criterion(CriterionName::Quality, Scoring::Judge));
    assert!(prompt.starts_with(JUDGE_PREAMBLE));
    assert!(prompt.contains("Criterion: quality"));
    assert!(prompt.contains("Return a `score` from 0 to 100"));
    assert!(prompt.ends_with("# Rubric\n\nScore it."));
}

#[test]
fn metric_criteria_are_told_not_to_score() {
    let prompt = build_system_prompt(&criterion(CriterionName::Speed, Scoring::Metric));
    assert!(prompt.contains("Do NOT return a score"));
}

#[test]
fn user_prompt_follows_the_declared_input_order() {
    let inputs = AssembledInputs {
        blocks: vec![
            InputBlock {
                slice: InputSlice::Metrics,
                label: "METRICS (json)",
                body: "{}".to_owned(),
            },
            InputBlock {
                slice: InputSlice::UserMessages,
                label: "USER MESSAGES (chronological)",
                body: "1. \"hi\"".to_owned(),
            },
        ],
        metrics: None,
    };
    let prompt = build_user_prompt(&inputs, None);
    let metrics_at = prompt.find("### METRICS").expect("metrics block");
    let messages_at = prompt.find("### USER MESSAGES").expect("messages block");
    assert!(metrics_at < messages_at);
    assert!(!prompt.contains("### SCORE"));
}

#[test]
fn fixed_score_is_appended_last() {
    let inputs = AssembledInputs::default();
    let prompt = build_user_prompt(&inputs, Some(72));
    assert!(prompt.ends_with("The score is fixed at 72 by policy. Explain it; do not change it."));
}

#[test]
fn judged_schema_requires_a_score_and_metric_schema_forbids_one() {
    let judged = judge_output_schema(CriterionName::Quality, Scoring::Judge);
    let required = judged["required"].as_array().expect("required");
    assert!(required.iter().any(|v| v == "score"));
    let metric = judge_output_schema(CriterionName::Speed, Scoring::Metric);
    assert!(metric["properties"].get("score").is_none());
    let intent = judge_output_schema(CriterionName::Intent, Scoring::Judge);
    assert!(intent["properties"].get("title").is_some());
}

#[test]
fn judge_output_parses_with_optional_fields_absent() {
    let out: JudgeOutput =
        serde_json::from_str(r#"{"rationale":"fine","evidence":[]}"#).expect("parses");
    assert!(out.score.is_none());
    assert!(out.title.is_none());
}
