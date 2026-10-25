//! Renders every plane's export into one zip: the install planes through
//! their own export, the planes core owns through core's.

use std::io::{Cursor, Write};

use sqlx::PgPool;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

use super::manifest::{ManifestPlane, build_manifest, render_manifest};
use super::{MANIFEST_FILE, TREE_PREFIX};
use crate::error::{AdminError, AdminResult};
use crate::repositories::sync::core_plane_state::find_core_plane_state;
use crate::repositories::sync::core_planes::{CORE_PLANE_IDS, CorePlanes};
use crate::repositories::sync::registry::planes;
use crate::repositories::sync::state::find_sync_state;

/// A finished export, ready to send as an attachment.
#[derive(Debug, Clone)]
pub struct ExportZip {
    pub filename: String,
    pub bytes: Vec<u8>,
    pub planes: usize,
    pub rows: usize,
}

fn zip_error(e: zip::result::ZipError) -> AdminError {
    AdminError::invalid("archive could not be written", e)
}

fn write_file(
    writer: &mut zip::ZipWriter<Cursor<Vec<u8>>>,
    path: &str,
    body: &str,
) -> AdminResult<()> {
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    writer.start_file(path, options).map_err(zip_error)?;
    writer
        .write_all(body.as_bytes())
        .map_err(|e| AdminError::invalid("archive could not be written", e))
}

pub async fn build_export_zip(
    pool: &PgPool,
    core_planes: &CorePlanes,
    actor: &str,
) -> AdminResult<ExportZip> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let mut exported = Vec::new();
    let mut rows = 0usize;
    for plane in &planes() {
        let Some(export) = plane.export(pool).await? else {
            continue;
        };
        write_file(
            &mut writer,
            &format!("{TREE_PREFIX}{}", plane.source_file()),
            &export.body,
        )?;
        rows += export.row_count;
        let state = find_sync_state(pool, plane.id()).await?;
        exported.push(ManifestPlane {
            id: plane.id().to_owned(),
            file: plane.source_file().to_owned(),
            declared_hash: state.as_ref().map(|s| s.declared_hash.clone()),
            applied_hash: state.as_ref().and_then(|s| s.applied_hash.clone()),
            applied_mode: state.as_ref().and_then(|s| s.applied_mode.clone()),
            applied_at: state.as_ref().and_then(|s| s.applied_at),
            row_count: export.row_count,
        });
    }
    for id in CORE_PLANE_IDS {
        let Some(body) = core_planes.export(id).await? else {
            continue;
        };
        let file = core_planes.diff(id).await?.declared_path;
        write_file(&mut writer, &format!("{TREE_PREFIX}{file}"), &body)?;
        let state = find_core_plane_state(pool, id).await?;
        exported.push(ManifestPlane {
            id: id.to_owned(),
            file,
            declared_hash: state.as_ref().and_then(|s| s.declared_hash.clone()),
            applied_hash: state
                .as_ref()
                .and_then(|s| s.last_apply_declared_hash.clone()),
            applied_mode: state.as_ref().and_then(|s| s.last_apply_mode.clone()),
            applied_at: state.as_ref().and_then(|s| s.last_apply_at),
            row_count: 0,
        });
    }
    let manifest = build_manifest(actor, exported);
    let planes = manifest.planes.len();
    write_file(&mut writer, MANIFEST_FILE, &render_manifest(&manifest)?)?;
    let bytes = writer.finish().map_err(zip_error)?.into_inner();
    Ok(ExportZip {
        filename: format!(
            "configuration-v{}-{}.zip",
            manifest.release,
            manifest.exported_at.format("%Y%m%d-%H%M")
        ),
        bytes,
        planes,
        rows,
    })
}
