//! Drives one fixture through the gateway's real inbound parser and outbound
//! adapter to a wiremock upstream, capturing what the provider receives.

use std::sync::Once;

use bytes::Bytes;
use futures_util::StreamExt;
use serde_json::{Value, json};
use systemprompt::ai::UpstreamTarget;
use systemprompt::gateway::protocol::canonical::ContentBlockKind;
use systemprompt::gateway::protocol::inbound::anthropic_messages::AnthropicMessagesInbound;
use systemprompt::gateway::protocol::inbound::openai_chat::OpenAiChatInbound;
use systemprompt::gateway::protocol::inbound::openai_responses::OpenAiResponsesInbound;
use systemprompt::gateway::{
    CanonicalEvent, GatewayUpstreamRegistry, InboundAdapter, OutboundCtx, OutboundOutcome,
};
use systemprompt::wire::WireProtocol;
use systemprompt::wire::anthropic::is_forwardable_request_header;
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::fixtures::{Fixture, Inbound};
use crate::providers::{Lane, pointed_at};

pub(crate) const KEY: &str = "sk-harness-fixture";
pub(crate) const REPLY_TEXT: &str = "harness-ok";

#[derive(Debug)]
pub(crate) struct Captured {
    pub body: Value,
    pub headers: Vec<(String, String)>,
    pub client_reply: String,
}

impl Captured {
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

fn inbound_adapter(inbound: Inbound) -> Box<dyn InboundAdapter> {
    match inbound {
        Inbound::Anthropic => Box::new(AnthropicMessagesInbound),
        Inbound::OpenAiChat => Box::new(OpenAiChatInbound),
        Inbound::OpenAiResponses => Box::new(OpenAiResponsesInbound),
    }
}

fn upstream_reply(wire: WireProtocol) -> Value {
    match wire {
        WireProtocol::Anthropic => json!({
            "id": "msg_harness", "type": "message", "role": "assistant", "model": "m",
            "content": [{ "type": "text", "text": REPLY_TEXT }],
            "stop_reason": "end_turn", "stop_sequence": null,
            "usage": { "input_tokens": 3, "output_tokens": 2 },
        }),
        WireProtocol::OpenAiChat => json!({
            "id": "chatcmpl-harness", "object": "chat.completion", "created": 0, "model": "m",
            "choices": [{ "index": 0, "finish_reason": "stop",
                "message": { "role": "assistant", "content": REPLY_TEXT } }],
            "usage": { "prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5 },
        }),
        WireProtocol::OpenAiResponses => json!({
            "id": "resp_harness", "object": "response", "model": "m", "status": "completed",
            "output": [{ "type": "message", "id": "msg_1", "role": "assistant", "status": "completed",
                "content": [{ "type": "output_text", "text": REPLY_TEXT, "annotations": [] }] }],
            "usage": { "input_tokens": 3, "output_tokens": 2, "total_tokens": 5 },
        }),
        WireProtocol::Gemini => json!({
            "candidates": [{ "content": { "role": "model", "parts": [{ "text": REPLY_TEXT }] },
                "finishReason": "STOP" }],
            "usageMetadata": { "promptTokenCount": 3, "candidatesTokenCount": 2, "totalTokenCount": 5 },
        }),
    }
}

// The same one-line reply as `upstream_reply`, framed as the wire streams it.
pub(crate) fn upstream_stream(wire: WireProtocol) -> String {
    let frame = |event: Option<&str>, data: Value| {
        let name = event.map_or_else(String::new, |name| format!("event: {name}\n"));
        format!("{name}data: {data}\n\n")
    };
    match wire {
        WireProtocol::Anthropic => [
            frame(Some("message_start"), json!({ "type": "message_start", "message": {
                "id": "msg_harness", "type": "message", "role": "assistant", "model": "m",
                "content": [], "stop_reason": null, "usage": { "input_tokens": 3, "output_tokens": 1 } } })),
            frame(Some("content_block_start"), json!({ "type": "content_block_start", "index": 0,
                "content_block": { "type": "text", "text": "" } })),
            frame(Some("content_block_delta"), json!({ "type": "content_block_delta", "index": 0,
                "delta": { "type": "text_delta", "text": REPLY_TEXT } })),
            frame(Some("content_block_stop"), json!({ "type": "content_block_stop", "index": 0 })),
            frame(Some("message_delta"), json!({ "type": "message_delta",
                "delta": { "stop_reason": "end_turn", "stop_sequence": null }, "usage": { "output_tokens": 2 } })),
            frame(Some("message_stop"), json!({ "type": "message_stop" })),
        ]
        .concat(),
        WireProtocol::OpenAiChat => [
            frame(None, json!({ "id": "c", "object": "chat.completion.chunk", "created": 0, "model": "m",
                "choices": [{ "index": 0, "delta": { "role": "assistant", "content": REPLY_TEXT }, "finish_reason": null }] })),
            frame(None, json!({ "id": "c", "object": "chat.completion.chunk", "created": 0, "model": "m",
                "choices": [{ "index": 0, "delta": {}, "finish_reason": "stop" }],
                "usage": { "prompt_tokens": 3, "completion_tokens": 2, "total_tokens": 5 } })),
            "data: [DONE]\n\n".to_owned(),
        ]
        .concat(),
        WireProtocol::OpenAiResponses => {
            let done = upstream_reply(wire);
            let item = done["output"][0].clone();
            [
                frame(Some("response.created"), json!({ "type": "response.created", "response": {
                    "id": "resp_harness", "object": "response", "model": "m", "status": "in_progress", "output": [] } })),
                frame(Some("response.output_item.added"), json!({ "type": "response.output_item.added", "output_index": 0, "item": item })),
                frame(Some("response.output_text.delta"), json!({ "type": "response.output_text.delta",
                    "item_id": "msg_1", "output_index": 0, "content_index": 0, "delta": REPLY_TEXT })),
                frame(Some("response.output_item.done"), json!({ "type": "response.output_item.done", "output_index": 0, "item": item })),
                frame(Some("response.completed"), json!({ "type": "response.completed", "response": done })),
            ]
            .concat()
        },
        WireProtocol::Gemini => frame(None, upstream_reply(wire)),
    }
}

// Sends `fixture` to the lane's model, as the gateway would.
pub(crate) async fn dispatch(fixture: &Fixture, lane: &Lane) -> Result<Captured, String> {
    dispatch_model(fixture, lane, lane.model.as_str()).await
}

// Why: reqwest is built without a default rustls provider; core installs ring
// at server start, which this in-process harness never runs.
fn install_crypto_provider() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("install the ring crypto provider");
    });
}

