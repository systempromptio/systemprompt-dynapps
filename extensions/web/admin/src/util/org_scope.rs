//! Resolving the organization a listing may span for a given caller.
//!
//! This instance is single-tenant: every `admin` is a system-wide admin, so in
//! practice an admin resolves to [`OrgScope::AllOrganizations`] and this module
//! narrows nobody. It is kept whole because pooled multi-tenancy may return —
//! re-tightening `is_platform_admin` (see `middleware/mod.rs`) is the entirety
//! of that change, and every narrowing below starts working again.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::organizations::crud;
use crate::types::UserContext;

/// Whether a listing spans every organization, is pinned to one, or belongs to
/// a caller who is in none.
///
/// `AllOrganizations` is the cross-customer view. Naming the wide case rather
/// than spelling it as an absent filter keeps it out of reach of a caller who
/// never considered tenancy, which is why this type has no `Default`.
///
/// `NoOrganization` is a caller with no membership, held apart from
/// `Only("")` even though both reach the SQL as a predicate matching nothing.
/// The distinction is for the call site, not the query: a handler can tell a
/// reader that they belong to no organization, where a bare empty slug is
/// indistinguishable from a real organization that happens to have no
/// activity, and a page of zeros cannot say which it is showing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrgScope {
    AllOrganizations,
    Only(String),
    NoOrganization,
}

impl OrgScope {
    // Why: The null-skip bind every scoped query takes. `AllOrganizations`
    // becomes `NULL`, which the `($n IS NULL OR …)` predicate reads as "do not
    // narrow"; `Only` becomes the key to match. `NoOrganization` binds the
    // empty slug, which no organization can hold, so the predicate matches
    // nothing — the narrowest scope, for the caller entitled to nothing.
    #[must_use]
    pub const fn as_slug(&self) -> Option<&str> {
        match self {
            Self::AllOrganizations => None,
            Self::Only(key) => Some(key.as_str()),
            Self::NoOrganization => Some(""),
        }
    }

    // Why: the caller's own organization, or `NoOrganization`. Every user
    // should have a membership — provisioning joins them to the default
    // organization — so reaching `NoOrganization` means a user slipped
    // through, and the page should say so rather than show zeros.
    #[must_use]
    pub fn own(slug: Option<String>) -> Self {
        slug.filter(|s| !s.is_empty())
            .map_or(Self::NoOrganization, Self::Only)
    }
}

pub async fn listing_scope(pool: &PgPool, user_ctx: &UserContext) -> OrgScope {
    if user_ctx.is_platform_admin {
        return OrgScope::AllOrganizations;
    }
    // Why: An unattached caller is scoped to nothing, never widened. Widening
    // them to `AllOrganizations` would hand the least-attached caller the
    // widest view.
    OrgScope::own(own_org_slug(pool, &user_ctx.user_id).await)
}

async fn own_org_slug(pool: &PgPool, user_id: &UserId) -> Option<String> {
    match crud::find_organization_for_user(pool, user_id).await {
        Ok(slug) => slug,
        Err(e) => {
            tracing::warn!(error = %e, "find_organization_for_user failed; scoping to no organization");
            None
        },
    }
}

// Why: Whether `user_ctx` may act on `target`'s account.
//
// A platform admin may act on anyone. Everyone else may act only within their
// own organization, and only when both sides actually have one — two
// unattached users are not colleagues, so `(None, None)` is not a match.
pub async fn may_administer(pool: &PgPool, user_ctx: &UserContext, target: &UserId) -> bool {
    if user_ctx.is_platform_admin {
        return true;
    }
    let own = crud::find_organization_for_user(pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();
    let theirs = crud::find_organization_for_user(pool, target)
        .await
        .unwrap_or_default();
    matches!((own, theirs), (Some(a), Some(b)) if a == b)
}
