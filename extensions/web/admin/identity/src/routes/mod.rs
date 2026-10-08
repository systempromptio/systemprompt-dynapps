//! Route fragments the admin composition root merges.
//!
//! Each sits at the position and behind the role gate it has always had; the
//! sign-in routes in [`public`] mount before the session gate. Fragment paths
//! are relative to the mount point the root nests them under.

pub mod account_ssr;
mod admin_groups;
pub mod api;
pub mod people_ssr;
pub mod public;
