//! Claude Code's `--output-format stream-json` lines, decoded into the run
//! events the farm stores.
//!
//! One assistant message carries several content blocks, so one line can
//! yield several events; lines the farm does not record (rate limits, hook
//! starts) yield none.

use serde::Deserialize;
use systemprompt::identifiers::SessionId;

pub use super::stream_payload::{cap_text, to_payload};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
pub struct Usage {
    #[serde(default, rename = "input_tokens")]
    pub input: i64,
    #[serde(default, rename = "output_tokens")]
    pub output: i64,
    #[serde(default, rename = "cache_read_input_tokens")]
    pub cache_read_input: i64,
    #[serde(default, rename = "cache_creation_input_tokens")]
    pub cache_creation_input: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct ResultSummary {
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default)]
    pub is_error: Option<bool>,
    #[serde(default)]
    pub num_turns: Option<i64>,
    #[serde(default)]
    pub duration_ms: Option<i64>,
    #[serde(default)]
    pub duration_api_ms: Option<i64>,
    #[serde(default)]
    pub total_cost_usd: Option<f64>,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub session_id: Option<SessionId>,
    #[serde(default)]
    pub usage: Usage,
}

impl ResultSummary {
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.subtype.as_deref() == Some("success") && !self.is_error.unwrap_or(false)
    }
}

// JSON: protocol boundary — a tool call's `input` follows the tool's own
// schema and is relayed verbatim into the event payload.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    Init {
        session_id: SessionId,
        model: String,
        tools: Vec<String>,
        cwd: String,
    },
    Hook {
        name: String,
        outcome: Option<String>,
    },
    AssistantText(String),
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
    Result(ResultSummary),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Line {
    System(SystemLine),
    Assistant {
        message: Message,
    },
    User {
        message: Message,
    },
    Result(ResultSummary),
    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
struct SystemLine {
    #[serde(default)]
    subtype: String,
    #[serde(default)]
    session_id: Option<SessionId>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    tools: Vec<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    hook_name: Option<String>,
    #[serde(default)]
    outcome: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Message {
    #[serde(default)]
    content: MessageContent,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MessageContent {
    Blocks(Vec<Block>),
    Text(serde::de::IgnoredAny),
}

impl Default for MessageContent {
    fn default() -> Self {
        Self::Blocks(Vec::new())
    }
}

// JSON: protocol boundary — `tool_use.input` is tool-defined.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Block {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        #[serde(default)]
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        #[serde(default)]
        content: ToolResultContent,
        #[serde(default)]
        is_error: Option<bool>,
    },
    #[serde(other)]
    Other,
}

#[derive(Debug, Default, Deserialize)]
#[serde(untagged)]
enum ToolResultContent {
    Text(String),
    Parts(Vec<TextPart>),
    #[default]
    Missing,
}

#[derive(Debug, Deserialize)]
struct TextPart {
    #[serde(default)]
    text: Option<String>,
}

impl ToolResultContent {
    fn flatten(self) -> String {
        match self {
            Self::Text(text) => text,
            Self::Parts(parts) => parts
                .into_iter()
                .filter_map(|part| part.text)
                .collect::<Vec<_>>()
                .join("\n"),
            Self::Missing => String::new(),
        }
    }
}

#[must_use]
pub fn parse_stream_line(line: &str) -> Vec<StreamEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    match serde_json::from_str::<Line>(trimmed) {
        Ok(Line::System(system)) => system_events(system),
        Ok(Line::Assistant { message } | Line::User { message }) => block_events(message),
        Ok(Line::Result(summary)) => vec![StreamEvent::Result(summary)],
        Ok(Line::Other) | Err(_) => Vec::new(),
    }
}

fn system_events(system: SystemLine) -> Vec<StreamEvent> {
    match system.subtype.as_str() {
        "init" => vec![StreamEvent::Init {
            session_id: system
                .session_id
                .unwrap_or_else(|| SessionId::new(String::new())),
            model: system.model.unwrap_or_default(),
            tools: system.tools,
            cwd: system.cwd.unwrap_or_default(),
        }],
        "hook_response" => vec![StreamEvent::Hook {
            name: system.hook_name.unwrap_or_default(),
            outcome: system.outcome,
        }],
        _ => Vec::new(),
    }
}

fn block_events(message: Message) -> Vec<StreamEvent> {
    let MessageContent::Blocks(blocks) = message.content else {
        return Vec::new();
    };
    blocks
        .into_iter()
        .filter_map(|block| match block {
            Block::Text { text } if !text.trim().is_empty() => {
                Some(StreamEvent::AssistantText(text))
            },
            Block::ToolUse { id, name, input } => Some(StreamEvent::ToolUse { id, name, input }),
            Block::ToolResult {
                tool_use_id,
                content,
                is_error,
            } => Some(StreamEvent::ToolResult {
                tool_use_id,
                content: content.flatten(),
                is_error: is_error.unwrap_or(false),
            }),
            Block::Text { .. } | Block::Other => None,
        })
        .collect()
}
