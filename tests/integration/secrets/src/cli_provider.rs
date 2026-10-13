//! The provider over the fake `op` CLI: an item read whole with `op item
//! get`, the per-reference `op read` fallback when the item cannot be parsed,
//! and a non-zero exit surfaced as an error naming the status.
//!
//! The CLI backend clears the child's environment and passes through only
//! the variables `op` needs, so `FAKE_OP_STORE` is handed to the fake through
//! [`CliBackend::new`]'s passthrough list rather than the config path.

use std::time::Duration;

use dynapps_test_common::{FakeConnect, FakeOpCli, skip_or_panic};
use serde_json::json;
use systemprompt_secrets_onepassword::cli::CliBackend;
use systemprompt_secrets_onepassword::{OnePasswordError, OnePasswordProvider};
use zeroize::Zeroizing;

use crate::support::{ITEM, TOKEN, VAULT, config, value_of};

fn cli_provider(cli: &FakeOpCli) -> OnePasswordProvider {
    let block = json!({
        "vault": VAULT,
        "items": {
            "anthropic": format!("op://{VAULT}/{ITEM}/anthropic"),
            "github_token": format!("op://{VAULT}/{ITEM}/github_token"),
        },
    });
    let cfg = config(&block);
    let backend = CliBackend::new(
        cli.op_path().display().to_string(),
        Zeroizing::new(TOKEN.to_owned()),
        cli.env(),
        Duration::from_secs(10),
    );
    OnePasswordProvider::with_backend(Box::new(backend), &cfg, None)
}

fn installed() -> Option<FakeOpCli> {
    if !FakeOpCli::available() {
        skip_or_panic("python3", "the fake op CLI is a python script");
        return None;
    }
    Some(FakeOpCli::install(TOKEN))
}

#[tokio::test]
async fn cli_reads_the_item_whole() {
    let Some(cli) = installed() else {
        return;
    };
    let connect = FakeConnect::start(TOKEN).await;
    let vault = connect.add_vault(VAULT);
    connect.add_item(
        &vault,
        ITEM,
        &[("anthropic", "sk-ant-cli"), ("github_token", "ghp-cli")],
    );
    cli.write_store(&connect);

    let doc = cli_provider(&cli).fetch().await.expect("op item get");
    assert_eq!(value_of(&doc, "anthropic").as_deref(), Some("sk-ant-cli"));
    assert_eq!(value_of(&doc, "github_token").as_deref(), Some("ghp-cli"));
}

// A numeric item id makes `op item get`'s JSON unparseable as an item, which
// is the fallback's trigger; `op read` still resolves each reference.
#[tokio::test]
async fn cli_falls_back_to_op_read_per_reference() {
    let Some(cli) = installed() else {
        return;
    };
    cli.write_vaults(json!([{
        "id": "vault-1", "name": VAULT,
        "items": [{
            "id": 42, "title": ITEM, "vault": { "id": "vault-1" },
            "fields": [
                { "id": "f1", "label": "anthropic", "type": "CONCEALED", "value": "sk-ant-read" },
                { "id": "f2", "label": "github_token", "type": "CONCEALED", "value": "ghp-read" },
            ],
        }],
    }]));

    let doc = cli_provider(&cli).fetch().await.expect("op read fallback");
    assert_eq!(value_of(&doc, "anthropic").as_deref(), Some("sk-ant-read"));
    assert_eq!(value_of(&doc, "github_token").as_deref(), Some("ghp-read"));
}

#[tokio::test]
async fn cli_surfaces_a_non_zero_exit() {
    let Some(cli) = installed() else {
        return;
    };
    cli.set_fail(6, "[ERROR] simulated outage");
    let err = cli_provider(&cli).fetch().await.expect_err("op failed");
    match &err {
        OnePasswordError::Cli {
            status,
            stderr_excerpt,
        } => {
            assert!(status.contains('6'), "{status}");
            assert!(
                stderr_excerpt.contains("simulated outage"),
                "{stderr_excerpt}"
            );
        },
        other => panic!("expected a Cli error, got {other:?}"),
    }
    assert!(!err.to_string().contains(TOKEN));
}

#[tokio::test]
async fn cli_with_the_wrong_token_is_refused() {
    let Some(cli) = installed() else {
        return;
    };
    let other = FakeOpCli::install("a-different-token");
    let connect = FakeConnect::start(TOKEN).await;
    let vault = connect.add_vault(VAULT);
    connect.add_item(&vault, ITEM, &[("anthropic", "x"), ("github_token", "y")]);
    other.write_store(&connect);
    drop(cli);
    let err = cli_provider(&other)
        .fetch()
        .await
        .expect_err("token mismatch");
    assert!(matches!(err, OnePasswordError::Cli { .. }), "{err:?}");
}
