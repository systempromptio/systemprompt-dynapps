//! Attributes each request of a thread to a position in its history.
//!
//! A request is attributed to the index of the assistant row it produced: its
//! own history length, minus the reply it appended on completion. A request
//! whose history was not retained falls back to its recorded message count.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::MessagesByRequest;
use crate::repositories::analytics::context_detail::{ContextMessageRow, ContextRequestRow};

pub(super) type Attribution<'a> = HashMap<usize, &'a ContextRequestRow>;

pub(super) fn attribute<'a>(
    messages: &MessagesByRequest<'_>,
    reqs: &[&'a ContextRequestRow],
    history: &[&ContextMessageRow],
) -> Attribution<'a> {
    let mut out = HashMap::new();
    for r in reqs {
        let rows = messages.get(r.id.as_str());
        let len = match rows {
            Some(rows) => rows.len(),
            None => match usize::try_from(r.message_count) {
                Ok(count) => count,
                Err(error) => {
                    tracing::warn!(request_id = %r.id, message_count = r.message_count, %error,
                        "Skipping transcript attribution with invalid message count");
                    continue;
                },
            },
        };
        let appended_reply = r.status == "completed"
            && rows.is_none_or(|rows| rows.last().is_some_and(|m| m.role == "assistant"));
        let n = if appended_reply {
            len.saturating_sub(1)
        } else {
            len
        };
        if history.get(n).is_some_and(|m| m.role == "assistant") {
            out.insert(n, *r);
        }
    }
    out
}

pub(super) fn turn_started_at(
    attributed: &Attribution<'_>,
    idx: usize,
    fallback: DateTime<Utc>,
) -> DateTime<Utc> {
    attributed
        .iter()
        .filter(|(i, _)| **i > idx)
        .min_by_key(|(i, _)| *i)
        .map_or(fallback, |(_, r)| r.created_at)
}
