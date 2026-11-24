//! The artifacts broker end to end: a Drive page's capabilities, declared by
//! the agent that writes it, and the page's tool calls, run as the person
//! viewing it through core's own MCP proxy once that person has allowed them.
//!
//! One throwaway database carries the full schema. The runtime boots on a
//! profile over it with the document store on and a live-pages policy of the
//! suite's own, the API serves from that runtime on the loopback port the
//! profile names, and two servers are hosted in-process: the `artifacts-fake`
//! toolset by core's toolset host, and the extension's `documents` server by
//! its own router. The agent declares the page through `documents`
//! (`manifest`); every other request goes as the browser sends it, the web
//! login's cookie from the site's own page, to `/api/v1/artifacts/...`:
//! consent (`consent`), tool calls with their approvals and cache (`calls`)
//! and the page republishing itself (`publish`). The run ends by reading the
//! audit rows back.

mod calls;
mod consent;
mod fake;
mod manifest;
mod publish;
mod stack;

use serde_json::json;
use systemprompt::identifiers::{DocumentId, UserId};

use crate::{globals, principal};
use fake::{ECHO, SECRET, WRITE};
use stack::{As, SERVER, Stack};

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

// Nobody signed in (a public link's visitor, an invitee who has not claimed
// the invite) and a person who cannot see the page get no capability at all.
async fn outsiders_get_nothing(stack: &Stack, outsider: &str, page: &DocumentId) {
    let base = format!("/api/v1/artifacts/{}", page.as_str());
    let routes = [
        (reqwest::Method::GET, format!("{base}/permissions"), None),
        (
            reqwest::Method::POST,
            format!("{base}/mcp/call"),
            Some(json!({ "server": SERVER, "tool": ECHO })),
        ),
        (
            reqwest::Method::POST,
            format!("{base}/mcp/tools"),
            Some(json!({})),
        ),
        (
            reqwest::Method::POST,
            format!("{base}/artifact/publish"),
            Some(json!({ "html": "<!doctype html>", "base_revision": "x" })),
        ),
    ];
    for (method, path, body) in routes {
        let anonymous = stack
            .send(method.clone(), &path, As::Nobody, body.as_ref())
            .await;
        assert_eq!(
            anonymous.status, 401,
            "{path} with no session: {}",
            anonymous.body
        );
        let hidden = stack
            .send(method, &path, As::Cookie(outsider), body.as_ref())
            .await;
        assert_eq!(
            hidden.status, 404,
            "{path} for a person who cannot see it: {}",
            hidden.body
        );
    }
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
    let owner_cookie = stack.session_cookie(&db.pool, &owner).await;
    let outsider_cookie = stack.session_cookie(&db.pool, &outsider).await;
    let agent = stack.agent_bearer(&db.pool, &owner).await;
    let undeclared = Box::pin(stack.owned_document(&owner, "dynapps-html")).await;

    let page = manifest::the_agent_declares_a_page(&stack, &db.pool, &owner, &owner_cookie).await;
    calls::before_consent(&stack, &owner_cookie, &page).await;
    let who = consent::Who {
        owner: &owner_cookie,
        outsider: &outsider_cookie,
        agent: &agent,
    };
    consent::only_the_viewer_decides(&stack, &who, &page).await;
    calls::reads_and_their_cache(&stack, &owner_cookie, owner.as_str(), &page).await;
    calls::writes_run_once_per_approval(&stack, &owner_cookie, &agent, &page).await;
    calls::the_server_still_decides(&stack, &db.pool, &owner_cookie, &page).await;
    calls::list_and_describe(&stack, &owner_cookie, &page).await;
    calls::consent_gates_the_cache(&stack, &owner_cookie, &page).await;
    outsiders_get_nothing(&stack, &outsider_cookie, &page).await;
    let people = publish::People {
        owner: &owner,
        owner_cookie: &owner_cookie,
        reader: &outsider,
        reader_cookie: &outsider_cookie,
    };
    publish::the_page_republishes_itself(&stack, &people, &page, &undeclared).await;

    let row = |user: &UserId, tool: &str, outcome: &str| {
        let server = if tool == "-" {
            "no-such-server"
        } else {
            SERVER
        };
        let tool = if tool == "-" { ECHO } else { tool };
        (
            user.as_str().to_owned(),
            server.to_owned(),
            tool.to_owned(),
            outcome.to_owned(),
        )
    };
    assert_eq!(
        audit_rows(&db.pool, &page).await,
        vec![
            row(&owner, ECHO, "approval_required"),
            row(&owner, SECRET, "not_in_manifest"),
            row(&owner, "-", "bad_request"),
            row(&owner, ECHO, "ok"),
            row(&owner, ECHO, "cached"),
            row(&owner, ECHO, "ok"),
            row(&owner, ECHO, "ok"),
            row(&owner, WRITE, "approval_required"),
            row(&owner, WRITE, "ok"),
            row(&owner, WRITE, "approval_required"),
            row(&owner, WRITE, "approval_required"),
            row(&owner, WRITE, "ok"),
            row(&owner, WRITE, "ok"),
            row(&owner, SECRET, "blocked_by_policy"),
            row(&owner, ECHO, "approval_required"),
            row(&owner, ECHO, "ok"),
            row(&owner, ECHO, "not_in_manifest"),
            row(&outsider, ECHO, "not_found"),
            row(&outsider, ECHO, "approval_required"),
        ],
        "one row per call a signed-in viewer made; the anonymous calls have no viewer to record"
    );
    assert_eq!(
        audit_columns(&db.pool).await,
        AUDIT_COLUMNS.map(str::to_owned).to_vec(),
        "the audit never holds a tool's input or result"
    );

    drop(stack);
    db.cleanup().await;
}
