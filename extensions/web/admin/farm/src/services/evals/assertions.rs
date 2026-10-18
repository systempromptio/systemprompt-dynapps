//! The deterministic half of an eval: each case assertion checked against
//! the finished run's row, its event stream and the workspace files the
//! runner stored for it ([`RunFiles`]).
//!
//! Pure, so it runs the same in the orchestrator and in unit tests.

use serde::Serialize;
use systemprompt::agent_runs::{AgentRunRow, RunEvent};

use super::run_files::{self, RunFiles};
use crate::services::farm_sync::eval_yaml::Assertion;
use crate::types::RunEventPayload;

use systemprompt_web_admin_core::util::tool_allowlist::{SKILL_TOOL, tool_permitted};

#[derive(Debug)]
pub struct AssertionContext<'a> {
    pub run: &'a AgentRunRow,
    pub events: &'a [RunEvent],
    pub files: &'a RunFiles,
    pub agent_tools: &'a [String],
    pub output_path: Option<&'a str>,
    pub final_text: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AssertionResult {
    pub kind: String,
    pub passed: bool,
    pub detail: String,
}

impl AssertionResult {
    fn new(kind: &str, passed: bool, detail: impl Into<String>) -> Self {
        Self {
            kind: kind.to_owned(),
            passed,
            detail: detail.into(),
        }
    }
}

#[must_use]
pub fn evaluate_assertions(
    assertions: &[Assertion],
    ctx: &AssertionContext<'_>,
) -> Vec<AssertionResult> {
    assertions.iter().map(|a| evaluate_one(a, ctx)).collect()
}

fn evaluate_one(assertion: &Assertion, ctx: &AssertionContext<'_>) -> AssertionResult {
    match assertion {
        Assertion::FileExists { path } => file_exists(ctx, path),
        Assertion::OutputContains { path, text } => {
            output_check(ctx, "output_contains", path.as_deref(), |body| {
                let found = body.contains(text.as_str());
                (found, format!("text {text:?} {}", found_word(found)))
            })
        },
        Assertion::Regex { path, pattern } => match regex::Regex::new(pattern) {
            Ok(re) => output_check(ctx, "regex", path.as_deref(), |body| {
                let found = re.is_match(body);
                (found, format!("pattern {pattern:?} {}", found_word(found)))
            }),
            Err(err) => AssertionResult::new("regex", false, format!("invalid pattern: {err}")),
        },
        Assertion::NoToolOutsideAllowlist { .. } => tool_allowlist(ctx),
        Assertion::MaxTurns { value } => {
            let limit = i64::from(*value);
            ctx.run.turns.map_or_else(
                || AssertionResult::new("max_turns", false, "run recorded no turn count"),
                |turns| {
                    AssertionResult::new(
                        "max_turns",
                        i64::from(turns) <= limit,
                        format!("{turns} turns, limit {limit}"),
                    )
                },
            )
        },
        Assertion::MaxCostUsd { value } => ctx.run.cost_microdollars.map_or_else(
            || AssertionResult::new("max_cost_usd", false, "run recorded no cost"),
            |micro| {
                let usd = f64::from(i32::try_from(micro).unwrap_or(i32::MAX)) / 1_000_000.0;
                AssertionResult::new(
                    "max_cost_usd",
                    usd <= *value,
                    format!("${usd:.4} spent, limit ${value:.4}"),
                )
            },
        ),
        Assertion::SkillInvoked { skill } => skill_invoked(ctx, skill),
    }
}

const fn found_word(found: bool) -> &'static str {
    if found { "found" } else { "not found" }
}

fn file_exists(ctx: &AssertionContext<'_>, path: &str) -> AssertionResult {
    if run_files::normalise(path).is_none() {
        return AssertionResult::new(
            "file_exists",
            false,
            format!("{path} escapes the workspace"),
        );
    }
    match ctx.files.get(path) {
        Some(_) => AssertionResult::new("file_exists", true, path),
        None => AssertionResult::new("file_exists", false, format!("{path} is missing")),
    }
}

fn output_check(
    ctx: &AssertionContext<'_>,
    kind: &str,
    path: Option<&str>,
    check: impl FnOnce(&str) -> (bool, String),
) -> AssertionResult {
    let (source, body) = match path.or(ctx.output_path) {
        Some(rel) => {
            if run_files::normalise(rel).is_none() {
                return AssertionResult::new(kind, false, format!("{rel} escapes the workspace"));
            }
            match ctx.files.get(rel) {
                Some(body) => (rel.to_owned(), body.to_owned()),
                None => {
                    return AssertionResult::new(
                        kind,
                        false,
                        format!("cannot read {rel}: the run stored no such file"),
                    );
                },
            }
        },
        None => match ctx.final_text {
            Some(text) => ("final result".to_owned(), text.to_owned()),
            None => return AssertionResult::new(kind, false, "run produced no result text"),
        },
    };
    let (passed, detail) = check(&body);
    AssertionResult::new(kind, passed, format!("{detail} in {source}"))
}

fn payloads(events: &[RunEvent]) -> impl Iterator<Item = RunEventPayload> + '_ {
    events.iter().map(|e| e.payload.clone())
}

fn tool_allowlist(ctx: &AssertionContext<'_>) -> AssertionResult {
    const KIND: &str = "no_tool_outside_allowlist";
    if ctx.agent_tools.is_empty() {
        return AssertionResult::new(KIND, true, "agent declares no tool allowlist");
    }
    let mut outside: Vec<String> = payloads(ctx.events)
        .filter_map(|p| match p {
            RunEventPayload::ToolUse { name, .. } => Some(name),
            _ => None,
        })
        .filter(|name| !tool_permitted(name, ctx.agent_tools))
        .collect();
    outside.sort();
    outside.dedup();
    if outside.is_empty() {
        AssertionResult::new(KIND, true, "every tool call was on the allowlist")
    } else {
        AssertionResult::new(
            KIND,
            false,
            format!("outside allowlist: {}", outside.join(", ")),
        )
    }
}

fn normalise(name: &str) -> String {
    name.trim().to_lowercase().replace('_', "-")
}

// JSON: protocol boundary — a `ToolUse` input is whatever the tool's own
// schema says; the Skill tool's `skill` / `name` key is read and nothing else.
fn skill_invoked(ctx: &AssertionContext<'_>, skill: &str) -> AssertionResult {
    let wanted = normalise(skill);
    let hit = payloads(ctx.events).find_map(|p| match p {
        RunEventPayload::ToolUse { name, input, .. } if name == SKILL_TOOL => ["skill", "name"]
            .iter()
            .filter_map(|key| input.get(*key).and_then(serde_json::Value::as_str))
            .find(|v| normalise(v).contains(&wanted))
            .map(|v| format!("Skill tool called with {v}")),
        RunEventPayload::Hook { name, .. } if normalise(&name).contains(&wanted) => {
            Some(format!("hook {name}"))
        },
        _ => None,
    });
    hit.map_or_else(
        || AssertionResult::new("skill_invoked", false, format!("{skill} was never invoked")),
        |detail| AssertionResult::new("skill_invoked", true, detail),
    )
}
