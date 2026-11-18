//! Where a call's Graph token comes from.
//!
//! In production it is the connector broker
//! (`/api/public/connectors/microsoft-365/token`), asked with the caller's
//! own bearer and the broker secret exactly as core asks it for an external
//! MCP server, so the broker answers for that user only and refreshes the
//! token itself. Tests inject any [`GraphTokens`].

use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::manifest::Config;
use systemprompt::mcp::services::client::external_auth::{
    BROKER_SECRET_KEY, accessor_url, fetch_external_bearer,
};
use systemprompt::traits::SecretLookup;

use super::GraphError;

pub const BROKER_PATH: &str = "/api/public/connectors/microsoft-365/token";

/// Held as `dyn GraphTokens` so a call takes the broker in production and a
/// fixed token in tests.
#[async_trait]
pub trait GraphTokens: Send + Sync + fmt::Debug {
    async fn access_token(&self, stale: Option<&str>) -> Result<String, GraphError>;
}

/// The broker's address, its secret and the server it answers for; held for
/// the server's lifetime and bound to one caller per call.
#[derive(Clone)]
pub struct Broker {
    accessor: String,
    secrets: Arc<dyn SecretLookup>,
    server: String,
}

impl fmt::Debug for Broker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Broker")
            .field("accessor", &self.accessor)
            .field("server", &self.server)
            .finish_non_exhaustive()
    }
}

impl Broker {
    pub fn new(
        accessor: impl Into<String>,
        secrets: Arc<dyn SecretLookup>,
        server: impl Into<String>,
    ) -> Self {
        Self {
            accessor: accessor.into(),
            secrets,
            server: server.into(),
        }
    }

    pub fn for_instance(
        config: &Config,
        secrets: Arc<dyn SecretLookup>,
        server: impl Into<String>,
    ) -> Self {
        Self::new(
            accessor_url(&config.api_external_url, BROKER_PATH, config),
            secrets,
            server,
        )
    }

    pub fn for_caller(&self, bearer: String, purpose: Option<String>) -> BrokerTokens {
        BrokerTokens {
            broker: self.clone(),
            bearer,
            purpose,
        }
    }
}

pub struct BrokerTokens {
    broker: Broker,
    bearer: String,
    purpose: Option<String>,
}

impl fmt::Debug for BrokerTokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BrokerTokens")
            .field("broker", &self.broker)
            .field("purpose", &self.purpose)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl GraphTokens for BrokerTokens {
    async fn access_token(&self, _stale: Option<&str>) -> Result<String, GraphError> {
        let secret = self
            .broker
            .secrets
            .lookup(BROKER_SECRET_KEY)
            .filter(|secret| !secret.expose().is_empty())
            .ok_or_else(|| {
                GraphError::Config(format!(
                    "secret `{BROKER_SECRET_KEY}` is not granted to this server"
                ))
            })?;
        let authorization = fetch_external_bearer(
            &self.broker.accessor,
            &self.bearer,
            secret.expose(),
            &self.broker.server,
            self.purpose.as_deref(),
        )
        .await?;
        Ok(bearer_credential(&authorization)?.to_owned())
    }
}

// Why: the broker answers with the whole `Authorization` value core's
// external-server proxy sends as is (`Bearer <token>`); the Graph client adds
// its own scheme, so only the credential is kept, and anything but a bearer
// credential is refused rather than forwarded to Graph.
pub fn bearer_credential(authorization: &str) -> Result<&str, GraphError> {
    let value = authorization.trim();
    let credential = match value.split_once(char::is_whitespace) {
        Some((scheme, credential)) if scheme.eq_ignore_ascii_case("bearer") => credential.trim(),
        Some(_) => "",
        None if value.eq_ignore_ascii_case("bearer") => "",
        None => value,
    };
    if credential.is_empty() || credential.contains(char::is_whitespace) {
        return Err(GraphError::Config(
            "the connector broker returned no bearer credential".to_owned(),
        ));
    }
    Ok(credential)
}
