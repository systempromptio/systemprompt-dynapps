//! The `SharePoint` tools against a simulated Graph: the site, list and item
//! paths, the Belgian `search`, `$select` and `$expand`, the list item
//! bodies passed through as given, host names and ids that are not what
//! they claim refused before any request, and the domain's slice of the
//! pinned Belgian surface.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::sharepoint::{
    LIST_FIELDS, SITE_FIELDS, create_sharepoint_list_item, get_sharepoint_site,
    get_sharepoint_site_by_path, list_sharepoint_list_items, list_sharepoint_lists,
    list_site_drives, search_sharepoint_sites, update_sharepoint_list_item,
};

use crate::m365_pim_support::{args, assert_domain_registered, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

const SITE: &str = "contoso.sharepoint.com,2c712f2b-aaaa,9b5c-bbbb";

async fn sharepoint() -> Sim {
    Sim::start(|seen, _| match seen.method.as_str() {
        "GET" if !seen.path.ends_with("/v1.0/sites/s1") && !seen.path.contains(".com") => {
            Reply::json(200, &collection(&[json!({"id": "x"})], None))
        },
        _ => Reply::json(200, &json!({"id": "x"})),
    })
    .await
}

#[test]
fn m365_sharepoint_every_tool_is_registered_with_its_belgian_hints() {
    assert_domain_registered("sharepoint", 8);
}

#[tokio::test]
async fn m365_sharepoint_search_sites_sends_the_keyword_and_site_fields() {
    let sim = sharepoint().await;
    let reply = search_sharepoint_sites(
        &graph(&sim),
        args(json!({"query": "it's sales", "skip": 4})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/sites");
    assert_eq!(calls[0].q("search"), Some("it''s sales"));
    assert_eq!(calls[0].q("$select"), Some(SITE_FIELDS));
    assert_eq!(calls[0].q("$skip"), Some("4"));
    assert_eq!(reply.data, vec![json!({"id": "x"})]);
}

#[tokio::test]
async fn m365_sharepoint_get_site_takes_a_composite_site_id() {
    let sim = sharepoint().await;
    let reply = get_sharepoint_site(&graph(&sim), args(json!({"site_id": SITE})))
        .await
        .unwrap();

    assert_eq!(sim.calls()[0].path, format!("/v1.0/sites/{SITE}"));
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": {"id": "x"}, "message": "success"})
    );
}

#[tokio::test]
async fn m365_sharepoint_get_site_by_path_addresses_host_and_encoded_path() {
    let sim = sharepoint().await;
    let g = graph(&sim);
    get_sharepoint_site_by_path(
        &g,
        args(json!({"hostname": "contoso.sharepoint.com", "site_path": "/sites/Sales Team/"})),
    )
    .await
    .unwrap();
    get_sharepoint_site_by_path(
        &g,
        args(json!({"hostname": "contoso.sharepoint.com", "site_path": "/"})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(
        calls[0].path,
        "/v1.0/sites/contoso.sharepoint.com:/sites/Sales%20Team"
    );
    assert_eq!(calls[1].path, "/v1.0/sites/contoso.sharepoint.com");
}

#[tokio::test]
async fn m365_sharepoint_list_lists_and_site_drives_page_with_skip() {
    let sim = sharepoint().await;
    let g = graph(&sim);
    list_sharepoint_lists(&g, args(json!({"site_id": "s1", "skip": 2})))
        .await
        .unwrap();
    list_site_drives(&g, args(json!({"site_id": "s1"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/sites/s1/lists");
    assert_eq!(calls[0].q("$select"), Some(LIST_FIELDS));
    assert_eq!(calls[0].q("$skip"), Some("2"));
    assert_eq!(calls[1].path, "/v1.0/sites/s1/drives");
    assert_eq!(calls[1].q("$select"), None);
}

#[tokio::test]
async fn m365_sharepoint_list_items_expands_fields_unless_told_not_to() {
    let sim = sharepoint().await;
    let g = graph(&sim);
    list_sharepoint_list_items(&g, args(json!({"site_id": "s1", "list_id": "l1"})))
        .await
        .unwrap();
    list_sharepoint_list_items(
        &g,
        args(json!({"site_id": "s1", "list_id": "l1", "expand_fields": false, "top": 7})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/sites/s1/lists/l1/items");
    assert_eq!(calls[0].q("$expand"), Some("fields"));
    assert_eq!(calls[0].q("$top"), Some("25"));
    assert_eq!(calls[1].q("$expand"), None);
    assert_eq!(calls[1].q("$top"), Some("7"));
}

#[tokio::test]
async fn m365_sharepoint_create_and_update_list_items_send_the_fields_as_given() {
    let sim = sharepoint().await;
    let g = graph(&sim);
    let fields = json!({"Title": "Q1", "Amount": 12.5, "Done": false});
    create_sharepoint_list_item(
        &g,
        args(json!({"site_id": "s1", "list_id": "l1", "fields": fields})),
    )
    .await
    .unwrap();
    update_sharepoint_list_item(
        &g,
        args(json!({"site_id": "s1", "list_id": "l1", "item_id": "7", "fields": {"Done": true}})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("POST", "/v1.0/sites/s1/lists/l1/items")
    );
    assert_eq!(calls[0].json(), json!({"fields": fields}));
    assert_eq!(
        (calls[1].method.as_str(), calls[1].path.as_str()),
        ("PATCH", "/v1.0/sites/s1/lists/l1/items/7/fields")
    );
    assert_eq!(calls[1].json(), json!({"Done": true}));
}

#[tokio::test]
async fn m365_sharepoint_refuses_bad_hosts_paths_and_ids_before_graph() {
    let sim = sharepoint().await;
    let g = graph(&sim);
    let errors = [
        get_sharepoint_site_by_path(&g, args(json!({"hostname": "evil/x", "site_path": "/"})))
            .await
            .unwrap_err(),
        get_sharepoint_site_by_path(
            &g,
            args(json!({"hostname": "contoso.sharepoint.com", "site_path": "/sites/../x"})),
        )
        .await
        .unwrap_err(),
        get_sharepoint_site(&g, args(json!({"site_id": "s1/lists"})))
            .await
            .unwrap_err(),
        list_sharepoint_list_items(&g, args(json!({"site_id": "s1", "list_id": "../l"})))
            .await
            .unwrap_err(),
        update_sharepoint_list_item(
            &g,
            args(json!({"site_id": "s1", "list_id": "l1", "item_id": "7?x", "fields": {}})),
        )
        .await
        .unwrap_err(),
    ];
    assert!(errors.iter().all(is_invalid), "{errors:?}");
    assert!(sim.calls().is_empty());
}
