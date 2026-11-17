//! A reply an earlier process left unfinished.
//!
//! The registry is per process, so an assistant message with no
//! `completed_at` that nothing here has claimed was being written when the
//! server stopped. It is settled on the first touch of its thread after the
//! restart (a prompt, an `interrupt` or a `/log` reader), as the holder of
//! the thread's claim: its text is rebuilt from its stored events, the
//! events it never wrote are added (`session.text.ended`, then
//! `session.execution.failed` with error type `restarted`) and it is
//! compacted. A reply whose terminal event was written before the stop is
//! only compacted, with the ending that event says.

// JSON: the stored events and messages are `OpenCode` documents.
use serde_json::Value;
use systemprompt::identifiers::UserId;

use crate::error::ChatsError;
use crate::ids::MessageId;
use crate::reply::document::{Ending, Fold, ReplyDoc};
use crate::reply::events::{self, Tokens};
use crate::reply::model::{ReplyError, raw_finish_of};
use crate::reply::registry::Claim;
use crate::reply::runner::Writer;
use crate::state::ChatsState;
use crate::store::{MAX_PAGE, MessageRecord, ModelRef, Role};

pub async fn settle_stale(
    state: &ChatsState,
    user: &UserId,
    claim: &Claim,
) -> Result<(), ChatsError> {
    let thread = claim.thread();
    let Some(rows) = state.store.list_messages(user, thread, None, 1).await? else {
        return Ok(());
    };
    let Some(last) = rows.first() else {
        return Ok(());
    };
    if last.role != Role::Assistant || last.completed_at.is_some() {
        return Ok(());
    }
    let doc = stored_doc(last);
    let (fold, closed) = replay(state, user, claim, &doc.id).await?;
    let already_closed = closed.is_some();
    let ending = closed.unwrap_or_else(|| Ending::Failed(Ending::restarted()));
    tracing::info!(%thread, reply = %doc.id, "dynwork chats settling a reply an earlier process left");
    let writer = Writer {
        state,
        user,
        claim,
        doc: &doc,
    };
    if writer
        .settle(&fold, &ending, Tokens::default(), already_closed)
        .await
        .is_err()
    {
        tracing::warn!(%thread, "dynwork chats stale reply not settled");
    }
    Ok(())
}

fn stored_doc(record: &MessageRecord) -> ReplyDoc {
    let text = |path: &[&str]| {
        path.iter()
            .try_fold(&record.info, |node, key| node.get(key))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned()
    };
    ReplyDoc {
        id: record.id.clone(),
        model: ModelRef {
            provider_id: text(&["model", "providerID"]),
            model_id: text(&["model", "id"]),
        },
        created: record
            .info
            .get("time")
            .and_then(|time| time.get("created"))
            .and_then(Value::as_i64)
            .unwrap_or_else(|| record.created_at.timestamp_millis()),
    }
}

async fn replay(
    state: &ChatsState,
    user: &UserId,
    claim: &Claim,
    reply: &MessageId,
) -> Result<(Fold, Option<Ending>), ChatsError> {
    let mut fold = Fold::default();
    let mut closed = None;
    let mut after = 0;
    loop {
        let page = state
            .store
            .events_after(user, claim.thread(), after, MAX_PAGE)
            .await?
            .unwrap_or_default();
        let full = page.len() >= MAX_PAGE as usize;
        for record in &page {
            after = record.seq;
            if record.message.as_ref() != Some(reply) {
                continue;
            }
            closed = apply(&mut fold, &record.kind, &record.data).or(closed);
        }
        if !full {
            return Ok((fold, closed));
        }
    }
}

fn apply(fold: &mut Fold, kind: &str, data: &Value) -> Option<Ending> {
    let text = |key: &str| data.get(key).and_then(Value::as_str).unwrap_or("");
    match kind {
        events::TEXT_STARTED => fold.text_open = true,
        events::TEXT_DELTA => fold.text.push_str(text("delta")),
        events::TEXT_ENDED => {
            fold.text_open = false;
            text("text").clone_into(&mut fold.text);
        },
        events::STEP_ENDED => {
            let tokens = data
                .get("tokens")
                .map(Tokens::from_json)
                .unwrap_or_default();
            fold.usage.input = tokens.input;
            fold.usage.output = tokens.output;
            fold.usage.reasoning = tokens.reasoning;
            fold.usage.cache_read = tokens.cache_read;
            fold.usage.cache_write = tokens.cache_write;
            fold.usage.raw_finish = Some(raw_finish_of(text("rawFinish")));
        },
        events::EXECUTION_SUCCEEDED => return Some(Ending::Succeeded),
        events::EXECUTION_INTERRUPTED => return Some(Ending::Interrupted),
        events::EXECUTION_FAILED => {
            let error = data.get("error");
            let field = |key: &str| {
                error
                    .and_then(|error| error.get(key))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned()
            };
            return Some(Ending::Failed(ReplyError::new(
                &field("type"),
                field("message"),
            )));
        },
        _ => {},
    }
    None
}
