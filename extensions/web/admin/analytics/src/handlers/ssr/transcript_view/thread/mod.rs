//! Walks one thread's canonical history into turns and steps.
//!
//! The canonical history is the stored message array of the thread's latest
//! request that has one — the gateway persists the whole array on every
//! request, so the newest is the complete transcript and everything before it
//! is a prefix.
//!
//! A `user` row starts a turn unless the assistant row before it carried a
//! tool use — then it carries that row's results and folds into its tool
//! steps. A `tool` row is always a result. An `assistant` row becomes a text
//! step (when any text survives the marker strip) plus one tool step per
//! marker, whose arguments come from the attributed request's
//! `ai_request_tool_calls` rows when their count matches and from the marker's
//! own JSON otherwise.
//!
//! Results pair by `tool_use` id when both sides carry one, and by position
//! among the latest assistant row's still-open steps only when neither does.
//! A step whose ledger row holds a result payload keeps it and never takes
//! message text.
//!
//! Message inserts are best-effort, so the history walked is the latest one
//! that was actually retained, not simply the latest request's.

mod attribution;
mod results;

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::conversation::{
    Counters, StepView, THREAD_LABEL_CHARS, ThreadView, TurnView, is_long, local_time, single_line,
};
use super::markers::{parse_assistant, strip_system_reminders, tidy_lines};
use super::steps::{tool_step, turn_view};
use super::{meta_view, preview, short_id};
use crate::repositories::analytics::context_detail::{ContextMessageRow, ContextRequestRow};
use crate::repositories::analytics::context_tool_calls::ContextToolCallRow;
use attribution::{Attribution, attribute, turn_started_at};
use results::closes_tool_group;
use systemprompt_web_admin_core::ssr::entity_urls::request_detail_url;

type MessagesByRequest<'a> = HashMap<&'a str, Vec<&'a ContextMessageRow>>;
type ToolsByRequest<'a> = HashMap<&'a str, Vec<&'a ContextToolCallRow>>;

pub(super) struct ThreadBuilder<'a, 'c> {
    counters: &'c mut Counters,
    messages: &'c MessagesByRequest<'a>,
    tools: &'c ToolsByRequest<'a>,
    open: Vec<OpenTool>,
}

struct OpenTool {
    step: usize,
    id: Option<String>,
    has_ledger_result: bool,
}

struct TurnDraft {
    prompt: String,
    ts: DateTime<Utc>,
    steps: Vec<StepView>,
}

#[derive(Default)]
struct FoldState {
    turns: Vec<TurnView>,
    draft: Option<TurnDraft>,
    system_prompt: Option<String>,
    system_prompt_chars: usize,
    prev_tool_use: bool,
}

impl<'a, 'c> ThreadBuilder<'a, 'c> {
    pub(super) const fn new(
        counters: &'c mut Counters,
        messages: &'c MessagesByRequest<'a>,
        tools: &'c ToolsByRequest<'a>,
    ) -> Self {
        Self {
            counters,
            messages,
            tools,
            open: Vec::new(),
        }
    }

