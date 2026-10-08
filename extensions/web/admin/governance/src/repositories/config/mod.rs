//! The configured gateway access surface: the ACL the gateway enforces over
//! routes, and the after-the-fact detector that re-runs it over traffic that
//! already went through.

pub mod acl_detect;
pub mod gateway_acl;
