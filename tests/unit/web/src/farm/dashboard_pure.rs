//! Pure halves of the Farm console's `dashboard` module.

use systemprompt_web_admin_agent_farm::dashboard::{NONE, count_or_dash, task_tiles};
use systemprompt_web_admin_agent_farm::models::TaskOut;

fn task(state: &str) -> (String, TaskOut) {
    (
        "t".to_owned(),
        TaskOut {
            id: "x".to_owned(),
            state: state.to_owned(),
            ..TaskOut::default()
        },
    )
}

#[test]
fn task_tiles_count_each_state_running_first() {
    let tasks = vec![
        task("succeeded"),
        task("failed"),
        task("succeeded"),
        task("running"),
    ];
    let tiles = task_tiles(Some(&tasks), "lab b");
    let labels: Vec<&str> = tiles.iter().map(|t| t.label.as_str()).collect();
    assert_eq!(labels, ["Tasks", "running", "succeeded", "failed"]);
    assert_eq!(tiles[0].value, "4");
    assert_eq!(tiles[2].value, "2");
    assert_eq!(tiles[3].tone, Some("err"));
    assert_eq!(tiles[2].href, "/admin/farm/tasks?tenant=lab%20b");
}

#[test]
fn an_unreadable_block_is_a_dash_that_still_links() {
    let tiles = task_tiles(None, "t");
    assert_eq!(tiles.len(), 1);
    assert_eq!(tiles[0].value, NONE);
    assert_eq!(tiles[0].href, "/admin/farm/tasks?tenant=t");
    assert_eq!(count_or_dash(None), NONE);
    assert_eq!(count_or_dash(Some(3)), "3");
}
