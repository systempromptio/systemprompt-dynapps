//! `/admin/access/groups` — every group, with its membership, entitlement and
//! thirty-day spend.
//!
//! `unassigned` is a row here like any other. It is derived rather than
//! stored — anyone with no `group_members` row falls into it — and the whole
//! point of the page is that the number next to it is visible rather than
//! implicit, so it is listed and badged, never filtered out.

use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::types::GroupsPageData;

mod data;
mod view;

pub(crate) const UNASSIGNED_GROUP: &str = "unassigned";

pub(crate) async fn groups_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let rows = data::load_group_rows(&pool).await;
    let data = GroupsPageData {
        page: "groups",
        title: "Groups",
        stats: view::group_stats(&rows),
        groups: rows,
        can_manage: user_ctx.is_admin,
    };

    Ok(super::render_typed_page(
        &engine, "groups", &data, &user_ctx, &mkt_ctx,
    ))
}
