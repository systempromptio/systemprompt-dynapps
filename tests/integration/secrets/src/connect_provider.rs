//! The provider over a loopback 1Password Connect server: the mapped keys
//! come back, a rate limit is retried, a refused token fails closed with no
//! retry, and a server slower than `timeout_secs` is a timeout.

use std::time::Duration;

use axum::http::StatusCode;
use dynapps_test_common::FakeConnect;
use serde_json::json;
use systemprompt::config::DynSecretsProvider as _;
use systemprompt_secrets_onepassword::OnePasswordError;

use crate::support::{ITEM, TOKEN, VAULT, connect_block, provider, value_of};

async fn seeded() -> FakeConnect {
    let connect = FakeConnect::start(TOKEN).await;
    let vault = connect.add_vault(VAULT);
    connect.add_item(
        &vault,
        ITEM,
        &[
            ("anthropic", "sk-ant-from-connect"),
            ("github_token", "ghp-from-connect"),
        ],
    );
    connect
}

#[tokio::test]
async fn connect_resolves_every_mapped_key_with_the_bearer_token() {
    let connect = seeded().await;
    let provider = provider(&connect_block(connect.base_url())).expect("provider");
    let doc = provider.fetch().await.expect("fetch");

    let mut keys = doc.key_names();
    keys.sort();
    assert_eq!(keys, ["anthropic", "github_token"]);
    assert_eq!(
        value_of(&doc, "anthropic").as_deref(),
        Some("sk-ant-from-connect")
    );
    assert_eq!(
        value_of(&doc, "github_token").as_deref(),
        Some("ghp-from-connect")
    );
    assert!(
        connect
            .requests()
            .iter()
            .all(|r| r.authorization.as_deref() == Some(&format!("Bearer {TOKEN}"))),
        "every request carries the token as a bearer credential"
    );
    assert!(
        provider.describe().contains("connect"),
        "{}",
        provider.describe()
    );
    assert!(
        !provider.describe().contains(TOKEN),
        "describe never names the token"
    );
}

#[tokio::test]
async fn connect_retries_a_rate_limit_and_then_succeeds() {
    let connect = seeded().await;
    connect.rate_limit_once();
    let provider = provider(&connect_block(connect.base_url())).expect("provider");
    let doc = provider.fetch_boxed().await.expect("the 429 is retried");
    assert_eq!(
        value_of(&doc, "anthropic").as_deref(),
        Some("sk-ant-from-connect")
    );
    let first = &connect.requests()[..2];
    assert_eq!(
        first[0].path, first[1].path,
        "the rate-limited request is re-sent"
    );
}

#[tokio::test]
async fn connect_fails_closed_on_a_refused_token() {
    let connect = seeded().await;
    connect.fail_next(1, StatusCode::UNAUTHORIZED);
    let provider = provider(&connect_block(connect.base_url())).expect("provider");
    let err = provider
        .fetch()
        .await
        .expect_err("a 401 is not retried into success");
    assert!(
        matches!(err, OnePasswordError::Status { code: 401, .. }),
        "{err:?}"
    );
    assert_eq!(connect.requests().len(), 1, "a 401 is fatal, not retried");
    assert!(
        !err.to_string().contains(TOKEN),
        "the error never names the token"
    );
}

#[tokio::test]
async fn connect_wrong_token_is_refused_by_the_server() {
    let connect = seeded().await;
    let mut block = connect_block(connect.base_url());
    block["token_env"] = json!("SOME_OTHER_TOKEN_VAR");
    let cfg = crate::support::config(&block);
    let provider = systemprompt_secrets_onepassword::OnePasswordProvider::from_config(
        &cfg,
        None,
        &|name: &str| (name == "SOME_OTHER_TOKEN_VAR").then(|| "not-the-token".to_owned()),
    )
    .expect("provider");
    let err = provider
        .fetch()
        .await
        .expect_err("an unknown token is refused");
    assert!(
        matches!(err, OnePasswordError::Status { code: 401, .. }),
        "{err:?}"
    );
}

#[tokio::test]
async fn connect_times_out_against_a_slow_server() {
    let connect = seeded().await;
    connect.delay(Duration::from_secs(3));
    let mut block = connect_block(connect.base_url());
    block["timeout_secs"] = json!(1);
    block["retries"] = json!(0);
    let provider = provider(&block).expect("provider");
    let err = provider
        .fetch()
        .await
        .expect_err("slower than timeout_secs");
    assert!(matches!(err, OnePasswordError::Timeout(1)), "{err:?}");
}

#[tokio::test]
async fn connect_reports_a_missing_vault_by_name() {
    let connect = FakeConnect::start(TOKEN).await;
    let provider = provider(&connect_block(connect.base_url())).expect("provider");
    let err = provider.fetch().await.expect_err("no vault");
    assert!(err.to_string().contains(VAULT), "{err}");
}
