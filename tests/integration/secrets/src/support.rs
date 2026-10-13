//! Shared construction for the 1Password suites: a provider over a config
//! block, a token lookup that never touches the process environment, and the
//! vault every suite seeds.

use serde_json::{Value, json};
use systemprompt::config::SecretsDocument;
use systemprompt_secrets_onepassword::{OnePasswordConfig, OnePasswordError, OnePasswordProvider};

pub(crate) const TOKEN: &str = "fake-1password-token";
pub(crate) const VAULT: &str = "Dynapps";
pub(crate) const ITEM: &str = "dynapps-secrets";

pub(crate) fn config(block: &Value) -> OnePasswordConfig {
    let Value::Object(map) = block else {
        panic!("config block is not an object");
    };
    OnePasswordConfig::from_map(map).expect("valid onepassword block")
}

// A provider whose token lookup answers [`TOKEN`] for `token_env` alone.
pub(crate) fn provider(block: &Value) -> Result<OnePasswordProvider, OnePasswordError> {
    let cfg = config(block);
    let token_env = cfg.token_env.clone();
    OnePasswordProvider::from_config(cfg, None, &move |name: &str| {
        (name == token_env).then(|| TOKEN.to_owned())
    })
}

pub(crate) fn connect_block(url: &str) -> Value {
    json!({
        "backend": "connect",
        "connect_url": url,
        "vault": VAULT,
        "items": {
            "anthropic": format!("op://{VAULT}/{ITEM}/anthropic"),
            "github_token": format!("op://{VAULT}/{ITEM}/github_token"),
        },
        "timeout_secs": 2,
        "retries": 2,
    })
}

// The fields core requires of every secrets document, so a fetched
// document carrying only the mapped keys can be parsed and read back.
pub(crate) fn baseline_fields() -> serde_json::Map<String, Value> {
    let Value::Object(map) = json!({
        "database_url": "postgres://unused:unused@localhost:5432/postgres",
        "oauth_at_rest_pepper": "secrets-suite-pepper-not-a-real-secret",
        "mcp_credential_broker_secret": "secrets-suite-broker-not-a-real-secret",
        "manifest_signing_secret_seed": "c2VjcmV0cy1zdWl0ZS1zZWVkLW5vdC1yZWFsLTAwMDA=",
        "encryption_master_key": "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
    }) else {
        unreachable!("literal object");
    };
    map
}

// `key` of a fetched document, read through core's own parse.
pub(crate) fn value_of(doc: &SecretsDocument, key: &str) -> Option<String> {
    let mut doc = doc.clone();
    let present = doc.key_names();
    for (name, value) in baseline_fields() {
        if !present.contains(&name) {
            doc.merge_field(name, value);
        }
    }
    let secrets = doc.into_secrets().expect("document parses as secrets");
    secrets.get(key).cloned()
}
