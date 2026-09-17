//! The planes this instance syncs, in page order.
//!
//! One line per plane; the page renders one card per entry here, so adding
//! a plane is an `impl` and a line. `access_control` first because it is
//! the entitlement, `groups` next because the rules name its ids, and
//! `gateway_policies` last because it is the one plane the console also
//! edits in place.

use super::access_control::AccessControlPlane;
use super::gateway_policies::GatewayPoliciesPlane;
use super::groups::GroupsPlane;
use super::plane::SyncPlane;

#[must_use]
pub fn planes() -> Vec<Box<dyn SyncPlane>> {
    vec![
        Box::new(AccessControlPlane),
        Box::new(GroupsPlane),
        Box::new(GatewayPoliciesPlane),
    ]
}

#[must_use]
pub fn find_plane(id: &str) -> Option<Box<dyn SyncPlane>> {
    planes().into_iter().find(|p| p.id() == id)
}
