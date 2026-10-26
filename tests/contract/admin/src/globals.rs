//! Process-wide globals the admin handlers read, installed once per test
//! binary.
//!
//! Core's config, profile and secrets are `OnceLock`s, so they, the services
//! view the routers are handed and the test's signing keys are installed
//! behind a `Once` and every token minted in
//! [`crate::principal`] validates against the same issuer. The profile is a
//! checked-in fixture: the developer's `.systemprompt` profile is git-ignored,
//! so relying on it would pass locally and silently skip in CI.

use std::path::PathBuf;
use std::sync::{Once, OnceLock};

use systemprompt::manifest::services::ServicesView;
use systemprompt_security::keys::SigningKeys;

static INIT: Once = Once::new();
static READY: OnceLock<bool> = OnceLock::new();
static KEYS: OnceLock<SigningKeys> = OnceLock::new();
static SERVICES: OnceLock<ServicesView> = OnceLock::new();

const ISSUER: &str = "http://localhost:8099";

pub(crate) use dynapps_test_common::repo_root;

pub(crate) fn init() -> bool {
    INIT.call_once(|| {
        let ready = try_init();
        READY
            .set(ready)
            .expect("call_once runs this exactly once, so READY is unset");
    });
    *READY.get().unwrap_or(&false)
}

fn try_init() -> bool {
    let profile = write_fixture_profile();

    systemprompt::config::ProfileBootstrap::init_from_path(&profile)
        .expect("initialise the contract fixture profile");
    // Why: tokio panics on `block_on` inside a `#[tokio::test]` runtime, so the
    // async secrets bootstrap runs on a throwaway runtime on its own thread.
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("runtime for the secrets bootstrap")
                    .block_on(systemprompt::config::SecretsBootstrap::try_init())
                    .expect("load the fixture profile's secrets");
            })
            .join()
            .expect("the secrets bootstrap thread did not panic");
    });
    systemprompt::config::try_init_config(None).expect("build config from the fixture profile");
    let state = systemprompt::loader::ServicesState::load()
        .expect("load the services tree the fixture profile points at");
    SERVICES
        .set(state.view())
        .expect("try_init runs once, so SERVICES is unset");

    KEYS.set(dynapps_test_common::ephemeral_signing_keys())
        .expect("try_init runs once, so KEYS is unset");
    true
}

pub(crate) fn services() -> &'static ServicesView {
    SERVICES
        .get()
        .expect("globals::init installs the services view")
}

pub(crate) fn signing_keys() -> &'static SigningKeys {
    KEYS.get().expect("globals::init installs the signing keys")
}

const FIXTURE_PROFILE: &str = include_str!("../fixtures/profile.yaml");

// `anthropic` must exist: core's services loader demotes a provider with no
// credential to `surface: backend`, which hides both fixture routes from the
// per-user catalog before any ACL rule is consulted.
fn fixture_secrets() -> String {
    let database_url = std::env::var("SYSTEMPROMPT_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://unused:unused@localhost:5432/postgres".to_owned());
    format!(
        r#"{{
  "database_url": "{database_url}",
  "oauth_at_rest_pepper": "contract-suite-pepper-not-a-real-secret",
  "mcp_credential_broker_secret": "contract-suite-broker-not-a-real-secret",
  "manifest_signing_secret_seed": "Y29udHJhY3Qtc3VpdGUtc2VlZC1ub3QtcmVhbC0wMDA=",
  "encryption_master_key": "{MASTER_KEY_HEX}",
  "anthropic": "contract-suite-anthropic-key-not-a-real-secret"
}}"#
    )
}

// `secret_crypto::load_master_key` refuses anything but exactly 32 bytes.
const MASTER_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

// Profile path validation requires `paths.bin` to exist; a runner that built
// only this workspace has no `target/debug`, so an empty `bin/` stands in.
fn write_fixture_profile() -> PathBuf {
    let root = repo_root();
    // Why: the tree is rewritten on every init; two concurrent runs sharing a
    // path race into "No such file or directory" mid-copy.
    let dir = root.join(format!(
        "tests/target/contract-profile-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(dir.join("bin")).expect("create fixture profile and bin directories");

    let yaml = FIXTURE_PROFILE
        .replace("__PROFILE_DIR__", &dir.to_string_lossy())
        .replace("__REPO__", &root.to_string_lossy())
        .replace(
            "jwt_issuer: http://localhost:8099",
            &format!("jwt_issuer: {ISSUER}"),
        );
    std::fs::write(dir.join("profile.yaml"), yaml).expect("write fixture profile");
    std::fs::write(dir.join("secrets.json"), fixture_secrets()).expect("write fixture secrets");
    write_fixture_services(&root, &dir.join("services"));

    dir.join("profile.yaml")
}

// One Anthropic endpoint serving the fixture models, and two routes with fixed
// ids and no default model, so an unrouted model exists for the ACL detector
// cases; the rest of `services/` is copied as-is.
const FIXTURE_CATALOG: [(&str, &str); 5] = [
    (
        "credentials.yaml",
        include_str!("../fixtures/credentials.yaml"),
    ),
    ("providers.yaml", include_str!("../fixtures/providers.yaml")),
    ("models.yaml", include_str!("../fixtures/models.yaml")),
    (
        "deployments.yaml",
        include_str!("../fixtures/deployments.yaml"),
    ),
    ("gateway.yaml", include_str!("../fixtures/gateway.yaml")),
];
// The instance ships no identity-backed server; the accessor routes under
// /api/public/identity/ are exercised against this fixture instead.
const FIXTURE_IDENTITY_SERVER: &str = include_str!("../fixtures/sfcore_dev.yaml");
const IDENTITY_SERVER_INCLUDE: &str = "  - ../mcp/sfcore_dev.yaml\n";

fn write_fixture_services(root: &std::path::Path, dest: &std::path::Path) {
    if dest.exists() {
        std::fs::remove_dir_all(dest).expect("clear the previous fixture services tree");
    }
    copy_tree(&root.join("services"), dest);
    for (file, yaml) in FIXTURE_CATALOG {
        std::fs::write(dest.join("ai").join(file), yaml).expect("write fixture catalog file");
    }
    add_identity_server(dest);
}

fn add_identity_server(dest: &std::path::Path) {
    std::fs::write(dest.join("mcp/sfcore_dev.yaml"), FIXTURE_IDENTITY_SERVER)
        .expect("write fixture sfcore_dev.yaml");
    let config_path = dest.join("config/config.yaml");
    let config = std::fs::read_to_string(&config_path).expect("read fixture config.yaml");
    let anchor = "  - ../mcp/dynbrain.yaml\n";
    assert!(
        config.contains(anchor),
        "services/config/config.yaml must include ../mcp/dynbrain.yaml"
    );
    std::fs::write(
        &config_path,
        config.replacen(anchor, &format!("{anchor}{IDENTITY_SERVER_INCLUDE}"), 1),
    )
    .expect("include fixture sfcore_dev.yaml");
}

fn copy_tree(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).expect("create fixture services directory");
    for entry in std::fs::read_dir(from).expect("read services directory") {
        let entry = entry.expect("read services entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy services file");
        }
    }
}

pub(crate) fn jwt_issuer() -> String {
    systemprompt::manifest::Config::get()
        .expect("config installed by init()")
        .jwt_issuer
        .clone()
}
