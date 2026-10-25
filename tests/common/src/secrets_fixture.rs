//! A bootstrapped profile and secrets store for suites that read secrets.
//!
//! It carries an `encryption_master_key` for the at-rest sealing path
//! (`secret_crypto`), and optionally a signing key and `Config` for suites
//! that build the web router.
//!
//! The key is served the way production reads it, through core's
//! `SecretsBootstrap`; nothing is placed in the process environment. Both
//! bootstraps are process-wide, so the first call installs and later calls
//! in the same process reuse the installed store.

use std::path::PathBuf;

pub const MASTER_KEY_HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

const FIXTURE_PROFILE: &str = include_str!("../../contract/admin/fixtures/profile.yaml");

fn secrets(signing_key_pem: Option<&str>) -> String {
    serde_json::json!({
        "database_url": "postgres://unused:unused@localhost:5432/postgres",
        "oauth_at_rest_pepper": "secrets-fixture-pepper-not-a-real-secret",
        "manifest_signing_secret_seed": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "encryption_master_key": MASTER_KEY_HEX,
        "signing_key_pem": signing_key_pem,
    })
    .to_string()
}

pub async fn bootstrap_master_key() -> tempfile::TempDir {
    bootstrap(None).await
}

// Why: the web router resolves its signing keys from the process-wide
// `Config` and `SecretsBootstrap`, so a suite that builds it needs both.
pub async fn bootstrap_signing_config() -> tempfile::TempDir {
    use base64::Engine as _;
    let key = systemprompt::security::keys::RsaSigningKey::generate().expect("signing key");
    let pem = key.to_pkcs8_pem().expect("signing key pem");
    let dir = bootstrap(Some(base64::engine::general_purpose::STANDARD.encode(pem))).await;
    std::os::unix::fs::symlink(crate::repo_path("services"), dir.path().join("services"))
        .expect("link the repository services tree into the profile");
    systemprompt::config::try_init_config(None).expect("config from the fixture profile");
    dir
}

async fn bootstrap(signing_key_pem: Option<String>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("profile directory");
    let root: PathBuf = crate::repo_root();
    let yaml = FIXTURE_PROFILE
        .replace("__PROFILE_DIR__", &dir.path().to_string_lossy())
        .replace("__REPO__", &root.to_string_lossy());
    std::fs::create_dir_all(dir.path().join("bin")).expect("bin directory");
    std::fs::write(dir.path().join("profile.yaml"), yaml).expect("fixture profile");
    std::fs::write(
        dir.path().join("secrets.json"),
        secrets(signing_key_pem.as_deref()),
    )
    .expect("fixture secrets");
    if !systemprompt::config::ProfileBootstrap::is_initialized() {
        systemprompt::config::ProfileBootstrap::init_from_path(&dir.path().join("profile.yaml"))
            .expect("profile bootstrap");
    }
    systemprompt::config::SecretsBootstrap::try_init()
        .await
        .expect("secrets bootstrap");
    dir
}
