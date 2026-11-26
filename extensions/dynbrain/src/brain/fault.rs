//! What an operation answers when it cannot do what was asked, as data: the
//! REST layer maps each variant to a status and the MCP server to a tool
//! error, and neither has to look inside the store.
//!
//! A user who cannot read an article or a space is told it was not found,
//! never that it is forbidden, so the answer does not reveal what exists.
//! `Forbidden` is for someone who can read what they ask about but needs more
//! access to change it.
//!
//! In proxy mode a refusal of the source is a [`SourceFault`] with the
//! source's own message: `Refused` is a `forbidden`, `NotFound` a
//! `not_found`, `Invalid` a refused argument and `Unavailable` a source that
//! could not be reached. Permissions are not this instance's to manage there:
//! `ManagedAtSource`.

use systemprompt::mcp::McpDomainError;

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
    #[error("Invalid tool input: {0}")]
    Arguments(#[source] serde_json::Error),
    #[error("the reply could not be encoded")]
    Encode(#[source] serde_json::Error),
    #[error("the permissions of this instance's documentation are managed at its source")]
    ManagedAtSource,
    #[error(transparent)]
    Source(#[from] SourceFault),
}

/// How the source of a proxy instance refused a call.
#[derive(Debug, thiserror::Error)]
pub enum SourceFault {
    #[error("the source refused the call: {0}")]
    Refused(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Invalid(String),
    #[error("the source of this instance is unavailable")]
    Unavailable(#[source] Box<McpDomainError>),
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
