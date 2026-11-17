//! Pure halves of the Farm console's `cost` module.

use chrono::NaiveDate;
use systemprompt_web_admin_agent_farm::cost::{aggregate, resolve_range, sessions};
use systemprompt_web_admin_agent_farm::models::{TaskOut, TaskUsage};
use systemprompt_web_admin_agent_farm::types::SessionTotalRow;

fn task(agent: Option<&str>, owner: Option<&str>, session: Option<&str>) -> (String, TaskOut) {
    (
        "t1".to_owned(),
        TaskOut {
            id: "x".to_owned(),
            agent: agent.map(ToOwned::to_owned),
            owner: owner.map(ToOwned::to_owned),
            usage: Some(TaskUsage {
                // farm-api's own cost is never used.
                cost_eur: Some(99.0),
                gateway_session_id: session.map(ToOwned::to_owned),
                ..TaskUsage::default()
            }),
            ..TaskOut::default()
        },
    )
}

fn total(session: &str, micros: i64) -> SessionTotalRow {
    SessionTotalRow {
        session_id: session.to_owned(),
        calls: 2,
        input_tokens: 10,
        output_tokens: 5,
        cache_read_tokens: 100,
        cost_microdollars: micros,
        ..SessionTotalRow::default()
    }
}

fn day(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap_or_default()
}

#[test]
fn default_range_is_the_last_seven_days() {
    let r = resolve_range(None, None, day("2026-10-27"));
    assert_eq!((r.from, r.to), (day("2026-10-21"), day("2026-10-27")));
    assert!(r.notes.is_empty());
    assert_eq!(
        r.instants(),
        (
            "2026-10-21T00:00:00Z".to_owned(),
            "2026-10-28T00:00:00Z".to_owned()
        )
    );
}

#[test]
fn bad_and_reversed_dates_are_corrected_with_a_note() {
    let r = resolve_range(Some("yesterday"), Some("2026-10-27"), day("2026-10-27"));
    assert_eq!(r.from, day("2026-10-21"));
    assert!(r.notes[0].contains("not a date"));
    let r = resolve_range(Some("2026-10-27"), Some("2026-10-01"), day("2026-10-27"));
    assert_eq!((r.from, r.to), (day("2026-10-01"), day("2026-10-27")));
    assert!(r.notes[0].contains("swapped"));
}

#[test]
fn unlinked_tasks_are_counted_but_add_nothing() {
    let report = aggregate(
        &[
            task(Some("a"), Some("p"), Some("sess_a1")),
            task(Some("a"), Some("p"), None),
            task(None, None, Some("sess_b2")),
        ],
        &[total("sess_a1", 1_000_000), total("sess_b2", 500_000)],
    );
    assert_eq!(report.total_cost, "$1.50");
    assert_eq!(report.tasks_label, "3 tasks");
    assert_eq!(report.calls_label, "4 model calls");
    assert_eq!(report.unlinked_label.as_deref(), Some("1 without SP link"));
    assert_eq!(report.empty_label, None);
    assert_eq!(report.input_tokens, 20);
    assert_eq!(report.cache_read_tokens, 200);
    assert_eq!(report.by_agent[0].key, "a");
    assert_eq!(report.by_agent[0].tasks, 2);
    assert_eq!(report.by_agent[0].unlinked, 1);
    assert_eq!(report.by_agent[1].key, "free prompt");
    assert_eq!(report.by_person[1].key, "unknown");
}

#[test]
fn a_linked_task_with_no_calls_is_reported_and_sorts_last() {
    let report = aggregate(
        &[
            task(Some("a"), None, Some("sess_a1")),
            task(Some("b"), None, Some("sess_b2")),
        ],
        &[total("sess_b2", 2_000_000)],
    );
    assert_eq!(report.by_agent[0].key, "b");
    assert_eq!(report.by_agent[1].cost, "$0.00");
    assert_eq!(
        report.empty_label.as_deref(),
        Some("1 linked with no calls in systemprompt")
    );
    assert_eq!(report.unlinked_label, None);
}

#[test]
fn a_task_read_from_all_tenants_without_a_tenant_is_unknown_not_star() {
    let (_, mut t) = task(Some("a"), None, None);
    t.tenant = None;
    let report = aggregate(&[("*".to_owned(), t)], &[]);
    assert_eq!(report.by_tenant[0].key, "unknown");
    let (_, mut t) = task(Some("a"), None, None);
    t.tenant = None;
    let report = aggregate(&[("lab-b".to_owned(), t)], &[]);
    assert_eq!(report.by_tenant[0].key, "lab-b");
}

#[test]
fn sessions_are_asked_once_and_bad_ids_never() {
    let ids = sessions(&[
        task(None, None, Some("sess_a1")),
        task(None, None, Some("sess_a1")),
        task(None, None, Some("not-a-session")),
        task(None, None, None),
    ]);
    assert_eq!(ids, vec!["sess_a1".to_owned()]);
}
