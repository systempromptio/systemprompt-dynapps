//! Two companies on one installation, end to end through the `microsoft-365`
//! MCP binary, with Entra and Graph simulated per tenant
//! (`tests/fixtures/oauth/m365_sim.py`, loopback only, no real accounts).
//!
//! Company A's person (Ana, tenant A) and company B's (Bruno, tenant B) each
//! connect their own Microsoft 365 account through the real connector routes,
//! read their own mailbox through the MCP binary, whose Graph token comes from
//! the connector broker, request an upload from the `documents` server, upload
//! the bytes to the signed link, attach them to a draft, send it, mark mail as
//! read, put a file on `OneDrive` and save an attachment as a document. The
//! crossings are asserted on the same run: a tenant the installation does not
//! admit is never linked, one person cannot complete another's consent, a
//! token of tenant A never reads tenant B's mail, and none of Ana's file
//! references (upload, blob, document) resolves for Bruno. Every step prints
//! its case and result; `just m365-sim` runs it with output shown.

mod cases;
mod connect;
mod stack;

use serde_json::Value;

// Prints one case as it passes, so a run reads as the list of what held.
pub(super) fn case(number: &mut u32, text: &str) {
    *number += 1;
    println!("m365-sim caso {:02}: {text} ... ok", *number);
}

// The first value under `key` anywhere in a tool's payload.
pub(super) fn find<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    match value {
        Value::Object(fields) => fields
            .get(key)
            .or_else(|| fields.values().find_map(|inner| find(inner, key))),
        Value::Array(items) => items.iter().find_map(|inner| find(inner, key)),
        _ => None,
    }
}

pub(super) fn text_of(value: &Value, key: &str) -> String {
    find(value, key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("no `{key}` in {value}"))
        .to_owned()
}

#[tokio::test(flavor = "multi_thread")]
async fn m365_sim_two_companies_end_to_end() {
    let db = crate::tempdb::TempDb::create()
        .await
        .expect("the simulation needs SYSTEMPROMPT_TEST_DATABASE_URL");
    let sim = stack::start_sim().await;
    let stack = stack::start(&db.pool, &db.url(), sim).await;
    let ana = stack::seed_person(&db.pool, "Ana (empresa A)", "ana@empresa-a.example").await;
    let bruno = stack::seed_person(&db.pool, "Bruno (empresa B)", "bruno@empresa-b.example").await;
    let mut run = cases::Run::new(&stack, &db.pool, ana, bruno).await;
    let mut number = 0;
    run.connect_both(&mut number).await;
    run.read_mail(&mut number).await;
    run.upload_and_attach(&mut number).await;
    run.download_to_document(&mut number).await;
    run.references_stay_with_their_owner(&mut number).await;
    run.audit(&mut number).await;
    println!("m365-sim: {number} casos en verde");
    drop(run);
    drop(stack);
    db.cleanup().await;
}
