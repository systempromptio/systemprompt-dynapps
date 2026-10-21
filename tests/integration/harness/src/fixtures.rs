//! Sanitised client request fixtures, one per harness.

use std::path::PathBuf;

use serde_json::Value;

pub(crate) const HARNESSES: &[&str] = &[
    "claude_code",
    "claude_desktop",
    "opencode",
    "codex",
    "cursor",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Inbound {
    Anthropic,
    OpenAiChat,
    OpenAiResponses,
}

#[derive(Debug, Clone)]
pub(crate) struct Fixture {
    pub name: String,
    pub inbound: Inbound,
    pub headers: Vec<(String, String)>,
    pub body: Value,
}

impl Fixture {
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub(crate) fn inline(
        name: &str,
        inbound: Inbound,
        headers: &[(&str, &str)],
        body: Value,
    ) -> Self {
        Self {
            name: name.to_owned(),
            inbound,
            headers: headers
                .iter()
                .map(|(n, v)| ((*n).to_owned(), (*v).to_owned()))
                .collect(),
            body,
        }
    }
}

fn dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/harness")
}

// Request bodies recorded from the real CLIs (`just smoke-harness-local` with
// CAPTURE_DIR, sanitised by scripts/harness-sanitise-capture.py).
pub(crate) fn captured() -> Vec<Fixture> {
    let mut names: Vec<String> = std::fs::read_dir(dir().join("captured"))
        .expect("captured fixtures dir")
        .filter_map(Result::ok)
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|n| {
            n.strip_suffix(".json")
                .map(|stem| format!("captured/{stem}"))
        })
        .collect();
    names.sort();
    names.iter().map(|n| load(n)).collect()
}

pub(crate) fn load(name: &str) -> Fixture {
    let path = dir().join(format!("{name}.json"));
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc: Value = serde_json::from_str(&raw).expect("fixture is JSON");
    let inbound = match doc["inbound"].as_str() {
        Some("anthropic.messages") => Inbound::Anthropic,
        Some("openai.chat") => Inbound::OpenAiChat,
        Some("openai.responses") => Inbound::OpenAiResponses,
        other => panic!("{name}: unknown inbound wire {other:?}"),
    };
    let headers = doc["headers"]
        .as_object()
        .expect("fixture headers")
        .iter()
        .map(|(n, v)| (n.clone(), v.as_str().unwrap_or_default().to_owned()))
        .collect();
    Fixture {
        name: name.to_owned(),
        inbound,
        headers,
        body: doc["body"].clone(),
    }
}
