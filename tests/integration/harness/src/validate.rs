//! Strict validators for the request an upstream receives, one per wire.
//!
//! Each is deliberately stricter than Anthropic's own API (which merges
//! consecutive same-role turns): it is the bar `DeepSeek`, `MiniMax` and the
//! other Anthropic-compatible and `OpenAI`-compatible upstreams hold.

use std::collections::BTreeSet;

use serde_json::Value;
use systemprompt::wire::WireProtocol;

use crate::dispatch::{Captured, KEY, REPLY_TEXT};
use crate::fixtures::Fixture;
use crate::providers::Lane;

type Check = Result<(), String>;

pub(crate) fn all(fixture: &Fixture, lane: &Lane, captured: &Captured) -> Check {
    body(lane.wire, &captured.body)?;
    headers(fixture, lane, captured)?;
    if !captured.client_reply.contains(REPLY_TEXT) {
        return Err(format!(
            "client reply lost the upstream text: {}",
            captured.client_reply
        ));
    }
    Ok(())
}

pub(crate) fn body(wire: WireProtocol, body: &Value) -> Check {
    match wire {
        WireProtocol::Anthropic => anthropic(body),
        WireProtocol::OpenAiChat => openai_chat(body),
        WireProtocol::OpenAiResponses => openai_responses(body),
        WireProtocol::Gemini => gemini(body),
    }
}

fn blocks(message: &Value) -> Vec<&Value> {
    message["content"]
        .as_array()
        .map(|a| a.iter().collect())
        .unwrap_or_default()
}

fn ids<'a>(blocks: &[&'a Value], kind: &str, key: &str) -> Vec<&'a str> {
    blocks
        .iter()
        .filter(|b| b["type"] == kind)
        .filter_map(|b| b[key].as_str())
        .collect()
}

fn anthropic(body: &Value) -> Check {
    let messages = body["messages"]
        .as_array()
        .ok_or("messages is not an array")?;
    if messages.first().map(|m| &m["role"]) != Some(&Value::from("user")) {
        return Err("first message is not a user turn".to_owned());
    }
    let mut pending: BTreeSet<&str> = BTreeSet::new();
    let mut previous = "";
    for (i, message) in messages.iter().enumerate() {
        let role = message["role"]
            .as_str()
            .ok_or_else(|| format!("messages.{i}: no role"))?;
        // Claude Code sends harness context as mid-history `system` messages,
        // which the Messages API accepts between turns; they may never split a
        // tool_use from its results.
        if role == "system" {
            if !pending.is_empty() {
                return Err(format!("messages.{i}: system message before tool results"));
            }
            continue;
        }
        if role == previous {
            return Err(format!("messages.{i}: two consecutive {role} turns"));
        }
        let content = blocks(message);
        let results = ids(&content, "tool_result", "tool_use_id");
        let unique: BTreeSet<&str> = results.iter().copied().collect();
        match role {
            "user" => {
                if unique != pending || unique.len() != results.len() {
                    return Err(format!(
                        "messages.{i}: tool_results {results:?} do not answer tool_use ids {pending:?}"
                    ));
                }
                let lead = content
                    .iter()
                    .take_while(|b| b["type"] == "tool_result")
                    .count();
                if lead != results.len() {
                    return Err(format!("messages.{i}: tool_result blocks are not first"));
                }
                pending.clear();
            },
            "assistant" => {
                if !results.is_empty() {
                    return Err(format!("messages.{i}: tool_result in an assistant turn"));
                }
                pending = ids(&content, "tool_use", "id").into_iter().collect();
            },
            other => return Err(format!("messages.{i}: role {other} on the anthropic wire")),
        }
        previous = role;
    }
    if pending.is_empty() {
        Ok(())
    } else {
        Err(format!("tool_use ids {pending:?} never answered"))
    }
}

fn openai_chat(body: &Value) -> Check {
    let messages = body["messages"]
        .as_array()
        .ok_or("messages is not an array")?;
    let mut pending: BTreeSet<&str> = BTreeSet::new();
    let mut previous = "";
    for (i, message) in messages.iter().enumerate() {
        let role = message["role"]
            .as_str()
            .ok_or_else(|| format!("messages.{i}: no role"))?;
        match role {
            "tool" => {
                let id = message["tool_call_id"].as_str().unwrap_or_default();
                if !pending.remove(id) {
                    return Err(format!(
                        "messages.{i}: tool message {id} answers no open tool call"
                    ));
                }
            },
            "assistant" | "user" | "system" => {
                if !pending.is_empty() {
                    return Err(format!(
                        "messages.{i}: {role} before tool calls {pending:?} were answered"
                    ));
                }
                if role == "assistant" {
                    if previous == "assistant" {
                        return Err(format!("messages.{i}: two consecutive assistant turns"));
                    }
                    pending = message["tool_calls"]
                        .as_array()
                        .map(|calls| calls.iter().filter_map(|c| c["id"].as_str()).collect())
                        .unwrap_or_default();
                }
            },
            other => return Err(format!("messages.{i}: unknown role {other}")),
        }
        previous = role;
    }
    if pending.is_empty() {
        Ok(())
    } else {
        Err(format!("tool calls {pending:?} never answered"))
    }
}

