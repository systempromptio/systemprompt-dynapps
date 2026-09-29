//! Service layer between the admin handlers and the repositories.

pub(crate) mod analysis_report;
pub(crate) mod device_service;
pub(crate) mod jobs_service;
pub(crate) mod salesforce_jwt_bearer;

pub mod connector_oauth;
pub mod salesforce_orgs;

pub(crate) mod connector_accounts;
pub(crate) mod connector_readiness;
pub mod identity_token;
