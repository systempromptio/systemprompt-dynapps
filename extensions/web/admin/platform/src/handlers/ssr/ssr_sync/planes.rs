//! The Planes section of the Sources tab: one row per registered plane with
//! its drift and last apply, linking to the plane's one sync location.
//!
//! The row reads the same card the plane's own Sync tab renders, so the
//! counts here and there cannot disagree.

use serde::Serialize;
use sqlx::PgPool;

use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::repositories::sync::registry::planes;
use systemprompt_web_admin_core::ssr::sync_plane::{AppliedView, plane_card_by_id};

#[derive(Debug, Serialize)]
pub(crate) struct PlaneRowView {
    pub id: &'static str,
    pub label: &'static str,
    pub source_file: &'static str,
    pub declared_count: usize,
    pub in_db: usize,
    pub drift_count: usize,
    pub drift_label: String,
    pub is_clean: bool,
    pub unreadable: Option<String>,
    pub applied: Option<AppliedView>,
    pub review_url: String,
}

pub(super) async fn plane_rows(pool: &PgPool) -> AdminResult<Vec<PlaneRowView>> {
    let mut out = Vec::new();
    for plane in planes() {
        let Some(card) = plane_card_by_id(pool, plane.id(), "").await? else {
            continue;
        };
        out.push(PlaneRowView {
            id: card.id,
            label: card.label,
            source_file: card.source_file,
            declared_count: card.declared_count,
            in_db: card.in_db,
            drift_count: card.row_count,
            drift_label: format!("{} differ", card.row_count),
            is_clean: card.is_clean,
            unreadable: card.unreadable,
            applied: card.applied,
            review_url: card.sync_url,
        });
    }
    Ok(out)
}
