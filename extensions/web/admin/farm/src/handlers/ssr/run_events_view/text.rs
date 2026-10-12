//! The words an event shows: its title line, its body, and whether that body
//! is long enough to fold behind a one-line preview.

use crate::types::RunEventPayload;

pub const FOLD_CHARS: usize = 600;
pub(crate) const FOLD_LINES: usize = 12;
pub(crate) const PREVIEW_CHARS: usize = 160;

const SUBJECT_KEYS: &[&str] = &[
    "command",
    "file_path",
    "path",
    "pattern",
    "url",
    "query",
    "description",
    "prompt",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EventText {
    pub glyph: &'static str,
    pub title: String,
    pub body: String,
    pub is_error: bool,
}

pub(crate) fn event_text(payload: &RunEventPayload, tool_name: Option<&str>) -> EventText {
    let (glyph, title, body, is_error) = match payload {
        RunEventPayload::Init {
            model, tools, cwd, ..
        } => (
            "◆",
            "Session started".to_owned(),
            format!("{model} · {} tools · {cwd}", tools.len()),
            false,
        ),
        RunEventPayload::AssistantText { text } => {
            ("¶", "Assistant".to_owned(), text.clone(), false)
        },
        RunEventPayload::ToolUse { name, input, .. } => {
            ("→", tool_title(name, input), pretty_json(input), false)
        },
        RunEventPayload::ToolResult {
            content, is_error, ..
        } => (
            "←",
            format!("{} returned", tool_name.unwrap_or("Tool")),
            content.clone(),
            *is_error,
        ),
        RunEventPayload::Hook { name, decision } => (
            "⚑",
            format!("Hook {name}"),
            decision.clone().unwrap_or_default(),
            decision
                .as_deref()
                .is_some_and(|d| d.eq_ignore_ascii_case("deny")),
        ),
        RunEventPayload::Result {
            subtype,
            is_error,
            text,
            ..
        } => (
            "■",
            format!("Run finished · {subtype}"),
            text.clone().unwrap_or_default(),
            *is_error,
        ),
        RunEventPayload::Stderr { text } => ("!", "stderr".to_owned(), text.clone(), false),
        RunEventPayload::Runner { message } => ("⚙", "Runner".to_owned(), message.clone(), false),
    };
    EventText {
        glyph,
        title,
        body,
        is_error,
    }
}

// JSON: protocol boundary — a stored payload that did not decode into
// `RunEventPayload`, and tool inputs whose shape each tool defines.
pub(crate) fn raw_text(kind: &str, payload: &serde_json::Value) -> EventText {
    EventText {
        glyph: "?",
        title: format!("Unrecognised {kind} event"),
        body: pretty_json(payload),
        is_error: false,
    }
}

pub(crate) fn tool_title(name: &str, input: &serde_json::Value) -> String {
    let subject = SUBJECT_KEYS
        .iter()
        .find_map(|key| input.get(*key).and_then(serde_json::Value::as_str))
        .map(|s| first_line(s, PREVIEW_CHARS));
    match subject {
        Some(s) if !s.is_empty() => format!("{name} · {s}"),
        _ => name.to_owned(),
    }
}

pub(crate) fn is_long(body: &str) -> bool {
    body.chars().count() > FOLD_CHARS || body.lines().count() > FOLD_LINES
}

pub(crate) fn first_line(text: &str, max_chars: usize) -> String {
    let line = text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    if line.chars().count() > max_chars {
        let head: String = line.chars().take(max_chars).collect();
        format!("{head}…")
    } else {
        line.to_owned()
    }
}

// JSON: protocol boundary — undecoded payloads and tool inputs as stored.
fn pretty_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}
