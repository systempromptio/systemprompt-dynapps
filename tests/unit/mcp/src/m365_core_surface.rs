//! The `microsoft-365` tool surface against the fixture pinned from the
//! Belgian server (`fixtures/m365_tools.json`, from `_annotations.py` and
//! the `apps/` tools), the four tools served as destructive where Belgium
//! marks a plain write, and the broker the Graph token comes from.

use std::collections::HashSet;

use serde::Deserialize;
use serde_json::{Value, json};
use systemprompt::wire::schema::gemini_declaration_violations;
use systemprompt_mcp_microsoft_365::graph::{BROKER_PATH, Broker, GraphError, GraphTokens};
use systemprompt_mcp_microsoft_365::tools::{TOOLS, ToolKind, list_tools, spec};

use crate::m365_parity::served_destructive;
use crate::m365_support::{Reply, Secrets, Sim};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pinned {
    name: String,
    title: String,
    read_only_hint: bool,
    destructive_hint: bool,
    idempotent_hint: bool,
    open_world_hint: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Fixture {
    tool_count: usize,
    app_count: usize,
    tools: Vec<Pinned>,
    apps: Vec<Pinned>,
}

fn fixture() -> Fixture {
    let raw = include_str!("../fixtures/m365_tools.json");
    serde_json::from_str(raw).unwrap()
}

#[test]
fn m365_core_fixture_pins_the_belgian_surface() {
    let fixture = fixture();
    assert_eq!(fixture.tools.len(), fixture.tool_count);
    assert_eq!(fixture.apps.len(), fixture.app_count);
    assert_eq!((fixture.tool_count, fixture.app_count), (147, 3));
    let names: HashSet<&str> = fixture
        .tools
        .iter()
        .chain(&fixture.apps)
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(names.len(), 150, "names are unique");
    assert!(names.iter().all(|n| n.starts_with("d365_")));
    assert!(fixture.tools.iter().all(|t| t.open_world_hint));
}

#[test]
fn m365_core_every_registered_tool_matches_its_pinned_name_and_annotations() {
    let fixture = fixture();
    let tools = list_tools();
    assert_eq!(tools.len(), TOOLS.len());
    for tool in &tools {
        let pinned = fixture
            .tools
            .iter()
            .chain(&fixture.apps)
            .find(|p| p.name == tool.name)
            .unwrap_or_else(|| panic!("`{}` is not a Belgian tool", tool.name));
        let a = tool.annotations.as_ref().unwrap();
        assert_eq!(
            a.title.as_deref(),
            Some(pinned.title.as_str()),
            "{}",
            tool.name
        );
        assert_eq!(
            a.read_only_hint,
            Some(pinned.read_only_hint),
            "{}",
            tool.name
        );
        assert_eq!(
            a.destructive_hint,
            Some(served_destructive(&pinned.name, pinned.destructive_hint)),
            "{}",
            tool.name
        );
        assert_eq!(
            a.idempotent_hint,
            Some(pinned.idempotent_hint),
            "{}",
            tool.name
        );
        assert_eq!(
            a.open_world_hint,
            Some(pinned.open_world_hint),
            "{}",
            tool.name
        );
    }
    assert!(tools.iter().any(|t| t.name == "d365_list_mail"));
}

#[test]
fn m365_core_tools_that_can_destroy_or_overwrite_announce_destructive_hint() {
    let tools = list_tools();
    for name in [
        "d365_batch",
        "d365_share_file",
        "d365_create_mail_rule",
        "d365_upload_file",
    ] {
        let tool = tools.iter().find(|t| t.name == name).unwrap();
        let a = tool.annotations.as_ref().unwrap();
        let hints = (a.read_only_hint, a.destructive_hint, a.idempotent_hint);
        assert_eq!(hints, (Some(false), Some(true), Some(false)), "{name}");
        assert_eq!(spec(name).map(|s| s.kind), Some(ToolKind::Destructive));
    }
}

#[test]
fn m365_core_every_listed_input_schema_is_gemini_declarable() {
    for tool in list_tools() {
        let schema = Value::Object(tool.input_schema.as_ref().clone());
        let violations = gemini_declaration_violations(&schema);
        assert!(
            violations.is_empty(),
            "`{}`: {violations:?}\n{schema:#}",
            tool.name
        );
        if tool.name == "d365_list_mail" {
            assert_eq!(
                schema["properties"]["is_read"]["type"], "boolean",
                "{schema:#}"
            );
        }
    }
}

fn broker(base: &str, secret: Option<&str>) -> Broker {
    let secrets = secret.map_or_else(
        || Secrets::with(&[]),
        |value| Secrets::with(&[("mcp_credential_broker_secret", value)]),
    );
    Broker::new(format!("{base}{BROKER_PATH}"), secrets, "microsoft-365")
}

#[tokio::test]
async fn m365_core_broker_is_asked_with_the_callers_bearer_and_the_broker_secret() {
    let sim = Sim::start(|_, _| Reply::json(200, &json!({"access_token": "graph-token"}))).await;
    let tokens = broker(&sim.base, Some("s3cret")).for_caller("user-jwt".to_owned(), None);
    assert_eq!(tokens.access_token(None).await.unwrap(), "graph-token");

    let call = &sim.calls()[0];
    assert_eq!(call.method, "GET");
    assert_eq!(call.path, "/api/public/connectors/microsoft-365/token");
    assert_eq!(call.header("authorization"), Some("Bearer user-jwt"));
    assert_eq!(
        call.header("x-systemprompt-credential-broker"),
        Some("s3cret")
    );
    assert_eq!(
        call.header("x-systemprompt-mcp-server"),
        Some("microsoft-365")
    );
    assert!(
        !format!("{tokens:?}").contains("user-jwt"),
        "Debug hides the bearer"
    );
}

#[tokio::test]
async fn m365_core_broker_refusals_reach_the_model_as_connect_or_config_errors() {
    let sim = Sim::start(|_, _| Reply::json(404, &json!({"error": "not connected"}))).await;
    let error = broker(&sim.base, Some("s3cret"))
        .for_caller("user-jwt".to_owned(), None)
        .access_token(None)
        .await
        .unwrap_err();
    assert!(matches!(error, GraphError::NotConnected), "{error:?}");

    let error = broker(&sim.base, None)
        .for_caller("user-jwt".to_owned(), None)
        .access_token(None)
        .await
        .unwrap_err();
    assert!(matches!(error, GraphError::Config(_)), "{error:?}");
    assert_eq!(sim.calls().len(), 1, "no broker call without its secret");
}
