//! Shared helpers for the calendar, contacts and To Do tests of the
//! `microsoft-365` server: a Graph client on the simulated Graph, canned
//! replies, and the domain's slice of the pinned Belgian surface.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;
use systemprompt::identifiers::{AgentName, ContextId, SessionId, TraceId};
use systemprompt::models::execution::context::RequestContext;
use systemprompt_mcp_microsoft_365::graph::{GraphClient, GraphError};
use systemprompt_mcp_microsoft_365::tools::list_tools;

use crate::m365_parity::served_destructive;
use crate::m365_support::{FakeTokens, Reply, Sim, client, fast_limits};

pub(crate) fn graph(sim: &Sim) -> GraphClient {
    client(
        &sim.graph_url(),
        FakeTokens::new(&["graph-token"]),
        fast_limits(),
    )
}

pub(crate) fn args<T: serde::de::DeserializeOwned>(value: Value) -> T {
    serde_json::from_value(value).unwrap()
}

pub(crate) fn empty(status: u16) -> Reply {
    Reply {
        status,
        headers: Vec::new(),
        body: Vec::new(),
    }
}

pub(crate) fn is_invalid(error: &GraphError) -> bool {
    matches!(error, GraphError::Invalid(_))
}

pub(crate) fn context() -> RequestContext {
    RequestContext::new(
        SessionId::new("m365-pim-session"),
        TraceId::new("m365-pim-trace"),
        ContextId::try_new("00000000-0000-4000-8000-00000000365a").expect("context"),
        AgentName::try_new("m365-pim-agent").expect("agent"),
        systemprompt::identifiers::Actor::user(systemprompt::identifiers::UserId::new(
            "00000000-0000-4000-8000-00000000365a",
        )),
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Pinned {
    name: String,
    title: String,
    domain: String,
    read_only_hint: bool,
    destructive_hint: bool,
    idempotent_hint: bool,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    tools: Vec<Pinned>,
}

type Hints = (String, bool, bool, bool);

// Every Belgian tool of `domain` must be registered, with its pinned title
// and hints (save the hint deviations `m365_parity` records), and the
// description a model reads must not be empty.
pub(crate) fn assert_domain_registered(domain: &str, expected: usize) {
    let fixture: Fixture =
        serde_json::from_str(include_str!("../fixtures/m365_tools.json")).unwrap();
    let pinned: BTreeMap<String, Hints> = fixture
        .tools
        .into_iter()
        .filter(|tool| tool.domain == domain)
        .map(|t| {
            let destructive = served_destructive(&t.name, t.destructive_hint);
            let hints = (t.title, t.read_only_hint, destructive, t.idempotent_hint);
            (t.name, hints)
        })
        .collect();
    assert_eq!(pinned.len(), expected, "pinned {domain} tools");
    let registered: BTreeMap<String, Hints> = list_tools()
        .into_iter()
        .filter(|tool| pinned.contains_key(tool.name.as_ref()))
        .map(|tool| {
            assert!(
                tool.description.as_deref().is_some_and(|d| !d.is_empty()),
                "{} has a description",
                tool.name
            );
            let a = tool.annotations.unwrap();
            let hints = (
                a.title.unwrap(),
                a.read_only_hint.unwrap(),
                a.destructive_hint.unwrap(),
                a.idempotent_hint.unwrap(),
            );
            (tool.name.to_string(), hints)
        })
        .collect();
    assert_eq!(registered, pinned);
}
