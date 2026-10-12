//! The eval assertion evaluators against a temp workspace and synthetic
//! run events: one pass and one fail per assertion type.

use std::path::Path;

use chrono::Utc;
use serde_json::json;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::repositories::rows::{RunEventRow, RunRow};
use systemprompt_web_admin_farm::services::evals::{
    AssertionContext, AssertionResult, evaluate_assertions,
};
use systemprompt_web_admin_farm::services::farm_sync::Assertion;
use systemprompt_web_admin_farm::types::{FarmAgentId, FarmId, FarmRunId, RunEventPayload};

fn run_row(turns: Option<i32>, cost: Option<i64>) -> RunRow {
    let now = Utc::now();
    RunRow {
        id: FarmRunId::new("run-1"),
        farm_id: FarmId::new("farm"),
        agent_id: FarmAgentId::new("agent"),
        user_id: UserId::new("user-1"),
        triggered_via: "eval".to_owned(),
        status: "succeeded".to_owned(),
        prompt: "review".to_owned(),
        fixture: None,
        model: None,
        permission_mode: "acceptEdits".to_owned(),
        max_turns: 40,
        max_budget_usd: 5.0,
        timeout_seconds: 900,
        source_commit_sha: None,
        run_dir: None,
        gateway_session_id: None,
        claude_session_id: None,
        context_id: None,
        api_key_id: None,
        pid: None,
        started_at: Some(now),
        finished_at: Some(now),
        duration_ms: Some(1),
        num_turns: turns,
        input_tokens: None,
        output_tokens: None,
        cost_microdollars: cost,
        result_subtype: None,
        exit_code: Some(0),
        error: None,
        diff_summary: None,
        eval_result_id: None,
        cancel_requested_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn event(seq: i64, payload: &RunEventPayload) -> RunEventRow {
    RunEventRow {
        run_id: FarmRunId::new("run-1"),
        seq,
        kind: payload.kind().as_str().to_owned(),
        tool_name: None,
        tool_use_id: None,
        payload: serde_json::to_value(payload).expect("payload serializes"),
        created_at: Utc::now(),
    }
}

fn tool(seq: i64, name: &str, input: serde_json::Value) -> RunEventRow {
    event(
        seq,
        &RunEventPayload::ToolUse {
            tool_use_id: format!("t{seq}"),
            name: name.to_owned(),
            input,
        },
    )
}

struct Case<'a> {
    run: RunRow,
    events: Vec<RunEventRow>,
    workspace: &'a Path,
    tools: Vec<String>,
    output_path: Option<&'a str>,
    final_text: Option<&'a str>,
}

impl<'a> Case<'a> {
    fn new(workspace: &'a Path) -> Self {
        Self {
            run: run_row(Some(5), Some(1_500_000)),
            events: Vec::new(),
            workspace,
            tools: Vec::new(),
            output_path: None,
            final_text: None,
        }
    }

    fn check(&self, assertion: Assertion) -> AssertionResult {
        let ctx = AssertionContext {
            run: &self.run,
            events: &self.events,
            workspace: self.workspace,
            agent_tools: &self.tools,
            output_path: self.output_path,
            final_text: self.final_text,
        };
        evaluate_assertions(&[assertion], &ctx).remove(0)
    }
}

fn workspace() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(
        dir.path().join("REVIEW.md"),
        "Found sudo( in onchange at line 12\n",
    )
    .expect("write review");
    dir
}

#[test]
fn file_exists_passes_fails_and_rejects_escape() {
    let dir = workspace();
    let case = Case::new(dir.path());
    let ok = case.check(Assertion::FileExists {
        path: "REVIEW.md".into(),
    });
    assert!(ok.passed, "{ok:?}");
    assert_eq!(ok.kind, "file_exists");
    assert!(
        !case
            .check(Assertion::FileExists {
                path: "PLAN.md".into()
            })
            .passed
    );
    let escape = case.check(Assertion::FileExists {
        path: "../REVIEW.md".into(),
    });
    assert!(!escape.passed);
    assert!(escape.detail.contains("escapes"), "{escape:?}");
}

