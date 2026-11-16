//! Parity of the `microsoft-365` surface with the Belgian server, against
//! the pinned fixture (`fixtures/m365_tools.json`): exactly its 147 tools —
//! none missing, none extra — with their titles, hints and parameters (name,
//! JSON type, required). The three `apps/` entry points are pinned too and
//! must not be served. Every parameter that differs on purpose is listed in
//! `DEVIATIONS` with its reason, and a stale entry fails like a new drift.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;
use serde_json::{Map, Value};
use systemprompt_mcp_microsoft_365::tools::{TOOLS, list_tools, spec};

#[derive(Debug, Deserialize)]
struct Param {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    required: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pinned {
    name: String,
    title: String,
    read_only_hint: bool,
    destructive_hint: bool,
    idempotent_hint: bool,
    open_world_hint: bool,
    params: Vec<Param>,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    tools: Vec<Pinned>,
    apps: Vec<Pinned>,
}

const INLINE_OR_REF: &str = "decisions.md 8: base64 stays for small files (<= 64 KB) and \
    becomes optional, since the file layer takes the same file by reference in `file`";

// Parameters that differ from the Belgian signature on purpose: the tool,
// the drift as `parity_drift` words it, and why.
const DEVIATIONS: &[(&str, &str, &str)] = &[
    (
        "d365_add_mail_attachment",
        "d365_add_mail_attachment: `content_base64` required=false",
        INLINE_OR_REF,
    ),
    (
        "d365_add_mail_attachment",
        "d365_add_mail_attachment: `file` extra",
        INLINE_OR_REF,
    ),
    (
        "d365_upload_attachment_chunk",
        "d365_upload_attachment_chunk: `chunk_base64` required=false",
        INLINE_OR_REF,
    ),
    (
        "d365_upload_attachment_chunk",
        "d365_upload_attachment_chunk: `file` extra",
        INLINE_OR_REF,
    ),
    (
        "d365_upload_file",
        "d365_upload_file: `content_base64` required=false",
        INLINE_OR_REF,
    ),
    (
        "d365_upload_file",
        "d365_upload_file: `file` extra",
        INLINE_OR_REF,
    ),
];

fn fixture() -> Fixture {
    serde_json::from_str(include_str!("../fixtures/m365_tools.json")).unwrap()
}

// The names one side has and the other lacks: (missing, extra).
fn surface_diff<'a>(
    registered: &BTreeSet<&'a str>,
    pinned: &BTreeSet<&'a str>,
) -> (Vec<&'a str>, Vec<&'a str>) {
    let missing = pinned.difference(registered).copied().collect();
    let extra = registered.difference(pinned).copied().collect();
    (missing, extra)
}

// The JSON type a property schema declares, `null` aside.
fn schema_type(property: &Value) -> String {
    let declared = match property.get("type") {
        Some(Value::String(kind)) => Some(kind.clone()),
        Some(Value::Array(kinds)) => kinds
            .iter()
            .filter_map(Value::as_str)
            .find(|kind| *kind != "null")
            .map(str::to_owned),
        _ => None,
    };
    declared.unwrap_or_else(|| "untyped".to_owned())
}

// Every difference between a registered input schema and the Belgian
// parameters, one line each.
fn parity_drift(tool: &str, schema: &Map<String, Value>, pinned: &[Param]) -> Vec<String> {
    let empty = Map::new();
    let properties = schema
        .get("properties")
        .and_then(Value::as_object)
        .unwrap_or(&empty);
    let required: BTreeSet<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let mut drift = Vec::new();
    for param in pinned {
        let Some(property) = properties.get(&param.name) else {
            drift.push(format!("{tool}: `{}` missing", param.name));
            continue;
        };
        let kind = schema_type(property);
        if kind != param.kind {
            drift.push(format!(
                "{tool}: `{}` is {kind}, not {}",
                param.name, param.kind
            ));
        }
        let is_required = required.contains(param.name.as_str());
        if is_required != param.required {
            drift.push(format!("{tool}: `{}` required={is_required}", param.name));
        }
    }
    let known: BTreeSet<&str> = pinned.iter().map(|p| p.name.as_str()).collect();
    for name in properties
        .keys()
        .filter(|name| !known.contains(name.as_str()))
    {
        drift.push(format!("{tool}: `{name}` extra"));
    }
    drift
}

