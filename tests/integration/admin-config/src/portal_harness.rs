//! Shared driving for the fake-portal e2e: the real connector router
//! (`connector_api_router`) in-process, a person with a live login, the
//! browser leg of the OAuth dance against the fake, and the two halves of a
//! gateway MCP call -- the broker bearer fetch core makes
//! (`fetch_external_bearer`'s headers) and the forwarded MCP exchange.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use dynapps_test_common::fake_portal::env::BROKER_SECRET;
use dynapps_test_common::{PortalEnv, Principal};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use systemprompt::database::Database;
use systemprompt::identifiers::UserId;
use systemprompt::users::UserRepository;
use systemprompt_web_admin_platform::connector_api_router;
use systemprompt_web_admin_platform::service_accounts::ServiceAccounts;
use tower::ServiceExt;

use crate::fixtures::{insert_user, unique, user_id};
use crate::tempdb::TempDb;

pub(crate) struct Harness {
    pub(crate) env: PortalEnv,
    pub(crate) db: TempDb,
    pub(crate) router: Router,
    pub(crate) http: reqwest::Client,
}

// A person with a live login session and the JWT that carries it.
pub(crate) struct Person {
    pub(crate) id: UserId,
    pub(crate) jwt: String,
}

impl Harness {
    pub(crate) async fn create_or_skip() -> Option<Self> {
        let env = PortalEnv::start().await;
        let db = TempDb::create().await?;
        let database = Arc::new(Database::from_pools(
            Arc::clone(&db.pool),
            Some(Arc::clone(&db.pool)),
        ));
        let accounts = ServiceAccounts::new(UserRepository::new(&database));
        let router = connector_api_router(Arc::clone(&db.pool), accounts);
        let ca = reqwest::Certificate::from_pem(&env.portal.ca_pem()).expect("fake CA");
        let http = reqwest::Client::builder()
            .add_root_certificate(ca)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("http client");
        Some(Self {
            env,
            db,
            router,
            http,
        })
    }

    pub(crate) async fn person(&self, prefix: &str) -> Person {
        let raw = unique(prefix);
        insert_user(&self.db.pool, &raw).await;
        let session = unique("portal-session");
        sqlx::query("INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)")
            .bind(&session)
            .bind(&raw)
            .execute(self.db.pool.as_ref())
            .await
            .expect("insert live session");
        let email = format!("{raw}@example.test");
        let jwt = PortalEnv::mint(Principal {
            user_id: &raw,
            email: &email,
            session_id: Some(&session),
        });
        Person {
            id: user_id(&raw),
            jwt,
        }
    }

    // GET a connector route as `jwt`, with optional extra headers.
    pub(crate) async fn get(
        &self,
        uri: &str,
        jwt: &str,
        extra: &[(&str, &str)],
    ) -> (StatusCode, Option<String>, String) {
        let mut request = Request::get(uri).header(header::AUTHORIZATION, format!("Bearer {jwt}"));
        for (name, value) in extra {
            request = request.header(*name, *value);
        }
        let response = self
            .router
            .clone()
            .oneshot(request.body(Body::empty()).expect("request"))
            .await
            .expect("router");
        let status = response.status();
        let location = response
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned);
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        (
            status,
            location,
            String::from_utf8_lossy(&body).into_owned(),
        )
    }

    // The whole consent: start → the portal's `/authorize` (signed in as
    // `portal_subject`) → callback. Returns the callback's redirect.
    pub(crate) async fn connect(
        &self,
        person: &Person,
        provider: &str,
        portal_subject: &str,
    ) -> String {
        self.env.portal.sign_in_as(portal_subject);
        let (status, authorize, body) = self
            .get(&format!("/connectors/{provider}/start"), &person.jwt, &[])
            .await;
        assert!(status.is_redirection(), "start: {status} {body}");
        let authorize = authorize.expect("start redirects to the portal");
        assert!(
            authorize.starts_with(self.env.portal.base_url()),
            "{authorize}"
        );
        let response = self.http.get(&authorize).send().await.expect("authorize");
        assert_eq!(response.status(), StatusCode::FOUND);
        let callback = response.headers()[header::LOCATION.as_str()]
            .to_str()
            .expect("location")
            .to_owned();
        let callback = reqwest::Url::parse(&callback).expect("callback URL");
        assert_eq!(
            callback.path(),
            format!("/api/public/connectors/{provider}/callback")
        );
        let uri = format!(
            "/connectors/{provider}/callback?{}",
            callback.query().unwrap_or_default()
        );
        let (status, done, body) = self.get(&uri, &person.jwt, &[]).await;
        assert!(status.is_redirection(), "callback: {status} {body}");
        done.expect("callback redirects")
    }

    // What core's external-MCP proxy asks the broker for, header for header.
    pub(crate) async fn broker_token(
        &self,
        jwt: &str,
        provider: &str,
        caller: Option<&str>,
    ) -> (StatusCode, Option<String>) {
        let mut extra = vec![
            ("x-systemprompt-credential-broker", BROKER_SECRET),
            ("x-systemprompt-mcp-server", provider),
        ];
        if let Some(caller) = caller {
            extra.push(("x-systemprompt-caller", caller));
        }
        let (status, _, body) = self
            .get(&format!("/connectors/{provider}/token"), jwt, &extra)
            .await;
        let token = serde_json::from_str::<Value>(&body)
            .ok()
            .and_then(|v| v["access_token"].as_str().map(str::to_owned))
            // The broker answers the complete Authorization value; core forwards
            // it as-is, so the scheme is checked and the token compared bare.
            .map(|value| {
                value
                    .strip_prefix("Bearer ")
                    .expect("broker answers a Bearer authorization value")
                    .to_owned()
            });
        (status, token)
    }

    // The forwarded half: initialize, initialized, tools/list and a
    // `whoami` tools/call at `path` with `bearer`. Returns (tool names,
    // whoami text).
    pub(crate) async fn mcp_whoami(&self, path: &str, bearer: &str) -> (Vec<String>, String) {
        let endpoint = self.env.portal.endpoint(path);
        let post = |body: Value, session: Option<&str>| {
            let mut request = self
                .http
                .post(&endpoint)
                .bearer_auth(bearer)
                .header("Accept", "application/json, text/event-stream")
                .json(&body);
            if let Some(id) = session {
                request = request.header("Mcp-Session-Id", id);
            }
            request.send()
        };
        let init = post(
            json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
                "protocolVersion":"2025-03-26","capabilities":{},
                "clientInfo":{"name":"portal-e2e","version":"1"}}}),
            None,
        )
        .await
        .expect("initialize");
        assert!(init.status().is_success(), "initialize: {}", init.status());
        let session = init.headers()["mcp-session-id"]
            .to_str()
            .expect("session id")
            .to_owned();
        let notified = post(
            json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
            Some(&session),
        )
        .await
        .expect("initialized");
        assert!(notified.status().is_success());
        let rpc = |id: u64, method: &str, params: Value| {
            let body = json!({"jsonrpc":"2.0","id":id,"method":method,"params":params});
            let fut = post(body, Some(&session));
            async move { fut.await.expect("rpc").json::<Value>().await.expect("json") }
        };
        let list = rpc(2, "tools/list", json!({})).await;
        let tools = list["result"]["tools"]
            .as_array()
            .expect("tools")
            .iter()
            .filter_map(|t| t["name"].as_str().map(str::to_owned))
            .collect();
        let call = rpc(3, "tools/call", json!({"name":"whoami","arguments":{}})).await;
        let text = call["result"]["content"][0]["text"]
            .as_str()
            .expect("whoami text")
            .to_owned();
        (tools, text)
    }
}
