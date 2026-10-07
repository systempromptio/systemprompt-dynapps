//! `copy_extension_assets` job: copies every registered asset into `web/dist/`.
//!
//! Runs after the CSS and JS bundle jobs, whose output it picks up.

use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;

use systemprompt::config::AppPaths;
use systemprompt::extension::{AssetDefinition, ExtensionRegistry};
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct CopyExtensionAssetsJob;

impl CopyExtensionAssetsJob {
    pub async fn execute_copy(paths: &AppPaths) -> Result<JobResult, JobError> {
        let start_time = std::time::Instant::now();

        tracing::debug!("Copy extension assets job started");

        let registry = ExtensionRegistry::discover().map_err(|e| JobError::other(e.to_string()))?;
        let assets = registry.all_required_assets(paths);

        if assets.is_empty() {
            let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);
            tracing::debug!(duration_ms, "No extension assets to copy");
            return Ok(JobResult::success()
                .with_stats(0, 0)
                .with_duration(duration_ms));
        }

        let (copied, failed) = copy_all_assets(paths.web().dist(), assets).await?;

        let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::debug!(
            copied,
            failed,
            duration_ms,
            "Copy extension assets job completed"
        );

        Ok(JobResult::success()
            .with_stats(copied, failed)
            .with_duration(duration_ms))
    }
}

#[doc(hidden)]
pub async fn copy_all_assets(
    dist_dir: &Path,
    assets: Vec<(&str, AssetDefinition)>,
) -> Result<(u64, u64), JobError> {
    let version = javascript_version(&assets).await?;
    let mut copied = 0u64;
    let mut failed = 0u64;

    for (ext_id, asset) in assets {
        let result = if is_javascript(asset.destination()) {
            copy_versioned_module(dist_dir, &asset, &version).await
        } else {
            copy_asset(dist_dir, ext_id, &asset).await
        };
        match result {
            Ok(()) => copied += 1,
            Err(e) => {
                if asset.is_required() {
                    return Err(e);
                }
                tracing::warn!(
                    extension = %ext_id,
                    asset = %asset.source().display(),
                    error = %e,
                    "Optional asset copy failed"
                );
                failed += 1;
            },
        }
    }

    tokio::fs::create_dir_all(dist_dir.join("js")).await?;
    tokio::fs::write(dist_dir.join("js/module-version.txt"), version).await?;
    Ok((copied, failed))
}

fn is_javascript(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("js"))
}

// Why: browser caches include dependencies, so include the whole graph:
// changing a leaf must invalidate its importers too.
async fn javascript_version(assets: &[(&str, AssetDefinition)]) -> Result<String, JobError> {
    let mut modules: Vec<_> = assets
        .iter()
        .filter(|(_, asset)| is_javascript(asset.destination()))
        .collect();
    modules.sort_by(|(_, left), (_, right)| left.destination().cmp(right.destination()));
    let mut hash = Sha256::new();
    for (_, asset) in modules {
        match tokio::fs::read(asset.source()).await {
            Ok(bytes) => {
                hash.update(asset.destination().as_bytes());
                hash.update([0]);
                hash.update(bytes);
                hash.update([0]);
            },
            Err(error) if asset.is_required() => return Err(error.into()),
            Err(_) => {},
        }
    }
    Ok(hex::encode(hash.finalize()))
}

async fn copy_versioned_module(
    dist_dir: &Path,
    asset: &AssetDefinition,
    version: &str,
) -> Result<(), JobError> {
    let source = tokio::fs::read_to_string(asset.source()).await?;
    let destination = dist_dir.join(asset.destination());
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    // Why: browsers can read assets while the publish job replaces them.
    let temporary = destination.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    tokio::fs::write(&temporary, version_module_imports(&source, version)).await?;
    tokio::fs::rename(temporary, destination).await?;
    Ok(())
}

#[doc(hidden)]
pub fn version_module_imports(source: &str, version: &str) -> String {
    let mut output = String::with_capacity(source.len());
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let declaration = trimmed.starts_with("import ")
            || trimmed.starts_with("export ")
            || trimmed.starts_with("} from ");
        let specifier = if declaration {
            trimmed
                .split_once("from ")
                .map(|(_, path)| path)
                .or_else(|| trimmed.strip_prefix("import "))
        } else {
            None
        };
        let replacement = specifier.and_then(|path| {
            let quote = path.chars().next()?;
            if !matches!(quote, '\'' | '"') {
                return None;
            }
            let end = path[1..].find(quote)? + 1;
            let url = &path[1..end];
            if !(url.starts_with('.') || url.starts_with("/js/")) || !is_javascript(url) {
                return None;
            }
            Some((
                format!("{quote}{url}{quote}"),
                format!("{quote}{url}?v={version}{quote}"),
            ))
        });
        if let Some((old, new)) = replacement {
            output.push_str(&line.replacen(&old, &new, 1));
        } else {
            output.push_str(line);
        }
    }
    output
}

#[doc(hidden)]
pub async fn copy_asset(
    dist_dir: &Path,
    ext_id: &str,
    asset: &AssetDefinition,
) -> Result<(), JobError> {
    let dest_path = dist_dir.join(asset.destination());

    if let Some(parent) = dest_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    tokio::fs::copy(asset.source(), &dest_path).await?;

    tracing::debug!(
        extension = %ext_id,
        source = %asset.source().display(),
        destination = %dest_path.display(),
        "Copied extension asset"
    );

    Ok(())
}

#[async_trait::async_trait]
impl Job for CopyExtensionAssetsJob {
    fn name(&self) -> &'static str {
        "copy_extension_assets"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Copies extension assets (CSS, JS) to web dist directory"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let paths = ctx.get::<Arc<AppPaths>>()?.as_ref();
        Ok(Self::execute_copy(paths).await?)
    }
}

systemprompt::traits::submit_job!(&CopyExtensionAssetsJob);
