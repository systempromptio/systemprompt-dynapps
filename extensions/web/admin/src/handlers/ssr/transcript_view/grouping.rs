//! Which requests make up which thread, and whose bodies the reader needs.
//!
//! Kept beside the builder rather than inside it because the caller has to
//! resolve the same grouping before it can fetch anything: the transcript is
//! built from the tail of each thread, so the ids fetched and the ids the
//! builder goes looking for must be decided by one piece of code.

use std::collections::HashMap;

use systemprompt::identifiers::GatewayConversationId;

use crate::repositories::analytics::context_detail::ContextRequestRow;

// Why: the Postgres fn `conversation_request_kind` calls a request `utility`
// when its best-effort tool payload write was lost, so a context with no
// `turn` row still has a real transcript; callers then fall back to every row.
pub(super) fn has_turns(requests: &[ContextRequestRow]) -> bool {
    requests.iter().any(|r| r.effective_kind == "turn")
}

pub(super) fn group_threads(requests: &[ContextRequestRow]) -> Vec<Vec<&ContextRequestRow>> {
    let mut turns: Vec<&ContextRequestRow> = requests
        .iter()
        .filter(|r| r.effective_kind == "turn" || !has_turns(requests))
        .collect();
    turns.sort_by_key(|r| r.created_at);

    let mut groups: Vec<Vec<&ContextRequestRow>> = Vec::new();
    let mut group_of: HashMap<Option<&str>, usize> = HashMap::new();
    for r in turns {
        let key = r
            .gateway_conversation_id
            .as_ref()
            .map(GatewayConversationId::as_str);
        let idx = *group_of.entry(key).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[idx].push(r);
    }
    groups
}

const CANONICAL_LOOKBACK: usize = 5;

#[must_use]
pub fn transcript_request_ids(requests: &[ContextRequestRow]) -> TranscriptRequestIds {
    let groups = group_threads(requests);
    let messages = groups
        .iter()
        .flat_map(|reqs| {
            reqs.iter()
                .rev()
                .take(CANONICAL_LOOKBACK)
                .map(|r| r.id.as_str().to_owned())
        })
        .collect();
    let tool_calls = groups
        .iter()
        .flatten()
        .map(|r| r.id.as_str().to_owned())
        .collect();
    TranscriptRequestIds {
        messages,
        tool_calls,
    }
}

/// The two id sets [`transcript_request_ids`] resolves: the requests whose
/// stored history may be rendered, and every turn request, each of which may
/// own tool-call rows.
#[derive(Debug, Default)]
pub struct TranscriptRequestIds {
    pub messages: Vec<String>,
    pub tool_calls: Vec<String>,
}
