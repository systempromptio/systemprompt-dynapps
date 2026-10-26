//! A booted profile, secrets store and config for suites that read them.
//!
//! The secrets carry an `encryption_master_key` for the at-rest sealing path
//! (`secret_crypto`), and optionally a signing key for suites that build the
//! web router. The fixture owns the [`BootContext`] the way a server does and
//! hands the code under test the [`ExtensionBoot`] it would receive from its
//! extension context; nothing is installed process-wide.

use std::path::Path;
use std::sync::Arc;

use systemprompt::config::{LoadedProfile, SecretsService};
use systemprompt::loader::BootContext;
use systemprompt::manifest::services::root::ActiveServicesRoot;
use systemprompt::security::keys::SigningKeys;
use systemprompt::traits::SecretStore;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

pub const MASTER_KEY_HEX: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

pub(crate) const FIXTURE_PROFILE: &str = include_str!("../../contract/admin/fixtures/profile.yaml");

pub(crate) fn secrets_document(signing_key_pem: Option<&str>, extra: &[(&str, &str)]) -> String {
    let mut doc = serde_json::json!({
        "database_url": "postgres://unused:unused@localhost:5432/postgres",
        "oauth_at_rest_pepper": "secrets-fixture-pepper-not-a-real-secret",
        "manifest_signing_secret_seed": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "encryption_master_key": MASTER_KEY_HEX,
        "signing_key_pem": signing_key_pem,
    });
    for (key, value) in extra {
        doc[*key] = serde_json::Value::String((*value).to_owned());
    }
    doc.to_string()
}

#[derive(Debug)]
pub struct BootFixture {
    dir: tempfile::TempDir,
    boot: BootContext,
    keys: SigningKeys,
}

impl BootFixture {
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub const fn boot(&self) -> &BootContext {
        &self.boot
    }

    pub const fn signing_keys(&self) -> &SigningKeys {
        &self.keys
    }

    pub fn secrets(&self) -> Arc<dyn SecretStore> {
        Arc::<SecretsService>::clone(self.boot.secrets())
    }

    pub fn config(&self) -> Arc<systemprompt::manifest::Config> {
        let root = self.services_root();
        Arc::clone(
            self.boot
                .try_init_config(Some(root.path.as_path()))
                .expect("config from the fixture profile"),
        )
    }

    pub fn services_root(&self) -> ActiveServicesRoot {
        ActiveServicesRoot::baked(self.boot.profile())
    }

    pub fn extension_boot(&self) -> ExtensionBoot {
        let config = self.config();
        let authority = systemprompt::security::jwt::DeploymentTokenAuthority::new(
            self.keys.clone(),
            config.jwt_issuer.clone(),
        );
        ExtensionBoot::new(
            Arc::new(self.boot.profile().clone()),
            config,
            self.services_root(),
            Arc::new(authority),
            self.secrets(),
        )
    }
}

pub async fn bootstrap_master_key() -> BootFixture {
    bootstrap(None, &[]).await
}

pub async fn bootstrap_with_secrets(extra: &[(&str, &str)]) -> BootFixture {
    bootstrap(None, extra).await
}

pub async fn bootstrap_signing_config() -> BootFixture {
    use base64::Engine as _;
    let key = systemprompt::security::keys::RsaSigningKey::generate().expect("signing key");
    let pem = key.to_pkcs8_pem().expect("signing key pem");
    let fixture = bootstrap(
        Some(base64::engine::general_purpose::STANDARD.encode(pem)),
        &[],
    )
    .await;
    std::os::unix::fs::symlink(
        crate::repo_path("services"),
        fixture.path().join("services"),
    )
    .expect("link the repository services tree into the profile");
    fixture
}

async fn bootstrap(signing_key_pem: Option<String>, extra: &[(&str, &str)]) -> BootFixture {
    let dir = tempfile::tempdir().expect("profile directory");
    let root = crate::repo_root();
    let yaml = FIXTURE_PROFILE
        .replace("__PROFILE_DIR__", &dir.path().to_string_lossy())
        .replace("__REPO__", &root.to_string_lossy());
    std::fs::create_dir_all(dir.path().join("bin")).expect("bin directory");
    std::fs::write(dir.path().join("profile.yaml"), yaml).expect("fixture profile");
    std::fs::write(
        dir.path().join("secrets.json"),
        secrets_document(signing_key_pem.as_deref(), extra),
    )
    .expect("fixture secrets");
    let profile =
        LoadedProfile::from_path(&dir.path().join("profile.yaml")).expect("profile bootstrap");
    let secrets = SecretsService::load(Arc::clone(&profile))
        .await
        .expect("secrets bootstrap");
    BootFixture {
        dir,
        boot: BootContext::new(profile, secrets),
        keys: crate::ephemeral_signing_keys(),
    }
}
