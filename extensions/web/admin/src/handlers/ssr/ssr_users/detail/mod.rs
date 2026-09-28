//! `/admin/users/{id}` — one account, in seven tabs.
//!
//! Every tab is a link
//! (`?tab=identity|conversations|membership|access|devices|sessions|usage`),
//! so a view is bookmarkable and the server loads and renders only the tab the
//! reader asked for.

mod access_overview;
mod access_view;
mod context;
mod headline;
mod load;
mod usage;
mod view;
mod workspace_grant;

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories;
use crate::types::UserContext;

use context::{DetailTabView, UserDetailContext};
use headline::{BaseInputs, base_context};

use crate::handlers::ssr::list_view::DEFAULT_PAGE_SIZE;
use crate::handlers::ssr::page::Page;

const TABS: [(&str, &str); 7] = [
    ("identity", "Identity"),
    ("conversations", "Conversations"),
    ("membership", "Membership"),
    ("access", "Access & connections"),
    ("devices", "Devices"),
    ("sessions", "Sessions"),
    ("usage", "Usage"),
];

#[derive(Debug, Deserialize)]
pub(crate) struct DetailQuery {
    pub tab: Option<String>,
    pub page: Option<i64>,
}

fn may_open_profile(viewer: &UserContext, user_id_raw: &str) -> bool {
    viewer.is_console || viewer.user_id.as_str() == user_id_raw
}

pub(crate) async fn user_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
    Query(query): Query<DetailQuery>,
) -> AdminHtmlResult<Response> {
    if !may_open_profile(&shell.user, &user_id_raw) {
        return Err(AdminError::Forbidden("You can only view your own profile.".to_owned()).into());
    }

    let user_id = UserId::new(user_id_raw.as_str());
    let Some(detail) = repositories::users::queries::find_user_detail(&pool, &user_id).await?
    else {
        return Err(AdminError::NotFound("No such user.".to_owned()).into());
    };

    let tab = resolve_tab(query.tab.as_deref());
    let page = query.page.unwrap_or(0).max(0);
    let headline = load::load_headline(&pool, &user_id).await;
    let data = base_context(BaseInputs {
        user_id: &user_id,
        detail: &detail,
        headline,
        tab,
        viewer: &shell.user,
    });

    let data = Box::pin(fill_active_tab(
        ActiveTabRead {
            pool: &pool,
            user_id: &user_id,
            detail: &detail,
            tab,
            page,
            viewer: &shell.user,
        },
        data,
    ))
    .await;

    Ok(super::super::render_typed_page(
        &shell.engine,
        "user-detail",
        &data,
        &shell.user,
        &shell.marketplace,
    ))
}

struct ActiveTabRead<'a> {
    pool: &'a PgPool,
    user_id: &'a UserId,
    detail: &'a crate::types::UserDetail,
    tab: &'static str,
    page: i64,
    viewer: &'a UserContext,
}

async fn fill_active_tab(
    read: ActiveTabRead<'_>,
    mut data: UserDetailContext,
) -> UserDetailContext {
    let ActiveTabRead {
        pool,
        user_id,
        detail,
        tab,
        page,
        viewer,
    } = read;
    match tab {
        "conversations" => {
            data.conversations = Some(view::conversations_tab(
                &load::load_conversations(pool, user_id, page).await,
                user_id,
                viewer,
                page,
                load::CONVERSATION_PAGE_SIZE,
            ));
        },
        "membership" => {
            data.membership = Some(view::membership_tab(
                load::load_membership(pool, user_id).await,
            ));
        },
        "access" => {
            data.access = Some(view::access_tab(
                load::load_access(pool, user_id).await,
                user_id,
            ));
        },
        "devices" => {
            let rows = load::load_devices(pool, user_id).await;
            let tab_view = view::devices_tab(rows);
            data.kpis.devices = i64::try_from(tab_view.count).unwrap_or(0);
            data.devices = Some(tab_view);
        },
        "sessions" => {
            let rows = load::load_sessions(pool, user_id).await;
            data.sessions = Some(view::sessions_tab(&rows, user_id, page, DEFAULT_PAGE_SIZE));
        },
        "usage" => {
            data.usage = Some(view::usage_tab(
                load::load_usage(pool, user_id).await,
                user_id,
            ));
        },
        _ => {
            let identity = load::load_identity(pool, user_id, &detail.roles).await;
            let choices = super::scope_data::role_choices(&identity.roles);
            data.identity = Some(view::identity_tab(detail, &identity, choices));
        },
    }
    data
}

fn resolve_tab(raw: Option<&str>) -> &'static str {
    TABS.into_iter()
        .find(|(slug, _)| Some(*slug) == raw)
        .map_or("identity", |(slug, _)| slug)
}

fn tab_href(user_id: &UserId, slug: &str) -> String {
    let encoded = urlencoding::encode(user_id.as_str());
    format!("/admin/users/{encoded}?tab={slug}")
}

fn tab_links(user_id: &UserId, active: &'static str) -> Vec<DetailTabView> {
    TABS.into_iter()
        .map(|(slug, label)| DetailTabView {
            slug,
            label,
            href: tab_href(user_id, slug),
            is_active: slug == active,
        })
        .collect()
}
