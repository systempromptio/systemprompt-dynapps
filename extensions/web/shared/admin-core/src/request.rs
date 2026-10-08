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
    Ok(ServicesRootBootstrap::active_root_or(
        &ProfileBootstrap::get()?.paths.services,
    ))
}

// Why: core's composed services root is a content-addressed overlay the next
// refresh replaces, so any file edit must target the baked tree.
pub(crate) fn get_baked_services_path() -> AdminResult<PathBuf> {
    Ok(PathBuf::from(&ProfileBootstrap::get()?.paths.services))
}

pub fn get_gateway_file_path() -> AdminResult<PathBuf> {
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
