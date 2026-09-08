//! Parsing of the gateway's flattened tool-use markers.
//!
//! An assistant message is stored as text with each tool use flattened to
//! `[tool_use:NAME {json}]`, several per message when the model called
//! several tools. The JSON may span lines and nest brackets, so the marker is
//! walked with a string-aware depth counter rather than a regex; an
//! unterminated marker is left in the text as it is.

const MARKER_OPEN: &str = "[tool_use:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolUseMarker {
    pub name: String,
    pub input_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedAssistant {
    pub text: String,
    pub tool_uses: Vec<ToolUseMarker>,
}

#[must_use]
pub fn parse_assistant(body: &str) -> ParsedAssistant {
    let mut text = String::new();
    let mut tool_uses = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find(MARKER_OPEN) {
        let after = &rest[start + MARKER_OPEN.len()..];
        match parse_marker(after) {
            Some((marker, consumed)) => {
                text.push_str(&rest[..start]);
                tool_uses.push(marker);
                rest = &after[consumed..];
            }
            None => {
                text.push_str(&rest[..start + MARKER_OPEN.len()]);
                rest = after;
            }
        }
    }
    text.push_str(rest);
    ParsedAssistant {
        text: text.trim().to_owned(),
        tool_uses,
    }
}

// Why: `after` starts right past `[tool_use:`; returns the marker and how many
// bytes of `after` it occupied, including the closing `]`.
fn parse_marker(after: &str) -> Option<(ToolUseMarker, usize)> {
    let name_end = after.find(|c: char| c.is_whitespace() || c == ']')?;
    let name = after[..name_end].trim();
    if name.is_empty() {
        return None;
    }
    let mut pos = name_end;
    pos += after[pos..].len() - after[pos..].trim_start().len();
    let input_json = match after[pos..].chars().next() {
        Some(']') => {
            return Some((
                ToolUseMarker {
                    name: name.to_owned(),
                    input_json: "{}".to_owned(),
                },
                pos + 1,
            ));
        }
        Some('{' | '[') => {
            let len = json_len(&after[pos..])?;
            let json = &after[pos..pos + len];
            pos += len;
            json.to_owned()
        }
        _ => return None,
    };
    pos += after[pos..].len() - after[pos..].trim_start().len();
    if !after[pos..].starts_with(']') {
        return None;
    }
    Some((
        ToolUseMarker {
            name: name.to_owned(),
            input_json,
        },
        pos + 1,
    ))
}

// Why: byte length of the JSON value at the start of `s`, honouring strings
// and escapes so a `]` inside a string argument does not end the marker.
fn json_len(s: &str) -> Option<usize> {
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if in_string {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_string = true,
            '{' | '[' => depth += 1,
            '}' | ']' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i + c.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}

// JSON: the marker's arguments are per-tool shaped and only ever re-indented.
#[must_use]
pub(super) fn pretty_json_text(raw: &str) -> String {
    serde_json::from_str::<serde_json::Value>(raw)
        .ok()
        .and_then(|v| serde_json::to_string_pretty(&v).ok())
        .unwrap_or_else(|| raw.to_owned())
}