fn by_ordinal(event: CanonicalEvent, tools: &mut Vec<u32>) -> CanonicalEvent {
    let mut ordinal = |index: u32| {
        if !tools.contains(&index) {
            tools.push(index);
        }
        tools.iter().position(|i| *i == index).unwrap_or_default() as u32
    };
    match event {
        CanonicalEvent::ContentBlockStart { index, block }
            if matches!(block, ContentBlockKind::ToolUse { .. }) =>
        {
            CanonicalEvent::ContentBlockStart {
                index: ordinal(index),
                block,
            }
        },
        CanonicalEvent::ToolUseDelta {
            index,
            partial_json,
        } => CanonicalEvent::ToolUseDelta {
            index: ordinal(index),
            partial_json,
        },
        other => other,
    }
}

// What the mock upstream answers: a body and its content type.
#[derive(Debug, Clone)]
pub(crate) struct Reply {
    pub body: Vec<u8>,
    pub content_type: &'static str,
}

impl Reply {
    pub(crate) fn json(value: &Value) -> Self {
        Self {
            body: serde_json::to_vec(value).unwrap_or_default(),
            content_type: "application/json",
        }
    }

    pub(crate) fn sse(frames: &str) -> Self {
        Self {
            body: frames.as_bytes().to_vec(),
            content_type: "text/event-stream",
        }
    }
}

pub(crate) async fn dispatch_model(
    fixture: &Fixture,
    lane: &Lane,
    model_id: &str,
) -> Result<Captured, String> {
    let reply = if fixture.body["stream"] == Value::Bool(true) {
        Reply::sse(&upstream_stream(lane.wire))
    } else {
        Reply::json(&upstream_reply(lane.wire))
    };
    dispatch_reply(fixture, lane, model_id, &reply).await
}

