//! Pure halves of the Farm console's `schedules` module.

use systemprompt_web_admin_agent_farm::models::{Recurrence, ScheduleOut, ScheduleTask};
use systemprompt_web_admin_agent_farm::schedules::schedule_row;

#[test]
fn a_paused_schedule_has_no_next_fire_and_a_skipped_run_warns() {
    let row = schedule_row(
        &ScheduleOut {
            schedule_id: "01a1a000-0000".to_owned(),
            state: "paused".to_owned(),
            last_fire_state: Some("skipped_quota".to_owned()),
            missed_runs: Some(3),
            runs_count: Some(40),
            recurrence: Some(Recurrence {
                cron: "*/30 * * * *".to_owned(),
                max_runs: Some(100),
                ..Recurrence::default()
            }),
            task: Some(ScheduleTask::default()),
            ..ScheduleOut::default()
        },
        "lab-b",
    );
    assert_eq!(row.next_fire, "—");
    assert_eq!(row.state_tone, "warn");
    assert_eq!(row.last_tone, "warn");
    assert_eq!(row.runs, "40 (3 missed)");
    assert_eq!(row.window.as_deref(), Some("at most 100 runs"));
    assert_eq!(row.agent, "free prompt");
    assert_eq!(row.tenant, "lab-b");
}

#[test]
fn a_schedule_that_never_fired_says_so() {
    let row = schedule_row(&ScheduleOut::default(), "t");
    assert_eq!(row.last_state, "never fired");
    assert_eq!(row.last_fired, "—");
}
