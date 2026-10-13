//! `OnePasswordProvider`: what it says about itself, and a fetch over a stub
//! backend (the real backends are exercised against fakes in the integration
//! tier).

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{Value, json};
use systemprompt::config::DynSecretsProvider;
use systemprompt_secrets_onepassword::item::Item;
use systemprompt_secrets_onepassword::{
    Backend, OnePasswordConfig, OnePasswordError, OnePasswordProvider, OpReference,
};

struct Stub {
    items: Vec<Item>,
    fail_items: bool,
    reads: Mutex<Vec<String>>,
}

#[async_trait]
impl Backend for Stub {
    fn label(&self) -> &'static str {
        "stub"
    }

    async fn fetch_items(
        &self,
        _vault: &str,
        titles: &[String],
    ) -> Result<Vec<Item>, OnePasswordError> {
        if self.fail_items {
            return Err(OnePasswordError::cli(
                "exit status: 1",
                b"[ERROR] item not found",
            ));
        }
        Ok(self
            .items
            .iter()
            .filter(|i| titles.contains(&i.title))
            .cloned()
            .collect())
    }

    async fn read_reference(&self, reference: &OpReference) -> Result<String, OnePasswordError> {
        self.reads.lock().unwrap().push(reference.to_string());
        Ok("read-value".to_owned())
    }
}

fn config(block: Value) -> OnePasswordConfig {
    let Value::Object(map) = block else {
        panic!("not an object")
    };
    OnePasswordConfig::from_map(&map).unwrap()
}

fn items_config() -> OnePasswordConfig {
    config(json!({
        "vault": "dynapps-prod",
        "items": {
            "anthropic": "op://dynapps-prod/platform/anthropic",
            "gemini": "op://dynapps-prod/platform/gemini",
        },
    }))
}

fn platform() -> Item {
    serde_json::from_value(json!({
        "id": "p1",
        "title": "platform",
        "fields": [
            {"id": "a", "label": "anthropic", "type": "CONCEALED", "value": "sk-ant-secret"},
            {"id": "g", "label": "gemini", "type": "CONCEALED", "value": "gm-secret"},
        ],
    }))
    .unwrap()
}

fn stub(fail_items: bool) -> Box<dyn Backend> {
    Box::new(Stub {
        items: vec![platform()],
        fail_items,
        reads: Mutex::new(Vec::new()),
    })
}

#[test]
fn describe_names_backend_and_vault_only() {
    let provider = OnePasswordProvider::with_backend(stub(false), &items_config(), None);
    assert_eq!(provider.describe(), "onepassword stub vault dynapps-prod");
}

#[test]
fn refresh_hint_follows_refresh_secs() {
    let provider = OnePasswordProvider::with_backend(
        stub(false),
        &items_config(),
        Some(Duration::from_secs(300)),
    );
    assert_eq!(provider.refresh_hint(), Some(Duration::from_secs(300)));
    let none = OnePasswordProvider::with_backend(stub(false), &items_config(), None);
    assert_eq!(none.refresh_hint(), None);
}

#[test]
fn from_config_requires_the_token_from_env() {
    let empty = |_: &str| None::<String>;
    let err = OnePasswordProvider::from_config(items_config(), None, &empty)
        .err()
        .unwrap();
    assert!(
        matches!(err, OnePasswordError::TokenMissing { ref env } if env == "OP_SERVICE_ACCOUNT_TOKEN")
    );

    let blank = |_: &str| Some("  ".to_owned());
    assert!(OnePasswordProvider::from_config(items_config(), None, &blank).is_err());
}

#[test]
fn from_config_builds_cli_and_connect() {
    let env = |name: &str| (name.starts_with("OP_")).then(|| "ops_token_value".to_owned());
    let cli = OnePasswordProvider::from_config(items_config(), Some(120), &env).unwrap();
    assert_eq!(cli.describe(), "onepassword cli vault dynapps-prod");
    assert!(!cli.describe().contains("ops_token_value"));
    assert_eq!(cli.refresh_hint(), Some(Duration::from_secs(120)));

    let connect = config(json!({
        "backend": "connect",
        "connect_url": "http://127.0.0.1:1",
        "vault": "v",
        "item": "platform",
    }));
    let connect = OnePasswordProvider::from_config(connect, None, &env).unwrap();
    assert_eq!(connect.describe(), "onepassword connect vault v");
}

#[tokio::test]
async fn fetch_maps_each_key() {
    let provider = OnePasswordProvider::with_backend(stub(false), &items_config(), None);
    let doc = provider.fetch().await.unwrap();
    assert_eq!(doc.key_names(), vec!["anthropic", "gemini"]);
}

#[tokio::test]
async fn fetch_falls_back_to_reading_each_reference() {
    let provider = OnePasswordProvider::with_backend(stub(true), &items_config(), None);
    let doc = provider.fetch().await.unwrap();
    assert_eq!(doc.key_names(), vec!["anthropic", "gemini"]);
}

#[tokio::test]
async fn single_item_has_no_reference_fallback() {
    let cfg = config(json!({"vault": "dynapps-prod", "item": "platform"}));
    let provider = OnePasswordProvider::with_backend(stub(true), &cfg, None);
    assert!(matches!(
        provider.fetch().await,
        Err(OnePasswordError::Cli { .. })
    ));
}

#[tokio::test]
async fn fetch_boxed_maps_errors_to_external() {
    let cfg = config(json!({"vault": "dynapps-prod", "item": "platform"}));
    let provider = OnePasswordProvider::with_backend(stub(true), &cfg, None);
    let err = provider.fetch_boxed().await.unwrap_err();
    let rendered = format!("{err:?}");
    assert!(rendered.contains("onepassword"), "{rendered}");
}
