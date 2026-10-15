//! Installs the run's outbound allowlist before the agent starts.
//!
//! The spec's [`EgressPolicy`] names hosts, addresses and blocks; host names
//! are resolved here, once, and every address they return is allowed. The
//! ruleset is loaded with `nft -f -` while the worker still holds root, and
//! a self-test then expects a connection to [`PROBE_HOST`] to be refused.
//! Any failure is an error, so the run is reported `host_crashed` and the
//! agent never runs unconfined.

use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use systemprompt_web_admin_farm::services::runner::fly::egress::{EgressPolicy, Net, nft_ruleset};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, lookup_host};
use tokio::process::Command;

const NFT: &str = "nft";
const PROBE_HOST: &str = "example.com";
const PROBE_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) async fn install(policy: &EgressPolicy) -> Result<()> {
    let nets = resolve(policy).await?;
    load(&nft_ruleset(&nets)).await?;
    tracing::info!(
        egress_policy = %policy.label(),
        addresses = nets.len(),
        "egress allowlist installed"
    );
    self_test().await
}

async fn resolve(policy: &EgressPolicy) -> Result<Vec<Net>> {
    let mut nets = Vec::new();
    for entry in &policy.allow {
        if let Some(net) = Net::parse(entry) {
            nets.push(net);
            continue;
        }
        let addrs = lookup_host((entry.as_str(), 443))
            .await
            .with_context(|| format!("egress host {entry} did not resolve"))?;
        nets.extend(addrs.map(|addr| Net::host(addr.ip())));
    }
    nets.sort_by_key(|net| (net.addr, net.prefix));
    nets.dedup();
    Ok(nets)
}

async fn load(ruleset: &str) -> Result<()> {
    let mut child = Command::new(NFT)
        .args(["-f", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("nft could not be started")?;
    let mut stdin = child.stdin.take().context("nft stdin")?;
    stdin
        .write_all(ruleset.as_bytes())
        .await
        .context("nft ruleset not written")?;
    drop(stdin);
    let output = child
        .wait_with_output()
        .await
        .context("nft did not finish")?;
    if !output.status.success() {
        bail!(
            "nft refused the egress ruleset ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

async fn self_test() -> Result<()> {
    let Ok(mut addrs) = lookup_host((PROBE_HOST, 443)).await else {
        tracing::info!(
            probe = PROBE_HOST,
            "egress self-test: probe host unresolvable"
        );
        return Ok(());
    };
    let Some(addr) = addrs.next() else {
        return Ok(());
    };
    match tokio::time::timeout(PROBE_TIMEOUT, TcpStream::connect(addr)).await {
        Ok(Ok(_)) => bail!("egress self-test: {PROBE_HOST} ({addr}) is reachable"),
        Ok(Err(error)) => {
            tracing::info!(probe = PROBE_HOST, %addr, %error, "egress self-test: refused");
            Ok(())
        },
        Err(_) => {
            tracing::info!(probe = PROBE_HOST, %addr, "egress self-test: timed out");
            Ok(())
        },
    }
}
