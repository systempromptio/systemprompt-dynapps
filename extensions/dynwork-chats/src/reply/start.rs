//! Starting a reply on a prompt, and stopping one.
//!
//! A prompt is idempotent by its client-chosen `messageID`: the same id while
//! its reply runs, or after, answers with the same reply and starts nothing.
//! Another prompt while a reply runs is refused (`409 busy`), and so is a
//! prompt of a person who already has as many replies running as
//! [`Limits::max_running_replies`](crate::state::Limits::max_running_replies)
//! allows (`429 too_many_replies`): neither claims or writes anything. The
//! reply's id is derived from the prompt's time and ids, its model is the
//! prompt's, else the thread's, else the instance's default. Once the reply is
//! running the caller's other desktops get the change hint (not the one that
//! prompted, which follows the `/log`); a prompt that starts nothing sends
//! none.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;
use tokio_util::sync::CancellationToken;

use crate::auth::Caller;
use crate::error::ChatsError;
use crate::ids::{MessageId, ThreadId};
use crate::reply::document::{self, ReplyDoc};
use crate::reply::events;
use crate::reply::model::{ReplyModel, ReplyRequest};
use crate::reply::recover::settle_stale;
use crate::reply::registry::{Busy, Claim, Refused, Running};
use crate::reply::runner::{self, Job};
use crate::state::ChatsState;
use crate::store::{AppendOutcome, ModelRef, NewMessage, Role, ThreadRecord};

pub const HISTORY_LIMIT: u32 = 200;
pub const INTERRUPT_WAIT: Duration = Duration::from_secs(10);

/// A prompt as the route accepted it: text only in stage 1.
#[derive(Debug, Clone)]
pub struct Prompt {
    pub message: MessageId,
    pub text: String,
    pub model: Option<ModelRef>,
}

pub async fn live_thread(
    state: &ChatsState,
    user: &UserId,
    thread: &ThreadId,
) -> Result<ThreadRecord, ChatsError> {
    state
        .store
        .get_thread(user, thread)
        .await?
        .filter(|record| record.deleted_at.is_none())
        .ok_or(ChatsError::NotFound)
}

pub fn effective_model(
    prompt: &Prompt,
    record: &ThreadRecord,
    engine: &dyn ReplyModel,
) -> Option<ModelRef> {
    prompt
        .model
        .clone()
        .or_else(|| record.model.clone())
        .or_else(|| engine.default_model())
}

pub async fn start(
    state: &ChatsState,
    caller: &Caller,
    thread: &ThreadId,
    prompt: Prompt,
    engine: Arc<dyn ReplyModel>,
) -> Result<MessageId, ChatsError> {
    let user = &caller.user;
    let record = live_thread(state, user, thread).await?;
    let most = state.limits.max_running_replies;
    let mut claim = match state.replies.claim_reply(thread, user, most) {
        Ok(claim) => claim,
        Err(Refused::Busy(Busy(Some(running)))) if running.prompt == prompt.message => {
            return Ok(running.reply);
        },
        Err(Refused::Busy(_)) => return Err(ChatsError::Busy),
        Err(Refused::Full) => return Err(ChatsError::TooManyReplies),
    };
    settle_stale(state, user, &claim).await?;
    let model = effective_model(&prompt, &record, engine.as_ref()).ok_or(
        ChatsError::BadRequest("this thread has no model: send one with the prompt"),
    )?;
    let now = state.clock.now();
    let created = now.timestamp_millis();
    let reply = MessageId::derived_at(now, &format!("{thread}/{}", prompt.message));
    if let Some(earlier) = ask(state, user, thread, &prompt, &reply).await? {
        return Ok(earlier);
    }
    let rows = state
        .store
        .list_messages(user, thread, None, HISTORY_LIMIT)
        .await?
        .ok_or(ChatsError::NotFound)?;
    let doc = ReplyDoc {
        id: reply.clone(),
        model: model.clone(),
        created,
    };
    open_reply(state, user, thread, &doc, now).await?;
    if record.last_event_seq == 0 {
        announce(state, user, &claim, &record, &model).await?;
    }
    let cancel = CancellationToken::new();
    claim.run(Running {
        prompt: prompt.message,
        reply: reply.clone(),
        cancel: cancel.clone(),
        done: CancellationToken::new(),
    });
    runner::spawn(Job {
        state: state.clone(),
        user: user.clone(),
        claim,
        engine,
        doc,
        request: ReplyRequest {
            user: user.clone(),
            thread: thread.clone(),
            model,
            history: document::history(&rows),
        },
        prior: document::prior_tokens(&rows),
        cancel,
    });
    state.hints.changed(user, caller.peer).await;
    Ok(reply)
}

async fn open_reply(
    state: &ChatsState,
    user: &UserId,
    thread: &ThreadId,
    doc: &ReplyDoc,
    now: DateTime<Utc>,
) -> Result<(), ChatsError> {
    let answer = NewMessage {
        id: doc.id.clone(),
        role: Role::Assistant,
        info: doc.started(),
        parts: document::text_parts(""),
        completed_at: None,
    };
    match state
        .store
        .append_message(user, thread, &answer, now)
        .await?
    {
        AppendOutcome::Appended(_) => Ok(()),
        AppendOutcome::Duplicate(_) | AppendOutcome::NoThread => Err(ChatsError::IdTaken),
    }
}

async fn ask(
    state: &ChatsState,
    user: &UserId,
    thread: &ThreadId,
    prompt: &Prompt,
    reply: &MessageId,
) -> Result<Option<MessageId>, ChatsError> {
    let now = state.clock.now();
    let asked = NewMessage {
        id: prompt.message.clone(),
        role: Role::User,
        info: document::user_info(&prompt.message, &prompt.text, now.timestamp_millis(), reply),
        parts: document::text_parts(&prompt.text),
        completed_at: Some(now),
    };
    match state
        .store
        .append_message(user, thread, &asked, now)
        .await?
    {
        AppendOutcome::Appended(_) => Ok(None),
        AppendOutcome::Duplicate(existing) => document::reply_of(&existing)
            .filter(|_| existing.role == Role::User)
            .map(Some)
            .ok_or(ChatsError::IdTaken),
        AppendOutcome::NoThread => Err(ChatsError::NotFound),
    }
}

async fn announce(
    state: &ChatsState,
    user: &UserId,
    claim: &Claim,
    record: &ThreadRecord,
    model: &ModelRef,
) -> Result<(), ChatsError> {
    let created = events::session_created(&record.id, &record.title, model);
    let written = state
        .store
        .append_events(user, &record.id, &[created], state.clock.now())
        .await?
        .ok_or(ChatsError::NotFound)?;
    claim.publish(&written);
    Ok(())
}

pub async fn interrupt(
    state: &ChatsState,
    user: &UserId,
    thread: &ThreadId,
) -> Result<(), ChatsError> {
    live_thread(state, user, thread).await?;
    if let Some(running) = state.replies.running(thread) {
        running.cancel.cancel();
        if !state.replies.settled(thread, INTERRUPT_WAIT).await {
            tracing::warn!(%thread, "dynwork chats reply still ending after interrupt");
        }
        return Ok(());
    }
    settle_if_idle(state, user, thread).await
}

pub async fn settle_if_idle(
    state: &ChatsState,
    user: &UserId,
    thread: &ThreadId,
) -> Result<(), ChatsError> {
    match state.replies.claim(thread) {
        Ok(claim) => settle_stale(state, user, &claim).await,
        Err(Busy(_)) => Ok(()),
    }
}
