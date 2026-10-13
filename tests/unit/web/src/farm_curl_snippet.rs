//! The agent page's remote execution commands: the launch `curl` names the
//! public run endpoint on the external base, carries a valid JSON body that
//! survives shell single-quoting, and the poll command reads the events.

use systemprompt_web_admin_farm::types::{FarmAgentId, FarmId};
use systemprompt_web_admin_farm::views::curl::{launch_body, remote_commands};

#[test]
fn launch_command_posts_to_the_public_runs_endpoint() {
    let cmds = remote_commands(
        "https://gw.example.com/",
        &FarmId::new("acme"),
        &FarmAgentId::new("triage"),
        "Fix the bug",
    );
    assert!(
        cmds.launch
            .starts_with("curl -X POST https://gw.example.com/api/public/agent-farm/runs")
    );
    assert!(
        cmds.launch
            .contains("Authorization: Bearer $SYSTEMPROMPT_TOKEN")
    );
    assert!(cmds.launch.contains("Content-Type: application/json"));
    assert!(
        cmds.launch
            .contains(r#"-d '{"agent":"triage","farm":"acme","prompt":"Fix the bug"}'"#)
            || cmds
                .launch
                .contains(r#"-d '{"farm":"acme","agent":"triage","prompt":"Fix the bug"}'"#)
    );
}

#[test]
fn poll_command_reads_run_events() {
    let cmds = remote_commands(
        "https://gw.example.com",
        &FarmId::new("acme"),
        &FarmAgentId::new("triage"),
        "x",
    );
    assert!(
        cmds.poll
            .starts_with("curl https://gw.example.com/api/public/agent-farm/runs/$RUN_ID/events")
    );
    assert!(
        cmds.poll
            .contains("Authorization: Bearer $SYSTEMPROMPT_TOKEN")
    );
}

#[test]
fn body_is_json_with_the_three_fields() {
    let body = launch_body(
        &FarmId::new("acme"),
        &FarmAgentId::new("triage"),
        "line one\nline \"two\"",
    );
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("body is JSON");
    assert_eq!(parsed["farm"], "acme");
    assert_eq!(parsed["agent"], "triage");
    assert_eq!(parsed["prompt"], "line one\nline \"two\"");
}

#[test]
fn single_quotes_in_the_prompt_are_shell_escaped() {
    let cmds = remote_commands(
        "https://gw.example.com",
        &FarmId::new("acme"),
        &FarmAgentId::new("triage"),
        "don't stop",
    );
    assert!(cmds.launch.contains(r"don'\''t stop"));
    let quoted = cmds
        .launch
        .split_once("-d ")
        .map(|(_, rest)| rest)
        .expect("has a body");
    let unquoted = quoted
        .trim_start_matches('\'')
        .trim_end_matches('\'')
        .replace(r"'\''", "'");
    let parsed: serde_json::Value = serde_json::from_str(&unquoted).expect("body is JSON");
    assert_eq!(parsed["prompt"], "don't stop");
}
