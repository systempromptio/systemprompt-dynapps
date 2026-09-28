//! Who a user is for authorization purposes: the identity envelope (roles,
//! directory groups, status), the access profile the entitlement matrix
//! resolves against, and the scoped listing every person picker reads.

pub mod identity;
pub mod listing;
pub mod role;
