//! Production shapes that once broke an upstream, pinned per provider.

use serde_json::{Value, json};
use systemprompt::wire::WireProtocol;

use crate::dispatch::dispatch_model;
use crate::fixtures::{Fixture, Inbound};
use crate::providers::named;
use crate::validate;

const OPENCODE: &[(&str, &str)] = &[
    ("user-agent", "opencode/1.0.150 ai-sdk/provider-utils/3.0.0"),
    ("x-opencode-session", "ses_regression000000000001"),
];

fn chat(messages: &Value) -> Fixture {
    Fixture::inline(
        "opencode-inline",
        Inbound::OpenAiChat,
        OPENCODE,
        json!({ "model": "any", "stream": false, "max_tokens": 4096, "messages": messages }),
    )
}

fn parallel_reads() -> Value {
    json!([
        { "role": "system", "content": "You are opencode." },
        { "role": "user", "content": "Read both files." },
        { "role": "assistant", "content": "", "tool_calls": [
            { "id": "toolu_p1", "type": "function", "function": { "name": "read", "arguments": "{\"filePath\":\"a.rs\"}" } },
            { "id": "toolu_p2", "type": "function", "function": { "name": "read", "arguments": "{\"filePath\":\"b.rs\"}" } }
        ] },
        { "role": "tool", "tool_call_id": "toolu_p1", "content": "fn a() {}" },
        { "role": "tool", "tool_call_id": "toolu_p2", "content": "fn b() {}" }
    ])
}

async fn upstream_body(fixture: &Fixture, provider: &str, model: &str) -> Value {
    let entry = named(provider);
    let captured = dispatch_model(fixture, &entry, model)
        .await
        .unwrap_or_else(|e| panic!("{provider}/{model}: {e}"));
    if let Err(e) = validate::body(entry.wire, &captured.body) {
        panic!("{provider}/{model}: {e}\n{:#}", captured.body);
    }
    captured.body
}

// Request `ba6f9208`: a Claude-started `OpenCode` session with two parallel
// tool calls, switched to `deepseek-flash`. The two `role: "tool"` messages
// must reach `DeepSeek` as ONE user turn holding both results.
#[tokio::test]
async fn ba6f9208_parallel_tool_results_reach_deepseek_as_one_turn() {
    let body = upstream_body(&chat(&parallel_reads()), "deepseek/api", "deepseek-flash").await;
    let messages = body["messages"].as_array().unwrap();
    let last = messages.last().unwrap();
    assert_eq!(last["role"], "user");
    let answered: Vec<&str> = last["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|b| b["tool_use_id"].as_str())
        .collect();
    assert_eq!(answered, ["toolu_p1", "toolu_p2"]);
    for provider_model in [
        ("opencode-go/messages", "go-minimax-m3"),
        ("anthropic/api", "claude-haiku-5-5"),
    ] {
        upstream_body(&chat(&parallel_reads()), provider_model.0, provider_model.1).await;
    }
}

fn interrupted_text(result: &Value) -> bool {
    result.to_string().contains("tool call interrupted")
}

// A tool call the user interrupted: the next turn is plain user text. Every
// strict upstream must see the call answered by a synthetic error result.
#[tokio::test]
async fn interrupted_tool_call_is_answered_with_an_error_result() {
    let mut history = parallel_reads();
    let messages = history.as_array_mut().unwrap();
    messages.truncate(3);
    messages.push(json!({ "role": "user", "content": "Stop, do something else." }));
    let body = upstream_body(&chat(&history), "deepseek/api", "deepseek-flash").await;
    let turn = &body["messages"].as_array().unwrap()[2];
    let results: Vec<&Value> = turn["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["type"] == "tool_result")
        .collect();
    assert_eq!(results.len(), 2, "{turn:#}");
    assert!(
        results
            .iter()
            .all(|r| r["is_error"] == true && interrupted_text(r))
    );
    let chat_body = upstream_body(&chat(&history), "opencode-go/chat", "go-kimi-k3").await;
    let tool_messages: Vec<&Value> = chat_body["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|m| m["role"] == "tool")
        .collect();
    assert_eq!(tool_messages.len(), 2, "{chat_body:#}");
    assert!(tool_messages.iter().all(|m| interrupted_text(m)));
}

// Claude Code's own interrupted shape on the anthropic passthrough lane: the
// raw body is forwarded only when strict, so this one is rebuilt.
#[tokio::test]
async fn anthropic_passthrough_with_unanswered_tool_use_is_repaired() {
    let fixture = Fixture::inline(
        "claude-code-interrupted",
        Inbound::Anthropic,
        &[("anthropic-version", "2023-06-01")],
        json!({ "model": "any", "max_tokens": 1024, "messages": [
            { "role": "user", "content": "List files." },
            { "role": "assistant", "content": [{ "type": "tool_use", "id": "toolu_i1", "name": "Bash", "input": { "command": "ls" } }] },
            { "role": "user", "content": "[Request interrupted by user]" },
            { "role": "user", "content": [{ "type": "tool_result", "tool_use_id": "toolu_ghost", "content": "orphan" }] }
        ] }),
    );
    let body = upstream_body(&fixture, "deepseek/api", "deepseek-flash").await;
    let text = body.to_string();
    assert!(text.contains("tool call interrupted"), "{body:#}");
    assert!(
        !text.contains("toolu_ghost"),
        "orphan result survived: {body:#}"
    );
}

// One `OpenCode` session hopping Claude → `DeepSeek` → `OpenCode` Go →
// Claude, each hop carrying the whole history so far.
#[tokio::test]
async fn one_session_switching_providers_stays_well_formed() {
    let hops = [
        ("anthropic/api", "claude-haiku-5-5"),
        ("deepseek/api", "deepseek-flash"),
        ("opencode-go/chat", "go-kimi-k3"),
        ("opencode-go/messages", "go-qwen3.8-flash"),
        ("anthropic/api", "claude-sonnet-5-5"),
    ];
    let mut history = parallel_reads();
    for (turn, (provider, model)) in hops.iter().enumerate() {
        let body = upstream_body(&chat(&history), provider, model).await;
        let wire = named(provider).wire;
        if wire == WireProtocol::Anthropic {
            assert!(
                body.get("system").is_some(),
                "{provider}: system prompt dropped"
            );
        }
        let messages = history.as_array_mut().unwrap();
        messages.push(json!({
            "role": "assistant",
            "content": format!("answer {turn} from {model}"),
            "reasoning_content": "thinking without a signature",
            "tool_calls": [{ "id": format!("call_hop{turn}"), "type": "function",
                "function": { "name": "read", "arguments": "{\"filePath\":\"c.rs\"}" } }]
        }));
        messages.push(json!({ "role": "tool", "tool_call_id": format!("call_hop{turn}"), "content": "fn c() {}" }));
        messages.push(json!({ "role": "user", "content": format!("next question {turn}") }));
    }
}