    pub(super) fn build(mut self, index: usize, reqs: &[&'a ContextRequestRow]) -> ThreadView {
        let history = self.latest_retained_history(reqs);
        let attributed = attribute(self.messages, reqs, history);
        let mut state = FoldState::default();
        for (idx, m) in history.iter().enumerate() {
            self.fold_message(&mut state, idx, m, &attributed);
        }
        if let Some(done) = state.draft.take() {
            state.turns.push(self.finish(done));
        }
        thread_view(index, reqs, state)
    }

    fn latest_retained_history(
        &self,
        reqs: &[&'a ContextRequestRow],
    ) -> &'c [&'a ContextMessageRow] {
        let messages = self.messages;
        reqs.iter()
            .rev()
            .find_map(|r| messages.get(r.id.as_str()))
            .map_or(&[], Vec::as_slice)
    }

    fn fold_message(
        &mut self,
        state: &mut FoldState,
        idx: usize,
        m: &ContextMessageRow,
        attributed: &Attribution<'a>,
    ) {
        match m.role.as_str() {
            "system" => self.keep_system_prompt(state, m),
            "assistant" => {
                let turn = state.draft.get_or_insert_with(|| TurnDraft {
                    prompt: String::new(),
                    ts: m.created_at,
                    steps: Vec::new(),
                });
                state.prev_tool_use = self.assistant_steps(turn, m, attributed.get(&idx).copied());
            },
            _ => self.fold_prompt_or_result(state, idx, m, attributed),
        }
    }

    fn keep_system_prompt(&mut self, state: &mut FoldState, m: &ContextMessageRow) {
        if state.system_prompt.is_some() {
            return;
        }
        let body = self.counters.body(&m.content);
        state.system_prompt_chars = body.chars().count();
        state.system_prompt = Some(preview(&body));
    }

    fn fold_prompt_or_result(
        &mut self,
        state: &mut FoldState,
        idx: usize,
        m: &ContextMessageRow,
        attributed: &Attribution<'a>,
    ) {
        let is_result = m.role == "tool" || state.prev_tool_use;
        if is_result && let Some(turn) = state.draft.as_mut() {
            self.fold_result(turn, m);
            if closes_tool_group(&m.role) {
                state.prev_tool_use = false;
            }
            return;
        }
        if let Some(done) = state.draft.take() {
            state.turns.push(self.finish(done));
        }
        state.draft = Some(TurnDraft {
            prompt: self
                .counters
                .body(&tidy_lines(&strip_system_reminders(&m.content))),
            ts: turn_started_at(attributed, idx, m.created_at),
            steps: Vec::new(),
        });
        state.prev_tool_use = false;
    }

    fn assistant_steps(
        &mut self,
        turn: &mut TurnDraft,
        m: &ContextMessageRow,
        request: Option<&ContextRequestRow>,
    ) -> bool {
        let parsed = parse_assistant(&m.content);
        let (id_short, url) = request.map_or((None, None), |r| {
            (
                Some(short_id(r.id.as_str())),
                Some(request_detail_url(&r.id)),
            )
        });
        if !parsed.text.is_empty() {
            let text = self.counters.body(&tidy_lines(&parsed.text));
            turn.steps.push(StepView {
                is_assistant: true,
                is_tool: false,
                text_is_long: is_long(&text),
                text: Some(text),
                tool_name: None,
                tool_input_pretty: None,
                tool_result_pretty: None,
                artifact_url: None,
                artifact_structured: false,
                meta: request.map(meta_view),
                request_id_short: id_short.clone(),
                request_url: url.clone(),
            });
        }
        let rows = self.ledger_rows_in_marker_order(request, parsed.tool_uses.len());
        self.open.clear();
        for (i, marker) in parsed.tool_uses.iter().enumerate() {
            let row = rows.and_then(|rows| rows.get(i).copied());
            self.open.push(OpenTool {
                step: turn.steps.len(),
                id: row
                    .and_then(|r| r.ai_tool_call_id.as_ref())
                    .map(|id| id.as_str().to_owned()),
                has_ledger_result: row.is_some_and(|r| r.tool_result_payload.is_some()),
            });
            turn.steps
                .push(tool_step(marker, row, id_short.clone(), url.clone()));
            self.counters.tool_calls += 1;
        }
        !parsed.tool_uses.is_empty()
    }

    fn ledger_rows_in_marker_order(
        &self,
        request: Option<&ContextRequestRow>,
        marker_count: usize,
    ) -> Option<&'c Vec<&'a ContextToolCallRow>> {
        let tools = self.tools;
        request
            .and_then(|r| tools.get(r.id.as_str()))
            .filter(|rows| rows.len() == marker_count)
    }

    fn finish(&mut self, draft: TurnDraft) -> TurnView {
        self.counters.turn_number += 1;
        turn_view(
            self.counters.turn_number,
            draft.prompt,
            draft.ts,
            draft.steps,
        )
    }
}

fn thread_view(index: usize, reqs: &[&ContextRequestRow], state: FoldState) -> ThreadView {
    let label = state
        .turns
        .first()
        .map_or_else(|| format!("Thread {index}"), thread_label);
    ThreadView {
        index,
        is_main: index == 1,
        anchor: format!("thread-{index}"),
        label,
        started_local: reqs
            .first()
            .map_or_else(String::new, |r| local_time(r.created_at)),
        request_count: reqs.len(),
        model: reqs.iter().rev().find_map(|r| r.model.clone()),
        system_prompt: state.system_prompt,
        system_prompt_chars: state.system_prompt_chars,
        turns: state.turns,
    }
}

fn thread_label(turn: &TurnView) -> String {
    single_line(&turn.prompt, THREAD_LABEL_CHARS)
}
