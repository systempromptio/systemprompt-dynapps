//! The `agent-farm` MCP server: its sixteen tools are the whole surface,
//! every schema it lists is well-formed and Gemini-declarable, the
//! read-only annotation sits on exactly the tools that change nothing, and
//! `authorize_run` admits a person's own run, their project's run and an
//! administrator while refusing everyone else.
//!
//! `scope: all` is refused
//! without an administrator role.

use chrono::Utc;
use serde_json::Value;
use systemprompt::agent_runs::AgentRunRow;
use systemprompt::identifiers::{AgentRunId, Email, UserId};
use systemprompt::wire::schema::gemini_declaration_violations;
use systemprompt_mcp_agent_farm::tools::{SERVER_NAME, TOOL_NAMES, list_tools, schema_violations};
use systemprompt_mcp_agent_farm::typed::facts_scope;
use systemprompt_mcp_agent_farm::{RunOrigin, Viewer, authorize_run};
use systemprompt_web_admin_core::repositories::agent_run_facts::RunScope;
use systemprompt_web_shared::ProjectId;

const READ_ONLY: [&str; 13] = [
    "run_status",
    "run_events",
    "run_list",
    "run_facts",
    "run_stats",
    "farm_run_report",
    "farm_eval_results",
    "analytics_costs",
    "analytics_requests",
    "analytics_skills",
    "analytics_tools",
    "catalog_list",
    "farm_dashboard",
];

fn names(tools: &[rmcp::model::Tool]) -> Vec<String> {
    tools.iter().map(|t| t.name.to_string()).collect()
}

#[test]
fn the_sixteen_tools_are_the_whole_surface() {
    let tools = list_tools();
    assert_eq!(names(&tools), TOOL_NAMES);
    assert_eq!(SERVER_NAME, "agent-farm");
}

#[test]
fn every_listed_schema_is_well_formed() {
    let violations = schema_violations();
    assert!(violations.is_empty(), "{violations:#?}");
}

#[test]
fn every_input_schema_is_gemini_declarable_and_flat() {
    for tool in list_tools() {
        let schema = Value::Object(tool.input_schema.as_ref().clone());
        let violations = gemini_declaration_violations(&schema);
        assert!(
            violations.is_empty(),
            "tool `{}` is not Gemini-declarable: {violations:?}\n{schema:#}",
            tool.name
        );
        assert!(
            schema.get("$defs").is_none(),
            "`{}` inlines no enum",
            tool.name
        );
        let properties = schema["properties"].as_object().expect("object properties");
        for (name, property) in properties {
            assert!(
                property.get("type").is_some_and(Value::is_string),
                "`{}`.{name} must be a single typed scalar: {property:#}",
                tool.name
            );
        }
    }
}

#[test]
fn read_only_sits_on_exactly_the_tools_that_change_nothing() {
    for tool in list_tools() {
        let read_only = tool
            .annotations
            .as_ref()
            .and_then(|a| a.read_only_hint)
            .unwrap_or(false);
        assert_eq!(
            read_only,
            READ_ONLY.contains(&tool.name.as_ref()),
            "`{}` read_only annotation",
            tool.name
        );
        assert!(tool.meta.is_some(), "`{}` carries UI meta", tool.name);
        assert!(
            tool.output_schema.is_some(),
            "`{}` has an output schema",
            tool.name
        );
    }
}

fn viewer(id: &str, roles: &[&str], projects: &[&str]) -> Viewer {
    Viewer::from_parts(
        UserId::new(id),
        Email::try_new(format!("{id}@example.test")).expect("email"),
        roles.iter().map(|r| (*r).to_owned()).collect(),
        Vec::new(),
        projects.iter().map(|p| (*p).to_owned()).collect(),
    )
}

fn run_of(user: &str) -> AgentRunRow {
    let now = Utc::now();
    AgentRunRow {
        id: AgentRunId::generate(),
        origin_kind: "farm".to_owned(),
        origin_id: "farm-run-1".to_owned(),
        user_id: UserId::new(user),
        host: "claude_code".to_owned(),
        status: "running".to_owned(),
        request: Value::Null,
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
        started_at: None,
        finished_at: None,
        duration_ms: None,
        error_class: None,
        turns: None,
        input_tokens: None,
        output_tokens: None,
        cost_microdollars: None,
        structured_output: None,
        exit_code: None,
        error: None,
        diff_summary: None,
        harness_digest: None,
        cancel_requested_at: None,
        task_id: None,
        context_id: None,
        created_at: now,
        updated_at: now,
    }
}

#[test]
fn a_person_reaches_their_own_run() {
    let me = viewer("alice", &["developer"], &[]);
    assert!(authorize_run(&me, &run_of("alice"), &RunOrigin::default()).is_ok());
}

#[test]
fn a_person_reaches_a_run_that_ran_as_a_service_account_they_launched() {
    let me = viewer("alice", &["developer"], &[]);
    let origin = RunOrigin {
        launched_by: Some(UserId::new("alice")),
        project_id: None,
    };
    assert!(authorize_run(&me, &run_of("svc-farm"), &origin).is_ok());
}

#[test]
fn a_person_reaches_a_run_of_a_project_they_belong_to() {
    let me = viewer("alice", &["project_manager"], &["odoo-delivery"]);
    let origin = RunOrigin {
        launched_by: Some(UserId::new("bob")),
        project_id: Some(ProjectId::new("odoo-delivery")),
    };
    assert!(authorize_run(&me, &run_of("bob"), &origin).is_ok());
}

#[test]
fn a_cross_user_run_is_refused() {
    let me = viewer("alice", &["developer"], &["odoo-delivery"]);
    let error = authorize_run(&me, &run_of("bob"), &RunOrigin::default())
        .expect_err("bob's run is not alice's");
    assert!(error.message.starts_with("Forbidden:"), "{}", error.message);
}

#[test]
fn a_run_of_a_project_the_person_is_not_in_is_refused() {
    let me = viewer("alice", &["developer"], &["odoo-delivery"]);
    let origin = RunOrigin {
        launched_by: Some(UserId::new("bob")),
        project_id: Some(ProjectId::new("finance")),
    };
    assert!(authorize_run(&me, &run_of("bob"), &origin).is_err());
}

#[test]
fn an_administrator_reaches_every_run() {
    for role in ["admin", "platform_admin", "super_admin"] {
        let me = viewer("root", &[role], &[]);
        assert!(me.is_admin, "{role} grants administration");
        assert!(authorize_run(&me, &run_of("bob"), &RunOrigin::default()).is_ok());
    }
}

#[test]
fn scope_all_needs_an_administrator() {
    let dev = viewer("alice", &["developer"], &["odoo-delivery"]);
    let error = rmcp::ErrorData::from(
        facts_scope(&dev, "all").expect_err("developers never span every user"),
    );
    assert!(error.message.starts_with("Forbidden:"), "{}", error.message);
    assert!(matches!(facts_scope(&dev, "mine"), Ok(RunScope::Own(_))));
    assert!(matches!(facts_scope(&dev, "project"), Ok(RunScope::Projects(ids)) if ids.len() == 1));

    let admin = viewer("root", &["admin"], &[]);
    assert!(matches!(facts_scope(&admin, "all"), Ok(RunScope::All)));
    assert!(
        facts_scope(&admin, "project").is_err(),
        "no projects to scope to"
    );
    assert!(
        facts_scope(&admin, "everyone").is_err(),
        "unknown scope words are refused"
    );
}
