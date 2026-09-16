//! Helpers shared across admin handlers.

use std::path::PathBuf;

use serde::Serialize;
use systemprompt::config::ProfileBootstrap;
use systemprompt::loader::services_root::ServicesRootBootstrap;

use crate::error::AdminResult;

// Why: built only by `AdminError::into_response`, so a status and its body
// cannot be chosen independently of each other.
#[derive(Debug, Serialize)]
pub(crate) struct ErrorBody {
    pub error: String,
}

// Why: the tree the instance is actually serving — the composition of the
// baked tree and every pinned bundle when the profile lists sources, the
// baked tree alone otherwise. Every catalog page reads through here, which
// is what makes a bundled marketplace visible in the console.
pub(crate) fn get_services_path() -> AdminResult<PathBuf> {
    Ok(ServicesRootBootstrap::active_root_or(
        &ProfileBootstrap::get()?.paths.services,
    ))
}

// Why: the composed root is a content-addressed overlay the next refresh
// replaces, so anything that edits a file must edit the baked tree.
pub(crate) fn get_baked_services_path() -> AdminResult<PathBuf> {
    Ok(PathBuf::from(&ProfileBootstrap::get()?.paths.services))
}

// Why: the gateway editor writes the services file the loader reads, never the
// profile — the routes are implementation configuration shipped with the image.
pub(crate) fn get_gateway_file_path() -> AdminResult<PathBuf> {
    Ok(get_baked_services_path()?.join("ai").join("gateway.yaml"))
}
