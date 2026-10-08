//! Who a principal may view conversation history for.
//!
//! Self always; the `admin` and `auditor` roles keep the unrestricted view;
//! a `project_manager` also sees every live member of the projects they are a
//! live member of (supplied by [`get_history_scope`]).
//! Resolution is a pure function over the request context, so the rule is
//! pinned by unit tests without a database.
//!
//! Which of the two a page asks for is a product decision, not a permission
//! one. `/admin/history` is titled "My conversations" and takes
//! [`own_history_scope`] whoever is looking (a `project_manager` excepted,
//! who cannot reach the org listing and sees their members there), because a
//! page named "my" that silently widens to the whole company for an admin is a
//! surprise in the wrong direction. The org-wide listing lives at
//! `/admin/conversations` and takes [`history_scope_for`], which is also what
//! the readers use to decide whether a viewer may open one conversation.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::scope::membership::list_managed_member_ids;
use crate::types::UserContext;
use crate::types::constants::ROLE_PROJECT_MANAGER;

/// The set of user ids a viewer's history queries are constrained to.
/// `All` is the admin/auditor view with no constraint; `Users` is everyone
/// else's explicit allowlist, which always contains the viewer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HistoryScope {
    All,
    Users(Vec<UserId>),
}

impl HistoryScope {
    #[must_use]
    pub fn may_view(&self, target: &UserId) -> bool {
        match self {
            Self::All => true,
            Self::Users(ids) => ids.iter().any(|id| id == target),
        }
    }

    // Why: Postgres binds `None` as a NULL text[] and the query skips the scope
    // predicate; an empty Vec would instead match nobody.
    #[must_use]
    pub fn user_ids(&self) -> Option<Vec<String>> {
        match self {
            Self::All => None,
            Self::Users(ids) => Some(ids.iter().map(|id| id.as_str().to_owned()).collect()),
        }
    }
}

#[must_use]
pub fn resolve_history_scope(
    viewer: &UserId,
    has_full_view: bool,
    managed_member_ids: Vec<UserId>,
) -> HistoryScope {
    if has_full_view {
        return HistoryScope::All;
    }
    let mut ids = managed_member_ids;
    if !ids.iter().any(|id| id == viewer) {
        ids.push(viewer.clone());
    }
    HistoryScope::Users(ids)
}

#[must_use]
pub fn has_full_history_view(ctx: &UserContext) -> bool {
    ctx.is_console || ctx.roles.iter().any(|r| r.eq_ignore_ascii_case("auditor"))
}

#[must_use]
pub fn history_scope_for(ctx: &UserContext) -> HistoryScope {
    resolve_history_scope(&ctx.user_id, has_full_history_view(ctx), Vec::new())
}

#[must_use]
pub fn own_history_scope(ctx: &UserContext) -> HistoryScope {
    HistoryScope::Users(vec![ctx.user_id.clone()])
}

#[must_use]
pub fn manages_member_history(ctx: &UserContext) -> bool {
    !has_full_history_view(ctx)
        && ctx
            .roles
            .iter()
            .any(|r| r.eq_ignore_ascii_case(ROLE_PROJECT_MANAGER))
}

pub async fn get_history_scope(
    pool: &PgPool,
    ctx: &UserContext,
) -> Result<HistoryScope, sqlx::Error> {
    let managed = if manages_member_history(ctx) {
        list_managed_member_ids(pool, &ctx.user_id).await?
    } else {
        Vec::new()
    };
    Ok(resolve_history_scope(
        &ctx.user_id,
        has_full_history_view(ctx),
        managed,
    ))
}
