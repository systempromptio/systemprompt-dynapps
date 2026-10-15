//! Bounded OAuth exchanges; provider bodies and credentials never enter logs.

use super::{Consent, Grant, Provider};
use std::time::Duration;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};

pub(super) fn client() -> AdminResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(AdminError::internal)
}

pub async fn authorize(consent: Consent<'_>, provider: Provider) -> AdminResult<(String, Grant)> {
    if !provider.configured() {
        return Err(AdminError::Unavailable("Connector not configured".into()));
    }
    super::generic::authorize(consent, provider).await
}

pub(super) use super::tokens::refresh;
pub use super::tokens::{exchange, exchange_with_client, refresh_with_client};
