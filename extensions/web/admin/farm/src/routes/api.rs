//! Console JSON API for farms. Reads join the console tier, the sync action
//! the manage tier, at the composition root.

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use sqlx::PgPool;

use crate::handlers::admin;

pub fn farm_read_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/agent-farm/farms", get(admin::list_farms))
        .route("/agent-farm/farms/{farm_id}", get(admin::get_farm))
}

pub fn farm_write_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/agent-farm/farms/{farm_id}/sync", post(admin::sync_farm))
}
