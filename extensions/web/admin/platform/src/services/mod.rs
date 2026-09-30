//! Service layer between the platform handlers and the repositories.
//!
//! The connector OAuth broker and the connection snapshot, device credential
//! issuance, the Salesforce org registry and JWT-bearer grant, identity
//! tokens, and the jobs listing.

pub mod connector_oauth;
pub mod identity_token;
pub mod salesforce_orgs;

pub(crate) mod connector_accounts;
pub(crate) mod connector_readiness;
pub(crate) mod device_service;
pub(crate) mod jobs_service;
pub(crate) mod salesforce_jwt_bearer;
