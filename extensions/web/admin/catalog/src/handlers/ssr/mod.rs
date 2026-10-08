//! Server-rendered catalog pages outside the catalog family itself: the
//! retention lifecycle page and the Analysis pages for marketplace versions,
//! revisions, inventory actions and publication decisions.

pub(crate) mod analysis;
mod ssr_lifecycle;

pub(crate) use ssr_lifecycle::{lifecycle_archive_download, lifecycle_page};
