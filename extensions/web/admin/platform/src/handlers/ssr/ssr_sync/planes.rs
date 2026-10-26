//! The Planes section of the Sources tab: one row per plane, core's three
//! and the install's own, with its drift and last apply, linking to the
//! plane's one sync location. Core reports drift by entity, not by row, so a
//! core plane's row has no declared or in-database row counts.
//!
//! The row reads the same card the plane's own Sync tab renders, so the
//! counts here and there cannot disagree.

use serde::Serialize;
use sqlx::PgPool;

use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_core::repositories::sync::core_planes::{CORE_PLANE_IDS, CorePlanes};
use systemprompt_web_admin_core::repositories::sync::registry::planes;
use systemprompt_web_admin_core::ssr::core_plane::{CorePlaneCardOpts, core_plane_card};
use systemprompt_web_admin_core::ssr::sync_plane::{AppliedView, HashView, plane_card_by_id};

#[derive(Debug, Serialize)]
pub(crate) struct PlaneRowView {
    pub id: &'static str,
    pub label: &'static str,
    pub source_file: String,
    pub is_core: bool,
    pub declared_count: usize,
    pub in_db: usize,
    pub drift_count: usize,
    pub drift_label: String,
    pub is_clean: bool,
    pub unreadable: Option<String>,
    pub applied: Option<AppliedView>,
    pub review_url: String,
}

pub(super) async fn plane_rows(
    pool: &PgPool,
    boot: &systemprompt_web_admin_core::extension_boot::ExtensionBoot,
    core_planes: &CorePlanes,
) -> AdminResult<Vec<PlaneRowView>> {
    let mut out = Vec::new();
    let opts = CorePlaneCardOpts::default();
    for id in CORE_PLANE_IDS {
        let Some(card) = core_plane_card(pool, core_planes, id, &opts).await? else {
            continue;
        };
        out.push(PlaneRowView {
            id: card.id,
            label: card.label,
            source_file: card.source_file,
            is_core: true,
            declared_count: 0,
            in_db: 0,
            drift_count: card.row_count,
            drift_label: format!("{} entities differ", card.row_count),
            is_clean: card.is_clean,
            unreadable: None,
            applied: card.applied.map(|a| AppliedView {
                hash: a.hash,
                mode: a.mode,
                at: a.at,
                by: a.by,
                base_tree_hash: None::<HashView>,
                composed_hash: None,
                declared_changed: a.declared_changed,
            }),
            review_url: card.sync_url.to_owned(),
        });
    }
    for plane in planes(boot) {
        let Some(card) = plane_card_by_id(pool, boot, plane.id(), "").await? else {
            continue;
        };
        out.push(PlaneRowView {
            id: card.id,
            label: card.label,
            source_file: card.source_file.to_owned(),
            is_core: false,
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
