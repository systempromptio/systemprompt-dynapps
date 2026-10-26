//! The boot values a unit test hands code that reads them: a profile parsed
//! from the contract fixture over the repository's `services/` tree, the
//! services view of a tree on disk, a secrets store built from an in-memory
//! document, and the `ExtensionBoot` over them. Nothing touches a database or
//! installs process-wide state.

use std::path::Path;
use std::sync::Arc;

use systemprompt::config::{LoadedProfile, SecretsService};
use systemprompt::loader::{BootContext, ServicesState};
use systemprompt::manifest::services::ServicesView;
use systemprompt::manifest::services::root::ActiveServicesRoot;
use systemprompt::security::keys::{RsaSigningKey, SigningKeys};
use systemprompt::traits::SecretStore;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

const FIXTURE_PROFILE: &str = include_str!("../../../contract/admin/fixtures/profile.yaml");

pub(crate) struct Fixture {
    dir: tempfile::TempDir,
    boot: BootContext,
}

impl Fixture {
    pub(crate) fn new() -> Self {
        let dir = tempfile::tempdir().expect("profile directory");
        let yaml = FIXTURE_PROFILE
            .replace("__PROFILE_DIR__", &dir.path().to_string_lossy())
            .replace("__REPO__", &crate::support::repo_root().to_string_lossy());
        std::fs::create_dir_all(dir.path().join("bin")).expect("bin directory");
        std::os::unix::fs::symlink(
            crate::support::repo_root().join("services"),
            dir.path().join("services"),
        )
        .expect("link the repository services tree into the profile");
        let path = dir.path().join("profile.yaml");
        std::fs::write(&path, yaml).expect("fixture profile");
        let profile = LoadedProfile::from_path(&path).expect("load the fixture profile");
        let secrets = serde_json::from_value(serde_json::json!({
            "database_url": "postgres://unused:unused@localhost:5432/postgres",
            "oauth_at_rest_pepper": "unit-fixture-pepper-not-a-real-secret",
            "manifest_signing_secret_seed": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
            "encryption_master_key":
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        }))
        .expect("fixture secrets document");
        let secrets = SecretsService::from_secrets(Arc::clone(&profile), secrets);
        Self {
            dir,
            boot: BootContext::new(profile, secrets),
        }
    }

    pub(crate) fn path(&self) -> &Path {
        self.dir.path()
    }

    pub(crate) fn secrets(&self) -> Arc<dyn SecretStore> {
        Arc::<SecretsService>::clone(self.boot.secrets())
    }

    pub(crate) fn services_view(&self, config: &Path) -> ServicesView {
        ServicesState::load_from_path(config, self.boot.profile())
            .expect("load the services fixture")
            .view()
    }

    pub(crate) fn extension_boot(&self) -> ExtensionBoot {
        let root = ActiveServicesRoot::baked(self.boot.profile());
        let config = Arc::clone(
            self.boot
                .try_init_config(Some(root.path.as_path()))
                .expect("config from the fixture profile"),
        );
        let keys = SigningKeys::new(RsaSigningKey::generate().expect("signing key"), &[9; 32])
            .expect("signing keys");
        let authority = systemprompt::security::jwt::DeploymentTokenAuthority::new(
            keys,
            config.jwt_issuer.clone(),
        );
        ExtensionBoot::new(
            Arc::new(self.boot.profile().clone()),
            config,
            root,
            Arc::new(authority),
            self.secrets(),
        )
    }
}

pub(crate) fn empty_services_view() -> ServicesView {
    let fixture = Fixture::new();
    let path = fixture.path().join("empty-services.yaml");
    std::fs::write(&path, "{}").expect("write services fixture");
    fixture.services_view(&path)
}

pub(crate) fn services_view(config: &Path) -> ServicesView {
    Fixture::new().services_view(config)
}

pub(crate) fn secret_store() -> Arc<dyn SecretStore> {
    Fixture::new().secrets()
}
