//! The task that writes one reply.
//!
//! It runs on the registry's `BackgroundTasks`, holding the thread's
//! [`Claim`] until the reply has ended, so a follower disconnecting never
//! stops it. Each event is written to the store first and then published to
//! the thread's followers, so a reader that joins late finds in the store
//! everything the live feed already sent. The lifecycle is
//! `session.execution.started`, `session.step.started`, then for the text
//! `session.text.started` and one `session.text.delta` per chunk the model
//! emits, and at the end `session.text.ended` with the whole text and one of:
//! `session.step.ended` + `session.usage.updated` +
//! `session.execution.succeeded`; `session.execution.interrupted` (an
//! `interrupt`); `session.execution.failed` (a model error). The last events
//! are written, the reply is compacted into its message, and only then are
//! they published, so a client that reloads the history on the terminal
//! event finds the finished message. Then the person's desktops get the change
//! hint, whichever way the reply ended.

use std::sync::Arc;

use futures_util::StreamExt;
use systemprompt::identifiers::UserId;
use tokio_util::sync::CancellationToken;

use crate::ids::{MessageId, ThreadId};
use crate::reply::document::{Ending, Fold, ReplyDoc, text_parts};
use crate::reply::events::{self, Tokens};
use crate::reply::model::{ModelChunk, ReplyModel, ReplyRequest};
use crate::reply::registry::Claim;
use crate::state::ChatsState;
use crate::store::{CompletedReply, EventRecord, NewEvent};

pub const TASK_NAME: &str = "dynwork_chats_reply";

/// The reply could not be written on: the store failed (logged) or the
/// thread is gone.
#[derive(Debug, Clone, Copy)]
pub struct Stop;

/// Writes one reply's events, as the holder of the thread's claim.
#[derive(Debug)]
pub struct Writer<'a> {
    pub state: &'a ChatsState,
    pub user: &'a UserId,
    pub claim: &'a Claim,
    pub doc: &'a ReplyDoc,
}

impl Writer<'_> {
    const fn thread(&self) -> &ThreadId {
        self.claim.thread()
    }

    const fn reply(&self) -> &MessageId {
        &self.doc.id
    }

    pub async fn write(&self, events: &[NewEvent]) -> Result<Vec<EventRecord>, Stop> {
        match self
            .state
            .store
            .append_events(self.user, self.thread(), events, self.state.clock.now())
            .await
        {
            Ok(Some(records)) => Ok(records),
            Ok(None) => {
                tracing::debug!(thread = %self.thread(), "dynwork chats reply stopped: thread gone");
                Err(Stop)
            },
            Err(error) => {
                tracing::error!(error = ?error, thread = %self.thread(), "dynwork chats reply events not written");
                Err(Stop)
            },
        }
    }

    pub async fn emit(&self, events: &[NewEvent]) -> Result<(), Stop> {
        let records = self.write(events).await?;
        self.claim.publish(&records);
        Ok(())
    }

    pub async fn settle(
        &self,
        fold: &Fold,
        ending: &Ending,
        prior: Tokens,
        closed: bool,
    ) -> Result<(), Stop> {
        let thread = self.thread();
        let reply = self.reply();
        let doc = self.doc;
        let mut closing = Vec::new();
        if fold.text_open {
            closing.push(events::text_ended(thread, reply, &fold.text));
        }
        if !closed {
            match ending {
                Ending::Succeeded => {
                    let tokens = fold.tokens();
                    closing.push(events::step_ended(
                        thread,
                        reply,
                        fold.finish(),
                        fold.raw_finish(),
                        tokens,
                    ));
                    closing.push(events::usage_updated(thread, reply, prior.plus(tokens)));
                    closing.push(events::execution_succeeded(thread, reply));
                },
                Ending::Interrupted => closing.push(events::execution_interrupted(thread, reply)),
                Ending::Failed(error) => {
                    closing.push(events::execution_failed(thread, reply, error));
                },
            }
        }
        let records = self.write(&closing).await?;
        let now = self.state.clock.now();
        let completed = CompletedReply {
            message: reply.clone(),
            info: doc.finished(fold, ending, now.timestamp_millis()),
            parts: text_parts(&fold.text),
        };
        let compacted = match self
            .state
            .store
            .compact_reply(self.user, thread, &completed, now)
            .await
        {
            Ok(Some(_)) => true,
            Ok(None) => {
                tracing::debug!(%thread, "dynwork chats reply ended on a thread that is gone");
                false
            },
            Err(error) => {
                tracing::error!(error = ?error, %thread, "dynwork chats reply not compacted");
                return Err(Stop);
            },
        };
        self.claim.publish(&records);
        if compacted {
            self.state.hints.changed(self.user, None).await;
        }
        Ok(())
    }
}

/// One reply to run.
#[derive(Debug)]
pub struct Job {
    pub state: ChatsState,
    pub user: UserId,
    pub claim: Claim,
    pub engine: Arc<dyn ReplyModel>,
    pub doc: ReplyDoc,
    pub request: ReplyRequest,
    pub prior: Tokens,
    pub cancel: CancellationToken,
}

pub fn spawn(job: Job) {
    let tasks = job.state.replies.tasks().clone();
    tasks.spawn(TASK_NAME, run(job));
}

async fn run(job: Job) {
    let Job {
        state,
        user,
        claim,
        engine,
        doc,
        request,
        prior,
        cancel,
    } = job;
    let writer = Writer {
        state: &state,
        user: &user,
        claim: &claim,
        doc: &doc,
    };
    let mut fold = Fold::default();
    let Ok(ending) = generate(&writer, &mut fold, &engine, request, &cancel).await else {
        return;
    };
    if writer.settle(&fold, &ending, prior, false).await.is_err() {
        tracing::warn!(thread = %claim.thread(), "dynwork chats reply left unfinished");
    }
}

async fn generate(
    writer: &Writer<'_>,
    fold: &mut Fold,
    engine: &Arc<dyn ReplyModel>,
    request: ReplyRequest,
    cancel: &CancellationToken,
) -> Result<Ending, Stop> {
    let thread = writer.thread();
    let reply = writer.reply();
    let doc = writer.doc;
    writer
        .emit(&[
            events::execution_started(thread, reply),
            events::step_started(thread, reply, &doc.model, doc.created),
        ])
        .await?;
    let opened = tokio::select! {
        biased;
        () = cancel.cancelled() => return Ok(Ending::Interrupted),
        opened = engine.stream(request) => opened,
    };
    let mut stream = match opened {
        Ok(stream) => stream,
        Err(error) => return Ok(Ending::Failed(error)),
    };
    loop {
        let chunk = tokio::select! {
            biased;
            () = cancel.cancelled() => return Ok(Ending::Interrupted),
            chunk = stream.next() => chunk,
        };
        match chunk {
            None => return Ok(Ending::Succeeded),
            Some(ModelChunk::Text(text)) if text.is_empty() => {},
            Some(ModelChunk::Text(text)) => {
                let mut batch = Vec::with_capacity(2);
                if !fold.text_open {
                    fold.text_open = true;
                    batch.push(events::text_started(thread, reply));
                }
                batch.push(events::text_delta(thread, reply, &text));
                writer.emit(&batch).await?;
                fold.text.push_str(&text);
            },
            Some(ModelChunk::Usage(usage)) => fold.add_usage(&usage),
            Some(ModelChunk::Error(error)) => return Ok(Ending::Failed(error)),
        }
    }
}
