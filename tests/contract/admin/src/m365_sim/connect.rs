//! Connecting a Microsoft 365 account the way a browser does: the person's
//! session opens `/api/public/connectors/microsoft-365/start`, follows the
//! redirect to the simulated Entra (signing in as the account named by
//! `login_hint`), and brings the code back to the callback on the same
//! session. The outcome is read from `mcp_connector_accounts`.

use reqwest::StatusCode;
use reqwest::header::LOCATION;
use systemprompt::identifiers::UserId;

use super::stack::Stack;

const START: &str = "/api/public/connectors/microsoft-365/start";

// The connector account row of one person: status, tenant id and account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Linked {
    pub(super) status: String,
    pub(super) tenant: Option<String>,
    pub(super) account: Option<String>,
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("a client that does not follow redirects")
}

fn location(response: &reqwest::Response) -> String {
    response
        .headers()
        .get(LOCATION)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_else(|| panic!("a redirect, got {}", response.status()))
        .to_owned()
}

// Opens the consent on `bearer`'s session and signs in at Entra as
// `login`; returns the callback URL Entra sent the browser back to.
pub(super) async fn consent(stack: &Stack, bearer: &str, login: &str) -> String {
    let http = client();
    let start = http
        .get(format!("{}{START}", stack.api))
        .bearer_auth(bearer)
        .send()
        .await
        .expect("open the connector consent");
    if !start.status().is_redirection() {
        let status = start.status();
        let body = start.text().await.unwrap_or_default();
        panic!("the consent start answered {status}: {body}");
    }
    let authorize = location(&start);
    assert!(
        authorize.starts_with(&stack.sim.entra),
        "the consent goes to the configured Entra authority, not {authorize}"
    );
    let mut signed_in = url::Url::parse(&authorize).expect("the authorize URL parses");
    signed_in.query_pairs_mut().append_pair("login_hint", login);
    let entra = http
        .get(signed_in)
        .send()
        .await
        .expect("sign in at the simulated Entra");
    assert_eq!(entra.status(), StatusCode::FOUND, "Entra signs {login} in");
    location(&entra)
}

// Brings Entra's code back on `bearer`'s session; returns the status.
pub(super) async fn callback(bearer: &str, url: &str) -> StatusCode {
    client()
        .get(url)
        .bearer_auth(bearer)
        .send()
        .await
        .expect("return to the connector callback")
        .status()
}

pub(super) async fn linked(pool: &sqlx::PgPool, user: &UserId) -> Option<Linked> {
    sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT status, resource_id, account_name FROM mcp_connector_accounts
         WHERE user_id = $1 AND provider = 'microsoft-365'",
    )
    .bind(user.as_str())
    .fetch_optional(pool)
    .await
    .expect("read the connector account")
    .map(|(status, tenant, account)| Linked {
        status,
        tenant,
        account,
    })
}

pub(super) fn is_connected(row: Option<&Linked>, tenant: &str) -> bool {
    row.is_some_and(|row| row.status == "connected" && row.tenant.as_deref() == Some(tenant))
}
