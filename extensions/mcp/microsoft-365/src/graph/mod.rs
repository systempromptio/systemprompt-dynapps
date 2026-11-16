//! The Microsoft Graph client every `d365_*` tool calls through.
//!
//! One [`GraphClient`] serves one tool call for one user: it holds that
//! user's token source, fetches the token lazily, and renews it once when
//! Graph answers 401. Requests only ever reach the configured
//! [`GraphEndpoint`]; `@odata.nextLink` cursors are followed only on the same
//! origin, so the user's bearer never leaves Graph.

pub mod config;
pub mod error;
pub mod paging;
pub mod query;
pub mod sanitize;
mod send;
pub mod token;

use std::fmt;
use std::sync::Arc;

use reqwest::Method;
use tokio::sync::Mutex;

pub use config::{DEFAULT_GRAPH_URL, GRAPH_URL_SECRET, GraphEndpoint, GraphLimits, http_client};
pub use error::GraphError;
pub use paging::{Page, Window};
pub use query::GraphQuery;
pub use send::{GraphBody, GraphRequest, GraphResponse};
pub use token::{BROKER_PATH, Broker, BrokerTokens, GraphTokens};

/// What a [`GraphClient`] is built from.
pub struct GraphClientParts {
    pub endpoint: GraphEndpoint,
    pub http: reqwest::Client,
    pub tokens: Arc<dyn GraphTokens>,
    pub limits: GraphLimits,
}

impl fmt::Debug for GraphClientParts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphClientParts")
            .field("endpoint", &self.endpoint)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

pub struct GraphClient {
    endpoint: GraphEndpoint,
    http: reqwest::Client,
    tokens: Arc<dyn GraphTokens>,
    limits: GraphLimits,
    token: Mutex<Option<String>>,
}

impl fmt::Debug for GraphClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GraphClient")
            .field("endpoint", &self.endpoint)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

impl GraphClient {
    pub fn new(parts: GraphClientParts) -> Self {
        let GraphClientParts {
            endpoint,
            http,
            tokens,
            limits,
        } = parts;
        Self {
            endpoint,
            http,
            tokens,
            limits,
            token: Mutex::new(None),
        }
    }

    pub const fn endpoint(&self) -> &GraphEndpoint {
        &self.endpoint
    }

    pub const fn limits(&self) -> &GraphLimits {
        &self.limits
    }

    // JSON: Graph resources are open-world; tools pass them through as Graph
    // returns them.
    pub async fn get_json(
        &self,
        path: &str,
        query: &GraphQuery,
    ) -> Result<serde_json::Value, GraphError> {
        self.send(&GraphRequest::new(Method::GET, path).with_query(query.clone()))
            .await?
            .json()
    }

    pub async fn send(&self, request: &GraphRequest) -> Result<GraphResponse, GraphError> {
        send::execute(self, request).await
    }

    async fn current_token(&self) -> Result<String, GraphError> {
        let mut slot = self.token.lock().await;
        if let Some(token) = slot.as_ref() {
            return Ok(token.clone());
        }
        let fresh = self.tokens.access_token(None).await?;
        *slot = Some(fresh.clone());
        Ok(fresh)
    }

    async fn renewed_token(&self, stale: &str) -> Result<Option<String>, GraphError> {
        let fresh = self.tokens.access_token(Some(stale)).await?;
        if fresh == stale {
            return Ok(None);
        }
        *self.token.lock().await = Some(fresh.clone());
        Ok(Some(fresh))
    }
}