fn openai_responses(body: &Value) -> Check {
    let input = body["input"].as_array().ok_or("input is not an array")?;
    let mut pending: BTreeSet<&str> = BTreeSet::new();
    for (i, item) in input.iter().enumerate() {
        let call_id = item["call_id"].as_str().unwrap_or_default();
        match item["type"].as_str() {
            Some("function_call") => {
                pending.insert(call_id);
            },
            Some("function_call_output") => {
                if !pending.remove(call_id) {
                    return Err(format!(
                        "input.{i}: output {call_id} answers no function_call"
                    ));
                }
            },
            Some("message") if !pending.is_empty() => {
                return Err(format!(
                    "input.{i}: message before calls {pending:?} were answered"
                ));
            },
            _ => {},
        }
    }
    if pending.is_empty() {
        Ok(())
    } else {
        Err(format!("function calls {pending:?} never answered"))
    }
}

fn gemini(body: &Value) -> Check {
    let contents = body["contents"]
        .as_array()
        .ok_or("contents is not an array")?;
    let mut previous = "";
    let mut pending: Vec<&str> = Vec::new();
    for (i, content) in contents.iter().enumerate() {
        let role = content["role"]
            .as_str()
            .ok_or_else(|| format!("contents.{i}: no role"))?;
        if role == previous {
            return Err(format!("contents.{i}: two consecutive {role} turns"));
        }
        let parts: Vec<&Value> = content["parts"]
            .as_array()
            .map(|a| a.iter().collect())
            .unwrap_or_default();
        let named = |key: &str| -> Vec<&str> {
            parts
                .iter()
                .filter_map(|p| p[key]["name"].as_str())
                .collect()
        };
        if role == "user" {
            let mut answered = named("functionResponse");
            answered.sort_unstable();
            pending.sort_unstable();
            if answered != pending {
                return Err(format!(
                    "contents.{i}: responses {answered:?} do not answer calls {pending:?}"
                ));
            }
            pending.clear();
        } else {
            pending = named("functionCall");
        }
        previous = role;
    }
    if pending.is_empty() {
        Ok(())
    } else {
        Err(format!("function calls {pending:?} never answered"))
    }
}

fn headers(fixture: &Fixture, lane: &Lane, captured: &Captured) -> Check {
    let bearer = format!("Bearer {KEY}");
    let authed = captured.header("x-api-key") == Some(KEY)
        || captured.header("authorization") == Some(bearer.as_str())
        || captured.header("x-goog-api-key") == Some(KEY);
    if !authed {
        return Err("upstream request carries no provider credential".to_owned());
    }
    if let Some(ua) = &lane.user_agent
        && captured.header("user-agent") != Some(ua.as_str())
    {
        return Err(format!(
            "user-agent {:?}, provider declares {ua}",
            captured.header("user-agent")
        ));
    }
    let session_expected = lane
        .forward_headers
        .iter()
        .any(|h| h == "x-opencode-session")
        && fixture.header("x-opencode-session").is_some();
    if captured.header("x-opencode-session").is_some() != session_expected {
        return Err(format!(
            "x-opencode-session forwarded = {}, expected {session_expected}",
            !session_expected
        ));
    }
    if captured
        .headers
        .iter()
        .any(|(n, _)| n.starts_with("x-stainless-") || n.starts_with("x-claude-code-"))
    {
        return Err("client identity header leaked upstream".to_owned());
    }
    let beta = captured.header("anthropic-beta");
    if lane.wire != WireProtocol::Anthropic {
        if beta.is_some() || captured.header("anthropic-version").is_some() {
            return Err("anthropic headers sent to a non-anthropic wire".to_owned());
        }
        return Ok(());
    }
    if captured.header("anthropic-version").is_none() {
        return Err("anthropic wire without anthropic-version".to_owned());
    }
    if let Some(accepted) = &lane.accepted_betas
        && let Some(sent) = beta
    {
        let accepted: Vec<String> = accepted
            .iter()
            .filter_map(|b| {
                serde_json::to_value(b)
                    .ok()?
                    .as_str()
                    .map(ToOwned::to_owned)
            })
            .collect();
        if let Some(extra) = sent
            .split(',')
            .map(str::trim)
            .find(|b| !accepted.iter().any(|a| a == b))
        {
            return Err(format!(
                "beta {extra} sent to a provider that does not accept it"
            ));
        }
    }
    if beta.is_none_or(|b| !b.contains("context-management"))
        && captured.body.get("context_management").is_some()
    {
        return Err("context_management sent without the beta that gates it".to_owned());
    }
    Ok(())
}
