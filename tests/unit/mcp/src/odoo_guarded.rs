//! The guarded tools against an in-process Odoo: demo scope refuses before
//! anything is written, a demo record is unlinked, posted invoices block
//! anonymisation, and the client signs in once per key.

use std::sync::Arc;

use systemprompt_mcp_odoo::client::{OdooClient, OdooConnection};
use systemprompt_mcp_odoo::error::OdooError;
use systemprompt_mcp_odoo::server::call::OdooCall;
use systemprompt_mcp_odoo::server::guarded::erasure::anonymise;
use systemprompt_mcp_odoo::server::guarded::{LEAD, PARTNER, delete_record};

use crate::odoo_mock::{GOOD_KEY, OdooMock, UID};

async fn signed_in(mock: &OdooMock) -> OdooCall {
    let client = Arc::new(OdooClient::new().expect("client builds"));
    let conn = OdooConnection::new(&mock.url, "unit").expect("connection");
    let creds = client
        .sign_in(conn, "farm-bot".to_owned(), GOOD_KEY.to_owned())
        .await
        .expect("the good key signs in");
    OdooCall { client, creds }
}

fn lead_tagged(tag: &str) -> (serde_json::Value, serde_json::Value) {
    (
        serde_json::json!([{ "id": 41, "name": "Acme enquiry", "tag_ids": [5] }]),
        serde_json::json!([{ "id": 5, "name": tag }]),
    )
}

#[tokio::test]
async fn a_lead_without_a_demo_tag_is_refused_before_any_unlink() {
    let mock = OdooMock::start().await;
    let (lead, tag) = lead_tagged("customer");
    mock.put("crm.lead", &lead);
    mock.put("crm.tag", &tag);
    let call = signed_in(&mock).await;

    let err = delete_record(&call, LEAD, 41, "duplicate".to_owned())
        .await
        .expect_err("a real lead is refused under demo scope");

    assert!(err.message.contains("demo-"), "{}", err.message);
    assert!(mock.wrote("unlink").is_empty(), "nothing may be deleted");
}

#[tokio::test]
async fn a_demo_tagged_lead_is_unlinked_with_an_irreversible_notice() {
    let mock = OdooMock::start().await;
    let (lead, tag) = lead_tagged("demo-dedup");
    mock.put("crm.lead", &lead);
    mock.put("crm.tag", &tag);
    let call = signed_in(&mock).await;

    let (_artifact, summary) = delete_record(&call, LEAD, 41, "duplicate of 40".to_owned())
        .await
        .expect("a demo lead is deleted");

    assert!(summary.contains("irreversible"), "{summary}");
    assert_eq!(mock.wrote("unlink"), vec!["crm.lead".to_owned()]);
}

#[tokio::test]
async fn a_missing_record_is_reported_not_deleted() {
    let mock = OdooMock::start().await;
    let call = signed_in(&mock).await;

    let err = delete_record(&call, PARTNER, 99, "duplicate".to_owned())
        .await
        .expect_err("nothing to delete");

    assert!(
        err.message.contains("No partner with id 99"),
        "{}",
        err.message
    );
    assert!(mock.wrote("unlink").is_empty());
}

#[tokio::test]
async fn a_partner_with_a_posted_invoice_is_not_anonymised() {
    let mock = OdooMock::start().await;
    mock.put(
        "res.partner",
        &serde_json::json!([{ "id": 9, "name": "Erin Erasure", "category_id": [3] }]),
    );
    mock.put(
        "res.partner.category",
        &serde_json::json!([{ "id": 3, "name": "demo-gdpr" }]),
    );
    mock.put(
        "account.move",
        &serde_json::json!([
            { "id": 70, "name": "INV/2026/0001", "state": "posted", "move_type": "out_invoice" },
            { "id": 71, "name": "/", "state": "draft", "move_type": "out_invoice" }
        ]),
    );
    let call = signed_in(&mock).await;

    let err = anonymise(&call, 9, "art. 17 request".to_owned())
        .await
        .expect_err("posted invoices block erasure");

    assert!(err.message.contains("INV/2026/0001"), "{}", err.message);
    assert!(!err.message.contains("[71]"), "{}", err.message);
    assert!(mock.wrote("write").is_empty(), "nothing may be overwritten");
}

#[tokio::test]
async fn a_demo_partner_without_posted_entries_is_anonymised_with_its_leads() {
    let mock = OdooMock::start().await;
    mock.put(
        "res.partner",
        &serde_json::json!([{ "id": 9, "name": "Erin Erasure", "category_id": [3] }]),
    );
    mock.put(
        "res.partner.category",
        &serde_json::json!([{ "id": 3, "name": "demo-gdpr" }]),
    );
    mock.put(
        "crm.lead",
        &serde_json::json!([{ "id": 41, "name": "Erin's enquiry" }]),
    );
    let call = signed_in(&mock).await;

    let (_artifact, summary) = anonymise(&call, 9, "art. 17 request".to_owned())
        .await
        .expect("a demo partner is anonymised");

    assert!(summary.contains("irreversible"), "{summary}");
    let mut wrote = mock.wrote("write");
    wrote.sort();
    assert_eq!(wrote, vec!["crm.lead".to_owned(), "res.partner".to_owned()]);
}

#[tokio::test]
async fn the_client_signs_in_once_per_key_and_reuses_the_uid() {
    let mock = OdooMock::start().await;
    let client = OdooClient::new().expect("client builds");
    let conn = OdooConnection::new(&mock.url, "unit").expect("connection");

    for _ in 0..3 {
        let creds = client
            .sign_in(conn.clone(), "farm-bot".to_owned(), GOOD_KEY.to_owned())
            .await
            .expect("signs in");
        assert_eq!(i64::from(creds.uid), UID);
    }
    assert_eq!(mock.authentications(), 1);
}

#[tokio::test]
async fn a_rejected_key_is_an_access_error_naming_the_login() {
    let mock = OdooMock::start().await;
    let client = OdooClient::new().expect("client builds");
    let conn = OdooConnection::new(&mock.url, "unit").expect("connection");

    let err = client
        .sign_in(conn, "farm-bot".to_owned(), "wrong".to_owned())
        .await
        .expect_err("a bad key is refused");

    assert!(matches!(err, OdooError::AccessDenied(ref m) if m.contains("farm-bot")));
}

#[test]
fn a_credential_without_a_database_is_not_configured() {
    assert!(matches!(
        OdooConnection::new("https://odoo.example", " "),
        Err(OdooError::NotConfigured(_))
    ));
}
