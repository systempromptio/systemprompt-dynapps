//! Meta-validation of MCP tool schemas.
//!
//! Every tool an MCP server lists carries an `inputSchema` (and optionally an
//! `outputSchema`) that the gateway forwards to a model provider as written.
//! A provider refuses the whole request when one schema is malformed, so a bad
//! schema is caught here, where tools are listed, rather than as a 400 on
//! every request that offers the tool.
//!
//! Two layers of checks run on each schema:
//! - the JSON Schema meta-schema its `$schema` names (Draft 2020-12 when it
//!   names none; Draft 7 and the other bundled drafts when it does);
//! - the shapes providers refuse even when the meta-schema allows them: a
//!   top-level `type` other than `object`, a top-level `required` naming a key
//!   `properties` does not declare, and a `$ref` that leaves the document.

// JSON: tool schemas are JSON Schema documents at the MCP protocol boundary.
use serde_json::Value;
use std::fmt;

/// Which of a tool's two schemas a violation was found in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaSide {
    Input,
    Output,
}

impl SchemaSide {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Input => "inputSchema",
            Self::Output => "outputSchema",
        }
    }
}

/// One malformed schema, named by server, tool and side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolSchemaViolation {
    pub server: String,
    pub tool: String,
    pub side: SchemaSide,
    pub detail: String,
}

impl fmt::Display for ToolSchemaViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MCP server '{}', tool '{}': {} {}",
            self.server,
            self.tool,
            self.side.as_str(),
            self.detail
        )
    }
}

// JSON: one tool schema as the server lists it.
#[must_use]
pub fn schema_violations(schema: &Value) -> Vec<String> {
    let mut out = Vec::new();
    if !schema.is_object() {
        out.push("is not a JSON object".to_owned());
        return out;
    }
    if let Err(e) = jsonschema::meta::validate(schema) {
        out.push(format!("violates the JSON Schema meta-schema: {e}"));
    }
    if schema.get("type").and_then(Value::as_str) != Some("object") {
        out.push("must declare top-level `type: object`".to_owned());
    }
    out.extend(required_violations(schema));
    collect_external_refs(schema, &mut out);
    out
}

#[must_use]
pub fn tool_violations(
    server: &str,
    tool: &str,
    // JSON: the tool's inputSchema.
    input: &Value,
    // JSON: the tool's outputSchema, when it declares one.
    output: Option<&Value>,
) -> Vec<ToolSchemaViolation> {
    let sides = std::iter::once((SchemaSide::Input, input))
        .chain(output.map(|schema| (SchemaSide::Output, schema)));
    sides
        .flat_map(|(side, schema)| {
            schema_violations(schema)
                .into_iter()
                .map(move |detail| ToolSchemaViolation {
                    server: server.to_owned(),
                    tool: tool.to_owned(),
                    side,
                    detail,
                })
        })
        .collect()
}

// JSON: the top level of one tool schema.
fn required_violations(schema: &Value) -> Vec<String> {
    let Some(required) = schema.get("required") else {
        return Vec::new();
    };
    let Some(names) = required.as_array() else {
        return vec!["`required` must be an array of property names".to_owned()];
    };
    let properties = schema.get("properties").and_then(Value::as_object);
    names
        .iter()
        .filter_map(|name| match (name.as_str(), properties) {
            (None, _) => Some(format!("`required` holds a non-string entry {name}")),
            (Some(key), None) => Some(format!(
                "`required` names '{key}' but the schema declares no `properties`"
            )),
            (Some(key), Some(props)) if !props.contains_key(key) => Some(format!(
                "`required` names '{key}', which `properties` does not declare"
            )),
            _ => None,
        })
        .collect()
}

// JSON: any node of a tool schema, walked for `$ref`.
fn collect_external_refs(node: &Value, out: &mut Vec<String>) {
    match node {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "$ref"
                    && let Some(target) = value.as_str()
                    && !target.starts_with('#')
                {
                    out.push(format!(
                        "has a `$ref` to '{target}' outside the document; providers resolve none"
                    ));
                }
                collect_external_refs(value, out);
            }
        },
        Value::Array(items) => items
            .iter()
            .for_each(|item| collect_external_refs(item, out)),
        _ => {},
    }
}
