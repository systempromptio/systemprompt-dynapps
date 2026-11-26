//! The files of the Brain's articles.
//!
//! The bytes go to core's file storage (`ctx.file_storage()`, S3 in
//! production and the same store as chat uploads) under [`STORAGE_PREFIX`];
//! the Brain keeps only the storage reference. A file reaches an article in
//! two steps, out of the MCP channel: a one-time signed URL takes one POST of
//! the bytes, and the person who asked for it adopts the upload as a file of
//! an article they may write. Markdown names a file `asset://<id>` and never
//! holds a URL; a reader gets short-lived signed download URLs instead
//! ([`link`]). The type of a file is read from its bytes ([`sniff`]), and only
//! raster images are ever shown inside a page.

pub mod link;
pub mod markdown;
pub mod model;
pub mod sniff;

use std::fmt;
use std::sync::Arc;

use systemprompt::traits::FileStorage;

use link::LinkSigner;

pub const STORAGE_PREFIX: &str = "dynbrain";
pub const DEFAULT_MAX_UPLOAD_BYTES: u64 = 25 * 1024 * 1024;

/// What the native Brain needs to keep and serve files: core's storage, the
/// link signer, the instance's external URL and the upload size limit.
#[derive(Clone)]
pub struct AssetFiles {
    pub storage: Arc<dyn FileStorage>,
    pub signer: LinkSigner,
    pub base_url: String,
    pub max_upload_bytes: u64,
}

impl fmt::Debug for AssetFiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AssetFiles")
            .field("base_url", &self.base_url)
            .field("max_upload_bytes", &self.max_upload_bytes)
            .finish_non_exhaustive()
    }
}

impl AssetFiles {
    pub const fn new(storage: Arc<dyn FileStorage>, signer: LinkSigner, base_url: String) -> Self {
        Self {
            storage,
            signer,
            base_url,
            max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
        }
    }
}
