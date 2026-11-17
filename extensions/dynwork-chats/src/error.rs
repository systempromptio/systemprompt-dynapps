//! The extension's error. Each variant has a stable `key` that the HTTP layer
//! (a later phase) puts in `{ "error": { "key", "message" } }`.

use crate::ids::IdError;
use crate::store::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum ChatsError {
    #[error("invalid identifier")]
    Id(#[from] IdError),
    #[error("chat store failure")]
    Store(#[from] StoreError),
}

impl ChatsError {
    pub const fn key(&self) -> &'static str {
        match self {
            Self::Id(IdError::Cursor) => "invalid_cursor",
            Self::Id(IdError::Thread | IdError::Message) => "invalid_id",
            Self::Store(_) => "internal",
        }
    }
}
