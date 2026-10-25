//! The core-plane component: one plane core owns (access control,
//! governance, gateway policies), read from core's state and diff.
//!
//! It shows the declared hash, core's last explicit apply, and each entity
//! whose database value differs from the declaration. Its two actions are
//! core's: *Apply from code* (core's entity merge, which records conflicts
//! instead of overwriting what both sides changed) and *Export*. The partial
//! is `components/core-plane`; the JS is `components/sp-sync-plane.js`.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::extension::prelude::{DriftKind, DriftRow};

use crate::error::AdminResult;
use crate::repositories::sync::core_plane_state::{CorePlaneStateRow, find_core_plane_state};
use crate::repositories::sync::core_planes::{CorePlanes, core_plane_meta};
use crate::ssr::sync_plane::HashView;

#[derive(Debug, Clone, Serialize)]
pub struct CoreAppliedView {
    pub hash: Option<HashView>,
    pub by: String,
    pub at: String,
    pub mode: String,
    pub declared_changed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoreDriftRowView {
    pub entity: String,
    pub kind: &'static str,
    pub kind_label: &'static str,
    pub kind_tone: &'static str,
    pub declared: String,
    pub current: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorePlaneCardView {
    pub id: &'static str,
    pub label: &'static str,
    pub source_file: String,
    pub owner_url: &'static str,
    pub sync_url: &'static str,
    pub authority: &'static str,
    pub declared_hash: Option<HashView>,
    pub applied: Option<CoreAppliedView>,
    pub rows: Vec<CoreDriftRowView>,
    pub row_count: usize,
    pub is_clean: bool,
    pub apply_url: String,
    pub export_url: String,
    pub show_export: bool,
    pub entity_filter: String,
}

/// Which rows the card shows and where its apply posts.
#[derive(Debug, Clone, Default)]
pub struct CorePlaneCardOpts<'a> {
    pub entity_filter: &'a str,
    pub show_export: bool,
}

fn row_view(row: &DriftRow) -> CoreDriftRowView {
    let (kind_label, kind_tone) = match row.kind {
        DriftKind::Missing => ("Only in code", "ok"),
        DriftKind::Extra => ("Only in this database", "err"),
        DriftKind::Changed => ("Changed", "warn"),
    };
    CoreDriftRowView {
        entity: row.entity.clone(),
        kind: row.kind.as_str(),
        kind_label,
        kind_tone,
        declared: row.declared.clone().unwrap_or_default(),
        current: row.current.clone().unwrap_or_default(),
    }
}

fn applied_view(state: &CorePlaneStateRow, declared_hash: Option<&str>) -> Option<CoreAppliedView> {
    let at = state.last_apply_at?;
    let hash = state.last_apply_declared_hash.as_deref();
    Some(CoreAppliedView {
        hash: hash.and_then(HashView::of),
        by: state.last_apply_actor.clone().unwrap_or_default(),
        at: at.to_rfc3339(),
        mode: state.last_apply_mode.clone().unwrap_or_default(),
        declared_changed: declared_hash.is_some() && declared_hash != hash,
    })
}

pub async fn core_plane_card(
    pool: &PgPool,
    planes: &CorePlanes,
    id: &str,
    opts: &CorePlaneCardOpts<'_>,
) -> AdminResult<Option<CorePlaneCardView>> {
    let Some(meta) = core_plane_meta(id) else {
        return Ok(None);
    };
    let diff = planes.diff(id).await?;
    let state = find_core_plane_state(pool, id).await?;
    let rows: Vec<CoreDriftRowView> = diff
        .rows
        .iter()
        .filter(|r| opts.entity_filter.is_empty() || r.entity == opts.entity_filter)
        .map(row_view)
        .collect();
    Ok(Some(CorePlaneCardView {
        id: meta.id,
        label: meta.label,
        source_file: diff.declared_path.clone(),
        owner_url: meta.owner_url,
        sync_url: meta.sync_url,
        authority: diff.authority.as_str(),
        declared_hash: diff.declared_hash.as_deref().and_then(HashView::of),
        applied: state
            .as_ref()
            .and_then(|s| applied_view(s, diff.declared_hash.as_deref())),
        row_count: rows.len(),
        is_clean: diff.rows.is_empty(),
        rows,
        apply_url: format!("/api/public/admin/sync/planes/{}/apply", meta.id),
        export_url: format!("/api/public/admin/sync/planes/{}/export", meta.id),
        show_export: opts.show_export,
        entity_filter: opts.entity_filter.to_owned(),
    }))
}
