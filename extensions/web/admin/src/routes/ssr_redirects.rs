//! The redirect handlers the SSR router points at.
//!
//! They live beside the router rather than in it because the router file is a
//! table of route literals — the admin contract suite reads it as one — and a
//! handler body in the middle of that table is the thing that makes it stop
//! reading like a table.

use axum::Extension;
use axum::response::Redirect;

use crate::types::UserContext;

// Why: admins land on the analytics dashboard; everyone else still gets their
// profile — the non-admin gate would bounce them off /admin/analytics anyway.
pub(super) async fn root(Extension(user_ctx): Extension<UserContext>) -> Redirect {
    if user_ctx.is_console {
        Redirect::to("/admin/analytics")
    } else {
        Redirect::to("/admin/profile")
    }
}

pub(super) async fn catalog() -> Redirect {
    Redirect::permanent("/admin/catalog/plugins")
}

// Why: the singular path redirects rather than 404s — it was the sidebar link,
// and it is in bookmarks from before marketplaces had a page of their own.
pub(super) async fn marketplaces() -> Redirect {
    Redirect::permanent("/admin/catalog/marketplaces")
}
