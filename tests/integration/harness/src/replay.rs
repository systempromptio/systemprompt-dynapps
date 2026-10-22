//! Production replay: the turn normaliser must leave already-valid traffic
//! unchanged.
//!
//! Opt-in (`#[ignore]`): it reads a JSONL dump of `infra logs audit
//! --messages` records named by `HARNESS_REPLAY_FILE`, which holds production
//! content and is never committed. The audit stores each message flattened
//! (`[tool_use:name {json}]` markers, tool results as plain text), so ids are
//! reconstructed in order: a `tool` message, or the user turn right after an
//! assistant turn with tool calls, answers the open calls. A record is valid
//! when that reconstruction already alternates strictly; for every valid
//! record, `normalize_turns` must return it unchanged.
//!
//!     HARNESS_REPLAY_FILE=/path/audit.jsonl cargo nextest run \
//!       --manifest-path tests/Cargo.toml -p harness-integration-tests \
//!       --run-ignored only -E 'test(replay)' --no-capture

use serde_json::Value;
use systemprompt::wire::canonical::{
    CanonicalContent, CanonicalMessage, Role, TurnMerge, normalize_turns,
};

fn tool_uses(text: &str) -> (String, Vec<(String, Value)>) {
    let mut prose = String::new();
    let mut calls = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("[tool_use:")
            && let Some((name, input)) = rest.trim_end_matches(']').split_once(' ')
        {
            calls.push((
                name.to_owned(),
                serde_json::from_str(input).unwrap_or(Value::Null),
            ));
        } else {
            prose.push_str(line);
            prose.push('\n');
        }
    }
    (prose.trim().to_owned(), calls)
}

fn result(id: &str, text: &str) -> CanonicalContent {
    CanonicalContent::ToolResult {
        tool_use_id: id.to_owned(),
        content: vec![CanonicalContent::text(text)],
        is_error: false,
        structured_content: None,
        meta: None,
        cache_control: None,
    }
}

fn reconstruct(messages: &[Value]) -> Vec<CanonicalMessage> {
    let mut out: Vec<CanonicalMessage> = Vec::new();
    let mut open: Vec<String> = Vec::new();
    for (i, m) in messages.iter().enumerate() {
        let text = m["content"].as_str().unwrap_or_default();
        match m["role"].as_str().unwrap_or_default() {
            "assistant" => {
                let (prose, calls) = tool_uses(text);
                let mut content = Vec::new();
                if !prose.is_empty() {
                    content.push(CanonicalContent::text(prose));
                }
                open.clear();
                for (n, (name, input)) in calls.into_iter().enumerate() {
                    let id = format!("call_{i}_{n}");
                    open.push(id.clone());
                    content.push(CanonicalContent::ToolUse {
                        id,
                        name,
                        input,
                        signature: None,
                        cache_control: None,
                    });
                }
                out.push(CanonicalMessage {
                    role: Role::Assistant,
                    content,
                });
            },
            "tool" => {
                let id = if open.is_empty() {
                    format!("orphan_{i}")
                } else {
                    open.remove(0)
                };
                out.push(CanonicalMessage {
                    role: Role::Tool,
                    content: vec![result(&id, text)],
                });
            },
            "system" => out.push(CanonicalMessage {
                role: Role::System,
                content: vec![CanonicalContent::text(text)],
            }),
            _ => {
                let mut content: Vec<CanonicalContent> = std::mem::take(&mut open)
                    .into_iter()
                    .map(|id| result(&id, text))
                    .collect();
                if content.is_empty() {
                    content.push(CanonicalContent::text(text));
                }
                out.push(CanonicalMessage {
                    role: Role::User,
                    content,
                });
            },
        }
    }
    out
}

// Parts in order, labelled by side, with a boundary only around assistant
// turns: how the OpenAI wires render a run of tool and user messages does not
// depend on how that run was split into messages.
fn fingerprint(messages: &[CanonicalMessage], merge: TurnMerge) -> Vec<String> {
    let mut out = Vec::new();
    let mut previous = "";
    for m in messages {
        let side = side(m.role, merge);
        if side != previous || side == "a" {
            out.push(format!("--{side}"));
        }
        previous = side;
        for c in &m.content {
            out.push(match c {
                CanonicalContent::Text { text, .. } => format!("text:{}", text.len()),
                CanonicalContent::ToolUse { id, .. } => format!("use:{id}"),
                CanonicalContent::ToolResult { tool_use_id, .. } => format!("result:{tool_use_id}"),
                other => format!("{other:?}").chars().take(16).collect(),
            });
        }
    }
    out
}

fn side(role: Role, merge: TurnMerge) -> &'static str {
    match (role, merge) {
        (Role::Assistant, _) => "a",
        (Role::System, TurnMerge::UserAndTool) => "s",
        _ => "u",
    }
}

fn strictly_valid(messages: &[CanonicalMessage], merge: TurnMerge) -> bool {
    // On the OpenAI wires consecutive tool and user messages are the native
    // shape; on Anthropic every turn must alternate.
    let sides: Vec<&str> = messages.iter().map(|m| side(m.role, merge)).collect();
    let tolerated = |s: &str| s == "s" || (merge == TurnMerge::UserAndTool && s == "u");
    if sides.windows(2).any(|w| w[0] == w[1] && !tolerated(w[0])) {
        return false;
    }
    messages.iter().all(|m| {
        m.content
            .iter()
            .all(|c| !matches!(c, CanonicalContent::ToolResult { tool_use_id, .. } if tool_use_id.starts_with("orphan_")))
    }) && messages.last().is_none_or(|last| {
        last.role != Role::Assistant
            || !last.content.iter().any(|c| matches!(c, CanonicalContent::ToolUse { .. }))
    })
}

#[test]
#[ignore = "reads a production audit dump named by HARNESS_REPLAY_FILE"]
fn replay_leaves_valid_production_histories_unchanged() {
    let path = std::env::var("HARNESS_REPLAY_FILE").expect("set HARNESS_REPLAY_FILE");
    let raw = std::fs::read_to_string(&path).expect("read the replay dump");
    let mut tally: std::collections::BTreeMap<(String, &str), usize> =
        std::collections::BTreeMap::new();
    let mut changed: Vec<String> = Vec::new();
    for line in raw.lines().filter(|l| !l.trim().is_empty()) {
        let record: Value = serde_json::from_str(line).expect("replay record");
        let client = record["client"].as_str().unwrap_or("?").to_owned();
        let messages = reconstruct(record["messages"].as_array().map_or(&[][..], Vec::as_slice));
        // OpenAI-wire clients keep tool messages apart; Anthropic clients send
        // every non-assistant turn as user.
        let merge = if client == "opencode" {
            TurnMerge::UserAndTool
        } else {
            TurnMerge::UserSide
        };
        if !strictly_valid(&messages, merge) {
            *tally.entry((client, "invalid (normalised)")).or_default() += 1;
            continue;
        }
        let before = fingerprint(&messages, merge);
        let after = fingerprint(&normalize_turns(&messages, merge), merge);
        if before == after {
            *tally.entry((client, "valid, unchanged")).or_default() += 1;
        } else {
            *tally.entry((client.clone(), "valid, CHANGED")).or_default() += 1;
            changed.push(format!("{client} {}", record["id"]));
        }
    }
    for ((client, verdict), n) in &tally {
        println!("{client:15} {verdict:22} {n}");
    }
    assert!(changed.is_empty(), "valid histories changed: {changed:?}");
}
