//! People page routes.
//!
//! The users roster and detail, groups, projects and their reports, roles,
//! who holds a personal-subscription key,
//! and the access-control matrix. The composition root merges the devices
//! page between the last two, where it has always sat.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use sqlx::PgPool;

use crate::handlers;

pub fn directory_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/users", get(handlers::ssr::users_page))
        .route(
            "/users/{user_id}",
            get(handlers::ssr::user_detail_by_id_page),
        )
        .route("/user", get(handlers::ssr::user_detail_page))
        .route("/groups", get(handlers::ssr::groups_page))
        .route("/groups/{group_id}", get(handlers::ssr::group_detail_page))
        .route("/projects", get(handlers::ssr::projects_page))
        .route(
            "/projects/{project_id}",
            get(handlers::ssr::project_detail_page),
        )
        .route(
            "/projects/{project_id}/report",
            get(handlers::ssr::project_report_page),
        )
        .route("/roles", get(handlers::ssr::roles_page))
        .route(
            "/member-subscriptions",
            get(handlers::ssr::member_subscriptions_page),
        )
}

pub fn access_control_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/access-control", get(handlers::ssr::access_control_page))
}
