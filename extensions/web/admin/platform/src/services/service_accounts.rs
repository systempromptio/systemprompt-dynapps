//! Service-account identities, read and written through core's user
//! repository.
//!
//! Core owns `users.account_kind` and refuses every interactive login for a
//! service row; this crate owns what such an account may present to a
//! provider (`machine_credentials`). The router root builds one
//! [`ServiceAccounts`] and layers it as an `Extension`.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::UserId;
use systemprompt::users::{ArchiveParams, NewServiceAccount, User, UserRepository, UserStatus};
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use systemprompt_web_admin_core::types::roles_grant_manage;

#[derive(Debug, Clone)]
pub struct ServiceAccount {
    pub user_id: UserId,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for ServiceAccount {
    fn from(user: User) -> Self {
        Self {
            user_id: user.id,
            name: user.display_name.unwrap_or(user.name),
            email: user.email,
            created_at: user.created_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServiceAccounts {
    users: UserRepository,
}

impl ServiceAccounts {
    #[must_use]
    pub const fn new(users: UserRepository) -> Self {
        Self { users }
    }

    pub async fn is_service_account(&self, user: &UserId) -> AdminResult<bool> {
        Ok(self.find(user).await?.is_some())
    }

    pub async fn is_admin(&self, user: &UserId) -> AdminResult<bool> {
        let found = self
            .users
            .find_by_id(user)
            .await
            .map_err(AdminError::internal)?;
        Ok(found.is_some_and(|user| roles_grant_manage(&user.roles)))
    }

    pub async fn find(&self, user: &UserId) -> AdminResult<Option<ServiceAccount>> {
        let found = self
            .users
            .find_by_id(user)
            .await
            .map_err(AdminError::internal)?;
        Ok(found
            .filter(|user| user.account_kind.is_service() && user.status == UserStatus::Active)
            .map(ServiceAccount::from))
    }

    pub async fn list(&self) -> AdminResult<Vec<ServiceAccount>> {
        let rows = self
            .users
            .list_service_accounts()
            .await
            .map_err(AdminError::internal)?;
        Ok(rows
            .into_iter()
            .filter(|user| user.status == UserStatus::Active)
            .map(ServiceAccount::from)
            .collect())
    }

    pub async fn create(
        &self,
        name: &str,
        email: &str,
        created_by: &UserId,
    ) -> AdminResult<UserId> {
        if self
            .users
            .find_by_email(email)
            .await
            .map_err(AdminError::internal)?
            .is_some()
        {
            return Err(AdminError::Conflict(
                "An account with that email already exists".to_owned(),
            ));
        }
        let ensured = self
            .users
            .ensure_service_account(NewServiceAccount {
                name: name.to_owned(),
                email: email.to_owned(),
                created_by: created_by.clone(),
            })
            .await
            .map_err(AdminError::internal)?;
        if !ensured.created {
            return Err(AdminError::Conflict(
                "An account with that email already exists".to_owned(),
            ));
        }
        Ok(ensured.user.id)
    }

    pub async fn delete(&self, user: &UserId, actor: &UserId) -> AdminResult<()> {
        self.users
            .archive(
                user,
                ArchiveParams {
                    archived_by: Some(actor),
                    reason: Some("service account deleted from the admin console"),
                    legal_hold: false,
                },
            )
            .await
            .map_err(AdminError::internal)?;
        Ok(())
    }
}
