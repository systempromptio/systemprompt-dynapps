//! The remote execution commands the agent page shows: one `curl` that queues
//! a run through the public farm API as the token's owner, and one that reads
//! the run's events back. Both are built server-side so the base URL is the
//! gateway's external address, never whatever host the browser used.

use crate::types::{FarmAgentId, FarmId};
use serde::Serialize;
use serde_json::json;

const API_PATH: &str = "/api/public/agent-farm";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RemoteCommands {
    pub launch: String,
    pub poll: String,
}

fn shell_single_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

#[must_use]
pub fn launch_body(farm_id: &FarmId, agent_id: &FarmAgentId, prompt: &str) -> String {
    json!({ "farm": farm_id.as_str(), "agent": agent_id.as_str(), "prompt": prompt }).to_string()
}

#[must_use]
pub fn remote_commands(
    base: &str,
    farm_id: &FarmId,
    agent_id: &FarmAgentId,
    prompt: &str,
) -> RemoteCommands {
    let base = base.trim_end_matches('/');
    let body = shell_single_quote(&launch_body(farm_id, agent_id, prompt));
    let launch = format!(
        "curl -X POST {base}{API_PATH}/runs \\\n  -H \"Authorization: Bearer $SYSTEMPROMPT_TOKEN\" \\\n  -H \"Content-Type: application/json\" \\\n  -d {body}"
    );
    let poll = format!(
        "curl {base}{API_PATH}/runs/$RUN_ID/events \\\n  -H \"Authorization: Bearer $SYSTEMPROMPT_TOKEN\""
    );
    RemoteCommands { launch, poll }
}