#[test]
fn output_contains_reads_path_then_output_then_final_text() {
    let dir = workspace();
    let mut case = Case::new(dir.path());
    let explicit = Assertion::OutputContains {
        path: Some("REVIEW.md".into()),
        text: "sudo(".into(),
    };
    assert!(case.check(explicit).passed);
    let missing = Assertion::OutputContains {
        path: Some("REVIEW.md".into()),
        text: "N+1".into(),
    };
    assert!(!case.check(missing).passed);

    let implicit = || Assertion::OutputContains {
        path: None,
        text: "sudo(".into(),
    };
    assert!(
        !case.check(implicit()).passed,
        "no output path and no final text"
    );
    case.final_text = Some("done: sudo( flagged");
    assert!(case.check(implicit()).passed, "falls back to final text");
    case.output_path = Some("REVIEW.md");
    case.final_text = Some("nothing here");
    assert!(
        case.check(implicit()).passed,
        "agent output path wins over final text"
    );
}

#[test]
fn regex_matches_and_reports_bad_patterns() {
    let dir = workspace();
    let case = Case::new(dir.path());
    let hit = Assertion::Regex {
        path: Some("REVIEW.md".into()),
        pattern: r"line \d+".into(),
    };
    assert!(case.check(hit).passed);
    let miss = Assertion::Regex {
        path: Some("REVIEW.md".into()),
        pattern: r"col \d+".into(),
    };
    assert!(!case.check(miss).passed);
    let bad = case.check(Assertion::Regex {
        path: Some("REVIEW.md".into()),
        pattern: "(".into(),
    });
    assert!(!bad.passed);
    assert!(bad.detail.contains("invalid pattern"), "{bad:?}");
}

#[test]
fn no_tool_outside_allowlist_allows_skill_and_listed_mcp() {
    let dir = workspace();
    let mut case = Case::new(dir.path());
    case.tools = vec!["Read".into(), "Bash(git:*)".into(), "mcp__odoo".into()];
    case.events = vec![
        tool(1, "Read", json!({})),
        tool(2, "Bash", json!({})),
        tool(3, "Skill", json!({"skill": "x"})),
        tool(4, "mcp__odoo__search", json!({})),
    ];
    assert!(case.check(Assertion::NoToolOutsideAllowlist {}).passed);
    case.events.push(tool(5, "Write", json!({})));
    case.events.push(tool(6, "mcp__github__issue", json!({})));
    let outside = case.check(Assertion::NoToolOutsideAllowlist {});
    assert!(!outside.passed);
    assert!(outside.detail.contains("Write") && outside.detail.contains("mcp__github__issue"));
}

#[test]
fn max_turns_and_max_cost_compare_against_the_run_row() {
    let dir = workspace();
    let mut case = Case::new(dir.path());
    assert!(case.check(Assertion::MaxTurns { value: 5 }).passed);
    assert!(!case.check(Assertion::MaxTurns { value: 4 }).passed);
    assert!(case.check(Assertion::MaxCostUsd { value: 2.0 }).passed);
    assert!(!case.check(Assertion::MaxCostUsd { value: 1.0 }).passed);
    case.run = run_row(None, None);
    assert!(!case.check(Assertion::MaxTurns { value: 40 }).passed);
    assert!(!case.check(Assertion::MaxCostUsd { value: 2.0 }).passed);
}

#[test]
fn skill_invoked_normalises_dashes_and_reads_hooks() {
    let dir = workspace();
    let mut case = Case::new(dir.path());
    let wanted = || Assertion::SkillInvoked {
        skill: "odoo_module_review".into(),
    };
    assert!(!case.check(wanted()).passed);
    case.events = vec![tool(
        1,
        "Skill",
        json!({"skill": "dynapps-odoo-delivery:odoo-module-review"}),
    )];
    assert!(case.check(wanted()).passed);
    case.events = vec![tool(1, "Read", json!({"skill": "odoo-module-review"}))];
    assert!(!case.check(wanted()).passed, "only the Skill tool counts");
    case.events = vec![event(
        1,
        &RunEventPayload::Hook {
            name: "PostToolUse:odoo-module-review".into(),
            decision: None,
        },
    )];
    assert!(case.check(wanted()).passed);
}
