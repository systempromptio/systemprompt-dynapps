//! Normalize modality-specific BDA output while retaining extraction
//! provenance.

use crate::error::KnowledgeBankError;
use serde_json::Value; // JSON: BDA standard-output schemas differ by modality

pub fn normalize(bytes: &[u8]) -> Result<String, KnowledgeBankError> {
    let value: Value = serde_json::from_slice(bytes)?;
    let mut result = String::new();
    visit(&value, "BDA", &mut result, 0);
    Ok(result)
}

fn visit(value: &Value, path: &str, output: &mut String, depth: usize) {
    if depth > 48 {
        return;
    }
    match value {
        Value::Object(fields) => {
            let mut context = path.to_owned();
            for key in [
                "page_index",
                "page_number",
                "start_time",
                "end_time",
                "start_timestamp_millis",
                "end_timestamp_millis",
                "speaker_id",
            ] {
                if let Some(value) = fields.get(key) {
                    context.push_str(&format!(" {key}={value}"));
                }
            }
            for (key, child) in fields {
                if matches!(
                    key.as_str(),
                    "text"
                        | "markdown"
                        | "transcript"
                        | "summary"
                        | "caption"
                        | "description"
                        | "representation"
                ) && let Some(text) = child.as_str().filter(|s| !s.trim().is_empty())
                {
                    let provenance =
                        if matches!(key.as_str(), "summary" | "caption" | "description") {
                            "Generated description"
                        } else {
                            "Extracted content"
                        };
                    output.push_str(&format!("\n## {context} — {provenance}\n{text}\n"));
                } else if child.is_array() || child.is_object() {
                    visit(child, &format!("{context}/{key}"), output, depth + 1);
                }
            }
        },
        Value::Array(items) => {
            for (index, child) in items.iter().enumerate() {
                visit(child, &format!("{path}[{index}]"), output, depth + 1);
            }
        },
        _ => {},
    }
}
