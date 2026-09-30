//! Pairs tool-result rows with the open tool steps of the latest assistant row.
//!
//! The gateway stores a row's results as one text block with no boundary
//! between them, so id-less parallel results are shown together and labelled
//! as such.

use super::{ThreadBuilder, TurnDraft};
use crate::handlers::ssr::transcript_view::markers::pretty_json_text;
use crate::repositories::analytics::context_detail::ContextMessageRow;

pub(super) fn closes_tool_group(role: &str) -> bool {
    role != "tool"
}

fn result_text(body: &str, parallel_in_one_block: Option<usize>) -> String {
    parallel_in_one_block.map_or_else(
        || pretty_json_text(body),
        |count| format!("Results of {count} parallel calls, stored as one block:\n\n{body}"),
    )
}

impl ThreadBuilder<'_, '_> {
    pub(super) fn fold_result(&mut self, turn: &mut TurnDraft, m: &ContextMessageRow) {
        let target = match m.tool_call_id.as_ref() {
            Some(id) => self
                .open
                .iter()
                .position(|o| o.id.as_deref() == Some(id.as_str()))
                .or_else(|| self.first_without_ledger_result(true)),
            None => self.first_without_ledger_result(false),
        };
        let unclaimed = self.open.iter().filter(|o| !o.has_ledger_result).count();
        let Some(pos) = target else {
            if closes_tool_group(&m.role) {
                self.open.clear();
            }
            return;
        };
        let open = self.open.remove(pos);
        if !open.has_ledger_result
            && let Some(step) = turn.steps.get_mut(open.step)
        {
            let body = self.counters.body(&m.content);
            let shares_block = m.tool_call_id.is_none() && unclaimed > 1;
            step.tool_result_pretty = Some(result_text(&body, shares_block.then_some(unclaimed)));
        }
        if closes_tool_group(&m.role) {
            self.open.clear();
        }
    }

    fn first_without_ledger_result(&self, id_less_only: bool) -> Option<usize> {
        self.open
            .iter()
            .position(|o| !o.has_ledger_result && (!id_less_only || o.id.is_none()))
    }
}
