//! Core's secrets bootstrap with `secrets.source: external, kind:
//! onepassword` over the fake Connect server: the first load, a reload that
//! swaps one changed key, a reload with nothing changed, and a reload that
//! would change a pinned key and is refused.
//!
//! Each test boots its own profile and secrets store. The token is read from
//! the process environment by name; the lint bar forbids `set_var`, so
//! `token_env` names `HOME`, which every runner has, and the fake is started
//! with its value.

use dynapps_test_common::{FakeConnect, repo_root};
use std::sync::Arc;

use systemprompt::config::{LoadedProfile, SecretsService};

use crate::support::{ITEM, VAULT, baseline_fields};

const FIXTURE_PROFILE: &str = include_str!("../../../contract/admin/fixtures/profile.yaml");
const FILE_SECRETS: &str =
    "secrets:\n  secrets_path: secrets.json\n  validation: warn\n  source: file\n";

struct Booted {
    connect: FakeConnect,
    vault: String,
    item: String,
    secrets: Arc<SecretsService>,
    _dir: tempfile::TempDir,
}

fn home() -> String {
    std::env::var("HOME").expect("HOME is set on every runner")
}

async fn boot() -> Booted {
    let connect = FakeConnect::start(&home()).await;
    let vault = connect.add_vault(VAULT);
    let mut fields: Vec<(String, String)> = baseline_fields()
        .into_iter()
        .map(|(k, v)| (k, v.as_str().expect("string").to_owned()))
        .collect();
    fields.push(("github_token".to_owned(), "ghp-generation-one".to_owned()));
    let borrowed: Vec<(&str, &str)> = fields
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let item = connect.add_item(&vault, ITEM, &borrowed);

    let dir = tempfile::tempdir().expect("profile dir");
    std::fs::create_dir_all(dir.path().join("bin")).expect("bin dir");
    let external = format!(
        "secrets:\n  validation: warn\n  source: external\n  external:\n    kind: onepassword\n    \
         backend: connect\n    connect_url: {}\n    token_env: HOME\n    vault: {VAULT}\n    \
         item: {ITEM}\n    retries: 0\n",
        connect.base_url()
    );
    assert!(
        FIXTURE_PROFILE.contains(FILE_SECRETS),
        "fixture profile secrets block moved"
    );
    let root = repo_root();
    let yaml = FIXTURE_PROFILE
        .replace(FILE_SECRETS, &external)
        .replace(
            "__PROFILE_DIR__/services",
            &root.join("services").to_string_lossy(),
        )
        .replace("__PROFILE_DIR__", &dir.path().to_string_lossy())
        .replace("__REPO__", &root.to_string_lossy());
    let profile = dir.path().join("profile.yaml");
    std::fs::write(&profile, yaml).expect("write profile");

    let profile = LoadedProfile::from_path(&profile).expect("profile with an external source");
    let secrets = SecretsService::load(profile)
        .await
        .expect("first load through 1Password");
    Booted {
        connect,
        vault,
        item,
        secrets,
        _dir: dir,
    }
}

fn current(booted: &Booted, key: &str) -> Option<String> {
    booted.secrets.get().get(key).cloned()
}

#[tokio::test(flavor = "multi_thread")]
async fn reload_swaps_a_changed_key_and_ignores_an_unchanged_store() {
    let booted = boot().await;
    let first = booted.secrets.generation();
    assert_eq!(
        current(&booted, "github_token").as_deref(),
        Some("ghp-generation-one")
    );

    let unchanged = booted.secrets.reload().await.expect("reload");
    assert!(!unchanged.swapped, "{unchanged:?}");
    assert!(unchanged.changed_keys.is_empty());
    assert_eq!(unchanged.generation, first);

    booted.connect.set_field(
        &booted.vault,
        &booted.item,
        "github_token",
        "ghp-generation-two",
    );
    let swapped = booted.secrets.reload().await.expect("reload");
    assert!(swapped.swapped, "{swapped:?}");
    assert_eq!(swapped.changed_keys, ["github_token"]);
    assert_eq!(swapped.generation, first + 1);
    assert_eq!(booted.secrets.generation(), first + 1);
    assert_eq!(
        current(&booted, "github_token").as_deref(),
        Some("ghp-generation-two")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn reload_refuses_a_changed_pinned_key_and_keeps_the_generation() {
    let booted = boot().await;
    let first = booted.secrets.generation();
    let before = current(&booted, "encryption_master_key");

    booted.connect.set_field(
        &booted.vault,
        &booted.item,
        "encryption_master_key",
        "ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100",
    );
    let err = booted
        .secrets
        .reload()
        .await
        .expect_err("a pinned key may not change");
    let message = err.to_string();
    assert!(message.contains("encryption_master_key"), "{message}");
    assert!(
        !message.contains("ffeeddcc"),
        "the refusal never echoes the value"
    );
    assert_eq!(
        booted.secrets.generation(),
        first,
        "the old generation stays"
    );
    assert_eq!(current(&booted, "encryption_master_key"), before);
    let status = booted.secrets.status();
    assert!(
        status.last_error.is_some(),
        "the refusal is recorded: {status:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn reload_failure_upstream_keeps_the_current_generation() {
    let booted = boot().await;
    let first = booted.secrets.generation();
    booted
        .connect
        .fail_next(4, axum::http::StatusCode::SERVICE_UNAVAILABLE);
    booted.secrets.reload().await.expect_err("Connect is down");
    assert_eq!(booted.secrets.generation(), first);
    assert_eq!(
        current(&booted, "github_token").as_deref(),
        Some("ghp-generation-one")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn reload_refuses_a_changed_signing_key_pem() {
    let booted = boot().await;
    booted.connect.set_field(
        &booted.vault,
        &booted.item,
        "signing_key_pem",
        "generation-one-pem",
    );
    let first = booted.secrets.generation();
    let err = booted
        .secrets
        .reload()
        .await
        .expect_err("signing_key_pem is pinned");
    assert!(err.to_string().contains("signing_key_pem"), "{err}");
    assert_eq!(booted.secrets.generation(), first);
    assert_eq!(
        current(&booted, "signing_key_pem"),
        None,
        "the refused value never goes live"
    );
}
