//! Typed ids, the run and eval lifecycles, and the run event payload parsed
//! from Claude Code's `stream-json` output.
//!
//! Each status enum round-trips through the `TEXT` column its table checks,
//! so `as_str` is the only spelling written and `parse` the only one read.

use std::fmt;

use serde::{Deserialize, Serialize};
use systemprompt::identifiers::SessionId;
use systemprompt_web_shared::web_define_id;

web_define_id!(FarmId);
web_define_id!(FarmAgentId);
web_define_id!(FarmRunId);
web_define_id!(EvalSuiteId);
web_define_id!(EvalRunId);
web_define_id!(EvalResultId);

macro_rules! text_enum {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }

            #[must_use]
            pub fn parse(value: &str) -> Option<Self> {
                match value {
                    $($text => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

text_enum!(RunStatus {
    Queued => "queued",
    Preparing => "preparing",
    Running => "running",
    Succeeded => "succeeded",
    Failed => "failed",
    Cancelled => "cancelled",
    TimedOut => "timed_out",
});

impl RunStatus {
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::TimedOut
        )
    }
}

text_enum!(TriggeredVia {
    Dashboard => "dashboard",
    Api => "api",
    Eval => "eval",
});

text_enum!(RunEventKind {
    Init => "init",
    AssistantText => "assistant_text",
    ToolUse => "tool_use",
    ToolResult => "tool_result",
    Hook => "hook",
    Result => "result",
    Stderr => "stderr",
    Runner => "runner",
});

text_enum!(EvalRunStatus {
    Queued => "queued",
    Running => "running",
    Completed => "completed",
    Failed => "failed",
    Cancelled => "cancelled",
});

text_enum!(EvalResultStatus {
    Pending => "pending",
    Running => "running",
    Passed => "passed",
    Failed => "failed",
    Errored => "errored",
});

// JSON: protocol boundary — a tool call's `input` is whatever the tool's own
// schema says, relayed verbatim from Claude Code's stream-json and stored in
// the `payload` JSONB column; the farm never interprets it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RunEventPayload {
    Init {
        session_id: SessionId,
        model: String,
        tools: Vec<String>,
        cwd: String,
    },
    AssistantText {
        text: String,
    },
    ToolUse {
        tool_use_id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
    Hook {
        name: String,
        decision: Option<String>,
    },
    Result {
        subtype: String,
        is_error: bool,
        num_turns: i64,
        duration_ms: i64,
        cost_usd: Option<f64>,
        text: Option<String>,
    },
    Stderr {
        text: String,
    },
    Runner {
        message: String,
    },
}

impl RunEventPayload {
    #[must_use]
    pub const fn kind(&self) -> RunEventKind {
        match self {
            Self::Init { .. } => RunEventKind::Init,
            Self::AssistantText { .. } => RunEventKind::AssistantText,
            Self::ToolUse { .. } => RunEventKind::ToolUse,
            Self::ToolResult { .. } => RunEventKind::ToolResult,
            Self::Hook { .. } => RunEventKind::Hook,
            Self::Result { .. } => RunEventKind::Result,
            Self::Stderr { .. } => RunEventKind::Stderr,
            Self::Runner { .. } => RunEventKind::Runner,
        }
    }
}
