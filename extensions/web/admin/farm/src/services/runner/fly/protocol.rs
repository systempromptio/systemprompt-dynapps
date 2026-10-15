//! The wire contract between the API and a farm worker on a Fly Machine.
//!
//! The worker holds three things from its machine env: the `agent_runs` id
//! ([`RUN_ID_ENV`]), the API base it reports to ([`API_BASE_ENV`]) and a
//! run-scoped bearer ([`WORKER_TOKEN_ENV`]). Every route under
//! [`WORKER_PREFIX`] checks that bearer against the run's in-flight entry,
//! so the token is good for one run and only while the API is waiting on it.
//!
//! The run directory travels as a [`Bundle`] of files relative to the run
//! root: the materialised workspace and plugin tree down, the edited
//! workspace and logs back up. The worker uses the API's own run root when
//! it can, so the paths baked into the stored request stay true; when it
//! cannot, [`rebase_request`] moves every path in the request to its root.

use std::path::{Component, Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use systemprompt::agent_runs::{DiffSummary, RunCredentials, RunEvent, RunOutcome, RunRequest};
use systemprompt::identifiers::{ApiKeyId, ApiKeySecret, HostSessionId, SessionId, UserId};
use walkdir::WalkDir;

use crate::error::{FarmError, FarmResult};

pub const WORKER_PREFIX: &str = "/api/public/agent-farm-worker";
pub const RUN_ID_ENV: &str = "RUN_ID";
pub const API_BASE_ENV: &str = "API_BASE";
pub const WORKER_TOKEN_ENV: &str = "FARM_WORKER_TOKEN";
pub const OUTPUT_DIRS: [&str; 2] = ["workspace", "logs"];
pub const MAX_BUNDLE_BYTES: u64 = 48 * 1024 * 1024;

/// The credentials the API issued for the run, as the worker rebuilds them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerCredentials {
    pub user_id: UserId,
    pub api_key_id: ApiKeyId,
    pub pat: String,
    pub gateway_session_id: SessionId,
    pub hook_token: Option<String>,
    pub mcp_token: Option<String>,
}

impl WorkerCredentials {
    #[must_use]
    pub fn from_run(credentials: &RunCredentials) -> Self {
        Self {
            user_id: credentials.user_id.clone(),
            api_key_id: credentials.api_key_id.clone(),
            pat: credentials.pat.as_str().to_owned(),
            gateway_session_id: credentials.gateway_session_id.clone(),
            hook_token: credentials.hook_token.clone(),
            mcp_token: credentials.mcp_token.clone(),
        }
    }

    #[must_use]
    pub fn into_run(self) -> RunCredentials {
        RunCredentials {
            user_id: self.user_id,
            api_key_id: self.api_key_id,
            pat: ApiKeySecret::new(self.pat),
            gateway_session_id: self.gateway_session_id,
            hook_token: self.hook_token,
            hook_token_jti: None,
            hook_token_expires_at: None,
            mcp_token: self.mcp_token,
            mcp_token_jti: None,
            mcp_token_expires_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerSpec {
    pub agent_run_id: String,
    pub farm_run_id: String,
    pub server_root: PathBuf,
    pub request: RunRequest,
    pub credentials: WorkerCredentials,
    pub gateway_base_url: String,
    pub govern_url: String,
    pub track_url: String,
    pub seq_after: Option<i64>,
    pub heartbeat_seconds: u64,
    #[serde(default)]
    pub egress: super::egress::EgressPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleFile {
    pub path: String,
    pub content_b64: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    pub files: Vec<BundleFile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventBatch {
    pub events: Vec<RunEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionReport {
    pub session_id: HostSessionId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatReply {
    pub cancel: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerResult {
    pub outcome: RunOutcome,
    pub diff: Option<DiffSummary>,
}

#[must_use]
pub fn token_hash(token: &str) -> [u8; 32] {
    Sha256::digest(token.as_bytes()).into()
}

#[must_use]
pub fn worker_url(api_base: &str, agent_run_id: &str, leaf: &str) -> String {
    format!(
        "{}{WORKER_PREFIX}/runs/{agent_run_id}/{leaf}",
        api_base.trim_end_matches('/')
    )
}

fn relative_is_safe(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|c| matches!(c, Component::Normal(_)))
}

pub fn collect_bundle(root: &Path, dirs: &[&str]) -> FarmResult<Bundle> {
    let mut files = Vec::new();
    let mut total = 0u64;
    for dir in dirs {
        let base = root.join(dir);
        if !base.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&base).follow_links(false) {
            let entry = entry.map_err(|err| FarmError::Io(std::io::Error::other(err)))?;
            if !entry.file_type().is_file() {
                continue;
            }
            let Ok(relative) = entry.path().strip_prefix(root) else {
                continue;
            };
            let bytes = std::fs::read(entry.path())?;
            total += u64::try_from(bytes.len()).unwrap_or(u64::MAX);
            if total > MAX_BUNDLE_BYTES {
                return Err(FarmError::Validation(format!(
                    "run bundle exceeds {MAX_BUNDLE_BYTES} bytes"
                )));
            }
            files.push(BundleFile {
                path: relative.to_string_lossy().into_owned(),
                content_b64: STANDARD.encode(bytes),
            });
        }
    }
    Ok(Bundle { files })
}

pub fn write_bundle(root: &Path, bundle: &Bundle) -> FarmResult<usize> {
    for file in &bundle.files {
        let relative = Path::new(&file.path);
        if !relative_is_safe(relative) {
            return Err(FarmError::Validation(format!(
                "bundle path {:?} leaves the run directory",
                file.path
            )));
        }
        let bytes =
            STANDARD
                .decode(&file.content_b64)
                .map_err(|source| FarmError::BundleDecode {
                    path: file.path.clone(),
                    source,
                })?;
        let dest = root.join(relative);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(dest, bytes)?;
    }
    Ok(bundle.files.len())
}

fn rebase(path: &Path, from: &Path, to: &Path) -> PathBuf {
    path.strip_prefix(from)
        .map_or_else(|_| path.to_path_buf(), |rest| to.join(rest))
}

#[must_use]
pub fn rebase_request(request: &RunRequest, from: &Path, to: &Path) -> RunRequest {
    let mut request = request.clone();
    if from == to {
        return request;
    }
    request.prompt = request
        .prompt
        .replace(&*from.to_string_lossy(), &to.to_string_lossy());
    request.workspace = rebase(&request.workspace, from, to);
    request.add_dirs = request
        .add_dirs
        .iter()
        .map(|p| rebase(p, from, to))
        .collect();
    request.plugin_dirs = request
        .plugin_dirs
        .iter()
        .map(|p| rebase(p, from, to))
        .collect();
    request
}
