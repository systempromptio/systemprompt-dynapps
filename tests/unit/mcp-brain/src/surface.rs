use std::fs;

use dynapps_test_common::repo_path;
use serde_json::Value;
use systemprompt_mcp_brain::tools::{INSTRUCTIONS, TOOLS, list_tools, spec, tool_names};
use systemprompt_mcp_shared::tool_schema::tool_violations;

const EXPECTED: [&str; 18] = [
    "answer",
    "asset_link",
    "asset_list",
    "asset_upload",
    "create",
    "create_tree",
    "export",
    "get",
    "list",
    "list_spaces",
    "metadata",
    "metadata_schema",
    "move",
    "outline",
    "request_upload",
    "search",
    "trash",
    "update",
];

fn sorted(mut names: Vec<String>) -> Vec<String> {
    names.sort();
    names
}

#[test]
fn the_server_lists_exactly_the_eighteen_tools() {
    let listed = sorted(
        list_tools()
            .iter()
            .map(|tool| tool.name.to_string())
            .collect(),
    );
    assert_eq!(listed, EXPECTED);
    let declared = sorted(tool_names().iter().map(|name| (*name).to_owned()).collect());
    assert_eq!(declared, EXPECTED);
}

#[test]
fn the_names_carry_no_prefix_and_leave_out_git_and_versions() {
    for name in tool_names() {
        assert!(
            !name.starts_with("docs_") && !name.starts_with("brain_"),
            "{name} carries a prefix: the client adds the server's name"
        );
    }
    for gone in [
        "changes_status",
        "propose_changes",
        "review_proposal",
        "merge_proposal",
        "sync_status",
        "version",
    ] {
        assert!(spec(gone).is_none(), "{gone} belongs to the Git source");
    }
}

fn required(schema: &Value) -> Vec<String> {
    schema["required"]
        .as_array()
        .map(|names| {
            names
                .iter()
                .map(|name| name.as_str().expect("a name").to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn any_of(schema: &Value) -> Vec<Vec<String>> {
    schema["anyOf"]
        .as_array()
        .map(|groups| groups.iter().map(required).collect())
        .unwrap_or_default()
}

#[test]
fn each_schema_declares_the_arguments_it_requires() {
    let cases: [(&str, &[&str], &[&[&str]]); 18] = [
        ("search", &["query"], &[]),
        ("answer", &["question"], &[]),
        ("get", &[], &[&["path"], &["article_id"]]),
        ("list_spaces", &[], &[]),
        ("list", &["space"], &[]),
        ("outline", &[], &[&["node"], &["space"]]),
        ("export", &["space"], &[]),
        (
            "metadata",
            &[],
            &[&["article_id"], &["path"], &["article_ids"]],
        ),
        (
            "metadata_schema",
            &[],
            &[&["article_id"], &["path"], &["space"]],
        ),
        ("create", &["title"], &[&["parent_id"], &["space"]]),
        ("create_tree", &["nodes"], &[&["parent_id"], &["space"]]),
        ("update", &["article_id"], &[]),
        ("move", &["article_id", "new_parent_id"], &[]),
        ("trash", &["article_id"], &[]),
        ("request_upload", &[], &[&["article_id"], &["path"]]),
        ("asset_list", &[], &[&["article_id"], &["path"]]),
        ("asset_link", &["attachment_id"], &[]),
        ("asset_upload", &[], &[&["article_id"], &["path"]]),
    ];
    for (name, must, groups) in cases {
        let schema = (spec(name).expect("a listed tool").schema)();
        assert_eq!(required(&schema), must, "{name}: required");
        let expected: Vec<Vec<String>> = groups
            .iter()
            .map(|group| group.iter().map(|key| (*key).to_owned()).collect())
            .collect();
        assert_eq!(any_of(&schema), expected, "{name}: anyOf");
        for key in must.iter().chain(groups.iter().flat_map(|group| group.iter())) {
            assert!(
                schema["properties"][*key].is_object(),
                "{name}: `{key}` is required but not a property"
            );
        }
    }
}

#[test]
fn every_listed_schema_passes_meta_validation() {
    for tool in list_tools() {
        let input = Value::Object((*tool.input_schema).clone());
        let output = tool.output_schema.as_ref().map(|schema| Value::Object((**schema).clone()));
        let violations = tool_violations("brain", &tool.name, &input, output.as_ref());
        assert!(violations.is_empty(), "{violations:?}");
    }
}

#[test]
fn no_description_names_the_source_system() {
    let mut text = vec![INSTRUCTIONS.to_owned()];
    for tool in list_tools() {
        text.push(tool.name.to_string());
        text.push(tool.description.map(|description| description.to_string()).unwrap_or_default());
        text.push(Value::Object((*tool.input_schema).clone()).to_string());
    }
    for spec in TOOLS.iter() {
        text.push(spec.title.to_owned());
    }
    for piece in text {
        let lower = piece.to_lowercase();
        for banned in ["odoo", "knowledge"] {
            assert!(!lower.contains(banned), "`{banned}` in: {piece}");
        }
    }
}

#[test]
fn every_tool_says_what_it_does_in_a_full_description() {
    for tool in list_tools() {
        let description = tool.description.as_deref().expect("a description");
        assert!(description.len() > 150, "{} is too terse", tool.name);
    }
}

#[test]
fn search_keeps_the_pautas_agents_were_tested_with() {
    let search = spec("search").expect("search").description;
    assert!(search.contains("confidence: \"low\""));
    assert!(search.contains("say the answer is not documented"));
    assert!(search.contains("unrated"));
    let get = spec("get").expect("get").description;
    assert!(get.contains("markdown_url"));
    let answer = spec("answer").expect("answer").description;
    assert!(answer.contains("never writes the answer"));
}

#[test]
fn uploads_say_the_body_is_raw_and_a_form_is_refused() {
    for name in ["request_upload", "asset_upload"] {
        let description = spec(name).expect("a listed tool").description;
        assert!(description.contains("--data-binary"), "{name}");
        assert!(description.contains("never as a multipart form"), "{name}");
    }
}

#[test]
fn reads_are_read_only_and_trash_alone_is_destructive() {
    let reads = [
        "search",
        "answer",
        "get",
        "list_spaces",
        "list",
        "outline",
        "export",
        "metadata",
        "metadata_schema",
        "asset_list",
        "asset_link",
    ];
    for tool in list_tools() {
        let hints = tool.annotations.as_ref().expect("annotations");
        let name: &str = &tool.name;
        assert_eq!(hints.read_only_hint, Some(reads.contains(&name)), "{name}");
        assert_eq!(hints.destructive_hint, Some(name == "trash"), "{name}");
        assert_eq!(hints.open_world_hint, Some(false), "{name}");
    }
}

#[test]
fn no_tool_name_matches_the_governance_blocklist() {
    let config = fs::read_to_string(repo_path("services/governance/config.yaml"))
        .expect("the governance config");
    let patterns: Vec<&str> = config
        .lines()
        .skip_while(|line| !line.contains("id: tool_blocklist"))
        .skip_while(|line| !line.trim_start().starts_with("patterns:"))
        .skip(1)
        .map_while(|line| line.trim_start().strip_prefix("- "))
        .collect();
    assert!(
        patterns.contains(&"delete") && patterns.contains(&"erase"),
        "the blocklist was not read: {patterns:?}"
    );
    for name in tool_names() {
        let full = format!("mcp__brain__{name}");
        for pattern in &patterns {
            assert!(!full.contains(pattern), "{full} matches `{pattern}`");
        }
    }
}
