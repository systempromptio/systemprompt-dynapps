//! The replies running in this process, and the live feed of each thread.
//!
//! One [`Claim`] per thread at a time: a prompt holds it while it starts a
//! reply and its task holds it until the reply has ended, so a second prompt
//! is refused and a reply left unfinished by an earlier process can be told
//! apart from one running here (nothing here claims it). Each thread with a
//! claim or a follower has a broadcast the reply publishes its events to;
//! followers read the store for anything they missed. Everything lives in
//! this process: one replica, as the relay.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use systemprompt::traits::BackgroundTasks;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use crate::ids::{MessageId, ThreadId};
use crate::store::EventRecord;

pub const FEED_CAPACITY: usize = 1024;

/// A reply running in this process: the prompt that started it, its
/// assistant message, the token `interrupt` cancels and the one its task
/// cancels when it has ended.
#[derive(Debug, Clone)]
pub struct Running {
    pub prompt: MessageId,
    pub reply: MessageId,
    pub cancel: CancellationToken,
    pub done: CancellationToken,
}

#[derive(Debug)]
struct Slot {
    feed: broadcast::Sender<EventRecord>,
    claimed: bool,
    running: Option<Running>,
}

impl Slot {
    fn new() -> Self {
        Self {
            feed: broadcast::channel(FEED_CAPACITY).0,
            claimed: false,
            running: None,
        }
    }

    fn unused(&self) -> bool {
        !self.claimed && self.feed.receiver_count() == 0
    }
}

#[derive(Debug, Default)]
struct Inner {
    slots: Mutex<HashMap<ThreadId, Slot>>,
    tasks: BackgroundTasks,
}

/// The registry. Cheap to clone; one per process.
#[derive(Debug, Clone, Default)]
pub struct Replies(Arc<Inner>);

/// Why a claim was refused: the thread is starting or running a reply.
#[derive(Debug, Clone)]
pub struct Busy(pub Option<Running>);

impl Replies {
    pub fn tasks(&self) -> &BackgroundTasks {
        &self.0.tasks
    }

    fn with<T>(&self, apply: impl FnOnce(&mut HashMap<ThreadId, Slot>) -> T) -> T {
        let mut slots = self.0.slots.lock().unwrap_or_else(PoisonError::into_inner);
        apply(&mut slots)
    }

    pub fn claim(&self, thread: &ThreadId) -> Result<Claim, Busy> {
        self.with(|slots| {
            let slot = slots.entry(thread.clone()).or_insert_with(Slot::new);
            if slot.claimed {
                return Err(Busy(slot.running.clone()));
            }
            slot.claimed = true;
            Ok(Claim {
                replies: self.clone(),
                thread: thread.clone(),
                feed: slot.feed.clone(),
                done: None,
            })
        })
    }

    pub fn running(&self, thread: &ThreadId) -> Option<Running> {
        self.with(|slots| slots.get(thread).and_then(|slot| slot.running.clone()))
    }

    pub fn follow(&self, thread: &ThreadId) -> Follow {
        let receiver = self.with(|slots| {
            slots
                .entry(thread.clone())
                .or_insert_with(Slot::new)
                .feed
                .subscribe()
        });
        Follow {
            receiver,
            guard: FollowGuard {
                replies: self.clone(),
                thread: thread.clone(),
            },
        }
    }

    pub async fn settled(&self, thread: &ThreadId, limit: Duration) -> bool {
        let Some(running) = self.running(thread) else {
            return true;
        };
        tokio::time::timeout(limit, running.done.cancelled())
            .await
            .is_ok()
    }

    fn forget_if_unused(&self, thread: &ThreadId) {
        self.with(|slots| {
            if slots.get(thread).is_some_and(Slot::unused) {
                slots.remove(thread);
            }
        });
    }
}

/// The right to start, run or settle the reply of one thread. Dropping it
/// frees the thread; when it carried a running reply, that reply's `done` is
/// cancelled after the thread is free.
#[derive(Debug)]
pub struct Claim {
    replies: Replies,
    thread: ThreadId,
    feed: broadcast::Sender<EventRecord>,
    done: Option<CancellationToken>,
}

impl Claim {
    pub const fn thread(&self) -> &ThreadId {
        &self.thread
    }

    pub fn run(&mut self, running: Running) {
        self.done = Some(running.done.clone());
        self.replies.with(|slots| {
            if let Some(slot) = slots.get_mut(&self.thread) {
                slot.running = Some(running);
            }
        });
    }

    pub fn publish(&self, records: &[EventRecord]) {
        for record in records {
            if self.feed.send(record.clone()).is_err() {
                tracing::trace!(seq = record.seq, "dynwork chats event has no live follower");
            }
        }
    }
}

impl Drop for Claim {
    fn drop(&mut self) {
        self.replies.with(|slots| {
            if let Some(slot) = slots.get_mut(&self.thread) {
                slot.claimed = false;
                slot.running = None;
                if slot.unused() {
                    slots.remove(&self.thread);
                }
            }
        });
        if let Some(done) = &self.done {
            done.cancel();
        }
    }
}

/// A follower's subscription; dropping it forgets the thread's feed when
/// nothing else uses it.
#[derive(Debug)]
pub struct Follow {
    pub receiver: broadcast::Receiver<EventRecord>,
    guard: FollowGuard,
}

impl Follow {
    pub const fn thread(&self) -> &ThreadId {
        &self.guard.thread
    }
}

#[derive(Debug)]
struct FollowGuard {
    replies: Replies,
    thread: ThreadId,
}

impl Drop for FollowGuard {
    fn drop(&mut self) {
        self.replies.forget_if_unused(&self.thread);
    }
}
