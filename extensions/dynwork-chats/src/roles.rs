//! The roles of the user a model list is for.
//!
//! Core's own bridge profile takes them from the user the credential names,
//! read from the users table, so the access rules that admit a model see what
//! they see on `/v1/messages`. [`Roles`] is the seam: [`UserRoles`] reads
//! core's user store, a suite sets them by hand.

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::database::{Database, DbPool};
use systemprompt::identifiers::UserId;
use systemprompt::traits::auth::UserProvider;
use systemprompt::users::{UserRepository, UserService};

use crate::error::ChatsError;

/// The roles of a user, or `None` when the instance has no active user of
/// that id. `#[async_trait]` because [`crate::reply::source::Gateway`] holds it
/// as `Arc<dyn Roles>`.
#[async_trait]
pub trait Roles: Send + Sync + Debug {
    async fn of(&self, user: &UserId) -> Result<Option<Vec<String>>, ChatsError>;
}

/// [`Roles`] over core's user store.
pub struct UserRoles {
    users: UserService,
}

impl Debug for UserRoles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserRoles").finish_non_exhaustive()
    }
}

impl UserRoles {
    pub fn new(database: &Database) -> Self {
        let pool: DbPool = Arc::new(Database::from_pools(
            database.pool(),
            Some(database.write_pool()),
        ));
        Self {
            users: UserService::new(Arc::new(UserRepository::new(&pool))),
        }
    }
}

#[async_trait]
impl Roles for UserRoles {
    async fn of(&self, user: &UserId) -> Result<Option<Vec<String>>, ChatsError> {
        let found = UserProvider::find_by_id(&self.users, user).await?;
        Ok(found
            .filter(|found| found.is_active)
            .map(|found| found.roles))
    }
}
