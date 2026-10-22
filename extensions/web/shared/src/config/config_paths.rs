//! Content-source validation and profile-aware path resolution for the
//! `services/content/` config model.

use std::path::{Path, PathBuf};

use systemprompt::config::{AppPaths, ProfileBootstrap};
use systemprompt::loader::ServicesRootBootstrap;

use super::{ContentSourceRaw, ContentSourceValidated, ExtensionConfigErrors};

pub(super) fn validate_content_source(
    src: ContentSourceRaw,
    index: usize,
    base_path: &Path,
    errors: &mut ExtensionConfigErrors,
) -> Option<ContentSourceValidated> {
    let field_prefix = format!("content_sources[{index}]");

    if src.source_id.as_str().trim().is_empty() {
        errors.push(
            format!("{field_prefix}.source_id"),
            "source_id cannot be empty",
        );
        return None;
    }

    if src.category_id.as_str().trim().is_empty() {
        errors.push(
            format!("{field_prefix}.category_id"),
            "category_id cannot be empty",
        );
        return None;
    }

    let resolved_path = match resolve_content_source_path(&src.path, base_path) {
        Ok(path) => path,
        Err(source) => {
            errors.push_with_source(
                format!("{field_prefix}.path"),
                "profile paths unavailable for a services-relative content source",
                source,
            );
            return None;
        },
    };

    if src.enabled {
        let source_id = &src.source_id;
        if !resolved_path.exists() {
            errors.push_with_path(
                format!("{field_prefix}.path"),
                format!("Content source '{source_id}' path does not exist"),
                &resolved_path,
            );
            return None;
        }

        if !resolved_path.is_dir() {
            errors.push_with_path(
                format!("{field_prefix}.path"),
                format!("Content source '{source_id}' path is not a directory"),
                &resolved_path,
            );
            return None;
        }
    }

    let canonical_path = resolved_path.canonicalize().unwrap_or(resolved_path);

    Some(ContentSourceValidated {
        source_id: src.source_id,
        category_id: src.category_id,
        path: canonical_path,
        allowed_content_types: src.allowed_content_types,
        enabled: src.enabled,
        override_existing: src.override_existing,
    })
}
#[derive(Debug, thiserror::Error)]
pub enum ProfilePathsError {
    #[error("profile unavailable: {0}")]
    Profile(#[source] systemprompt::config::ProfileBootstrapError),
    #[error("application paths unavailable: {0}")]
    Paths(#[source] systemprompt::config::PathError),
}

fn services_dir() -> Result<PathBuf, ProfilePathsError> {
    let profile = ProfileBootstrap::get().map_err(ProfilePathsError::Profile)?;
    let paths = AppPaths::from_profile(
        &profile.paths,
        profile.path_resolution(),
        ServicesRootBootstrap::get().map(|root| root.path.as_path()),
    )
    .map_err(ProfilePathsError::Paths)?;
    Ok(paths.system().services().to_path_buf())
}

pub(super) fn resolve_blog_config_path() -> Result<PathBuf, ExtensionConfigErrors> {
    services_dir()
        .map(|services| services.join("config/blog.yaml"))
        .map_err(|source| {
            let mut errors = ExtensionConfigErrors::new("blog");
            errors.push_with_source("_paths", "profile paths unavailable", source);
            errors
        })
}

pub(super) fn resolve_content_source_path(
    path: &str,
    base_path: &Path,
) -> Result<PathBuf, ProfilePathsError> {
    if Path::new(path).is_absolute() {
        Ok(PathBuf::from(path))
    } else if let Some(services_relative) = path.strip_prefix("./") {
        let clean_path = services_relative
            .strip_prefix("services/")
            .unwrap_or(services_relative);
        Ok(services_dir()?.join(clean_path))
    } else {
        Ok(base_path.join(path))
    }
}
