//! The extension's error. Repository calls propagate with `?`; the HTTP and
//! MCP layers of later phases map each variant to their own answer.

use systemprompt::traits::FileStorageError;

use crate::ids::{ArticleId, SpaceId};
use crate::properties::PropertyError;

#[derive(Debug, thiserror::Error)]
pub enum BrainError {
    #[error("brain store failure")]
    Database(#[from] sqlx::Error),
    #[error("file storage failure")]
    Storage(#[from] FileStorageError),
    #[error("invalid properties")]
    Property(#[from] PropertyError),
    #[error("space {0} not found")]
    SpaceNotFound(SpaceId),
    #[error("article {0} not found")]
    ArticleNotFound(ArticleId),
    #[error("language `{lang}` is not one of the languages of space {space}")]
    LanguageNotInSpace { lang: String, space: SpaceId },
    #[error("article {0} cannot move below itself")]
    MoveIntoOwnSubtree(ArticleId),
    #[error("article {0} cannot move to another space")]
    MoveAcrossSpaces(ArticleId),
    #[error("article {0} is in the trash")]
    Trashed(ArticleId),
    #[error("invalid input: {0}")]
    Invalid(&'static str),
}
