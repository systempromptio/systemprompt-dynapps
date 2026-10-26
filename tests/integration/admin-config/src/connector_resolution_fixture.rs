//! TLS OAuth fixture driving the real encrypted broker and database locks.

use crate::fixtures::{insert_user, unique, user_id};
use crate::tempdb::TempDb;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_core::connections::report::VerificationReport;
use systemprompt_web_admin_core::error::AdminResult;
use systemprompt_web_admin_platform::connector_oauth::{self as oauth, Grant, Provider};
use systemprompt_web_admin_platform::repositories::connectors::{
    connector_accounts as accounts, connector_credentials as credentials,
};

struct Server(std::process::Child);
impl Drop for Server {
    fn drop(&mut self) {
        if self.0.kill().is_ok() {
            self.0.wait().ok();
        }
    }
}

pub(super) struct Harness {
    pub(super) db: TempDb,
    pub(super) user: UserId,
    pub(super) provider: Provider,
    pub(super) http: reqwest::Client,
    origin: String,
    services: systemprompt_web_admin_core::extension_services::ExtensionServices,
    secrets: std::sync::Arc<dyn systemprompt::traits::SecretStore>,
    _server: Server,
    _directory: tempfile::TempDir,
}

impl Harness {
    pub(super) async fn create_or_skip(mode: &str) -> Option<Self> {
        let directory = tempfile::tempdir().unwrap();
        let script = dynapps_test_common::repo_path("tests/fixtures/oauth/server.py");
        let mut server = Server(
            Command::new("python3")
                .arg("-u")
                .arg(script)
                .arg(directory.path())
                .arg(mode)
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let mut origin = String::new();
        BufReader::new(server.0.stdout.take().unwrap())
            .read_line(&mut origin)
            .unwrap();
        let origin = origin.trim().to_owned();
        let config = directory.path().join("services/config/config.yaml");
        std::fs::create_dir_all(config.parent().unwrap()).unwrap();
        std::fs::create_dir(directory.path().join("bin")).unwrap();
        std::fs::create_dir_all(directory.path().join("services/skills")).unwrap();
        for relative in [
            "services/web/config.yaml",
            "services/web/metadata.yaml",
            "services/content/config.yaml",
        ] {
            let destination = directory.path().join(relative);
            std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
            std::fs::copy(dynapps_test_common::repo_path(relative), destination).unwrap();
        }
        std::fs::write(&config, format!(
            "mcp_servers:\n  rotation-mcp:\n    server_type: external\n    endpoint: {origin}/mcp\n    enabled: true\n    display_in_web: false\n    oauth: {{required: false, scopes: [user], audience: mcp, client_id: null}}\n    connector: {{adapter: generic, scopes: [tools:read]}}\n"
        )).unwrap();
        let view = dynapps_test_common::offline_extension_services(
            dynapps_test_common::services_view(&config),
        );
        let services = &view;
        let secrets = dynapps_test_common::fixture_secret_store();
        let access = oauth::ConnectorAccess {
            services,
            secrets: secrets.as_ref(),
        };
        let ca = reqwest::Certificate::from_pem(
            &std::fs::read(directory.path().join("ca.pem")).unwrap(),
        )
        .unwrap();
        let http = reqwest::Client::builder()
            .add_root_certificate(ca)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap();
        let db = TempDb::create().await?;
        let raw = unique("rotation-user");
        insert_user(&db.pool, &raw).await;
        let user = user_id(&raw);
        let provider = Provider::try_from("rotation-mcp".to_owned()).unwrap();
        let callback = "https://dashboard.example.test/api/public/connectors/rotation-mcp/callback";
        let (_, mut grant) = oauth::generic::authorize_with_client(
            access,
            oauth::Consent {
                user: user.as_str(),
                state: "test-state",
                verifier: "test-verifier".into(),
            },
            provider.clone(),
            callback,
            &http,
        )
        .await
        .unwrap();
        oauth::exchange_with_client(services, &mut grant, "test-code", callback, &http)
            .await
            .unwrap();
        grant.expires_at = 0;
        let mut tx = db.pool.begin().await.unwrap();
        let mut account = accounts::get_locked_account(&mut tx, &user, provider.slug())
            .await
            .unwrap();
        account.status = "connected".into();
        account.auth_method = Some("oauth".into());
        account.verified_at = Some(chrono::Utc::now());
        credentials::store(
            &mut tx,
            &user,
            provider.slug(),
            &oauth::seal(secrets.as_ref(), &grant).unwrap(),
        )
        .await
        .unwrap();
        accounts::update_account(&mut tx, &user, &account)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        Some(Self {
            db,
            user,
            provider,
            http,
            origin,
            services: view,
            secrets,
            _server: server,
            _directory: directory,
        })
    }

    pub(super) async fn resolve(&self, probe: bool) -> (AdminResult<String>, VerificationReport) {
        let mut report = VerificationReport::for_provider(self.provider.slug());
        let access = oauth::ConnectorAccess {
            services: &self.services,
            secrets: self.secrets.as_ref(),
        };
        let result = oauth::TokenResolver::new(&self.db.pool, access, &self.user, &self.http)
            .resolve(self.provider.clone(), probe, &mut report)
            .await;
        (result, report.finish())
    }

    pub(super) async fn grant(&self) -> Option<Grant> {
        let mut tx = self.db.pool.begin().await.unwrap();
        let row = credentials::lock(&mut tx, &self.user, self.provider.slug())
            .await
            .unwrap();
        tx.commit().await.unwrap();
        row.map(|row| {
            oauth::open(
                &self.services,
                self.secrets.as_ref(),
                &row.grant,
                &self.user,
                &self.provider,
            )
            .unwrap()
        })
    }

    pub(super) async fn account(&self) -> accounts::ProviderConnection {
        accounts::list_accounts(&self.db.pool, &self.user)
            .await
            .unwrap()
            .remove(0)
    }

    pub(super) async fn requests(&self) -> u64 {
        self.http
            .get(format!("{}/stats", self.origin))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap()["refresh_requests"]
            .as_u64()
            .unwrap()
    }
}
