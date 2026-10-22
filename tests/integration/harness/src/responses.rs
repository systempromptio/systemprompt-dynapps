//! The response direction: an upstream reply carrying thinking and two
//! parallel tool calls, buffered and streamed, on every provider wire, must
//! reach every client format with both calls and the reasoning intact.

use serde_json::{Value, json};
use systemprompt::manifest::services::ProviderEntry;
use systemprompt::wire::WireProtocol;

use crate::dispatch::{Reply, dispatch_reply};
use crate::fixtures::{Fixture, Inbound, load};
use crate::providers::catalogue;

const THOUGHT: &str = "pondering-both-files";
const CLIENTS: &[&str] = &["claude_code", "opencode", "codex", "cursor"];

fn args(file: &str) -> String {
    json!({ "filePath": file }).to_string()
}

fn buffered(wire: WireProtocol) -> Value {
    match wire {
        WireProtocol::Anthropic => json!({
            "id": "msg_r", "type": "message", "role": "assistant", "model": "m",
            "content": [
                { "type": "thinking", "thinking": THOUGHT, "signature": "sig_r" },
                { "type": "tool_use", "id": "toolu_r1", "name": "read", "input": { "filePath": "a.rs" } },
                { "type": "tool_use", "id": "toolu_r2", "name": "read", "input": { "filePath": "b.rs" } }
            ],
            "stop_reason": "tool_use", "stop_sequence": null,
            "usage": { "input_tokens": 9, "output_tokens": 9 },
        }),
        WireProtocol::OpenAiChat => json!({
            "id": "chatcmpl-r", "object": "chat.completion", "created": 0, "model": "m",
            "choices": [{ "index": 0, "finish_reason": "tool_calls", "message": {
                "role": "assistant", "content": null, "reasoning_content": THOUGHT,
                "tool_calls": [
                    { "id": "call_r1", "type": "function", "function": { "name": "read", "arguments": args("a.rs") } },
                    { "id": "call_r2", "type": "function", "function": { "name": "read", "arguments": args("b.rs") } }
                ] } }],
            "usage": { "prompt_tokens": 9, "completion_tokens": 9, "total_tokens": 18 },
        }),
        WireProtocol::OpenAiResponses => json!({
            "id": "resp_r", "object": "response", "model": "m", "status": "completed",
            "output": [
                { "type": "reasoning", "id": "rs_r", "summary": [{ "type": "summary_text", "text": THOUGHT }] },
                { "type": "function_call", "id": "fc_1", "call_id": "call_r1", "name": "read", "arguments": args("a.rs"), "status": "completed" },
                { "type": "function_call", "id": "fc_2", "call_id": "call_r2", "name": "read", "arguments": args("b.rs"), "status": "completed" }
            ],
            "usage": { "input_tokens": 9, "output_tokens": 9, "total_tokens": 18 },
        }),
        WireProtocol::Gemini => json!({
            "candidates": [{ "finishReason": "STOP", "content": { "role": "model", "parts": [
                { "text": THOUGHT, "thought": true },
                { "functionCall": { "name": "read", "args": { "filePath": "a.rs" } } },
                { "functionCall": { "name": "read", "args": { "filePath": "b.rs" } } }
            ] } }],
            "usageMetadata": { "promptTokenCount": 9, "candidatesTokenCount": 9, "totalTokenCount": 18 },
        }),
    }
}

fn frames(events: &[(Option<&str>, Value)]) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for (name, data) in events {
        if let Some(name) = name {
            writeln!(out, "event: {name}").unwrap();
        }
        write!(out, "data: {data}\n\n").unwrap();
    }
    out
}

