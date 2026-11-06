//! What every `/drive` page shares: the document service the pages
//! authorize their loads with, the caller those loads run as, and the
//! refusal when the install runs no document store.

use std::sync::Arc;

use systemprompt::files::documents::DocumentService;
use systemprompt::models::auth::{AuthenticatedUser, Permission};

use systemprompt_web_admin_core::error::AdminError;
use systemprompt_web_admin_core::ssr::page::Page;
use systemprompt_web_admin_core::types::UserContext;

pub const USER_BASE_URL: &str = "/drive";

/// The document service the Drive pages authorize page loads with.
#[derive(Clone)]
pub struct DriveDocuments(pub Arc<DocumentService>);

impl std::fmt::Debug for DriveDocuments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DriveDocuments").finish_non_exhaustive()
    }
}

pub(crate) fn ensure_enabled(shell: &Page) -> Result<(), AdminError> {
    if shell.engine.documents_enabled() {
        Ok(())
    } else {
        Err(AdminError::NotFound(
            "Document store is disabled.".to_owned(),
        ))
    }
}

pub(crate) fn caller(user: &UserContext) -> AuthenticatedUser {
    let permission = if user.is_admin {
        Permission::Admin
    } else {
        Permission::User
    };
    AuthenticatedUser::new_with_roles(
        user.user_id.clone(),
        user.username.clone(),
        user.email.as_str().to_owned(),
        vec![permission],
        user.roles.clone(),
    )
}
