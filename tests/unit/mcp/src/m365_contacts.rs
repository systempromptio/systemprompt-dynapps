//! The personal contact tools against a simulated Graph: list fields,
//! paging and a quoted `$search` under `ConsistencyLevel: eventual` (on the
//! cursor page too), the create and update bodies, the delete reply, and
//! ids refused before Graph.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::contacts::{
    CONTACT_FIELDS, create_contact, delete_contact, get_contact, list_contacts, update_contact,
};

use crate::m365_pim_support::{args, assert_domain_registered, empty, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

async fn echo() -> Sim {
    Sim::start(|seen, _| match seen.method.as_str() {
        "DELETE" => empty(204),
        _ => Reply::json(200, &json!({"id": "c1", "givenName": "Ana"})),
    })
    .await
}

#[test]
fn m365_contacts_all_five_belgian_tools_are_registered_as_pinned() {
    assert_domain_registered("contacts", 5);
}

#[tokio::test]
async fn m365_contacts_list_selects_fields_and_pages() {
    let sim = Sim::start(|_, _| Reply::json(200, &collection(&[json!({"id": "c1"})], None))).await;
    let reply = list_contacts(&graph(&sim), args(json!({"top": 0, "skip": 10})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.path, "/v1.0/me/contacts");
    assert_eq!(call.q("$select"), Some(CONTACT_FIELDS));
    assert_eq!(call.q("$top"), Some("1"));
    assert_eq!(call.q("$skip"), Some("10"));
    assert_eq!(call.q("$search"), None);
    assert_eq!(call.header("consistencylevel"), None);
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": [{"id": "c1"}], "message": "success"})
    );
}

#[tokio::test]
async fn m365_contacts_search_is_quoted_and_eventual_on_every_page() {
    let sim = Sim::start(|seen, index| {
        if index == 0 {
            let next = format!(
                "http://{}/v1.0/me/contacts?page=2",
                seen.header("host").unwrap()
            );
            return Reply::json(200, &collection(&[], Some(&next)));
        }
        Reply::json(200, &collection(&[json!({"id": "c9"})], None))
    })
    .await;
    let reply = list_contacts(&graph(&sim), args(json!({"search": "o'brien \"jr\""})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2, "an empty page with a cursor is followed");
    assert_eq!(calls[0].q("$search"), Some("\"o''brien \\\"jr\\\"\""));
    assert_eq!(calls[0].q("$top"), Some("25"));
    assert!(
        calls
            .iter()
            .all(|c| c.header("consistencylevel") == Some("eventual"))
    );
    assert_eq!(reply.data, vec![json!({"id": "c9"})]);
}

#[tokio::test]
async fn m365_contacts_get_reads_the_whole_contact() {
    let sim = echo().await;
    let reply = get_contact(&graph(&sim), args(json!({"contact_id": "AAMkC="})))
        .await
        .unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "GET");
    assert_eq!(call.path, "/v1.0/me/contacts/AAMkC=");
    assert_eq!(call.raw_query, None);
    assert_eq!(reply.data["givenName"], "Ana");
}

#[tokio::test]
async fn m365_contacts_create_sends_only_the_fields_given() {
    let sim = echo().await;
    let full = args(json!({
        "given_name": "Ana",
        "surname": "Ruiz",
        "email": "ana@contoso.com",
        "phone": "+34 600 000 000",
        "company": "Contoso",
        "job_title": "CFO"
    }));
    create_contact(&graph(&sim), full).await.unwrap();
    create_contact(&graph(&sim), args(json!({"given_name": "Bo"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].method, "POST");
    assert_eq!(calls[0].path, "/v1.0/me/contacts");
    assert_eq!(
        calls[0].json(),
        json!({
            "givenName": "Ana",
            "surname": "Ruiz",
            "emailAddresses": [{"address": "ana@contoso.com"}],
            "businessPhones": ["+34 600 000 000"],
            "companyName": "Contoso",
            "jobTitle": "CFO"
        })
    );
    assert_eq!(calls[1].json(), json!({"givenName": "Bo"}));
}

#[tokio::test]
async fn m365_contacts_update_patches_only_what_was_given() {
    let sim = echo().await;
    let input = args(json!({"contact_id": "c1", "email": "new@contoso.com", "surname": ""}));
    let reply = update_contact(&graph(&sim), input).await.unwrap();

    let call = &sim.calls()[0];
    assert_eq!(call.method, "PATCH");
    assert_eq!(call.path, "/v1.0/me/contacts/c1");
    assert_eq!(
        call.json(),
        json!({"surname": "", "emailAddresses": [{"address": "new@contoso.com"}]})
    );
    assert_eq!(reply.data["id"], "c1");
}

#[tokio::test]
async fn m365_contacts_delete_answers_like_the_belgian_server_and_checks_ids() {
    let sim = echo().await;
    let reply = delete_contact(&graph(&sim), args(json!({"contact_id": "c1"})))
        .await
        .unwrap();
    let refused = delete_contact(&graph(&sim), args(json!({"contact_id": "c1/../c2"})))
        .await
        .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].method, "DELETE");
    assert_eq!(calls[0].path, "/v1.0/me/contacts/c1");
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": {"deleted": true, "contact_id": "c1"}, "message": "success"})
    );
    assert!(is_invalid(&refused));
}
