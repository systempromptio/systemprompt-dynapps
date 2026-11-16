//! The OneDrive read tools against a simulated Graph: the drive each one
//! addresses (`/me/drive` or `/drives/{id}`), drive-relative paths with
//! every segment percent-encoded, the search literal escaped, the Belgian
//! `$select`, `$top` and `$skip`, and ids or paths that try to leave the
//! drive refused before any request.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::drive::{
    DRIVE_ITEM_FIELDS, get_file_metadata, list_drives, list_file_permissions, list_files,
    search_files,
};
use systemprompt_mcp_microsoft_365::tools::reply::GraphList;

use crate::m365_pim_support::{args, graph, is_invalid};
use crate::m365_support::{Reply, Sim, collection};

async fn listing() -> Sim {
    Sim::start(|_, _| Reply::json(200, &collection(&[json!({"id": "i1"})], None))).await
}

#[tokio::test]
async fn m365_files_list_drives_reads_the_users_drives_with_skip() {
    let sim = listing().await;
    let reply = list_drives(&graph(&sim), args(json!({"skip": 5})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/drives");
    assert_eq!(calls[0].q("$skip"), Some("5"));
    assert_eq!(calls[0].q("$top"), None);
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": [{"id": "i1"}], "message": "success"})
    );
}

#[tokio::test]
async fn m365_files_list_files_addresses_the_root_or_an_encoded_folder() {
    let sim = listing().await;
    let g = graph(&sim);
    let root: GraphList = list_files(&g, args(json!({}))).await.unwrap();
    list_files(
        &g,
        args(json!({"folder_path": "/Docs/Q1 #2/", "drive_id": "b!drv-1", "top": 500, "skip": 3})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/drive/root/children");
    assert_eq!(calls[0].q("$select"), Some(DRIVE_ITEM_FIELDS));
    assert_eq!(calls[0].q("$top"), Some("25"));
    assert_eq!(
        calls[1].path,
        "/v1.0/drives/b!drv-1/root:/Docs/Q1%20%232:/children"
    );
    assert_eq!(calls[1].q("$top"), Some("50"), "top clamps to 50");
    assert_eq!(calls[1].q("$skip"), Some("3"));
    assert_eq!(root.data, vec![json!({"id": "i1"})]);
}

#[tokio::test]
async fn m365_files_search_files_escapes_the_query_literal() {
    let sim = listing().await;
    search_files(&graph(&sim), args(json!({"query": "it's Q1/2024 #final"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(
        calls[0].path,
        "/v1.0/me/drive/root/search(q='it%27%27s%20Q1%2F2024%20%23final')"
    );
    assert_eq!(calls[0].q("$select"), Some(DRIVE_ITEM_FIELDS));
    assert_eq!(calls[0].q("$top"), Some("25"));
}

#[tokio::test]
async fn m365_files_get_file_metadata_reads_one_item() {
    let sim = Sim::start(|_, _| Reply::json(200, &json!({"id": "01A", "name": "a.txt"}))).await;
    let g = graph(&sim);
    let mine = get_file_metadata(&g, args(json!({"item_id": "01A"})))
        .await
        .unwrap();
    get_file_metadata(&g, args(json!({"item_id": "01A", "drive_id": "b!x"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/drive/items/01A");
    assert_eq!(calls[1].path, "/v1.0/drives/b!x/items/01A");
    assert_eq!(
        serde_json::to_value(&mine).unwrap(),
        json!({"data": {"id": "01A", "name": "a.txt"}, "message": "success"})
    );
}

#[tokio::test]
async fn m365_files_list_file_permissions_pages_with_skip() {
    let sim = listing().await;
    list_file_permissions(&graph(&sim), args(json!({"item_id": "01A", "skip": 2})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/drive/items/01A/permissions");
    assert_eq!(calls[0].q("$skip"), Some("2"));
}

#[tokio::test]
async fn m365_files_reads_refuse_ids_and_paths_that_leave_the_drive() {
    let sim = listing().await;
    let g = graph(&sim);
    let errors = [
        list_files(&g, args(json!({"folder_path": "Docs/../../x"})))
            .await
            .unwrap_err(),
        list_files(&g, args(json!({"drive_id": "b/../x"})))
            .await
            .unwrap_err(),
        search_files(&g, args(json!({"query": "x", "drive_id": "a\\b"})))
            .await
            .unwrap_err(),
        get_file_metadata(&g, args(json!({"item_id": "01A/children"})))
            .await
            .unwrap_err(),
        list_file_permissions(&g, args(json!({"item_id": ".."})))
            .await
            .unwrap_err(),
    ];
    assert!(errors.iter().all(is_invalid), "{errors:?}");
    assert!(sim.calls().is_empty());
}
