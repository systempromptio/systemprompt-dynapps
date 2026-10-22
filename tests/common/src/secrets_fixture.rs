//! A bootstrapped profile and secrets store carrying an
//! `encryption_master_key`, for suites that exercise the at-rest sealing path
//! (`secret_crypto`).
//!
//! The key is served the way production reads it, through core's
//! `SecretsBootstrap`; nothing is placed in the process environment. Both
//! bootstraps are process-wide, so the first call installs and later calls
//! in the same process reuse the installed store.

use std::path::PathBuf;

pub const MASTER_KEY_HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

const FIXTURE_PROFILE: &str = include_str!("../../contract/admin/fixtures/profile.yaml");

fn secrets() -> String {
    serde_json::json!({
        "database_url": "postgres://unused:unused@localhost:5432/postgres",
        "oauth_at_rest_pepper": "secrets-fixture-pepper-not-a-real-secret",
        "manifest_signing_secret_seed": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "encryption_master_key": MASTER_KEY_HEX,
    })
    .to_string()
}

pub async fn bootstrap_master_key() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("profile directory");
    let root: PathBuf = crate::repo_root();
    let yaml = FIXTURE_PROFILE
        .replace("__PROFILE_DIR__", &dir.path().to_string_lossy())
        .replace("__REPO__", &root.to_string_lossy());
    std::fs::create_dir_all(dir.path().join("bin")).expect("bin directory");
    std::fs::write(dir.path().join("profile.yaml"), yaml).expect("fixture profile");
    std::fs::write(dir.path().join("secrets.json"), secrets()).expect("fixture secrets");
    if !systemprompt::config::ProfileBootstrap::is_initialized() {
        systemprompt::config::ProfileBootstrap::init_from_path(&dir.path().join("profile.yaml"))
            .expect("profile bootstrap");
    }
    systemprompt::config::SecretsBootstrap::try_init()
        .await
        .expect("secrets bootstrap");
    dir
}
