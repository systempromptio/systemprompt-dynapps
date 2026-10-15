//! What a run's MCP calls did, read from its `tool_use` / `tool_result`
//! events: calls per server split into reads and writes, and whether the
//! run delivered its summary to `DynBrain`.
//!
//! A call is a write when a segment of its tool name is a mutating verb
//! (`orm_write`, `orm_execute_kw`, `odoo_request_upload`, a `DynBrain`
//! `*_create`); everything else is a read. The governance chain gates the
//! same dynapps-odoo verbs (services/governance/config.yaml). Delivery is a
//! `DynBrain` write whose result was not an error.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use systemprompt::agent_runs::{RunEvent, RunEventPayload};

use crate::services::farm_sync::validate::mcp_server_of;

pub const DELIVERY_SERVER: &str = "dynbrain";

const WRITE_VERBS: [&str; 18] = [
    "create", "write", "unlink", "delete", "execute", "update", "upload", "publish", "add", "save",
    "store", "ingest", "upsert", "post", "put", "set", "remove", "append",
];

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerCalls {
    pub server: String,
    pub reads: i32,
    pub writes: i32,
    pub failed: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryStatus {
    Delivered,
    Failed,
    NotAttempted,
}

impl DeliveryStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::Failed => "failed",
            Self::NotAttempted => "not_attempted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolTally {
    pub servers: Vec<ServerCalls>,
    pub delivery: DeliveryStatus,
}

impl ToolTally {
    #[must_use]
    pub fn reads(&self) -> i32 {
        self.servers.iter().map(|s| s.reads).sum()
    }

    #[must_use]
    pub fn writes(&self) -> i32 {
        self.servers.iter().map(|s| s.writes).sum()
    }
}

#[must_use]
pub fn is_write_tool(tool: &str) -> bool {
    let name = tool.rsplit("__").next().unwrap_or(tool);
    name.split(['_', '-', '.'])
        .any(|segment| WRITE_VERBS.contains(&segment.to_ascii_lowercase().as_str()))
}

#[must_use]
pub fn tally(events: &[RunEvent]) -> ToolTally {
    let failed: HashMap<&str, bool> = events
        .iter()
        .filter_map(|e| match &e.payload {
            RunEventPayload::ToolResult {
                tool_use_id,
                is_error,
                ..
            } => Some((tool_use_id.as_str(), *is_error)),
            _ => None,
        })
        .collect();
    let mut servers: BTreeMap<String, ServerCalls> = BTreeMap::new();
    let (mut delivered, mut attempted) = (false, false);
    for (id, name) in events.iter().filter_map(|e| match &e.payload {
        RunEventPayload::ToolUse {
            tool_use_id, name, ..
        } => Some((tool_use_id.as_str(), name.as_str())),
        _ => None,
    }) {
        let Some(server) = name.contains("__").then(|| mcp_server_of(name)).flatten() else {
            continue;
        };
        let entry = servers
            .entry(server.to_owned())
            .or_insert_with(|| ServerCalls {
                server: server.to_owned(),
                ..ServerCalls::default()
            });
        let write = is_write_tool(name);
        let errored = failed.get(id).copied().unwrap_or(false);
        if write {
            entry.writes += 1;
        } else {
            entry.reads += 1;
        }
        if errored {
            entry.failed += 1;
        }
        if write && server == DELIVERY_SERVER {
            attempted = true;
            delivered |= failed.get(id).is_some_and(|e| !e);
        }
    }
    let delivery = match (delivered, attempted) {
        (true, _) => DeliveryStatus::Delivered,
        (false, true) => DeliveryStatus::Failed,
        (false, false) => DeliveryStatus::NotAttempted,
    };
    ToolTally {
        servers: servers.into_values().collect(),
        delivery,
    }
}
