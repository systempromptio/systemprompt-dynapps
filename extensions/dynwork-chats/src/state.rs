//! What every chats route shares, built once per process.
//!
//! The extension builds it, or a suite with a memory store, a fake device
//! check, a recorder of the change hints, its own clock and a scripted model.
//! It includes the replies running in this process.

use std::ops::Deref;
use std::sync::Arc;

use systemprompt::models::auth::TokenAuthority;

use crate::auth::DeviceCheck;
use crate::clock::Clock;
use crate::hint::Hints;
use crate::reply::model::ModelSource;
use crate::reply::registry::Replies;
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
