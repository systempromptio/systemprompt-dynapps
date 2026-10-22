//! The worker's HTTP client for the API's worker endpoints, and the
//! [`EventSink`] that streams the executor's events through it.
//!
//! Every call carries the run's worker token. Writes are retried a few times
//! with a short backoff, since a dropped event batch or result is a gap the
//! API cannot recover on its own.

use std::time::Duration;

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use serde::Serialize;
use serde::de::DeserializeOwned;
use systemprompt::agent_runs::{AgentRunsError, AgentRunsResult, EventSink, RunEvent};
use systemprompt::identifiers::HostSessionId;
use systemprompt_web_admin_core::util::outbound::validate_mesh_url;
use systemprompt_web_admin_farm::services::runner::fly::protocol::{
    EventBatch, SessionReport, worker_url,
};

const ATTEMPTS: u32 = 4;
const BACKOFF: Duration = Duration::from_secs(2);
const TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Debug, Clone)]
pub(crate) struct WorkerClient {
    http: reqwest::Client,
    api_base: String,
    run_id: String,
    bearer: String,
}

impl WorkerClient {
    pub(crate) fn new(api_base: &str, run_id: &str, token: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .context("worker HTTP client")?;
        validate_mesh_url(api_base).with_context(|| format!("worker API base {api_base}"))?;
        Ok(Self {
            http,
            api_base: api_base.trim_end_matches('/').to_owned(),
            run_id: run_id.to_owned(),
            bearer: format!("Bearer {token}"),
        })
    }

    fn url(&self, leaf: &str) -> String {
        worker_url(&self.api_base, &self.run_id, leaf)
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, leaf: &str) -> Result<T> {
        let response = self
            .http
            .get(self.url(leaf))
            .header(reqwest::header::AUTHORIZATION, &self.bearer)
            .send()
            .await
            .with_context(|| format!("GET {leaf}"))?;
        let status = response.status();
        if !status.is_success() {
            bail!("GET {leaf}: HTTP {status}");
        }
        response
            .json()
            .await
            .with_context(|| format!("GET {leaf} body"))
    }

    async fn post_once<B: Serialize + Sync>(&self, leaf: &str, body: &B) -> Attempt {
        let sent = self
            .http
            .post(self.url(leaf))
            .header(reqwest::header::AUTHORIZATION, &self.bearer)
            .json(body)
            .send()
            .await;
        let response = match sent {
            Ok(response) => response,
            Err(error) => return Attempt::Retry(anyhow::anyhow!("POST {leaf}: {error}")),
        };
        let status = response.status();
        let text = match response.text().await {
            Ok(text) => text,
            Err(error) => format!("<body unreadable: {error}>"),
        };
        if status.is_success() {
            Attempt::Done(text)
        } else if status.is_client_error() {
            Attempt::Refused(anyhow::anyhow!("POST {leaf}: HTTP {status}: {text}"))
        } else {
            Attempt::Retry(anyhow::anyhow!("POST {leaf}: HTTP {status}: {text}"))
        }
    }

    pub(crate) async fn post<B: Serialize + Sync>(&self, leaf: &str, body: &B) -> Result<String> {
        let mut last = anyhow::anyhow!("POST {leaf} was not attempted");
        for attempt in 1..=ATTEMPTS {
            match self.post_once(leaf, body).await {
                Attempt::Done(text) => return Ok(text),
                Attempt::Refused(error) => return Err(error),
                Attempt::Retry(error) => {
                    tracing::warn!(leaf, attempt, %error, "worker report failed; retrying");
                    last = error;
                    tokio::time::sleep(BACKOFF * attempt).await;
                },
            }
        }
        Err(last)
    }
}

#[derive(Debug)]
enum Attempt {
    Done(String),
    Refused(anyhow::Error),
    Retry(anyhow::Error),
}

#[derive(Debug)]
pub(crate) struct HttpEventSink<'a> {
    pub client: &'a WorkerClient,
}

fn sink_error(error: &anyhow::Error) -> AgentRunsError {
    AgentRunsError::Io(std::io::Error::other(error.to_string()))
}

#[async_trait]
impl EventSink for HttpEventSink<'_> {
    async fn emit(&self, batch: Vec<RunEvent>) -> AgentRunsResult<()> {
        self.client
            .post("events", &EventBatch { events: batch })
            .await
            .map(|_| ())
            .map_err(|e| sink_error(&e))
    }

    async fn session_observed(&self, session_id: &HostSessionId) -> AgentRunsResult<()> {
        let report = SessionReport {
            session_id: session_id.clone(),
        };
        self.client
            .post("session", &report)
            .await
            .map(|_| ())
            .map_err(|e| sink_error(&e))
    }
}
