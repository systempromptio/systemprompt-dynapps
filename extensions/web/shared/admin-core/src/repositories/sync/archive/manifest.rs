//! `MANIFEST.yaml`: what an export was taken from, so an import can say
//! where its archive came from and a reader can tell two exports apart.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AdminResult;

pub const MANIFEST_FORMAT: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestPlane {
    pub id: String,
    pub file: String,
    pub declared_hash: Option<String>,
    pub applied_hash: Option<String>,
    pub applied_mode: Option<String>,
    pub applied_at: Option<DateTime<Utc>>,
    pub row_count: usize,
}

/// The header of a configuration archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveManifest {
    pub format: u32,
    pub release: String,
    pub exported_at: DateTime<Utc>,
    pub exported_by: String,
    pub base_tree_hash: Option<String>,
    pub composed_hash: Option<String>,
    #[serde(default)]
    pub planes: Vec<ManifestPlane>,
}

pub fn build_manifest(
    boot: &crate::extension_boot::ExtensionBoot,
    actor: &str,
    planes: Vec<ManifestPlane>,
) -> ArchiveManifest {
    let (base_tree_hash, composed_hash) = crate::repositories::sync::sources::tree_hashes(
        boot.profile(),
        boot.active_services_root(),
    );
    ArchiveManifest {
        format: MANIFEST_FORMAT,
        release: env!("CARGO_PKG_VERSION").to_owned(),
        exported_at: Utc::now(),
        exported_by: actor.to_owned(),
        base_tree_hash,
        composed_hash,
        planes,
    }
}

pub fn render_manifest(manifest: &ArchiveManifest) -> AdminResult<String> {
    let body = serde_yaml::to_string(manifest)
        .map_err(|e| crate::error::AdminError::invalid("manifest could not be rendered", e))?;
    Ok(format!(
        "# Configuration archive — exported from the console.\n\
         # Each services/<file> is that plane's database rendered as its declaration.\n\
         {body}"
    ))
}

pub fn parse_manifest(text: &str) -> Result<ArchiveManifest, String> {
    serde_yaml::from_str(text).map_err(|e| e.to_string())
}
