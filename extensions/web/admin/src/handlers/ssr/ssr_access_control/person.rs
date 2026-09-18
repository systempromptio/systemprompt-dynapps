//! The "Check a person" tab: pick one account and see exactly what the
//! resolver gives them, rendered by the same view the person's own Access
//! tab uses.
//!
//! The picker is a plain form so the tab works without JavaScript; the
//! selected id comes from the query string, which is a validated source for
//! a `UserId`.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::pickable_users::{PICKABLE_USER_LIMIT, list_pickable_users};
use crate::handlers::ssr::ssr_users::detail::{AccessTabView, load_access_tab};
use crate::handlers::ssr::types::PickableUserView;
use crate::repositories;

#[derive(Debug, Serialize)]
pub(crate) struct PersonHeaderView {
    pub user_id: UserId,
    pub name: String,
    pub email: String,
    pub href: String,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct PersonCheckView {
    pub people: Vec<PickableUserView>,
    pub capped: bool,
    pub selected: Option<String>,
    pub not_found: bool,
    pub header: Option<PersonHeaderView>,
}

pub(super) async fn build(
    pool: &PgPool,
    selected: Option<&str>,
) -> (PersonCheckView, Option<AccessTabView>) {
    let people = list_pickable_users(pool, selected).await;
    let capped = people.len() >= PICKABLE_USER_LIMIT;
    let mut view = PersonCheckView {
        people,
        capped,
        selected: selected.map(str::to_owned),
        not_found: false,
        header: None,
    };
    let Some(selected) = selected else {
        return (view, None);
    };
    let user_id = UserId::new(selected);
    let detail = repositories::users::queries::find_user_detail(pool, &user_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "check a person: user lookup failed"))
        .ok()
        .flatten();
    let Some(detail) = detail else {
        view.not_found = true;
        return (view, None);
    };
    let name = detail
        .display_name
        .clone()
        .filter(|n| !n.trim().is_empty())
        .or_else(|| detail.email.as_ref().map(ToString::to_string))
        .unwrap_or_else(|| selected.to_owned());
    view.header = Some(PersonHeaderView {
        user_id: user_id.clone(),
        name,
        email: detail
            .email
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default(),
        href: format!("/admin/users/{}?tab=access", urlencoding::encode(selected)),
    });
    (view, Some(load_access_tab(pool, &user_id).await))
}
