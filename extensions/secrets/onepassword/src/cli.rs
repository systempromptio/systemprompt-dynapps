//! The `op` CLI backend — what 1Password supports for service accounts.
//!
//! The child gets a cleared environment: the token goes in as
//! `OP_SERVICE_ACCOUNT_TOKEN` and only the variables `op` needs to run are
//! passed through, all read once at build time through core's `EnvLookup`.
//! The token is never on argv; stdout is parsed and never logged; stderr is
//! kept, truncated, for the error.

use std::fmt;
use std::process::Stdio;
use std::time::Duration;

use async_trait::async_trait;
use tokio::process::Command;
use zeroize::Zeroizing;

use crate::backend::Backend;
use crate::error::OnePasswordError;
use crate::item::Item;
use crate::reference::OpReference;

pub const TOKEN_VAR: &str = "OP_SERVICE_ACCOUNT_TOKEN";

pub const PASSTHROUGH_VARS: [&str; 4] = ["PATH", "HOME", "XDG_CONFIG_HOME", "TMPDIR"];

pub struct CliBackend {
    op_bin: String,
    token: Zeroizing<String>,
    passthrough: Vec<(String, String)>,
    timeout: Duration,
}

impl fmt::Debug for CliBackend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CliBackend")
            .field("op_bin", &self.op_bin)
            .field("token", &"<redacted>")
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

impl CliBackend {
    #[must_use]
    pub const fn new(
        op_bin: String,
        token: Zeroizing<String>,
        passthrough: Vec<(String, String)>,
        timeout: Duration,
    ) -> Self {
        Self {
            op_bin,
            token,
            passthrough,
            timeout,
        }
    }

    pub fn passthrough_from(env: &dyn Fn(&str) -> Option<String>) -> Vec<(String, String)> {
        PASSTHROUGH_VARS
            .iter()
            .filter_map(|name| env(name).map(|value| ((*name).to_owned(), value)))
            .collect()
    }

    async fn run(&self, args: &[&str]) -> Result<Vec<u8>, OnePasswordError> {
        let child = Command::new(&self.op_bin)
            .args(args)
            .env_clear()
            .envs(
                self.passthrough
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str())),
            )
            .env(TOKEN_VAR, self.token.as_str())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| OnePasswordError::Spawn(format!("{}: {e}", self.op_bin)))?;
        let output = tokio::time::timeout(self.timeout, child.wait_with_output())
            .await
            .map_err(|_| OnePasswordError::Timeout(self.timeout.as_secs()))?
            .map_err(|e| OnePasswordError::Spawn(e.to_string()))?;
        if !output.status.success() {
            return Err(OnePasswordError::cli(
                output.status.to_string(),
                &output.stderr,
            ));
        }
        Ok(output.stdout)
    }
}

#[async_trait]
impl Backend for CliBackend {
    fn label(&self) -> &'static str {
        "cli"
    }

    async fn fetch_items(
        &self,
        vault: &str,
        titles: &[String],
    ) -> Result<Vec<Item>, OnePasswordError> {
        let mut items = Vec::with_capacity(titles.len());
        for title in titles {
            let stdout = self
                .run(&[
                    "item", "get", title, "--vault", vault, "--format", "json", "--reveal",
                ])
                .await?;
            let item: Item = serde_json::from_slice(&stdout).map_err(|e| {
                OnePasswordError::InvalidResponse(format!("op item get '{title}': {e}"))
            })?;
            items.push(item);
        }
        Ok(items)
    }

    async fn read_reference(&self, reference: &OpReference) -> Result<String, OnePasswordError> {
        let stdout = self
            .run(&["read", "--no-newline", &reference.to_string()])
            .await?;
        let value = String::from_utf8(stdout).map_err(|_| {
            OnePasswordError::InvalidResponse(format!("op read {reference}: not UTF-8"))
        })?;
        Ok(value.trim_end_matches(['\r', '\n']).to_owned())
    }
}
