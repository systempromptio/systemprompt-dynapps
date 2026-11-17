//! The read-only HTTP client to farm-api.
//!
//! One service credential against one base URL (ending in `/farm/api/v1`),
//! both named in [`crate::config::FarmConfig`]; the token itself is read from
//! the secrets store on every call. Every call is a `GET`; the
//! tenant is always named with `?tenant=`: a slug, or [`ALL_TENANTS`] on the
//! list calls, which farm-api resolves to every tenant the credential can see
//! (all of them with the platform read scope). The token only ever travels in
//! the `Authorization` header: no error, log line or page carries it.

mod error;
mod ids;
mod token;

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use systemprompt::models::net::{LoopbackPolicy, validate_outbound_url};
use systemprompt::traits::SecretStore;

pub use self::error::{FarmApiError, status_error};
use self::ids::task_path;
pub use self::ids::{is_agent_slug, is_task_id};
use self::token::Token;
use crate::config::FarmConfig;
use crate::models::{
    AgentList, AgentOut, CredentialList, ScheduleList, TaskList, TaskLogs, TaskOut, TaskResult,
    WhoAmI,
};

pub const ALL_TENANTS: &str = "*";

pub fn is_all_tenants(tenant: &str) -> bool {
    tenant.is_empty() || tenant == ALL_TENANTS
}

const HTTP_TIMEOUT: Duration = Duration::from_secs(10);
pub const PAGE_SIZE: u32 = 50;
pub const COST_PAGE_SIZE: u32 = 100;
pub const TAIL_LINES: u32 = 200;

#[derive(Clone)]
struct Inner {
    http: reqwest::Client,
    base: String,
    token: Token,
}

/// The farm-api client the pages share. `None` inside means the pod has no
/// credential: every call then fails with [`FarmApiError::NotConfigured`].
#[derive(Clone)]
pub struct FarmApi {
    inner: Option<Inner>,
}

impl fmt::Debug for FarmApi {
    // Why: the derived Debug would print the token.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FarmApi")
            .field("base", &self.inner.as_ref().map(|i| i.base.as_str()))
            .field("configured", &self.inner.is_some())
            .finish()
    }
}

impl FarmApi {
    pub fn from_config(config: &FarmConfig, secrets: Arc<dyn SecretStore>) -> Self {
        let key = config.token_secret.trim().to_owned();
        if key.is_empty() {
            return Self::disabled();
        }
        Self::build(
            &config.api_url,
            Token::Store {
                store: secrets,
                key,
            },
        )
    }

    pub fn new(base_url: &str, token: &str) -> Self {
        let token = token.trim().to_owned();
        if token.is_empty() {
            return Self::disabled();
        }
        Self::build(base_url, Token::Fixed(token))
    }

    fn build(base_url: &str, token: Token) -> Self {
        let base = base_url.trim().trim_end_matches('/').to_owned();
        if base.is_empty() {
            return Self::disabled();
        }
        let http = reqwest::Client::builder()
            .timeout(HTTP_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build();
        match http {
            Ok(http) => Self {
                inner: Some(Inner { http, base, token }),
            },
            Err(e) => {
                tracing::warn!(error = %e, "farm-api client could not be built");
                Self::disabled()
            },
        }
    }

    pub const fn disabled() -> Self {
        Self { inner: None }
    }

    pub const fn is_configured(&self) -> bool {
        self.inner.is_some()
    }

    pub async fn whoami(&self) -> Result<WhoAmI, FarmApiError> {
        self.get("/whoami", &[]).await
    }

    pub async fn list_tasks(
        &self,
        tenant: &str,
        cursor: Option<&str>,
    ) -> Result<TaskList, FarmApiError> {
        let limit = PAGE_SIZE.to_string();
        let mut query = vec![("tenant", tenant), ("limit", limit.as_str())];
        if let Some(cursor) = cursor {
            query.push(("cursor", cursor));
        }
        self.get("/tasks", &query).await
    }

    pub async fn list_tasks_between(
        &self,
        tenant: &str,
        since: &str,
        until: &str,
        cursor: Option<&str>,
    ) -> Result<TaskList, FarmApiError> {
        let limit = COST_PAGE_SIZE.to_string();
        let mut query = vec![
            ("tenant", tenant),
            ("limit", limit.as_str()),
            ("since", since),
            ("until", until),
        ];
        if let Some(cursor) = cursor {
            query.push(("cursor", cursor));
        }
        self.get("/tasks", &query).await
    }

    pub async fn list_agents(&self, tenant: &str) -> Result<AgentList, FarmApiError> {
        self.get("/agents", &[("tenant", tenant)]).await
    }

    pub async fn get_agent(&self, tenant: &str, slug: &str) -> Result<AgentOut, FarmApiError> {
        if !is_agent_slug(slug) {
            return Err(FarmApiError::BadSlug(slug.to_owned()));
        }
        self.get(&format!("/agents/{slug}"), &[("tenant", tenant)])
            .await
    }

    pub async fn list_schedules(&self, tenant: &str) -> Result<ScheduleList, FarmApiError> {
        self.get("/schedules", &[("tenant", tenant)]).await
    }

    pub async fn list_credentials(
        &self,
        tenant: &str,
        scope: &str,
    ) -> Result<CredentialList, FarmApiError> {
        self.get("/credentials", &[("tenant", tenant), ("scope", scope)])
            .await
    }

    pub async fn get_task(&self, tenant: &str, id: &str) -> Result<TaskOut, FarmApiError> {
        let path = task_path(id, "")?;
        self.get(&path, &[("tenant", tenant)]).await
    }

    pub async fn get_result(&self, tenant: &str, id: &str) -> Result<TaskResult, FarmApiError> {
        let path = task_path(id, "/result")?;
        self.get(&path, &[("tenant", tenant)]).await
    }

    pub async fn get_logs(&self, tenant: &str, id: &str) -> Result<TaskLogs, FarmApiError> {
        let path = task_path(id, "/logs")?;
        let tail = TAIL_LINES.to_string();
        self.get(&path, &[("tenant", tenant), ("tail_lines", tail.as_str())])
            .await
    }

    async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, FarmApiError> {
        let inner = self.inner.as_ref().ok_or(FarmApiError::NotConfigured)?;
        let token = inner.token.current().ok_or(FarmApiError::NotConfigured)?;
        let target = format!("{}{path}", inner.base);
        let url = validate_outbound_url(&target, LoopbackPolicy::Allow).map_err(|source| {
            FarmApiError::BadUrl {
                url: target,
                source,
            }
        })?;
        let response = inner
            .http
            .get(url.as_str())
            .bearer_auth(&token)
            .query(query)
            .send()
            .await
            .map_err(|e| FarmApiError::Unreachable(e.without_url()))?;
        let status = response.status();
        let body = response
            .bytes()
            .await
            .map_err(|e| FarmApiError::Unreachable(e.without_url()))?;
        if !status.is_success() {
            return Err(status_error(status.as_u16(), &body));
        }
        serde_json::from_slice(&body).map_err(FarmApiError::Decode)
    }
}
