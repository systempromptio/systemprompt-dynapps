//! The mail tools on the server's tool list: every mail tool of the pinned
//! Belgian fixture is registered with its title and hints, each input schema
//! is declarable to Gemini, and the file inputs are inline objects.

use std::collections::BTreeSet;

use serde_json::Value;
use systemprompt::wire::schema::gemini_declaration_violations;
use systemprompt_mcp_microsoft_365::tools::list_tools;

fn pinned_mail_tools() -> Vec<Value> {
    let fixture: Value = serde_json::from_str(include_str!("../fixtures/m365_tools.json")).unwrap();
    fixture["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|tool| tool["domain"] == "mail")
        .cloned()
        .collect()
}

#[test]
fn m365_mail_every_pinned_mail_tool_is_registered_with_its_annotations() {
    let pinned = pinned_mail_tools();
    assert_eq!(pinned.len(), 24);
    let tools = list_tools();
    for expected in &pinned {
        let name = expected["name"].as_str().unwrap();
        let tool = tools
            .iter()
            .find(|tool| tool.name == name)
            .unwrap_or_else(|| panic!("`{name}` is not registered"));
        let hints = tool.annotations.as_ref().unwrap();
        assert_eq!(hints.title.as_deref(), expected["title"].as_str(), "{name}");
        assert_eq!(
            hints.read_only_hint,
            expected["readOnlyHint"].as_bool(),
            "{name}"
        );
        assert_eq!(
            hints.destructive_hint,
            expected["destructiveHint"].as_bool(),
            "{name}"
        );
        assert_eq!(
            hints.idempotent_hint,
            expected["idempotentHint"].as_bool(),
            "{name}"
        );
        assert_eq!(hints.open_world_hint, Some(true), "{name}");
        assert!(
            tool.description.as_deref().is_some_and(|d| !d.is_empty()),
            "{name}"
        );
    }
    let names: BTreeSet<&str> = tools.iter().map(|tool| tool.name.as_ref()).collect();
    assert_eq!(names.len(), tools.len(), "no tool registered twice");
}

#[test]
fn m365_mail_input_schemas_are_declarable_and_keep_the_belgian_parameters() {
    let tools = list_tools();
    let schema_of = |name: &str| -> Value {
        let tool = tools.iter().find(|tool| tool.name == name).unwrap();
        Value::Object(tool.input_schema.as_ref().clone())
    };
    for expected in pinned_mail_tools() {
        let name = expected["name"].as_str().unwrap();
        let schema = schema_of(name);
        let violations = gemini_declaration_violations(&schema);
        assert!(
            violations.is_empty(),
            "`{name}`: {violations:?}\n{schema:#}"
        );
    }
    let add = schema_of("d365_add_mail_attachment");
    for parameter in [
        "message_id",
        "filename",
        "content_base64",
        "content_type",
        "file",
    ] {
        assert!(
            add["properties"].get(parameter).is_some(),
            "{parameter}: {add:#}"
        );
    }
    assert_eq!(add["properties"]["file"]["type"], "object");
    let required: BTreeSet<&str> = add["required"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert_eq!(required, BTreeSet::from(["filename", "message_id"]));
    let update = schema_of("d365_update_mail");
    assert_eq!(update["properties"]["is_read"]["type"], "boolean");
    assert_eq!(update["properties"]["categories"]["type"], "array");
    let rule = schema_of("d365_create_mail_rule");
    assert_eq!(rule["properties"]["conditions"]["type"], "object");
    assert_eq!(rule["properties"]["sequence"]["type"], "integer");
}
