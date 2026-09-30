//! Route fragments the admin composition root merges.
//!
//! Each sits at the position and behind the role gate it has always had; the
//! routers in [`public`] mount outside the admin session. Fragment paths are
//! relative to the mount point the root nests them under.

pub mod api;
pub mod platform_ssr;
pub mod public;
pub mod ssr_bridge;