fn streamed(wire: WireProtocol) -> String {
    match wire {
        WireProtocol::Anthropic => {
            let mut events = vec![
                (
                    Some("message_start"),
                    json!({ "type": "message_start", "message": {
                "id": "msg_r", "type": "message", "role": "assistant", "model": "m", "content": [],
                "stop_reason": null, "usage": { "input_tokens": 9, "output_tokens": 1 } } }),
                ),
                (
                    Some("content_block_start"),
                    json!({ "type": "content_block_start", "index": 0, "content_block": { "type": "thinking", "thinking": "" } }),
                ),
                (
                    Some("content_block_delta"),
                    json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "thinking_delta", "thinking": THOUGHT } }),
                ),
                (
                    Some("content_block_delta"),
                    json!({ "type": "content_block_delta", "index": 0, "delta": { "type": "signature_delta", "signature": "sig_r" } }),
                ),
                (
                    Some("content_block_stop"),
                    json!({ "type": "content_block_stop", "index": 0 }),
                ),
            ];
            for (index, id, file) in [(1, "toolu_r1", "a.rs"), (2, "toolu_r2", "b.rs")] {
                events.push((Some("content_block_start"), json!({ "type": "content_block_start", "index": index,
                    "content_block": { "type": "tool_use", "id": id, "name": "read", "input": {} } })));
                events.push((
                    Some("content_block_delta"),
                    json!({ "type": "content_block_delta", "index": index,
                    "delta": { "type": "input_json_delta", "partial_json": args(file) } }),
                ));
                events.push((
                    Some("content_block_stop"),
                    json!({ "type": "content_block_stop", "index": index }),
                ));
            }
            events.push((Some("message_delta"), json!({ "type": "message_delta",
                "delta": { "stop_reason": "tool_use", "stop_sequence": null }, "usage": { "output_tokens": 9 } })));
            events.push((Some("message_stop"), json!({ "type": "message_stop" })));
            frames(&events)
        },
        WireProtocol::OpenAiChat => {
            let chunk = |delta: Value, finish: Value| {
                json!({ "id": "chatcmpl-r", "object": "chat.completion.chunk",
                "created": 0, "model": "m", "choices": [{ "index": 0, "delta": delta, "finish_reason": finish }] })
            };
            let mut events = vec![
                (
                    None,
                    chunk(
                        json!({ "role": "assistant", "reasoning_content": THOUGHT }),
                        Value::Null,
                    ),
                ),
                (
                    None,
                    chunk(
                        json!({ "tool_calls": [{ "index": 0, "id": "call_r1", "type": "function",
                    "function": { "name": "read", "arguments": args("a.rs") } }] }),
                        Value::Null,
                    ),
                ),
                (
                    None,
                    chunk(
                        json!({ "tool_calls": [{ "index": 1, "id": "call_r2", "type": "function",
                    "function": { "name": "read", "arguments": args("b.rs") } }] }),
                        Value::Null,
                    ),
                ),
                (None, chunk(json!({}), json!("tool_calls"))),
            ];
            events.push((None, json!({ "id": "chatcmpl-r", "object": "chat.completion.chunk", "created": 0, "model": "m",
                "choices": [], "usage": { "prompt_tokens": 9, "completion_tokens": 9, "total_tokens": 18 } })));
            frames(&events) + "data: [DONE]\n\n"
        },
        WireProtocol::OpenAiResponses => {
            let done = buffered(WireProtocol::OpenAiResponses);
            let mut events = vec![(
                Some("response.created"),
                json!({ "type": "response.created",
                "response": { "id": "resp_r", "object": "response", "model": "m", "status": "in_progress", "output": [] } }),
            )];
            for (index, item) in done["output"].as_array().unwrap().iter().enumerate() {
                events.push((Some("response.output_item.added"), json!({ "type": "response.output_item.added", "output_index": index, "item": item })));
                if item["type"] == "reasoning" {
                    events.push((Some("response.reasoning_summary_text.delta"), json!({ "type": "response.reasoning_summary_text.delta",
                        "item_id": "rs_r", "output_index": index, "summary_index": 0, "delta": THOUGHT })));
                } else {
                    events.push((
                        Some("response.function_call_arguments.delta"),
                        json!({ "type": "response.function_call_arguments.delta",
                        "item_id": item["id"], "output_index": index, "delta": item["arguments"] }),
                    ));
                }
                events.push((Some("response.output_item.done"), json!({ "type": "response.output_item.done", "output_index": index, "item": item })));
            }
            events.push((
                Some("response.completed"),
                json!({ "type": "response.completed", "response": done }),
            ));
            frames(&events)
        },
        WireProtocol::Gemini => frames(&[(None, buffered(WireProtocol::Gemini))]),
    }
}

fn with_stream(fixture: &Fixture, stream: bool) -> Fixture {
    let mut fixture = fixture.clone();
    fixture.body["stream"] = Value::Bool(stream);
    fixture
}

fn check(reply: &str) -> Result<(), String> {
    for needle in ["a.rs", "b.rs"] {
        if !reply.contains(needle) {
            return Err(format!("tool call for {needle} lost"));
        }
    }
    if !reply.contains(THOUGHT) {
        return Err("reasoning lost".to_owned());
    }
    Ok(())
}

async fn cell(fixture: &Fixture, provider: &ProviderEntry, stream: bool) -> Result<(), String> {
    let model = provider.models.first().ok_or("no models")?;
    let reply = if stream {
        Reply::sse(&streamed(provider.wire))
    } else {
        Reply::json(&buffered(provider.wire))
    };
    let captured = dispatch_reply(
        &with_stream(fixture, stream),
        provider,
        model.id.as_str(),
        &reply,
    )
    .await?;
    if stream
        && fixture.inbound == Inbound::OpenAiChat
        && !captured
            .client_reply
            .contains(r#""tool_calls":[{"index":0"#)
    {
        return Err("first streamed tool_call is not index 0".to_owned());
    }
    check(&captured.client_reply).map_err(|e| {
        format!(
            "{e}: {}",
            captured.client_reply.chars().take(600).collect::<String>()
        )
    })
}

#[tokio::test]
async fn parallel_calls_and_reasoning_reach_every_client_buffered_and_streamed() {
    let providers = catalogue();
    let mut failures = Vec::new();
    let mut cells = 0;
    for client in CLIENTS {
        let fixture = load(client);
        for provider in &providers {
            for stream in [false, true] {
                cells += 1;
                if let Err(e) = cell(&fixture, provider, stream).await {
                    let mode = if stream { "stream" } else { "buffered" };
                    failures.push(format!(
                        "{} <- {} ({mode}): {e}",
                        fixture.name,
                        provider.name.as_str()
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {cells} cells failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
