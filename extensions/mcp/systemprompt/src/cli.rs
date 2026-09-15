//! Runs the `systemprompt` CLI on behalf of an MCP tool call.
//!
//! Models routinely append flags the CLI does not accept; those are stripped
//! before exec rather than surfaced as a usage error the model cannot act on.

use crate::tools::CliOutput;
use rmcp::ErrorData as McpError;
use std::path::PathBuf;
use systemprompt::config::ProfileBootstrap;
use tokio::process::Command;

/// Where the CLI lives and what directory it runs in.
///
/// Resolved once by the caller and passed down, rather than read from the
/// environment at each call: the environment is process-global, so a test that
/// pointed it at a stand-in binary changed the binary every other test in the
/// process would spawn.
#[derive(Debug)]
pub struct CliLocation {
    pub bin: PathBuf,
    pub workdir: PathBuf,
}

impl CliLocation {
    pub fn from_profile() -> Result<Self, McpError> {
        let profile = ProfileBootstrap::get()
            // Why: lint-ok: error-adapt — rmcp's ErrorData is a variant-less wire type
            .map_err(|e| McpError::internal_error(format!("Failed to get profile: {e}"), None))?;

        Ok(Self {
            bin: PathBuf::from(&profile.paths.bin).join("systemprompt"),
            workdir: PathBuf::from(&profile.paths.system),
        })
    }
}

// Why: Strip CLI flags that models routinely hallucinate onto `systemprompt`
// invocations: output-format toggles the gateway sets itself, and `--export`,
// which writes a CSV on the server's disk where no MCP client can read it,
// and takes a path argument the model would have to invent. Exposed behind
// `#[doc(hidden)]` so the external test workspace can assert the filter set;
// not part of the public API.
#[doc(hidden)]
pub fn filter_hallucinated_args(args: Vec<String>) -> Vec<String> {
    const HALLUCINATED_ARGS: &[&str] = &["--json", "--output-format", "--format"];
    const PATH_TAKING_ARGS: &[&str] = &["--export"];

    let mut out = Vec::with_capacity(args.len());
    let mut skip_value = false;
    for arg in args {
        if skip_value {
            skip_value = false;
            if !arg.starts_with('-') {
                continue;
            }
        }
        if HALLUCINATED_ARGS.contains(&arg.as_str()) {
            continue;
        }
        if PATH_TAKING_ARGS.contains(&arg.as_str()) {
            skip_value = true;
            continue;
        }
        if PATH_TAKING_ARGS.iter().any(|flag| {
            arg.strip_prefix(flag)
                .is_some_and(|rest| rest.starts_with('='))
        }) {
            continue;
        }
        out.push(arg);
    }
    out
}

// Why: a CLI call that never returns would hang the client's tool call; a
// bounded failure names the remedy (narrow the query) instead.
const CLI_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

pub(crate) async fn execute(
    location: &CliLocation,
    command: &str,
    auth_token: &str,
) -> Result<CliOutput, McpError> {
    let cli_path = &location.bin;
    let workdir = &location.workdir;

    // Why: lint-ok: error-adapt — rmcp's ErrorData is a variant-less wire type
    let args = shell_words::split(command).map_err(|e| {
        McpError::invalid_params(format!("Failed to parse command arguments: {e}"), None)
    })?;

    let args = filter_hallucinated_args(args);

    tracing::info!(
        cli_path = %cli_path.display(),
        workdir = %workdir.display(),
        args = ?args,
        "Executing CLI command"
    );

    let spawned = Command::new(cli_path)
        .kill_on_drop(true)
        .args(&args)
        .env("SYSTEMPROMPT_NON_INTERACTIVE", "1")
        .env("SYSTEMPROMPT_OUTPUT_FORMAT", "json")
        .env("SYSTEMPROMPT_AUTH_TOKEN", auth_token)
        .current_dir(workdir)
        .output();
    let output = tokio::time::timeout(CLI_TIMEOUT, spawned)
        .await
        // Why: lint-ok: error-adapt — rmcp's ErrorData is a variant-less wire type
        .map_err(|_elapsed| {
            McpError::internal_error(
                format!(
                    "CLI command did not finish within {}s; narrow the query",
                    CLI_TIMEOUT.as_secs()
                ),
                None,
            )
        })?
        // Why: lint-ok: error-adapt — rmcp's ErrorData is a variant-less wire type
        .map_err(|e| {
            McpError::internal_error(format!("Failed to execute CLI command: {e}"), None)
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success();

    tracing::info!(
        exit_code = exit_code,
        success = success,
        stdout_len = stdout.len(),
        stderr_len = stderr.len(),
        "CLI command completed"
    );

    Ok(CliOutput {
        stdout,
        stderr,
        exit_code,
        success,
    })
}
