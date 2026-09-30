//! Who a user is for authorization purposes.
//!
//! The identity envelope (roles, directory groups, status), the group
//! memberships it resolves to, the access profile the entitlement matrix
//! resolves against, the scoped listing every person picker reads, and the
//! Salesforce usernames a user is linked for.

pub mod group_membership;
pub mod identity;
pub mod listing;
pub mod role;
pub mod salesforce_identity;
