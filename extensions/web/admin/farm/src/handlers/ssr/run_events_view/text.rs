//! The words an event shows: its title line, its body, and whether that body
//! is long enough to fold behind a one-line preview.

use systemprompt::agent_runs::{HookDecision, RunEventPayload, RunnerLevel};

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
            cwd.as_ref().map_or_else(
                || format!("{model} · {} tools", tools.len()),
                |cwd| format!("{model} · {} tools · {cwd}", tools.len()),
            ),
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
        RunEventPayload::Hook {
            name,
            decision,
            policy,
        } => (
            "⚑",
            format!("Hook {name}"),
            hook_body(*decision, policy.as_deref()),
            *decision == HookDecision::Deny,
        ),
        RunEventPayload::Result {
            subtype,
            is_error,
            error,
            ..
        } => (
            "■",
            format!("Run finished · {subtype}"),
            error.clone().unwrap_or_default(),
            *is_error,
        ),
        RunEventPayload::Stderr { line } => ("!", "stderr".to_owned(), line.clone(), false),
        RunEventPayload::Runner { message, level } => (
            "⚙",
            "Runner".to_owned(),
            message.clone(),
            *level == RunnerLevel::Error,
        ),
    };
    EventText {
        glyph,
        title,
        body,
        is_error,
    }
}

fn hook_body(decision: HookDecision, policy: Option<&str>) -> String {
    let verdict = match decision {
        HookDecision::Allow => "allow",
        HookDecision::Deny => "deny",
        HookDecision::Pending => "pending",
    };
    policy.map_or_else(|| verdict.to_owned(), |p| format!("{verdict} · {p}"))
}

// JSON: protocol boundary — a tool's input, shaped by that tool's own schema.
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

// JSON: protocol boundary — tool inputs as the tool's own schema shapes them.
fn pretty_json(value: &serde_json::Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
}
