//! `OnePasswordConfig` parsed from the keys core leaves in
//! `ExternalSecretsConfig::config`.

use serde_json::{Map, Value, json};
use systemprompt::config::ExternalSecretsConfig;
use systemprompt_secrets_onepassword::config::{
    DEFAULT_CLI_TOKEN_ENV, DEFAULT_CONNECT_TOKEN_ENV, DEFAULT_OP_BIN, DEFAULT_RETRIES,
    DEFAULT_TIMEOUT_SECS, MAX_RETRIES, MAX_TIMEOUT_SECS,
};
use systemprompt_secrets_onepassword::{BackendKind, Mapping, OnePasswordConfig, OnePasswordError};

fn map(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(m) => m,
        _ => panic!("not an object"),
    }
}

fn parse(value: Value) -> Result<OnePasswordConfig, OnePasswordError> {
    OnePasswordConfig::from_map(&map(value))
}

fn items_block() -> Value {
    json!({
        "vault": "dynapps-prod",
        "items": {
            "anthropic": "op://dynapps-prod/platform/anthropic",
            "encryption_master_key": "op://dynapps-prod/identity/encryption_master_key",
        },
    })
}

#[test]
fn defaults_to_cli() {
    let cfg = parse(items_block()).unwrap();
    assert_eq!(cfg.backend, BackendKind::Cli);
    assert_eq!(cfg.op_bin, DEFAULT_OP_BIN);
    assert_eq!(cfg.token_env, DEFAULT_CLI_TOKEN_ENV);
    assert_eq!(cfg.timeout_secs, DEFAULT_TIMEOUT_SECS);
    assert_eq!(cfg.retries, DEFAULT_RETRIES);
    assert_eq!(cfg.connect_url, None);
    let Mapping::Items(items) = cfg.mapping else {
        panic!("expected items mapping");
    };
    assert_eq!(items.len(), 2);
    assert_eq!(items["anthropic"].item, "platform");
}

#[test]
fn parses_from_external_config_flattened_keys() {
    let ext: ExternalSecretsConfig = serde_json::from_value(json!({
        "kind": "onepassword",
        "refresh_secs": 300,
        "backend": "cli",
        "vault": "dynapps-prod",
        "items": { "anthropic": "op://dynapps-prod/platform/anthropic" },
    }))
    .unwrap();
    let cfg = OnePasswordConfig::from_external(&ext).unwrap();
    assert_eq!(cfg.vault, "dynapps-prod");
    assert_eq!(ext.refresh_secs, Some(300));
}

#[test]
fn connect_defaults_its_own_token_env_and_trims_url() {
    let cfg = parse(json!({
        "backend": "connect",
        "connect_url": "https://op-connect.internal:8443/",
        "vault": "v",
        "item": "platform",
    }))
    .unwrap();
    assert_eq!(cfg.backend, BackendKind::Connect);
    assert_eq!(cfg.token_env, DEFAULT_CONNECT_TOKEN_ENV);
    assert_eq!(
        cfg.connect_url.as_deref(),
        Some("https://op-connect.internal:8443")
    );
    assert_eq!(
        cfg.mapping,
        Mapping::SingleItem {
            item: "platform".to_owned()
        }
    );
}

#[test]
fn token_key_is_refused() {
    for key in ["token", "connect_token"] {
        let mut block = map(items_block());
        block.insert(key.to_owned(), json!("ops_secret"));
        let err = OnePasswordConfig::from_map(&block).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains(key), "{msg}");
        assert!(!msg.contains("ops_secret"), "{msg}");
    }
}

#[test]
fn unknown_key_is_refused() {
    let mut block = map(items_block());
    block.insert("vualt".to_owned(), json!("x"));
    assert!(OnePasswordConfig::from_map(&block).is_err());
}

#[test]
fn connect_requires_url_and_cli_refuses_one() {
    assert!(parse(json!({"backend": "connect", "vault": "v", "item": "i"})).is_err());
    assert!(
        parse(json!({"backend": "cli", "connect_url": "https://x", "vault": "v", "item": "i"}))
            .is_err()
    );
    assert!(
        parse(json!({"backend": "connect", "connect_url": "ftp://x", "vault": "v", "item": "i"}))
            .is_err()
    );
}

#[test]
fn exactly_one_mapping_form() {
    assert!(parse(json!({"vault": "v"})).is_err());
    assert!(parse(json!({"vault": "v", "item": "i", "items": {"k": "op://v/i/f"}})).is_err());
    assert!(parse(json!({"vault": "v", "items": {}})).is_err());
    assert!(parse(json!({"vault": "v", "items": {"k": "not-a-ref"}})).is_err());
}

#[test]
fn timeout_bounds() {
    let with = |t: u64| parse(json!({"vault": "v", "item": "i", "timeout_secs": t}));
    assert!(with(0).is_err());
    assert!(with(1).is_ok());
    assert!(with(MAX_TIMEOUT_SECS).is_ok());
    assert!(with(MAX_TIMEOUT_SECS + 1).is_err());
}

#[test]
fn retries_bounds() {
    let with = |r: u32| parse(json!({"vault": "v", "item": "i", "retries": r}));
    assert!(with(0).is_ok());
    assert!(with(MAX_RETRIES).is_ok());
    assert!(with(MAX_RETRIES + 1).is_err());
}

#[test]
fn empty_vault_and_token_env_are_refused() {
    assert!(parse(json!({"vault": " ", "item": "i"})).is_err());
    assert!(parse(json!({"vault": "v", "item": "i", "token_env": ""})).is_err());
}
