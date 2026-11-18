//! The `OneDrive` and `SharePoint` tools on the server's tool list: every input
//! schema is declarable to Gemini, the Belgian parameter names survive
//! (`type` on a sharing link, `fields` as an open object), and
//! `d365_upload_file` takes a file reference beside its inline base64.

use std::collections::BTreeSet;

use serde_json::Value;
use systemprompt::wire::schema::gemini_declaration_violations;
use systemprompt_mcp_microsoft_365::tools::list_tools;

fn pinned(domains: &[&str]) -> Vec<String> {
    let fixture: Value = serde_json::from_str(include_str!("../fixtures/m365_tools.json")).unwrap();
    fixture["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|tool| domains.iter().any(|domain| tool["domain"] == *domain))
        .map(|tool| tool["name"].as_str().unwrap().to_owned())
        .collect()
}

fn schema_of(name: &str) -> Value {
    let tool = list_tools()
        .into_iter()
        .find(|tool| tool.name == name)
        .unwrap_or_else(|| panic!("`{name}` is not registered"));
    Value::Object(tool.input_schema.as_ref().clone())
}

fn required(schema: &Value) -> BTreeSet<String> {
    schema["required"]
        .as_array()
        .map(|names| {
            names
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn m365_files_and_m365_sharepoint_schemas_are_declarable_with_belgian_parameters() {
    let names = pinned(&["onedrive", "sharepoint"]);
    assert_eq!(names.len(), 20);
    for name in &names {
        let schema = schema_of(name);
        let violations = gemini_declaration_violations(&schema);
        assert!(
            violations.is_empty(),
            "`{name}`: {violations:?}\n{schema:#}"
        );
    }
    let upload = schema_of("d365_upload_file");
    for parameter in [
        "folder_path",
        "filename",
        "content_base64",
        "drive_id",
        "file",
    ] {
        assert!(upload["properties"].get(parameter).is_some(), "{parameter}");
    }
    assert_eq!(upload["properties"]["file"]["type"], "object");
    assert_eq!(
        required(&upload),
        BTreeSet::from(["filename".to_owned(), "folder_path".to_owned()])
    );
    let share = schema_of("d365_share_file");
    assert_eq!(share["properties"]["type"]["type"], "string");
    assert_eq!(share["properties"]["scope"]["default"], "organization");
    let move_file = schema_of("d365_move_file");
    assert_eq!(required(&move_file), BTreeSet::from(["item_id".to_owned()]));
    let items = schema_of("d365_list_sharepoint_list_items");
    assert_eq!(items["properties"]["expand_fields"]["default"], true);
    for name in [
        "d365_create_sharepoint_list_item",
        "d365_update_sharepoint_list_item",
    ] {
        let schema = schema_of(name);
        assert_eq!(schema["properties"]["fields"]["type"], "object", "{name}");
        assert!(required(&schema).contains("fields"), "{name}");
    }
}
