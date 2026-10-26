//! Helpers shared across admin handlers.

use std::path::PathBuf;

use crate::error::{AdminError, AdminResult};
use crate::extension_boot::ExtensionBoot;
use axum::http::HeaderMap;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub(crate) struct ErrorBody {
    pub error: String,
}

#[must_use]
pub fn get_services_path(boot: &ExtensionBoot) -> PathBuf {
    boot.services_root().to_path_buf()
}

// Why: core's composed services root is a content-addressed overlay the next
// refresh replaces, so any file edit must target the baked tree — and a file a
// pinned bundle owns is not editable here at all.
pub(crate) fn get_baked_services_path(boot: &ExtensionBoot) -> PathBuf {
    boot.baked_services_root()
}

#[must_use]
pub fn get_gateway_file_path(boot: &ExtensionBoot) -> PathBuf {
    let root = if crate::repositories::sync::sources::dir_owner(boot.profile(), "ai").is_some() {
        get_services_path(boot)
    } else {
        get_baked_services_path(boot)
    };
    root.join("ai").join("gateway.yaml")
}

pub fn get_gateway_file_path_for_write(boot: &ExtensionBoot) -> AdminResult<PathBuf> {
    if let Some(bundle) = crate::repositories::sync::sources::dir_owner(boot.profile(), "ai") {
        return Err(AdminError::Conflict(format!(
            "ai/gateway.yaml is managed by the services bundle `{bundle}`; change it there"
        )));
    }
    Ok(get_baked_services_path(boot)
        .join("ai")
        .join("gateway.yaml"))
}

pub fn require_write_origin(boot: &ExtensionBoot, headers: &HeaderMap) -> AdminResult<()> {
    let expected = url::Url::parse(&boot.config().api_external_url)
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
