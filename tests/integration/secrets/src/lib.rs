//! The 1Password secrets source end to end: the provider against a fake
//! Connect server and a fake `op` CLI, and core's secrets bootstrap loading
//! and reloading through it from a profile with `source: external`.
//!
//! The bootstrap suites install process-wide `OnceLock`s (profile, secrets
//! store), so they rely on nextest running each test in its own process.

#[cfg(test)]
mod support;

#[cfg(test)]
mod connect_provider;

#[cfg(test)]
mod cli_provider;

#[cfg(test)]
mod bootstrap_reload;
