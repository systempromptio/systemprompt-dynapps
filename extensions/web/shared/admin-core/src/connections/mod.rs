//! One public connection model for browser accounts and every enrolled bridge.
//!
//! [`Connection`] and [`ConnectionSnapshot`] are the shape every reader agrees
//! on — the Connectors page, the bridge manifest, the marketplace filter and
//! the `connector` authorization dimension. [`readiness`] decides whether a
//! connection will carry a person's calls, [`report`] is the step-by-step
//! outcome of a probe, and [`source`] is where a snapshot comes from: the
//! crate that owns connector credentials registers the one implementation.

pub mod readiness;
pub mod report;
pub mod source;

use serde::Serialize;
use systemprompt::identifiers::UserId;

use report::VerificationReport;

#[derive(Debug, Clone, Serialize)]
pub struct Connection {
    pub provider: String,
    pub display_name: String,
    pub requires_auth: bool,
    pub session_attested: bool,
    pub configured: bool,
    pub entitled: bool,
    pub status: String,
    pub auth_method: Option<String>,
    pub account_id: Option<String>,
    pub account_name: Option<String>,
    pub resource_id: Option<String>,
    pub resource_name: Option<String>,
    pub error_code: Option<String>,
    pub verified_at: Option<String>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionSnapshot {
    pub version: u32,
    pub user_id: UserId,
    pub revision: i64,
    pub connections: Vec<Connection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification: Option<VerificationReport>,
}
