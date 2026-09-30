//! Who a user is for authorization purposes.
//!
//! The identity envelope (roles, directory groups, status), the group
//! memberships it resolves to, the access profile the entitlement matrix
//! resolves against, and the scoped listing every person picker reads.

pub mod group_membership;
pub mod identity;
pub mod listing;
pub mod role;
