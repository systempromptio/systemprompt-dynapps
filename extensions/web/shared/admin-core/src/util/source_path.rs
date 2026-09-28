//! Display form of a config file's path, as the catalog pages print it.

use std::path::Path;

pub fn display_source_path(path: &Path, services_path: &Path) -> String {
    path.strip_prefix(services_path)
        .ok()
        .and_then(|p| p.to_str())
        .map_or_else(|| path.display().to_string(), |s| format!("services/{s}"))
}
