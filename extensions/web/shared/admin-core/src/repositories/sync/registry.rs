//! The planes this instance syncs, in page order.
//!
//! Each plane registers itself with an `order` where it is implemented, so
//! the page renders one card per registration and adding a plane is an
//! `impl` and an `inventory::submit!`. `access_control` first because it is
//! the entitlement, `groups` next because the rules name its ids, and
//! `gateway_policies` after it because the console also edits it in place,
//! then the two planes whose runtime input is a file core reads at boot:
//! `gateway_routes` and `governance`.

use super::plane::SyncPlane;
use crate::extension_boot::ExtensionBoot;

/// One plane's registration; `order` fixes its position on the page, and
/// `build` makes the plane over the boot values it reads its declaration from.
#[derive(Debug, Clone, Copy)]
pub struct PlaneRegistration {
    pub order: u8,
    pub build: fn(&ExtensionBoot) -> Box<dyn SyncPlane>,
}

inventory::collect!(PlaneRegistration);

#[must_use]
pub fn planes(boot: &ExtensionBoot) -> Vec<Box<dyn SyncPlane>> {
    let mut registrations: Vec<&PlaneRegistration> =
        inventory::iter::<PlaneRegistration>.into_iter().collect();
    registrations.sort_by_key(|r| r.order);
    registrations.into_iter().map(|r| (r.build)(boot)).collect()
}

#[must_use]
pub fn find_plane(boot: &ExtensionBoot, id: &str) -> Option<Box<dyn SyncPlane>> {
    planes(boot).into_iter().find(|p| p.id() == id)
}
