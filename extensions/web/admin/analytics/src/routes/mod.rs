//! Route fragments the admin composition root merges.
//!
//! Each sits at the position and behind the role gate it has always had.
//! Fragment paths are relative to the mount point the root nests them under;
//! [`hooks_track`] is the one fragment mounted outside the admin plane,
//! beside the governance webhooks.

pub mod activity_ssr;
pub mod analysis_ssr;
pub mod api;
pub mod export_ssr;
pub mod hooks_track;
pub mod observability_ssr;
pub mod overview;
