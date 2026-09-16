//! The planes this instance syncs, in page order.
//!
//! One line per plane. `groups` (groups.yaml → groups, projects and directory
//! mappings) and `gateway_policies` are the next two and are not built; the
//! page renders one card per entry here, so adding either is an `impl` and a
//! line.

use super::access_control::AccessControlPlane;
use super::plane::SyncPlane;

#[must_use]
pub fn planes() -> Vec<Box<dyn SyncPlane>> {
    vec![Box::new(AccessControlPlane)]
}

#[must_use]
pub fn find_plane(id: &str) -> Option<Box<dyn SyncPlane>> {
    planes().into_iter().find(|p| p.id() == id)
}
