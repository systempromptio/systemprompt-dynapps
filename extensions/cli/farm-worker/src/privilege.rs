//! The worker's two identities: root while it installs the egress rules,
//! then [`AGENT_USER`] for good before it writes the run directory or starts
//! the agent.
//!
//! The image installs the worker setuid root and executable only by the
//! `farmrun` group (the `app` server user is in it, the agent user is not),
//! so the worker starts with an effective uid of 0 whichever user Fly
//! launches it as. [`become_root`] makes that the real uid too, so `nft`
//! runs as root. [`drop_to_agent`] then sets every uid and gid to the
//! agent's and clears the supplementary groups; nothing after it, and
//! nothing the agent spawns, can regain root or touch the ruleset.

use std::path::Path;

use anyhow::{Context, Result, bail};
use nix::unistd::{Uid, User, geteuid, setgroups, setresgid, setresuid};

pub(crate) const AGENT_USER: &str = "farm-agent";

pub(crate) fn become_root() -> Result<()> {
    if !geteuid().is_root() {
        bail!("worker is not running with root privileges; egress rules cannot be installed");
    }
    setresuid(Uid::from_raw(0), Uid::from_raw(0), Uid::from_raw(0)).context("setresuid(0)")
}

pub(crate) fn drop_to_agent(root: &Path) -> Result<()> {
    let user = User::from_name(AGENT_USER)
        .context("user database")?
        .with_context(|| format!("user {AGENT_USER} does not exist"))?;
    std::fs::create_dir_all(root).with_context(|| format!("create {}", root.display()))?;
    std::os::unix::fs::chown(root, Some(user.uid.as_raw()), Some(user.gid.as_raw()))
        .with_context(|| format!("chown {}", root.display()))?;
    setgroups(&[]).context("setgroups")?;
    setresgid(user.gid, user.gid, user.gid).context("setresgid")?;
    setresuid(user.uid, user.uid, user.uid).context("setresuid")?;
    if setresuid(Uid::from_raw(0), Uid::from_raw(0), Uid::from_raw(0)).is_ok() {
        bail!("root could be regained after dropping to {AGENT_USER}");
    }
    tracing::info!(
        user = AGENT_USER,
        uid = user.uid.as_raw(),
        "privileges dropped"
    );
    Ok(())
}
