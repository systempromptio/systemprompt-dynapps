//! `systemprompt-farm-worker` — the entrypoint of a farm runner machine.
//!
//! The API launches a pooled Fly Machine with this binary as its entrypoint
//! and three env vars: `RUN_ID`, `API_BASE` and `FARM_WORKER_TOKEN`. The
//! worker runs that one run (see [`run`]), reports its result and exits, and
//! the machine stops with it. It needs no profile, database or secrets: all
//! it knows arrives over the worker API, scoped to the one run.
//!
//! Whatever fails before the executor ran is still reported, as a
//! `host_crashed` result, so the API finishes the row at once instead of
//! waiting out its watchdog.

mod client;
mod egress;
mod privilege;
mod run;

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use systemprompt_web_admin_farm::services::runner::fly::protocol::{
    API_BASE_ENV, RUN_ID_ENV, WORKER_TOKEN_ENV,
};

use crate::client::WorkerClient;

#[derive(Debug, Parser)]
#[command(
    name = "systemprompt-farm-worker",
    about = "Run one agent farm run on this machine and report it to the API"
)]
struct Cli {
    #[arg(long, env = RUN_ID_ENV, help = "The agent_runs id to run")]
    run: String,
    #[arg(long, env = API_BASE_ENV, help = "The API base the worker endpoints live under")]
    api_base: String,
    #[arg(
        long,
        help = "Run directory to use instead of the API's own run path (paths are rebased)"
    )]
    root: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    systemprompt::logging::init_console_logging();
    let cli = Cli::parse();
    let token = std::env::var(WORKER_TOKEN_ENV)
        .with_context(|| format!("{WORKER_TOKEN_ENV} is not set"))?;
    let client = WorkerClient::new(&cli.api_base, &cli.run, &token)?;
    let result = match run::run(&client, cli.root).await {
        Ok(result) => result,
        Err(error) => {
            tracing::error!(
                error = format!("{error:#}"),
                "farm run failed before it finished"
            );
            run::crashed(&format!("worker: {error:#}"))
        },
    };
    let status = result.outcome.status;
    client
        .post("result", &result)
        .await
        .context("result not reported")?;
    tracing::info!(run = %cli.run, status = status.as_str(), "farm run reported");
    Ok(())
}
