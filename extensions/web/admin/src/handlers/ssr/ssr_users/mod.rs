//! The People pages: the `/admin/users` roster and the per-user detail page.
//!
//! Both are flat and URL-driven — every filter, sort, page and tab is a query
//! parameter, so any view an operator is looking at is a link they can send to
//! someone else.

pub(crate) mod detail;
mod roster;
mod scope_data;

pub(super) const BASE_URL: &str = "/admin/users";

pub(crate) use detail::user_detail_page as user_detail_by_id_page;
pub(crate) use roster::users_page;

use axum::extract::Query;
use axum::response::Redirect;

use crate::types::IdQuery;

pub(crate) async fn user_detail_page(Query(params): Query<IdQuery>) -> Redirect {
    params.id().map_or_else(
        || Redirect::permanent(BASE_URL),
        |id| Redirect::permanent(&format!("{BASE_URL}/{}", urlencoding::encode(id))),
    )
}
