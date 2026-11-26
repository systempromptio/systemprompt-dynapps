//! What an operation answers when it cannot do what was asked, as data: the
//! REST layer maps each variant to a status and the MCP server to a tool
//! error, and neither has to look inside the store.
//!
//! A user who cannot read an article or a space is told it was not found,
//! never that it is forbidden, so the answer does not reveal what exists.
//! `Forbidden` is for someone who can read what they ask about but needs more
//! access to change it.

use crate::error::BrainError;
use crate::properties::PropertyError;

#[derive(Debug, thiserror::Error)]
pub enum BrainFault {
    #[error("not found")]
    NotFound,
    #[error("this needs more access than you have")]
    Forbidden,
    #[error("{0}")]
    Invalid(&'static str),
    #[error(transparent)]
    Refused(BrainError),
    #[error(transparent)]
    Property(PropertyError),
    #[error("{0}")]
    Unavailable(&'static str),
    #[error("the file is larger than the {0} bytes allowed")]
    TooLarge(u64),
    #[error("brain store failure")]
    Internal(#[source] BrainError),
}

impl From<BrainError> for BrainFault {
    fn from(error: BrainError) -> Self {
        match error {
            BrainError::SpaceNotFound(_) | BrainError::ArticleNotFound(_) => Self::NotFound,
            BrainError::Database(_) | BrainError::Storage(_) => Self::Internal(error),
            BrainError::Property(source) => Self::Property(source),
            other => Self::Refused(other),
        }
    }
}

impl From<PropertyError> for BrainFault {
    fn from(error: PropertyError) -> Self {
        Self::Property(error)
    }
}
