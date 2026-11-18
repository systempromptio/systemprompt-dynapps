//! A tenant taken off `microsoft_365_allowed_tenants` after its person
//! connected. The run's secrets are rewritten and the live secrets service
//! reloads them while the API serves, as an operator's edit lands without a
//! restart; the next token asked of the connector broker
//! (`/api/public/connectors/microsoft-365/token`, with the broker secret core
//! presents) is refused, the sealed grant is deleted and the account waits for
//! a reconnect. It runs last and on Bruno (tenant B), so no earlier case loses
//! his connection; Ana (tenant A, still admitted) keeps getting her token.

use reqwest::StatusCode;
use serde_json::Value;
use systemprompt::identifiers::UserId;

use super::stack::{M365, Stack};
use super::{case, connect, find};

const TOKEN: &str = "/api/public/connectors/microsoft-365/token";
const BROKER_HEADER: &str = "x-systemprompt-credential-broker";
const ALLOWED_TENANTS: &str = "microsoft_365_allowed_tenants";

fn read_secrets(stack: &Stack) -> Value {
    let text = std::fs::read_to_string(&stack.secrets).expect("read the run's secrets");
    serde_json::from_str(&text).expect("the run's secrets are JSON")
}

// Asks the broker for `user`'s token on a fresh session, the way core's MCP
// client does: the person's bearer plus the broker secret.
async fn broker_token(stack: &Stack, pool: &sqlx::PgPool, user: &UserId) -> (StatusCode, Value) {
    let caller = stack.sign_in(pool, user, M365).await;
    let secrets = read_secrets(stack);
    let broker = secrets["mcp_credential_broker_secret"]
        .as_str()
        .expect("the run's broker secret");
    let response = reqwest::Client::new()
        .get(format!("{}{TOKEN}", stack.api))
        .bearer_auth(&caller.bearer)
        .header(BROKER_HEADER, broker)
        .send()
        .await
        .expect("ask the connector broker for a token");
    let status = response.status();
    let body = response.json().await.unwrap_or(Value::Null);
    (status, body)
}

fn has_token(body: &Value) -> bool {
    find(body, "access_token")
        .and_then(Value::as_str)
        .is_some_and(|token| !token.is_empty())
}

async fn sealed_grants(pool: &sqlx::PgPool, user: &UserId) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT count(*) FROM mcp_connector_credentials
         WHERE user_id = $1 AND provider = 'microsoft-365'",
    )
    .bind(user.as_str())
    .fetch_one(pool)
    .await
    .expect("count the sealed grants")
}

async fn allow_only(stack: &Stack, tenants: &str) {
    let mut secrets = read_secrets(stack);
    secrets[ALLOWED_TENANTS] = Value::String(tenants.to_owned());
    std::fs::write(&stack.secrets, secrets.to_string()).expect("write the run's secrets");
    let reloaded = stack
        .ctx
        .secrets()
        .reload()
        .await
        .expect("reload the run's secrets");
    assert!(
        reloaded.swapped,
        "the allow-list change was not picked up: {reloaded:?}"
    );
}

pub(super) async fn tenant_off_the_list(
    stack: &Stack,
    pool: &sqlx::PgPool,
    ana: &UserId,
    bruno: &UserId,
    n: &mut u32,
) {
    let (status, body) = broker_token(stack, pool, bruno).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "Bruno before the withdrawal: {body}"
    );
    assert!(has_token(&body), "Bruno before the withdrawal: {body}");
    assert_eq!(sealed_grants(pool, bruno).await, 1, "Bruno's sealed grant");

    let tenant_a = stack.sim.tenants["a"].as_str().expect("tenant A");
    allow_only(stack, tenant_a).await;

    let (status, body) = broker_token(stack, pool, bruno).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "Bruno after: {body}");
    assert!(!has_token(&body), "a withdrawn tenant got a token: {body}");
    assert_eq!(
        sealed_grants(pool, bruno).await,
        0,
        "the sealed grant of a withdrawn tenant survived"
    );
    let row = connect::linked(pool, bruno).await;
    assert_eq!(
        row.as_ref().map(|row| row.status.as_str()),
        Some("reconnect_required"),
        "Bruno's account: {row:?}"
    );
    let (status, body) = broker_token(stack, pool, ana).await;
    assert!(
        status == StatusCode::OK && has_token(&body),
        "Ana (tenant A, still admitted): {status} {body}"
    );
    case(
        n,
        "retirar tenant: se quita el tenant B de microsoft_365_allowed_tenants; el broker ya no da token a Bruno, borra su grant sellado y deja la cuenta en reconnect_required (Ana, tenant A, sigue recibiendo el suyo)",
    );
}