#[test]
fn m365_parity_registers_exactly_the_147_belgian_tools() {
    let fixture = fixture();
    let tools = list_tools();
    let registered: BTreeSet<&str> = tools.iter().map(|tool| tool.name.as_ref()).collect();
    let pinned: BTreeSet<&str> = fixture.tools.iter().map(|t| t.name.as_str()).collect();
    let (missing, extra) = surface_diff(&registered, &pinned);
    assert!(missing.is_empty(), "missing Belgian tools: {missing:?}");
    assert!(extra.is_empty(), "tools Belgium does not have: {extra:?}");
    assert_eq!(registered.len(), tools.len(), "no tool is listed twice");
    assert_eq!((tools.len(), TOOLS.len()), (147, 147));
}

#[test]
fn m365_parity_surface_diff_names_what_is_missing_and_what_is_extra() {
    let registered = BTreeSet::from(["d365_a", "d365_b", "d365_extra"]);
    let pinned = BTreeSet::from(["d365_a", "d365_b", "d365_gone"]);
    assert_eq!(
        surface_diff(&registered, &pinned),
        (vec!["d365_gone"], vec!["d365_extra"])
    );
}

#[test]
fn m365_parity_the_three_belgian_apps_are_pinned_but_not_served() {
    let fixture = fixture();
    let served: BTreeSet<String> = list_tools()
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect();
    assert_eq!(fixture.apps.len(), 3);
    for app in &fixture.apps {
        assert!(app.name.starts_with("d365_app_"), "{}", app.name);
        assert!(!served.contains(&app.name), "{} is served", app.name);
        assert!(spec(&app.name).is_none(), "{} has a spec", app.name);
    }
}

#[test]
fn m365_parity_every_tool_keeps_its_belgian_title_and_hints() {
    let fixture = fixture();
    let pinned: BTreeMap<&str, &Pinned> =
        fixture.tools.iter().map(|t| (t.name.as_str(), t)).collect();
    for tool in list_tools() {
        let want = pinned[tool.name.as_ref()];
        let hints = tool.annotations.expect("annotated");
        let got = (
            hints.title.unwrap_or_default(),
            hints.read_only_hint,
            hints.destructive_hint,
            hints.idempotent_hint,
            hints.open_world_hint,
        );
        let expected = (
            want.title.clone(),
            Some(want.read_only_hint),
            Some(want.destructive_hint),
            Some(want.idempotent_hint),
            Some(want.open_world_hint),
        );
        assert_eq!(got, expected, "{}", tool.name);
    }
}

#[test]
fn m365_parity_every_parameter_matches_belgium_or_is_a_recorded_deviation() {
    let fixture = fixture();
    let pinned: BTreeMap<&str, &Pinned> =
        fixture.tools.iter().map(|t| (t.name.as_str(), t)).collect();
    let mut drift: BTreeSet<String> = BTreeSet::new();
    for tool in list_tools() {
        let want = pinned[tool.name.as_ref()];
        drift.extend(parity_drift(&tool.name, &tool.input_schema, &want.params));
    }
    let recorded: BTreeSet<String> = DEVIATIONS
        .iter()
        .map(|(_, line, why)| {
            assert!(!why.is_empty(), "{line} needs a reason");
            (*line).to_owned()
        })
        .collect();
    let unexplained: Vec<&String> = drift.difference(&recorded).collect();
    let stale: Vec<&String> = recorded.difference(&drift).collect();
    assert!(
        unexplained.is_empty(),
        "parameter drift from Belgium: {unexplained:#?}"
    );
    assert!(
        stale.is_empty(),
        "recorded deviations that no longer hold: {stale:#?}"
    );
    assert!(
        DEVIATIONS
            .iter()
            .all(|(tool, line, _)| line.starts_with(&format!("{tool}: ")))
    );
}

#[test]
fn m365_parity_drift_reports_missing_extra_retyped_and_optional_parameters() {
    let schema: Map<String, Value> = serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "id": {"type": "string"},
            "top": {"type": ["string", "null"]},
            "new": {"type": "boolean"}
        },
        "required": []
    }))
    .unwrap();
    let pinned = [
        Param {
            name: "id".to_owned(),
            kind: "string".to_owned(),
            required: true,
        },
        Param {
            name: "top".to_owned(),
            kind: "integer".to_owned(),
            required: false,
        },
        Param {
            name: "gone".to_owned(),
            kind: "string".to_owned(),
            required: false,
        },
    ];
    assert_eq!(
        parity_drift("d365_x", &schema, &pinned),
        vec![
            "d365_x: `id` required=false",
            "d365_x: `top` is string, not integer",
            "d365_x: `gone` missing",
            "d365_x: `new` extra",
        ]
    );
}
