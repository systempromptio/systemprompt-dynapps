//! The account list behind every server-rendered person picker.
//!
//! A `<select>` of every account works without JavaScript and is enough at
//! this instance's size; the list is capped so a large directory renders a
//! page rather than a stall, and the cap is reported by the caller. A failed
//! listing is returned to the caller, which records it on its page.

use sqlx::PgPool;

use crate::repositories::principal::listing::list_users;
use crate::repositories::scope::SubjectScope;
use crate::ssr::types::pickable_user::PickableUserView;

const PICKABLE_USER_LIMIT: usize = 200;

pub async fn list_pickable_users(
    pool: &PgPool,
    selected: Option<&str>,
) -> Result<Vec<PickableUserView>, sqlx::Error> {
    let users = list_users(pool, &SubjectScope::All).await?;
    let picked = users
        .into_iter()
        .take(PICKABLE_USER_LIMIT)
        .map(|u| {
            let id = u.user_id.as_str().to_owned();
            PickableUserView {
                label: u
                    .email
                    .as_ref()
                    .map(ToString::to_string)
                    .or(u.display_name)
                    .unwrap_or_else(|| id.clone()),
                selected: selected == Some(id.as_str()),
                id,
            }
        })
        .collect();
    Ok(picked)
}
