//! The runs explorer's view rules: status and trigger badges keyed by the
//! stored text, which statuses count as live, and how the page's query maps
//! to the repository filter and sort.

use chrono::{Duration, Utc};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin_farm::repositories::filters::RunSort;
use systemprompt_web_admin_farm::types::{FarmAgentId, FarmId, RunStatus, TriggeredVia};
use systemprompt_web_admin_farm::views::runs_query::{
    RunSortColumn, RunsQuery, run_filter, run_sort,
};
use systemprompt_web_admin_farm::views::status::{is_live, status_badge, trigger_badge};

fn query(pairs: &[(&str, &str)]) -> RunsQuery {
    let mut q = RunsQuery::default();
    for (name, value) in pairs {
        let v = Some((*value).to_owned());
        match *name {
            "farm_id" => q.farm_id = v,
            "agent_id" => q.agent_param = v,
            "user_id" => q.user_param = v,
            "status" => q.status = v,
            "trigger" => q.trigger = v,
            "sort" => q.sort = v,
            "dir" => q.dir = v,
            "preset" => q.preset = v,
            _ => {},
        }
    }
    q
}

#[test]
fn every_status_has_its_own_tone_and_label() {
    let cases = [
        ("queued", "muted", "Queued"),
        ("preparing", "info", "Preparing"),
        ("running", "accent", "Running"),
        ("succeeded", "ok", "Succeeded"),
        ("failed", "err", "Failed"),
        ("cancelled", "muted", "Cancelled"),
        ("timed_out", "warn", "Timed out"),
    ];
    for (key, tone, text) in cases {
        let badge = status_badge(key);
        assert_eq!(badge.tone, tone, "{key}");
        assert_eq!(badge.text, text, "{key}");
        assert_eq!(badge.key, key);
    }
}

#[test]
fn an_unknown_status_reads_unknown_and_keeps_its_key() {
    let badge = status_badge("exploded");
    assert_eq!(badge.tone, "muted");
    assert_eq!(badge.text, "Unknown");
    assert_eq!(badge.key, "exploded");
}

#[test]
fn trigger_badges_name_where_the_run_came_from() {
    assert_eq!(trigger_badge("dashboard").text, "Dashboard");
    assert_eq!(trigger_badge("api").text, "API");
    assert_eq!(trigger_badge("api").tone, "accent");
    assert_eq!(trigger_badge("eval").text, "Eval");
    assert_eq!(trigger_badge("cron").text, "Unknown");
}

#[test]
fn only_unfinished_statuses_are_live() {
    for live in ["queued", "preparing", "running"] {
        assert!(is_live(live), "{live}");
    }
    for done in ["succeeded", "failed", "cancelled", "timed_out", "bogus"] {
        assert!(!is_live(done), "{done}");
    }
}

#[test]
fn query_maps_every_filter_onto_the_repository_filter() {
    let until = Utc::now();
    let since = until - Duration::days(7);
    let q = query(&[
        ("farm_id", "delivery"),
        ("agent_id", "reviewer"),
        ("user_id", "user-1"),
        ("status", "timed_out"),
        ("trigger", "api"),
    ]);
    let f = run_filter(&q, since, until);
    assert_eq!(f.farm_id.as_ref().map(FarmId::as_str), Some("delivery"));
    assert_eq!(
        f.agent_id.as_ref().map(FarmAgentId::as_str),
        Some("reviewer")
    );
    assert_eq!(f.user_id.as_ref().map(UserId::as_str), Some("user-1"));
    assert_eq!(f.status, Some(RunStatus::TimedOut));
    assert_eq!(f.triggered_via, Some(TriggeredVia::Api));
    assert_eq!(f.since, Some(since));
    assert_eq!(f.until, Some(until));
}

#[test]
fn empty_and_unknown_values_widen_the_filter_instead_of_emptying_it() {
    let now = Utc::now();
    let q = query(&[
        ("farm_id", ""),
        ("user_id", ""),
        ("status", "exploded"),
        ("trigger", "cron"),
    ]);
    let f = run_filter(&q, now, now);
    assert!(f.farm_id.is_none());
    assert!(f.user_id.is_none());
    assert!(f.status.is_none());
    assert!(f.triggered_via.is_none());
}

#[test]
fn sort_defaults_to_newest_first_and_only_start_time_reverses() {
    assert_eq!(
        run_sort(&query(&[])),
        (RunSortColumn::Started, RunSort::CreatedDesc)
    );
    assert_eq!(
        run_sort(&query(&[("sort", "started"), ("dir", "asc")])),
        (RunSortColumn::Started, RunSort::CreatedAsc)
    );
    assert_eq!(
        run_sort(&query(&[("sort", "duration"), ("dir", "asc")])),
        (RunSortColumn::Duration, RunSort::DurationDesc)
    );
    assert_eq!(run_sort(&query(&[("sort", "turns")])).1, RunSort::TurnsDesc);
    assert_eq!(run_sort(&query(&[("sort", "cost")])).1, RunSort::CostDesc);
    assert_eq!(
        run_sort(&query(&[("sort", "nonsense")])).0,
        RunSortColumn::Started
    );
}

#[test]
fn rebuilt_urls_drop_only_what_they_are_asked_to() {
    let q = query(&[
        ("preset", "7d"),
        ("farm_id", "delivery"),
        ("status", "failed"),
    ]);
    assert_eq!(
        q.url_without(&["status"]),
        "/admin/farm/runs?preset=7d&farm_id=delivery"
    );
    assert_eq!(
        q.url_prefix(&["farm_id", "status"]),
        "/admin/farm/runs?preset=7d&"
    );
    assert_eq!(query(&[]).url_without(&[]), "/admin/farm/runs");
}
