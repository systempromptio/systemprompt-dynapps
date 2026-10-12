//! Entry point for the Dynapps binary.
//!
//! Thin by design: every capability is registered at compile time by the
//! extension crates under `extensions/`, and this delegates to the published
//! `systemprompt` core runtime.

use systemprompt_dynapps as _;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Box::pin(systemprompt_dynapps::cli::run()).await
}
