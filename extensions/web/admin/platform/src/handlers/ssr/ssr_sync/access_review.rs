//! The Access review tab of `/admin/sync`: the access-control drift as one
//! row per entity, each settled on its own — apply code, keep the database,
//! or export — with core's whole-plane card below for the rare wholesale
//! move.

use serde::Serialize;
use sqlx::PgPool;

use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::repositories::sync::access_control::{
    PLANE_ID, declared_now, drift_now,
};
use systemprompt_web_admin_core::repositories::sync::attention::{
    ReviewSplit, ReviewedEntity, reviews_for,
};
use systemprompt_web_admin_core::repositories::sync::core_planes::CorePlanes;
use systemprompt_web_admin_core::ssr::core_plane::{
    CorePlaneCardOpts, CorePlaneCardView, core_plane_card,
};
use systemprompt_web_admin_core::ssr::entity_panel::entity_access_url;

#[derive(Debug, Serialize)]
pub(crate) struct ReviewRowView {
    #[serde(flatten)]
    pub entity: ReviewedEntity,
    pub entity_url: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub(crate) struct AccessReviewView {
    pub pending: Vec<ReviewRowView>,
    pub kept: Vec<ReviewRowView>,
    pub pending_count: usize,
    pub kept_count: usize,
    pub not_live_count: usize,
    pub unreadable: Option<String>,
    pub export_url: String,
    pub plane: Option<CorePlaneCardView>,
}

fn rows(list: Vec<ReviewedEntity>) -> Vec<ReviewRowView> {
    list.into_iter()
        .map(|entity| ReviewRowView {
            entity_url: entity_access_url(&entity.review.entity_type, &entity.review.entity_id),
            entity,
        })
        .collect()
}

async fn split(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    pool: &PgPool,
) -> AdminResult<ReviewSplit> {
    let declared = declared_now(&services.config()).await?;
    let drift = drift_now(pool, &declared).await?;
    Ok(reviews_for(pool, &drift).await?)
}

pub(super) async fn access_review(
    services: &systemprompt_web_admin_core::extension_services::ExtensionServices,
    pool: &PgPool,
    core_planes: &CorePlanes,
) -> AdminResult<AccessReviewView> {
    let opts = CorePlaneCardOpts {
        entity_filter: "",
        show_export: true,
    };
    let mut view = AccessReviewView {
        export_url: format!("/api/public/admin/sync/planes/{PLANE_ID}/export"),
        plane: core_plane_card(pool, core_planes, PLANE_ID, &opts).await?,
        ..AccessReviewView::default()
    };
    match split(services, pool).await {
        Ok(split) => {
            view.not_live_count = split.pending.iter().filter(|r| r.review.not_live).count();
            view.pending_count = split.pending.len();
            view.pending = rows(split.pending);
            view.kept_count = split.kept.len();
            view.kept = rows(split.kept);
        },
        Err(e) => view.unreadable = Some(e.to_string()),
    }
    Ok(view)
}
