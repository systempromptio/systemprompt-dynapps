//! The Brain of an instance in proxy mode: every operation is a call to a
//! tool of the source, an MCP server declared in `services/mcp/`, made as the
//! person who asks.
//!
//! Which tool answers an operation, and under which argument names, is the
//! instance's configuration ([`crate::config::ProxyConfig`]); nothing here
//! knows what the source is. The answer is the source's JSON as it came,
//! because its field names are the ones the native Brain answers with. The
//! identity travels the way core sends it to any external MCP server: when the
//! source's declaration has `external_auth`, core asks the instance for a
//! short-lived token of that person and sends it, so the source decides what
//! the person may read or write. Permissions are therefore the source's own,
//! and the grant operations answer `managed_at_source`.
//!
//! `get` names the source's files the way the native Brain names its own:
//! [`files`] turns them into `asset://<id>` and adds a short-lived source
//! link per file in `asset_urls`. The file bytes never pass through here.

mod brain;
pub mod files;
pub mod source;

use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
// JSON: tool arguments and answers are the JSON the MCP protocol carries.
use serde_json::Value;
use systemprompt::models::execution::context::RequestContext;

use super::Reply;
use super::fault::{BrainFault, SourceFault};
use super::operation::Operation;
use crate::config::ProxyConfig;

/// One tool call to the source, as the person the context names. Held as
/// `Arc<dyn Source>` by the proxy Brain, hence `#[async_trait]`.
#[async_trait]
pub trait Source: Send + Sync + Debug {
    // JSON: the tool's arguments and its answer.
    async fn call(
        &self,
        context: &RequestContext,
        tool: &str,
        arguments: Value,
    ) -> Result<Value, SourceFault>;
}

/// The Brain of a proxy instance.
#[derive(Debug, Clone)]
pub struct ProxyBrain {
    source: Arc<dyn Source>,
    config: ProxyConfig,
}

impl ProxyBrain {
    pub fn new(source: Arc<dyn Source>, config: ProxyConfig) -> Self {
        Self { source, config }
    }

    pub const fn config(&self) -> &ProxyConfig {
        &self.config
    }

    // JSON: our arguments in, the source's answer out.
    pub async fn relay(
        &self,
        context: &RequestContext,
        operation: Operation,
        args: Value,
    ) -> Reply<Value> {
        let mapping = self
            .config
            .operations
            .get(&operation)
            .ok_or(BrainFault::Unavailable(
                "the source of this instance has no tool for this operation",
            ))?;
        let answer = self
            .source
            .call(context, &mapping.tool, mapping.translate(args))
            .await?;
        if operation == Operation::Get {
            return Ok(files::with_assets(self, context, answer).await);
        }
        Ok(answer)
    }
}
