//! The process-wide profile, secrets and signing authority the SSO driver
//! reads, installed once per test process.
//!
//! Core's config, secrets and signing authority are `OnceLock`s, which is why
//! every SSO test depends on nextest running each test in its own process.
//! The profile reuses the contract suite's checked-in fixture over a
//! per-process services tree: every entry of the repository's `services/` is
//! linked in, except `web/config`, a real directory the test writes its
//! `entra-oidc.yaml` into before the SSO driver discovers providers. The
//! secrets file carries the fake OIDC `IdP`'s client secret under the key the
//! provider resolves it by.

use std::path::PathBuf;
use std::sync::{Once, OnceLock};

use dynapps_test_common::repo_root;

static INIT: Once = Once::new();
static READY: OnceLock<bool> = OnceLock::new();

const FIXTURE_PROFILE: &str = include_str!("../../../../contract/admin/fixtures/profile.yaml");
// `secret_crypto::load_master_key` refuses anything but exactly 32 bytes.
const MASTER_KEY_HEX: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

// The tenant every OIDC test's fake `IdP` serves, so the client secret in
// the fixture secrets file matches the fake's.
pub(crate) const OIDC_TENANT: &str = "00000000-0000-4000-8000-0000000000aa";
pub(crate) const OIDC_SECRET_KEY: &str = "entra_oidc_client_secret";
// A secrets key whose value the fake token endpoint refuses.
pub(crate) const OIDC_WRONG_SECRET_KEY: &str = "entra_oidc_wrong_secret";
pub(crate) const EXTERNAL_URL: &str = "http://localhost:8099";

pub(crate) fn init() -> bool {
    INIT.call_once(|| {
        READY
            .set(try_init())
            .expect("call_once runs this exactly once");
    });
    *READY.get().unwrap_or(&false)
}

fn try_init() -> bool {
    let profile = write_profile();
    systemprompt::config::ProfileBootstrap::init_from_path(&profile)
        .expect("initialise the SSO fixture profile");
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
                    .expect("load the fixture secrets");
            })
            .join()
            .expect("the secrets bootstrap thread did not panic");
    });
    systemprompt::config::try_init_config(None).expect("build config from the fixture profile");
    let key = systemprompt_security::keys::RsaSigningKey::generate().expect("ephemeral RSA key");
    systemprompt_security::keys::authority::install(key).expect("install signing authority");
    true
}

fn secrets() -> String {
    let database_url = std::env::var("SYSTEMPROMPT_TEST_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .unwrap_or_else(|_| "postgres://unused:unused@localhost:5432/postgres".to_owned());
    serde_json::json!({
        "database_url": database_url,
        "oauth_at_rest_pepper": "sso-suite-pepper-not-a-real-secret",
        "mcp_credential_broker_secret": "sso-suite-broker-not-a-real-secret",
        "manifest_signing_secret_seed": "c3NvLXN1aXRlLXNlZWQtbm90LXJlYWwtMDAwMDAwMDA=",
        "encryption_master_key": MASTER_KEY_HEX,
        "anthropic": "sso-suite-anthropic-key-not-a-real-secret",
        OIDC_SECRET_KEY: format!("fake-oidc-secret-{OIDC_TENANT}"),
        OIDC_WRONG_SECRET_KEY: "not-the-secret-the-idp-holds",
    })
    .to_string()
}

fn profile_dir() -> PathBuf {
    repo_root().join(format!("tests/target/sso-profile-{}", std::process::id()))
}

pub(crate) fn services_web_config_dir() -> PathBuf {
    profile_dir().join("services/web/config")
}

fn link_entries(from: &std::path::Path, into: &std::path::Path, except: &str) {
    std::fs::create_dir_all(into).expect("create linked services directory");
    for entry in std::fs::read_dir(from).expect("read repository services") {
        let entry = entry.expect("services entry");
        let name = entry.file_name();
        if name == except {
            continue;
        }
        let target = into.join(&name);
        if !target.exists() {
            std::os::unix::fs::symlink(entry.path(), &target).expect("link services entry");
        }
    }
}

// Profile path validation requires `paths.bin` to exist, so an empty `bin/`
// stands in.
fn write_profile() -> PathBuf {
    let root = repo_root();
    let dir = profile_dir();
    std::fs::create_dir_all(dir.join("bin")).expect("create fixture profile directories");
    let services = dir.join("services");
    link_entries(&root.join("services"), &services, "web");
    link_entries(&root.join("services/web"), &services.join("web"), "config");
    link_entries(
        &root.join("services/web/config"),
        &services.join("web/config"),
        "entra-oidc.yaml",
    );
    let yaml = FIXTURE_PROFILE
        .replace("__PROFILE_DIR__", &dir.to_string_lossy())
        .replace("__REPO__", &root.to_string_lossy());
    std::fs::write(dir.join("profile.yaml"), yaml).expect("write fixture profile");
    std::fs::write(dir.join("secrets.json"), secrets()).expect("write fixture secrets");
    dir.join("profile.yaml")
}
