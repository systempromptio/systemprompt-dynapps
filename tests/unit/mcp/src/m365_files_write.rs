//! The `OneDrive` write tools against a simulated Graph: the exact request
//! body of a new folder, a sharing link and a move or rename, the Belgian
//! reply of a delete Graph answers with 204, and the drive tools' slice of
//! the pinned Belgian surface.

use serde_json::json;
use systemprompt_mcp_microsoft_365::tools::drive_write::{
    create_folder, delete_file, move_file, share_file,
};

use crate::m365_pim_support::{args, assert_domain_registered, empty, graph, is_invalid};
use crate::m365_support::{Reply, Sim};

async fn echo() -> Sim {
    Sim::start(|seen, _| match seen.method.as_str() {
        "DELETE" => empty(204),
        _ => Reply::json(200, &json!({"id": "it1"})),
    })
    .await
}

#[test]
fn m365_files_every_onedrive_tool_is_registered_with_its_belgian_hints() {
    assert_domain_registered("onedrive", 12);
}

#[tokio::test]
async fn m365_files_create_folder_posts_a_folder_that_renames_on_conflict() {
    let sim = echo().await;
    let g = graph(&sim);
    create_folder(&g, args(json!({"parent_path": "/", "folder_name": "New"})))
        .await
        .unwrap();
    create_folder(
        &g,
        args(json!({"parent_path": "Documents/Reports", "folder_name": "2024", "drive_id": "b!d"})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("POST", "/v1.0/me/drive/root/children")
    );
    assert_eq!(
        calls[0].json(),
        json!({"name": "New", "folder": {}, "@microsoft.graph.conflictBehavior": "rename"})
    );
    assert_eq!(
        calls[1].path,
        "/v1.0/drives/b!d/root:/Documents/Reports:/children"
    );
}

#[tokio::test]
async fn m365_files_share_file_defaults_to_a_view_link_inside_the_organization() {
    let sim = echo().await;
    share_file(&graph(&sim), args(json!({"item_id": "01A"})))
        .await
        .unwrap();

    let calls = sim.calls();
    assert_eq!(calls[0].path, "/v1.0/me/drive/items/01A/createLink");
    assert_eq!(
        calls[0].json(),
        json!({"type": "view", "scope": "organization"}),
        "no public link unless one is asked for"
    );
}

#[tokio::test]
async fn m365_files_share_file_makes_a_public_link_only_when_asked_explicitly() {
    let sim = echo().await;
    share_file(
        &graph(&sim),
        args(json!({"item_id": "01A", "type": "edit", "scope": "anonymous"})),
    )
    .await
    .unwrap();

    assert_eq!(
        sim.calls()[0].json(),
        json!({"type": "edit", "scope": "anonymous"})
    );
}

#[tokio::test]
async fn m365_files_move_file_patches_only_what_is_given() {
    let sim = echo().await;
    let g = graph(&sim);
    move_file(
        &g,
        args(json!({"item_id": "01A", "destination_folder_id": "01F", "new_name": "b.txt"})),
    )
    .await
    .unwrap();
    move_file(&g, args(json!({"item_id": "01A", "new_name": "c.txt"})))
        .await
        .unwrap();
    let bad = move_file(
        &g,
        args(json!({"item_id": "01A", "destination_folder_id": "../root"})),
    )
    .await
    .unwrap_err();

    let calls = sim.calls();
    assert_eq!(calls.len(), 2, "the bad folder id never reaches Graph");
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("PATCH", "/v1.0/me/drive/items/01A")
    );
    assert_eq!(
        calls[0].json(),
        json!({"parentReference": {"id": "01F"}, "name": "b.txt"})
    );
    assert_eq!(calls[1].json(), json!({"name": "c.txt"}));
    assert!(is_invalid(&bad));
}

#[tokio::test]
async fn m365_files_delete_file_answers_like_the_belgian_server() {
    let sim = echo().await;
    let reply = delete_file(
        &graph(&sim),
        args(json!({"item_id": "01A", "drive_id": "b!d"})),
    )
    .await
    .unwrap();

    let calls = sim.calls();
    assert_eq!(
        (calls[0].method.as_str(), calls[0].path.as_str()),
        ("DELETE", "/v1.0/drives/b!d/items/01A")
    );
    assert_eq!(
        serde_json::to_value(&reply).unwrap(),
        json!({"data": {"deleted": true, "item_id": "01A"}, "message": "success"})
    );
}
