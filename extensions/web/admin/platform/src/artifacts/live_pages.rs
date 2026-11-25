//! What a live page may reach on this instance: its servers, the operator's
//! policy and a session with core's MCP proxy, shared by the broker and by
//! the `documents` server that records the agent's declarations.
//!
//! A declaration is checked the way a call will be: each server is declared
//! and enabled here, the policy allows it and each tool, and the server lists
//! each tool for the person declaring it (their agent's own bearer, through
//! the proxy), so a page never declares a tool its author cannot see.
//!
//! Each declared server is shown to the viewer by its name (`names`): the
//! `display_name` the live-pages policy gives it, else its connector's
//! `display_name` in the instance's MCP config, else its id.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;
// JSON: the manifest is stored as JSONB; tool listings are MCP payloads.
use serde_json::Value;
use systemprompt::identifiers::{DocumentId, McpServerId, UserId};
use systemprompt::manifest::services::ServicesConfig;
use systemprompt::traits::ConfigWatch;

use super::manifest::{CONTRACT_VERSION, Capabilities, DeclarationRefused};
use super::policy::{LivePagesPolicy, LivePagesPolicyError};
use super::proxy::LoopbackProxy;
use crate::repositories::artifact_capabilities;

/// The instance's live-page surface: policy, servers and the proxy.
#[derive(Clone)]
pub struct LivePages {
    inner: Arc<Inner>,
}

struct Inner {
    policy: LivePagesPolicy,
    services: ConfigWatch<ServicesConfig>,
    proxy: LoopbackProxy,
}

impl std::fmt::Debug for LivePages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LivePages").finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LivePagesError {
    #[error(transparent)]
    Policy(#[from] LivePagesPolicyError),
    #[error("the MCP proxy client could not be built: {0}")]
    Client(#[from] reqwest::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ServerName {
    pub(crate) id: String,
    pub(crate) name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ServerStanding {
    Unknown,
    Off,
    Closed,
    Open,
}

impl LivePages {
    pub fn new(
        services_root: &Path,
        services: ConfigWatch<ServicesConfig>,
        api_internal_url: &str,
    ) -> Result<Self, LivePagesError> {
        Ok(Self {
            inner: Arc::new(Inner {
                policy: LivePagesPolicy::load(services_root)?,
                services,
                proxy: LoopbackProxy::new(api_internal_url)?,
            }),
        })
    }

    pub(crate) fn standing(&self, server: &str) -> ServerStanding {
        match self.inner.services.get().mcp_servers.get(server) {
            None => ServerStanding::Unknown,
            Some(deployment) if !deployment.enabled => ServerStanding::Off,
            Some(_) if !self.inner.policy.allows_server(server) => ServerStanding::Closed,
            Some(_) => ServerStanding::Open,
        }
    }

    pub(crate) fn allows(&self, server: &str, tool: &str) -> bool {
        self.inner.policy.allows(server, tool)
    }

    pub(crate) fn names(&self, capabilities: Option<&Capabilities>) -> Vec<ServerName> {
        let services = self.inner.services.get();
        capabilities
            .map_or(&[][..], Capabilities::servers)
            .iter()
            .map(|declared| {
                let id = declared.server.as_str();
                let connector = services
                    .mcp_servers
                    .get(id)
                    .and_then(|deployment| deployment.connector.as_ref())
                    .and_then(|connector| connector.display_name.as_deref());
                let name = self
                    .inner
                    .policy
                    .display_name(id)
                    .or(connector)
                    .filter(|name| !name.trim().is_empty())
                    .unwrap_or(id);
                ServerName {
                    id: id.to_owned(),
                    name: name.to_owned(),
                }
            })
            .collect()
    }

    pub(crate) fn proxy(&self) -> &LoopbackProxy {
        &self.inner.proxy
    }

    pub async fn check(
        &self,
        capabilities: &Capabilities,
        bearer: &str,
    ) -> Result<(), DeclarationRefused> {
        for declared in capabilities.servers() {
            let server = declared.server.as_str();
            match self.standing(server) {
                ServerStanding::Unknown => {
                    return refused(format!("this instance has no MCP server `{server}`"));
                },
                ServerStanding::Off => {
                    return refused(format!("the MCP server `{server}` is switched off"));
                },
                ServerStanding::Closed => {
                    return refused(format!(
                        "the live-pages policy does not let a page use `{server}`"
                    ));
                },
                ServerStanding::Open => {},
            }
            if let Some(tool) = declared
                .tools
                .iter()
                .find(|tool| !self.allows(server, tool))
            {
                return refused(format!(
                    "the live-pages policy does not let a page call `{tool}` on `{server}`"
                ));
            }
            let listed = self.listed(server, bearer).await?;
            if let Some(tool) = declared
                .tools
                .iter()
                .find(|tool| !listed.contains(tool.as_str()))
            {
                return refused(format!("`{server}` lists no tool `{tool}` for you"));
            }
        }
        Ok(())
    }

    async fn listed(
        &self,
        server: &str,
        bearer: &str,
    ) -> Result<BTreeSet<String>, DeclarationRefused> {
        let unreachable = |error: super::error::ArtifactCallError| {
            DeclarationRefused::Instance(format!(
                "`{server}` could not be asked for its tools: {}",
                error.code().as_str()
            ))
        };
        let mut session = self
            .proxy()
            .open(&McpServerId::new(server), bearer)
            .await
            .map_err(unreachable)?;
        let tools = session.tools().await;
        session.close().await;
        Ok(tools
            .map_err(unreachable)?
            .iter()
            .filter_map(|tool| tool.get("name").and_then(Value::as_str))
            .map(str::to_owned)
            .collect())
    }

    pub async fn record(
        &self,
        pool: &PgPool,
        document: &DocumentId,
        capabilities: &Capabilities,
        author: &UserId,
    ) -> Result<(), sqlx::Error> {
        if capabilities.is_empty() {
            return artifact_capabilities::delete_artifact_capabilities(pool, document).await;
        }
        let stored = serde_json::to_value(capabilities)
            .map_err(|error| sqlx::Error::Encode(Box::new(error)))?;
        artifact_capabilities::set_artifact_capabilities(
            pool,
            document,
            &stored,
            CONTRACT_VERSION,
            author,
        )
        .await
    }

    pub(crate) async fn manifest(
        pool: &PgPool,
        document: &DocumentId,
    ) -> Result<Option<(Capabilities, String)>, sqlx::Error> {
        let Some(row) = artifact_capabilities::find_artifact_capabilities(pool, document).await?
        else {
            return Ok(None);
        };
        let capabilities = serde_json::from_value(row.capabilities)
            .map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
        Ok(Some((capabilities, row.contract_version)))
    }
}

const fn refused(why: String) -> Result<(), DeclarationRefused> {
    Err(DeclarationRefused::Instance(why))
}
