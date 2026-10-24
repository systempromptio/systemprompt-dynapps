//! Helpers shared across admin handlers.

use std::path::PathBuf;

use axum::http::HeaderMap;
use serde::Serialize;
use systemprompt::config::ProfileBootstrap;
use systemprompt::loader::services_root::ServicesRootBootstrap;

use crate::error::{AdminError, AdminResult};

#[derive(Debug, Serialize)]
pub(crate) struct ErrorBody {
    pub error: String,
}

pub fn get_services_path() -> AdminResult<PathBuf> {
    Ok(ServicesRootBootstrap::require()?.path)
}

// Why: core's composed services root is a content-addressed overlay the next
// refresh replaces, so any file edit must target the baked tree — and a file a
// pinned bundle owns is not editable here at all.
pub(crate) fn get_baked_services_path() -> AdminResult<PathBuf> {
    Ok(PathBuf::from(&ProfileBootstrap::get()?.paths.services))
}

pub fn get_gateway_file_path() -> AdminResult<PathBuf> {
    let root = if crate::repositories::sync::sources::dir_owner("ai").is_some() {
        get_services_path()?
    } else {
        get_baked_services_path()?
    };
    Ok(root.join("ai").join("gateway.yaml"))
}

pub fn get_gateway_file_path_for_write() -> AdminResult<PathBuf> {
    if let Some(bundle) = crate::repositories::sync::sources::dir_owner("ai") {
        return Err(AdminError::Conflict(format!(
            "ai/gateway.yaml is managed by the services bundle `{bundle}`; change it there"
        )));
    }
    Ok(get_baked_services_path()?.join("ai").join("gateway.yaml"))
}

pub fn require_write_origin(headers: &HeaderMap) -> AdminResult<()> {
    let profile = ProfileBootstrap::get().map_err(AdminError::internal)?;
    let expected = url::Url::parse(&profile.server.api_external_url)
        .map_err(AdminError::internal)?
        .origin()
        .ascii_serialization();
    if headers.get("origin").and_then(|v| v.to_str().ok()) != Some(expected.as_str()) {
        return Err(AdminError::Forbidden(
            "Same-origin browser request required".to_owned(),
        ));
    }
    Ok(())
}
