//! Pure halves of the Farm console's `views` module.

use std::collections::HashMap;

use systemprompt_web_admin_agent_farm::models::{ResultObject, TaskList, TaskOut, TaskResult, TaskUsage};
use systemprompt_web_admin_agent_farm::views::{TasksInput, format_duration, format_time, result_view, state_tone, task_row, tasks_page};

#[test]
fn tones_cover_every_farm_state() {
    assert_eq!(state_tone("succeeded"), "ok");
    assert_eq!(state_tone("failed"), "err");
    assert_eq!(state_tone("running"), "accent");
    assert_eq!(state_tone("queued"), "info");
    assert_eq!(state_tone("pending"), "info");
    assert_eq!(state_tone("cancelled"), "muted");
    assert_eq!(state_tone("expired"), "muted");
    assert_eq!(state_tone("something-new"), "warn");
}

#[test]
fn duration_renders_for_humans() {
    assert_eq!(format_duration(Some(22.0)), "22 s");
    assert_eq!(format_duration(Some(605.0)), "10 min 5 s");
    assert_eq!(format_duration(Some(3600.0)), "1 h 0 min");
    assert_eq!(format_duration(None), "—");
}

#[test]
fn times_are_utc_and_bad_values_pass_through() {
    assert_eq!(
        format_time(Some("2026-10-27T07:10:20Z")),
        "2026-10-27 07:10:20 UTC"
    );
    assert_eq!(format_time(Some("not a date")), "not a date");
    assert_eq!(format_time(None), "—");
}

#[test]
fn row_links_keep_the_tenant_and_label_free_prompts() {
    let task = TaskOut {
        id: "01a17c32-14a4".to_owned(),
        tenant: Some("lab-b".to_owned()),
        state: "running".to_owned(),
        usage: Some(TaskUsage {
            cost_eur: Some(0.25),
            gateway_session_id: Some("sess_9e95".to_owned()),
            ..TaskUsage::default()
        }),
        ..TaskOut::default()
    };
    let costs = HashMap::from([("sess_9e95".to_owned(), 4_557)]);
    let row = task_row(&task, "dynapps-internal", Some(&costs));
    assert_eq!(row.href, "/admin/farm/tasks/01a17c32-14a4?tenant=lab-b");
    assert_eq!(row.agent, "free prompt");
    assert!(row.is_prompt);
    // systemprompt's cost in USD; farm-api's cost_eur is ignored.
    assert_eq!(row.cost, "$0.0046");
    assert_eq!(row.owner, "—");
    // Without systemprompt, or without a gateway session: no cost.
    assert_eq!(
        task_row::<std::collections::hash_map::RandomState>(&task, "lab-b", None).cost,
        "—"
    );
    let unlinked = TaskOut {
        usage: None,
        ..task
    };
    assert_eq!(task_row(&unlinked, "lab-b", Some(&costs)).cost, "—");
}

#[test]
fn summary_shows_when_the_result_has_one() {
    let view = result_view(&TaskResult {
        summary: Some("Reviewed 3 files".to_owned()),
        result: Some(ResultObject {
            size_bytes: Some(5802),
            sha256: Some("abc".to_owned()),
        }),
    });
    assert_eq!(view.summary.as_deref(), Some("Reviewed 3 files"));
    assert_eq!(view.facts[0].value, "5802 bytes");
}

#[test]
fn next_page_link_carries_cursor_and_tenant() {
    let list = TaskList {
        items: Vec::new(),
        next_cursor: Some("159".to_owned()),
    };
    let ctx = tasks_page(&TasksInput {
        tenant: "lab-b",
        tenants: &[],
        cursor: Some("200"),
        list: Some(&list),
        costs: None,
    });
    assert_eq!(
        ctx.next_url.as_deref(),
        Some("/admin/farm/tasks?tenant=lab-b&cursor=159")
    );
    assert_eq!(
        ctx.first_url.as_deref(),
        Some("/admin/farm/tasks?tenant=lab-b")
    );
}
