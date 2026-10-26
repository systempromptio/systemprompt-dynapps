//! Runs the `systemprompt` CLI on behalf of an MCP tool call.
//!
//! Models routinely append flags the CLI does not accept; those are stripped
//! before exec rather than surfaced as a usage error the model cannot act on.

use crate::tools::CliOutput;
use rmcp::ErrorData as McpError;
use std::path::PathBuf;
use systemprompt::manifest::Profile;
use tokio::process::Command;

/// Why a CLI invocation could not produce output.
///
/// rmcp's `ErrorData` is a variant-less wire type, so the typed cause lives
/// here and is projected onto the wire exactly once, in `From`.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("command arguments do not parse")]
    Arguments(#[from] shell_words::ParseError),
    #[error("CLI command did not finish within {}s; narrow the query", CLI_TIMEOUT.as_secs())]
    Timeout,
    #[error("CLI command could not be executed")]
    Spawn(#[source] Box<std::io::Error>),
}

impl From<std::io::Error> for CliError {
    fn from(error: std::io::Error) -> Self {
        Self::Spawn(Box::new(error))
    }
}

impl From<CliError> for McpError {
    fn from(error: CliError) -> Self {
        let message = match &error {
            CliError::Arguments(source) => format!("{error}: {source}"),
            CliError::Spawn(source) => format!("{error}: {source}"),
            CliError::Timeout => error.to_string(),
        };
        match error {
            CliError::Arguments(_) => Self::invalid_params(message, None),
            CliError::Timeout | CliError::Spawn(_) => Self::internal_error(message, None),
        }
    }
}

/// Where the CLI lives and what directory it runs in.
///
/// Resolved once by the caller and passed down, rather than read from the
/// environment at each call: the environment is process-global, so a test that
/// pointed it at a stand-in binary changed the binary every other test in the
/// process would spawn.
#[derive(Debug, Clone)]
pub struct CliLocation {
    pub bin: PathBuf,
    pub workdir: PathBuf,
}

impl CliLocation {
    #[must_use]
    pub fn from_profile(profile: &Profile) -> Self {
        Self {
            bin: PathBuf::from(&profile.paths.bin).join("systemprompt"),
            workdir: PathBuf::from(&profile.paths.system),
        }
    }
}

// Why: `#[doc(hidden)]` so the external test workspace can assert the filter
// set; `--export` writes a CSV on the server's disk no MCP client can read.
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

const CLI_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

pub(crate) async fn execute(
    location: &CliLocation,
    command: &str,
    auth_token: &str,
) -> Result<CliOutput, CliError> {
    let cli_path = &location.bin;
    let workdir = &location.workdir;

    let args = shell_words::split(command)?;

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
        .map_err(|_elapsed| CliError::Timeout)??;

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
