//! The stored `OpenCode` v2 message documents of a turn, and what a reply
//! reads back from them.
//!
//! `dynwork_chat_messages.info` holds the whole v2 message, which the history
//! route serves as it is. A user message is `{ id, type: "user", text, time,
//! metadata: { assistantMessageID } }`; the metadata names the reply the
//! prompt started, so a retried prompt answers with the same reply. An
//! assistant message starts as `{ id, type: "assistant", agent, model,
//! content: [], time }` and is completed when its reply ends: its text as one
//! `text` part, `finish`, `cost`, `tokens`, `time.completed` and, when it did
//! not succeed, `error` (`aborted` for an interrupt, as the desktop writes
//! it). A provider may report usage more than once (input first, output at
//! the end), so each count keeps its largest value and the finish its latest.
//! The history sent to the model is oldest first, without unfinished or empty
//! messages, two messages of one role in a row joined, since providers expect
//! the roles to alternate.

// JSON: the message documents are `OpenCode` wire shapes stored as jsonb.
use serde_json::{Value, json};

use crate::ids::MessageId;
use crate::reply::events::{AGENT, Tokens, error_json, model_json};
use crate::reply::model::{ReplyError, Turn, Usage, finish_of};
use crate::store::{MessageRecord, ModelRef, Role};

pub const METADATA_REPLY: &str = "assistantMessageID";

/// How a reply ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ending {
    Succeeded,
    Interrupted,
    Failed(ReplyError),
}

impl Ending {
    pub fn aborted() -> ReplyError {
        ReplyError::new("aborted", "Step interrupted")
    }

    pub fn restarted() -> ReplyError {
        ReplyError::new(
            "restarted",
            "The server restarted while this reply was being written",
        )
    }
}

/// What a reply has produced so far.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fold {
    pub text: String,
    pub text_open: bool,
    pub usage: Usage,
}

impl Fold {
    pub fn add_usage(&mut self, usage: &Usage) {
        let current = &mut self.usage;
        current.input = current.input.max(usage.input);
        current.output = current.output.max(usage.output);
        current.reasoning = current.reasoning.max(usage.reasoning);
        current.cache_read = current.cache_read.max(usage.cache_read);
        current.cache_write = current.cache_write.max(usage.cache_write);
        if usage.raw_finish.is_some() {
            current.raw_finish = usage.raw_finish;
        }
    }

    pub fn raw_finish(&self) -> &'static str {
        self.usage.raw_finish.unwrap_or("stop")
    }

    pub fn finish(&self) -> &'static str {
        finish_of(self.raw_finish())
    }

    pub const fn tokens(&self) -> Tokens {
        Tokens::of(&self.usage)
    }
}

pub fn user_info(id: &MessageId, text: &str, created: i64, reply: &MessageId) -> Value {
    json!({
        "id": id,
        "type": "user",
        "text": text,
        "time": { "created": created },
        "metadata": { METADATA_REPLY: reply },
    })
}

pub fn text_parts(text: &str) -> Value {
    if text.is_empty() {
        json!([])
    } else {
        json!([{ "type": "text", "text": text }])
    }
}

pub fn reply_of(prompt: &MessageRecord) -> Option<MessageId> {
    prompt
        .info
        .get("metadata")
        .and_then(|metadata| metadata.get(METADATA_REPLY))
        .and_then(Value::as_str)
        .and_then(|id| id.parse().ok())
}

/// The assistant message of one reply.
#[derive(Debug, Clone)]
pub struct ReplyDoc {
    pub id: MessageId,
    pub model: ModelRef,
    pub created: i64,
}

impl ReplyDoc {
    pub fn started(&self) -> Value {
        json!({
            "id": self.id,
            "type": "assistant",
            "agent": AGENT,
            "model": model_json(&self.model),
            "content": [],
            "time": { "created": self.created },
        })
    }

    pub fn finished(&self, fold: &Fold, ending: &Ending, completed: i64) -> Value {
        let mut info = json!({
            "id": self.id,
            "type": "assistant",
            "agent": AGENT,
            "model": model_json(&self.model),
            "content": text_parts(&fold.text),
            "cost": 0,
            "tokens": fold.tokens().to_json(),
            "time": { "created": self.created, "completed": completed },
        });
        let (finish, raw, error) = match ending {
            Ending::Succeeded => (fold.finish(), Some(fold.raw_finish()), None),
            Ending::Interrupted => ("error", None, Some(Ending::aborted())),
            Ending::Failed(error) => ("error", None, Some(error.clone())),
        };
        if let Some(fields) = info.as_object_mut() {
            fields.insert("finish".to_owned(), Value::from(finish));
            if let Some(raw) = raw {
                fields.insert("rawFinish".to_owned(), Value::from(raw));
            }
            if let Some(error) = error {
                fields.insert("error".to_owned(), error_json(&error));
            }
        }
        info
    }
}

pub fn message_text(record: &MessageRecord) -> String {
    match record.role {
        Role::User => record
            .info
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned(),
        Role::Assistant => record
            .info
            .get("content")
            .and_then(Value::as_array)
            .map(|parts| {
                parts
                    .iter()
                    .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
                    .filter_map(|part| part.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default(),
    }
}

pub fn history(rows: &[MessageRecord]) -> Vec<Turn> {
    let mut turns: Vec<Turn> = Vec::new();
    for row in rows.iter().rev() {
        if row.role == Role::Assistant && row.completed_at.is_none() {
            continue;
        }
        let text = message_text(row);
        if text.trim().is_empty() {
            continue;
        }
        match turns.last_mut() {
            Some(last) if last.role == row.role => {
                last.text.push_str("\n\n");
                last.text.push_str(&text);
            },
            _ => turns.push(Turn {
                role: row.role,
                text,
            }),
        }
    }
    turns
}

pub fn prior_tokens(rows: &[MessageRecord]) -> Tokens {
    rows.iter()
        .filter(|row| row.role == Role::Assistant && row.completed_at.is_some())
        .filter_map(|row| row.info.get("tokens"))
        .map(Tokens::from_json)
        .fold(Tokens::default(), Tokens::plus)
}
