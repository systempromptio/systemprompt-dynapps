//! What every chats route shares, built once per process.
//!
//! The extension builds it, or a suite with a memory store, a fake device
//! check, a recorder of the change hints, its own clock, a scripted model and
//! short [`Limits`]. It includes the replies running in this process.

use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use systemprompt::models::auth::TokenAuthority;

use crate::auth::DeviceCheck;
use crate::clock::Clock;
use crate::hint::Hints;
use crate::http::log::HEARTBEAT;
use crate::reply::registry::{MAX_RUNNING_REPLIES, Replies};
use crate::reply::runner::REPLY_IDLE;
use crate::reply::source::ModelSource;
use crate::store::ChatStore;

/// The collaborators behind [`ChatsState`].
#[derive(Debug)]
pub struct Shared {
    pub store: Arc<dyn ChatStore>,
    pub authority: Arc<dyn TokenAuthority>,
    pub devices: Arc<dyn DeviceCheck>,
    pub clock: Arc<dyn Clock>,
    pub models: Arc<dyn ModelSource>,
    pub hints: Arc<dyn Hints>,
    pub replies: Replies,
    pub limits: Limits,
}

/// When the server acts on its own.
///
/// The `/log` heartbeat, on which a phone's device is checked again; the
/// silence after which a model's stream is given up; and how many replies one
/// person may have running at once. Production takes the defaults; a suite
/// shortens the times.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub heartbeat: Duration,
    pub reply_idle: Duration,
    pub max_running_replies: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            heartbeat: HEARTBEAT,
            reply_idle: REPLY_IDLE,
            max_running_replies: MAX_RUNNING_REPLIES,
        }
    }
}

/// The state of the chats router: cheap to clone, one [`Shared`] behind it.
#[derive(Debug, Clone)]
pub struct ChatsState(Arc<Shared>);

impl Deref for ChatsState {
    type Target = Shared;

    fn deref(&self) -> &Shared {
        &self.0
    }
}

impl ChatsState {
    pub fn new(shared: Shared) -> Self {
        Self(Arc::new(shared))
    }
}
