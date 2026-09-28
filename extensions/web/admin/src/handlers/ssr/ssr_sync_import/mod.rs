//! `/admin/sync/import/{stage}` — what an uploaded archive would do to this
//! database, plane by plane, before any of it is applied.
//!
//! The stage holds each plane's file as text; every plane's drift is
//! computed from that text through the same code the owner page uses for
//! the file on disk, so the preview is the exact picture an apply would
//! act on. Entries the instance does not project are listed for what they
//! are — things to commit to the repository or publish as a bundle — and
//! projected planes the archive did not carry are named so a partial
//! archive is never mistaken for a full one.

mod view;

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use sqlx::PgPool;

use self::view::AbsentPlaneView;
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::sync_plane::{PlaneCardOpts, plane_card};
use crate::repositories::sync::archive::staging::StagingStore;
use crate::repositories::sync::plane::DeclarationSource;
use crate::repositories::sync::registry::planes;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

pub(crate) async fn import_preview_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(stage_id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let Some(stage) = StagingStore::global().get(&stage_id) else {
        return Err(AdminError::NotFound(
            "No staged import with that id — it may have expired.".to_owned(),
        )
        .into());
    };
    let apply_url = view::apply_url(&stage.id);
    let mut cards = Vec::new();
    let mut absent = Vec::new();
    for plane in planes() {
        match stage.planes.get(plane.id()) {
            Some(text) => {
                let opts = PlaneCardOpts {
                    entity_filter: "",
                    apply_url: Some(apply_url.clone()),
                    show_export: false,
                    source_label: Some("the archive"),
                };
                cards.push(
                    plane_card(&pool, plane.as_ref(), DeclarationSource::Text(text), &opts).await?,
                );
            },
            None => absent.push(AbsentPlaneView {
                label: plane.label(),
                source_file: plane.source_file(),
                owner_url: plane.owner_url(),
            }),
        }
    }
    let page = view::page_data(
        &stage,
        view::PreviewPlanes { cards, absent },
        user_ctx.is_admin,
    );
    Ok(super::render_typed_page(
        &engine,
        "sync-import",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