pub(crate) async fn dispatch_reply(
    fixture: &Fixture,
    lane: &Lane,
    model_id: &str,
    reply: &Reply,
) -> Result<Captured, String> {
    install_crypto_provider();
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(reply.body.clone(), reply.content_type),
        )
        .mount(&server)
        .await;
    let (catalog, hop) = pointed_at(lane, &server.uri(), model_id)?;
    let model = catalog
        .model(model_id)
        .ok_or_else(|| format!("{model_id} is not a catalog model"))?;

    let inbound = inbound_adapter(fixture.inbound);
    let raw = Bytes::from(serde_json::to_vec(&fixture.body).map_err(|e| e.to_string())?);
    let mut request = inbound
        .parse_request(&raw)
        .map_err(|e| format!("inbound parse: {e}"))?;
    request.model = model.id.clone();
    let raw_body = (inbound.passthrough_wire() == Some(hop.wire)).then_some(&raw);
    let forward: Vec<(String, String)> = fixture
        .headers
        .iter()
        .filter(|(n, _)| is_forwardable_request_header(n))
        .cloned()
        .collect();

    let call = UpstreamTarget::resolve(
        &std::sync::Arc::new(systemprompt::security::credential::TokenCache::default()),
        &catalog,
        &hop,
        KEY,
    )
    .map_err(|e| format!("resolve: {e}"))?
    .call()
    .await
    .map_err(|e| format!("call: {e}"))?;
    let http = reqwest::Client::new();
    let ctx = OutboundCtx {
        target: &hop,
        upstream: &call,
        http: &http,
        request: &request,
        upstream_model: &hop.upstream_model,
        model_limits: Some(model.limits),
        automatic_prompt_caching: true,
        forward_headers: &forward,
        raw_body,
    };
    let registry = GatewayUpstreamRegistry::new();
    let adapter = registry
        .get(hop.wire.as_tag())
        .ok_or("no outbound adapter for the wire")?;
    let body = adapter
        .build_body(&ctx)
        .map_err(|e| format!("build body: {e}"))?;
    let outcome = adapter
        .send(ctx, &body)
        .await
        .map_err(|e| format!("send: {e}"))?;
    let client_reply = match outcome {
        OutboundOutcome::Buffered(canonical) => {
            String::from_utf8_lossy(&inbound.render_response(&canonical)).into_owned()
        },
        OutboundOutcome::RawBuffered { body, .. } => String::from_utf8_lossy(&body).into_owned(),
        OutboundOutcome::Streaming(mut events) => {
            let mut out = String::new();
            let mut tools: Vec<u32> = Vec::new();
            while let Some(event) = events.next().await {
                let event = event.map_err(|e| format!("upstream stream: {e}"))?;
                // Mirrors the gateway stream tap, which renumbers tool blocks
                // for clients that index tool calls by ordinal.
                let event = if inbound.indexes_tool_calls_by_ordinal() {
                    by_ordinal(event, &mut tools)
                } else {
                    event
                };
                if let Some(frame) = inbound.render_event(&event, model_id) {
                    out.push_str(&String::from_utf8_lossy(&frame));
                }
            }
            out
        },
        OutboundOutcome::RawStreaming { mut stream, .. } => {
            let mut out = String::new();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(|e| format!("upstream stream: {e}"))?;
                out.push_str(&String::from_utf8_lossy(&chunk));
            }
            out
        },
    };

    let received = server.received_requests().await.unwrap_or_default();
    let sent = received.last().ok_or("upstream received no request")?;
    let body: Value = serde_json::from_slice(&sent.body)
        .map_err(|e| format!("upstream body is not JSON: {e}"))?;
    let headers = sent
        .headers
        .iter()
        .map(|(n, v)| {
            (
                n.as_str().to_owned(),
                v.to_str().unwrap_or_default().to_owned(),
            )
        })
        .collect();
    Ok(Captured {
        body,
        headers,
        client_reply,
    })
}
