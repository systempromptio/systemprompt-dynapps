//! `POST /users` — operator-created accounts and their credential bootstrap.
//!
//! Split from `users.rs` at the 300-line ceiling. A user created here gets no
//! credential of its own: Astound SSO is the only door, so the row waits to be
//! adopted by the first assertion carrying the matching email and a mapped
//! Active Directory group.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::error::{AdminError, AdminResult};
use crate::repositories;
use crate::repositories::organizations::crud::OrgAssignment;
use crate::types::{CreateUserRequest, UserContext};

// Why: there is no link to hand back — the note tells the operator what to say
// instead, and it differs when the email's domain claims no organization.
#[derive(Debug, serde::Serialize)]
pub(crate) struct CreatedUserResponse {
    pub user: crate::types::UserSummary,
    pub invite_note: String,
}

pub(crate) async fn create_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Json(body): Json<CreateUserRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    // Why: creating a colleague with an elevated role is a platform-admin act,
    // not an org-admin one.
    if !user_ctx.is_platform_admin && body.roles.iter().any(|r| r != "user") {
        return Err(AdminError::Forbidden(
            "Only platform administrators may create users with elevated roles".to_owned(),
        ));
    }
    let created = repositories::users::mutations::create_user(&pool, &body).await?;
    let user = created.summary;

    let invite_note = sign_in_note(&created.org);

    let p = Arc::clone(&pool);
    let uid = user_ctx.user_id.clone();
    let new_user_id = user.user_id.clone();
    let name = user
        .display_name
        .clone()
        .unwrap_or_else(|| user.user_id.as_str().to_owned());
    tokio::spawn(async move {
        activity::record(
            &p,
            NewActivity::entity_created(&uid, ActivityEntity::User, new_user_id.as_str(), &name),
        )
        .await;
    });
    Ok((
        StatusCode::CREATED,
        Json(CreatedUserResponse { user, invite_note }),
    )
        .into_response())
}

// Why: an admin-created row needs no sign-in link. Astound SSO is the only
// door, and with `auto_provision` on the AD group is the invitation — the
// account created here is adopted by the first assertion carrying a mapped
// group and the matching email.
//
// The three cases are told apart deliberately. A default-organization fallback
// still produces a working account, so it is not a failure to report as one —
// but it is a seat consumed on an organization the operator did not pick, and
// silently reporting it as an ordinary domain match is how a customer's seat
// count drifts without anyone noticing.
fn sign_in_note(org: &OrgAssignment) -> String {
    match org {
        OrgAssignment::ByDomain(_) => {
            "User created. They sign in with Astound SSO — there is no link to send. \
             Access follows their Active Directory group membership."
        },
        OrgAssignment::ByDefault(_) => {
            "User created, but no organization claims this email domain — they joined \
             the default organization and consume a seat there. They sign in with \
             Astound SSO; claim the domain to attach them to the right organization."
        },
        OrgAssignment::Unassigned => {
            "User created, but no organization claims this email domain and there is \
             no default organization. They can still sign in with Astound SSO; claim \
             the domain to attach them to an organization."
        },
    }
    .to_owned()
}
