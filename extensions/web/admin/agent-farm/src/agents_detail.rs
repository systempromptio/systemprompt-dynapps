//! The single agent page's view model.

use serde::Serialize;
use crate::models::AgentOut;
use crate::views::{KeyValue, format_duration, opt_text};
use crate::agents::{scope_tone, yes_no};

#[derive(Debug, Serialize)]
pub struct NamePair {
    pub name: String,
    pub target: String,
}

#[derive(Debug, Serialize)]
pub struct AgentDetailView {
    pub slug: String,
    pub scope: String,
    pub scope_tone: &'static str,
    pub description: String,
    pub config: Vec<KeyValue>,
    pub limits: Vec<KeyValue>,
    pub instructions: Option<String>,
    pub instructions_note: Option<&'static str>,
    pub egress: Vec<String>,
    pub credentials: Vec<String>,
    pub denied_tools: Vec<String>,
    pub env_credentials: Vec<NamePair>,
    pub mcp_servers: Vec<NamePair>,
}

pub fn limit_rows(agent: &AgentOut) -> Vec<KeyValue> {
    let l = agent.default_limits.clone().unwrap_or_default();
    vec![
        KeyValue {
            label: "Timeout",
            value: format_duration(l.timeout_s.map(|v| v as f64)),
        },
        KeyValue {
            label: "CPU",
            value: opt_text(l.cpu.as_deref()),
        },
        KeyValue {
            label: "Memory",
            value: opt_text(l.memory.as_deref()),
        },
        KeyValue {
            label: "Work volume",
            value: opt_text(agent.work_size.as_deref()),
        },
    ]
}

pub fn agent_detail(agent: &AgentOut) -> AgentDetailView {
    let scope = agent.scope.clone().unwrap_or_default();
    let caps = agent.capabilities.clone().unwrap_or_default();
    let personal = scope == "personal";
    AgentDetailView {
        slug: agent.slug.clone(),
        scope_tone: scope_tone(&scope),
        description: opt_text(agent.description.as_deref()),
        config: vec![
            KeyValue {
                label: "Model",
                value: opt_text(agent.model_alias.as_deref()),
            },
            KeyValue {
                label: "Harness",
                value: opt_text(agent.harness.as_deref()),
            },
            KeyValue {
                label: "Plugin agent",
                value: opt_text(agent.plugin_agent.as_deref()),
            },
            KeyValue {
                label: "Needs a workspace",
                value: yes_no(agent.requires_workspace).to_owned(),
            },
            KeyValue {
                label: "Owner",
                value: opt_text(agent.owner.as_deref()),
            },
            KeyValue {
                label: "Image",
                value: opt_text(agent.image.as_deref()),
            },
            KeyValue {
                label: "Resource profile",
                value: opt_text(agent.profile.as_deref()),
            },
        ],
        limits: limit_rows(agent),
        instructions: if personal {
            None
        } else {
            agent.instructions.clone().filter(|i| !i.trim().is_empty())
        },
        instructions_note: personal
            .then_some("The instructions of a personal agent are private to its owner."),
        egress: caps.egress,
        credentials: caps.credentials,
        denied_tools: caps.denied_tools,
        env_credentials: agent
            .env_credentials
            .iter()
            .map(|(var, credential)| NamePair {
                name: var.clone(),
                target: credential.clone(),
            })
            .collect(),
        mcp_servers: agent
            .mcp_servers
            .iter()
            .map(|m| NamePair {
                name: m.name.clone(),
                target: opt_text(m.credential.as_deref()),
            })
            .collect(),
        scope,
    }
}
