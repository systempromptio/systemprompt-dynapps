//! Typed ids and the farm run and eval lifecycles. The run event contract,
//! the host kinds and the executor's own status and error class are core's
//! (`systemprompt::agent_runs`), re-exported here.
//!
//! Each status enum round-trips through the `TEXT` column its table checks,
//! so `as_str` is the only spelling written and `parse` the only one read.
//! The farm's `RunStatus` mirrors the executor's lifecycle one to one
//! (`RunStatus::from_agent_run`).

use std::fmt;

use serde::{Deserialize, Serialize};
pub use systemprompt::agent_runs::{
    ErrorClass, HookDecision, HostKind, RunEvent, RunEventKind, RunEventPayload,
    RunStatus as AgentRunStatus, RunnerLevel,
};
use systemprompt_web_shared::web_define_id;

web_define_id!(ClientId);
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

    #[must_use]
    pub const fn from_agent_run(status: AgentRunStatus) -> Self {
        match status {
            AgentRunStatus::Queued => Self::Queued,
            AgentRunStatus::Preparing => Self::Preparing,
            AgentRunStatus::Running => Self::Running,
            AgentRunStatus::Succeeded => Self::Succeeded,
            AgentRunStatus::Failed => Self::Failed,
            AgentRunStatus::Cancelled => Self::Cancelled,
            AgentRunStatus::TimedOut => Self::TimedOut,
        }
    }
}

text_enum!(TriggeredVia {
    Dashboard => "dashboard",
    Api => "api",
    Eval => "eval",
    Mcp => "mcp",
    Console => "console",
    Demo => "demo",
    Scheduler => "scheduler",
});

impl TriggeredVia {
    pub const ALL: [Self; 7] = [
        Self::Dashboard,
        Self::Api,
        Self::Eval,
        Self::Mcp,
        Self::Console,
        Self::Demo,
        Self::Scheduler,
    ];
}

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
