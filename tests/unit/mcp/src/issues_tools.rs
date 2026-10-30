//! Issue reports on the `systemprompt` server: core's four issue tools keep
//! names that none of the server's own tools use, `files.issues` is on, the
//! `issues` stream that core announces reports on is declared, and the
//! commons plugin carries the `report_issue` skill.

use std::path::PathBuf;

use serde_yaml::Value;
use systemprompt::system::issues::mcp::IssuesMcpTools;
use systemprompt_mcp_agent::tools::list_tools;

const EXPECTED: [&str; 4] = [
    "report_issue",
    "issue_attachment",
    "issues_list",
    "issue_status",
];

fn repo_file(rel: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(rel);
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

#[test]
fn cores_issue_tools_do_not_collide_with_the_servers_own() {
    assert_eq!(IssuesMcpTools::tool_names(), EXPECTED);
    let own: Vec<String> = list_tools().iter().map(|t| t.name.to_string()).collect();
    for name in EXPECTED {
        assert!(
            !own.iter().any(|n| n == name),
            "{name} is already a systemprompt tool"
        );
    }
}

#[test]
fn issue_reports_are_enabled_and_announced_on_the_issues_stream() {
    let files = repo_file("services/config/files.yaml");
    assert_eq!(files["files"]["issues"]["enabled"].as_bool(), Some(true));

    let streams = repo_file("services/notifications/streams.yaml");
    let declared = streams["streams"]
        .as_sequence()
        .is_some_and(|s| s.iter().any(|v| v["id"].as_str() == Some("issues")));
    assert!(
        declared,
        "no `issues` stream in services/notifications/streams.yaml"
    );
}

#[test]
fn every_signed_in_user_gets_the_skill_through_commons() {
    let plugin = &repo_file("services/plugins/dynapps-commons/config.yaml")["plugin"];
    let skills: Vec<&str> = plugin["skills"]["include"]
        .as_sequence()
        .map(|s| s.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    assert!(skills.contains(&"report_issue"));

    let skill = repo_file("services/skills/report_issue/config.yaml");
    assert_eq!(skill["id"].as_str(), Some("report_issue"));
    assert_eq!(skill["enabled"].as_bool(), Some(true));
}
