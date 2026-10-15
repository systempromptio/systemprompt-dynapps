//! The eval assertion evaluators against a temp workspace and synthetic
//! run events: one pass and one fail per assertion type.

use std::path::Path;

use chrono::Utc;
use serde_json::json;
use systemprompt::agent_runs::AgentRunRow;
use systemprompt::identifiers::{AgentRunId, UserId};
use systemprompt_web_admin_farm::services::evals::{
    AssertionContext, AssertionResult, evaluate_assertions,
};
use systemprompt_web_admin_farm::services::farm_sync::Assertion;
use systemprompt_web_admin_farm::types::{HookDecision, RunEvent, RunEventPayload};

fn agent_run_row(turns: Option<i32>, cost: Option<i64>) -> AgentRunRow {
    let now = Utc::now();
    AgentRunRow {
        id: AgentRunId::new("agent-run-1"),
        origin_kind: "agent_farm_run".to_owned(),
        origin_id: "run-1".to_owned(),
        user_id: UserId::new("user-1"),
        host: "claude-code".to_owned(),
        status: "succeeded".to_owned(),
        request: json!({}),
        attempt: 1,
        not_before: None,
        claimed_by: None,
        lease_until: None,
        claimed_at: None,
        run_dir: None,
        gateway_session_id: None,
        api_key_id: None,
        host_session_id: None,
        pid: None,
        started_at: Some(now),
        finished_at: Some(now),
        duration_ms: Some(1),
        error_class: Some("ok".to_owned()),
        turns,
        input_tokens: None,
        output_tokens: None,
        cost_microdollars: cost,
        structured_output: None,
        exit_code: Some(0),
        error: None,
        diff_summary: None,
        cancel_requested_at: None,
        task_id: None,
        context_id: None,
        created_at: now,
        updated_at: now,
    }
}

fn event(seq: i64, payload: RunEventPayload) -> RunEvent {
    RunEvent {
        seq,
        kind: payload.kind(),
        tool_name: None,
        tool_use_id: payload.tool_use_id().map(str::to_owned),
        payload,
        created_at: Utc::now(),
    }
}

fn tool(seq: i64, name: &str, input: serde_json::Value) -> RunEvent {
    event(
        seq,
        RunEventPayload::ToolUse {
            tool_use_id: format!("t{seq}"),
            name: name.to_owned(),
            input,
        },
    )
}

struct Case<'a> {
    run: AgentRunRow,
    events: Vec<RunEvent>,
    workspace: &'a Path,
    tools: Vec<String>,
    output_path: Option<&'a str>,
    final_text: Option<&'a str>,
}

impl<'a> Case<'a> {
    fn new(workspace: &'a Path) -> Self {
        Self {
            run: agent_run_row(Some(5), Some(1_500_000)),
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
    case.tools = vec!["Read".into(), "Bash(git:*)".into(), "mcp__dynapps-odoo".into()];
    case.events = vec![
        tool(1, "Read", json!({})),
        tool(2, "Bash", json!({})),
        tool(3, "Skill", json!({"skill": "x"})),
        tool(4, "mcp__dynapps-odoo__search", json!({})),
    ];
    assert!(case.check(Assertion::NoToolOutsideAllowlist {}).passed);
    case.events.push(tool(5, "Write", json!({})));
    case.events.push(tool(6, "mcp__github__issue", json!({})));
    let outside = case.check(Assertion::NoToolOutsideAllowlist {});
    assert!(!outside.passed);
    assert!(outside.detail.contains("Write") && outside.detail.contains("mcp__github__issue"));
}

#[test]
fn max_turns_and_max_cost_compare_against_the_recorded_outcome() {
    let dir = workspace();
    let mut case = Case::new(dir.path());
    assert!(case.check(Assertion::MaxTurns { value: 5 }).passed);
    assert!(!case.check(Assertion::MaxTurns { value: 4 }).passed);
    assert!(case.check(Assertion::MaxCostUsd { value: 2.0 }).passed);
    assert!(!case.check(Assertion::MaxCostUsd { value: 1.0 }).passed);
    case.run = agent_run_row(None, None);
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
        RunEventPayload::Hook {
            name: "PostToolUse:odoo-module-review".into(),
            decision: HookDecision::Allow,
            policy: None,
        },
    )];
    assert!(case.check(wanted()).passed);
}
