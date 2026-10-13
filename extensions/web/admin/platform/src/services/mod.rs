//! Service layer between the platform handlers and the repositories.
//!
//! The connector OAuth broker and the connection snapshot, device credential
//! issuance, the Salesforce org registry and JWT-bearer grant, identity
//! tokens, the jobs listing, and the machine credentials service accounts
//! hold.

pub mod connector_oauth;
pub mod identity_token;
pub mod machine_credentials;
pub mod salesforce_orgs;
pub mod service_accounts;

pub(crate) mod connector_accounts;
pub(crate) mod connector_readiness;
pub(crate) mod device_service;
pub(crate) mod jobs_service;
pub(crate) mod salesforce_jwt_bearer;
