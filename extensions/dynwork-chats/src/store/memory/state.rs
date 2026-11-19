//! What the memory store holds: threads with their messages and events, and
//! the one change sequence.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;

use crate::ids::ThreadId;
use crate::store::{EventRecord, MessageRecord, ThreadRecord, page_limit};

#[derive(Debug)]
pub(super) struct StoredEvent {
    pub(super) record: EventRecord,
    pub(super) compacted_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub(super) struct Entry {
    pub(super) record: ThreadRecord,
    pub(super) last_message_seq: i64,
    pub(super) messages: Vec<MessageRecord>,
    pub(super) events: Vec<StoredEvent>,
}

#[derive(Debug, Default)]
pub(super) struct State {
    pub(super) change_seq: i64,
    pub(super) threads: HashMap<ThreadId, Entry>,
}

impl State {
    pub(super) const fn next_change(&mut self) -> i64 {
        self.change_seq += 1;
        self.change_seq
    }

    pub(super) fn live(&mut self, user: &UserId, id: &ThreadId) -> Option<&mut Entry> {
        self.threads
            .get_mut(id)
            .filter(|entry| entry.record.user == *user && entry.record.deleted_at.is_none())
    }
}

pub(super) fn usize_from(limit: u32) -> usize {
    usize::try_from(page_limit(limit)).unwrap_or(usize::MAX)
}
