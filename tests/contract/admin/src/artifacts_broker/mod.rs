//! The artifacts broker end to end: a Drive page's tool call runs as the
//! person viewing it, through core's own MCP proxy.
//!
//! One throwaway database carries the full schema. The runtime boots on a
//! profile over it with the document store on, the API serves from that
//! runtime on the loopback port the profile names, and the `artifacts-fake`
//! server is hosted in-process by core's toolset host. Every call goes as
//! the browser sends it (the web login's cookie, from the site's own page)
//! to `POST /api/v1/artifacts/{document}/mcp/call`; the broker forwards it to
//! core's proxy with the viewer's own credential, so the server sees the
//! viewer and core's per-tool access decides. Each case checks the answer's
//! status and `error_key`, and the run ends by reading the audit rows back.

mod fake;
mod stack;

use serde_json::{Value, json};
use systemprompt::identifiers::{DocumentId, UserId};

use crate::{globals, principal};
use fake::{ECHO, SECRET};
use stack::{SERVER, Stack};

const AUDIT_COLUMNS: [&str; 8] = [
    "id",
    "user_id",
    "document_id",
    "server",
    "tool",
    "outcome",
    "duration_ms",
    "called_at",
];

fn call(server: &str, tool: &str) -> Value {
    json!({ "server": server, "tool": tool, "input": { "text": "hola" } })
}

fn error_key(reply: &stack::Reply) -> String {
    reply.json()["error_key"]
        .as_str()
        .unwrap_or_else(|| panic!("no error_key in {}: {}", reply.status, reply.body))
        .to_owned()
}

async fn allowed_tool_runs_as_the_viewer(
    stack: &Stack,
    cookie: &str,
    owner: &UserId,
    document: &DocumentId,
) {
    let reply = stack
        .call(Some(cookie), document, &call(SERVER, ECHO))
        .await;
    assert_eq!(reply.status, 200, "an allowed tool runs: {}", reply.body);
    let result = reply.json();
    assert_eq!(
        result["structuredContent"]["caller"],
        owner.as_str(),
        "the server saw the viewer, not the broker: {result}"
    );
    assert_eq!(result["structuredContent"]["tool"], ECHO);
    assert_eq!(
        result["structuredContent"]["text"], "hola",
        "the input reached the tool"
    );
    assert!(
        result["content"].is_array(),
        "the result keeps core's shape: {result}"
    );
}

async fn refusals(stack: &Stack, owner: &str, outsider: &str, document: &DocumentId) {
    let secret = stack
        .call(Some(owner), document, &call(SERVER, SECRET))
        .await;
    assert_eq!(
        secret.status, 403,
        "a tool the viewer's role lacks: {}",
        secret.body
    );
    assert_eq!(error_key(&secret), "blocked_by_policy");

    let anonymous = stack.call(None, document, &call(SERVER, ECHO)).await;
    assert_eq!(anonymous.status, 401, "no session: {}", anonymous.body);

    let hidden = stack
        .call(Some(outsider), document, &call(SERVER, ECHO))
        .await;
    assert_eq!(
        hidden.status, 404,
        "a document the viewer cannot see: {}",
        hidden.body
    );
    assert_eq!(error_key(&hidden), "not_found");

    let unknown_server = stack
        .call(Some(owner), document, &call("no-such-server", ECHO))
        .await;
    assert_eq!(unknown_server.status, 400, "{}", unknown_server.body);
    assert_eq!(error_key(&unknown_server), "bad_request");

    let unknown_tool = stack
        .call(Some(owner), document, &call(SERVER, "no_such_tool"))
        .await;
    assert_eq!(unknown_tool.status, 400, "{}", unknown_tool.body);
    assert_eq!(error_key(&unknown_tool), "bad_request");
}

async fn audit_rows(
    pool: &sqlx::PgPool,
    document: &DocumentId,
) -> Vec<(String, String, String, String)> {
    sqlx::query_as::<_, (String, String, String, String, i32)>(
        "SELECT user_id, server, tool, outcome, duration_ms
           FROM artifact_tool_calls
          WHERE document_id = $1::uuid
          ORDER BY id",
    )
    .bind(document.as_str())
    .fetch_all(pool)
    .await
    .expect("read the audit rows")
    .into_iter()
    .map(|(user, server, tool, outcome, duration)| {
        assert!(duration >= 0, "a duration is never negative");
        (user, server, tool, outcome)
    })
    .collect()
}

async fn audit_columns(pool: &sqlx::PgPool) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT column_name::text FROM information_schema.columns
          WHERE table_name = 'artifact_tool_calls'
          ORDER BY ordinal_position",
    )
    .fetch_all(pool)
    .await
    .expect("read the audit table's columns")
}

#[tokio::test(flavor = "multi_thread")]
async fn artifacts_broker_runs_a_tool_as_the_viewer() {
    assert!(globals::init(), "the contract fixture profile boots");
    let db = crate::tempdb::TempDb::create()
        .await
        .expect("the broker suite needs SYSTEMPROMPT_TEST_DATABASE_URL");
    let credentials = principal::provision(&db.pool).await;
    let stack = stack::start(&db.pool, &db.url()).await;
    let owner = credentials.knowledge_worker_user_id.clone();
    let outsider = credentials.non_admin_user_id.clone();
    let document = Box::pin(stack.owned_document(&owner)).await;
    let owner_cookie = stack.session_cookie(&db.pool, &owner).await;
    let outsider_cookie = stack.session_cookie(&db.pool, &outsider).await;

    allowed_tool_runs_as_the_viewer(&stack, &owner_cookie, &owner, &document).await;
    refusals(&stack, &owner_cookie, &outsider_cookie, &document).await;

    let row = |user: &UserId, server: &str, tool: &str, outcome: &str| {
        (
            user.as_str().to_owned(),
            server.to_owned(),
            tool.to_owned(),
            outcome.to_owned(),
        )
    };
    assert_eq!(
        audit_rows(&db.pool, &document).await,
        vec![
            row(&owner, SERVER, ECHO, "ok"),
            row(&owner, SERVER, SECRET, "blocked_by_policy"),
            row(&outsider, SERVER, ECHO, "not_found"),
            row(&owner, "no-such-server", ECHO, "bad_request"),
            row(&owner, SERVER, "no_such_tool", "bad_request"),
        ],
        "one row per call a signed-in viewer made; the anonymous call has no viewer to record"
    );
    assert_eq!(
        audit_columns(&db.pool).await,
        AUDIT_COLUMNS.map(str::to_owned).to_vec(),
        "the audit never holds a tool's input or result"
    );

    drop(stack);
    db.cleanup().await;
}
