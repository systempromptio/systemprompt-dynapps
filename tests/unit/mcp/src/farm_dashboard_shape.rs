//! `farm_dashboard`'s JSON shape, pinned key by key, and the agent-farm
//! dashboard artifact (`services/artifacts/agent-farm/view.html`) reading
//! only fields that shape carries. A renamed or dropped field fails here
//! instead of rendering `undefined` in the artifact.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde_json::Value;
use systemprompt::identifiers::UserId;
use systemprompt_mcp_agent_farm::typed::{
    DashboardAgent, DashboardClient, DashboardDay, DashboardRun, DashboardTotals,
    FarmDashboardOutput,
};

fn sample() -> Value {
    let output = FarmDashboardOutput {
        scope: "all".to_owned(),
        window: "30d".to_owned(),
        client: String::new(),
        checked_at: "2026-10-15T00:00:00Z".to_owned(),
        totals: DashboardTotals::default(),
        clients: vec![DashboardClient {
            client: "dynapps".to_owned(),
            name: "Acme Logistics".to_owned(),
            runs: 2,
            succeeded: 1,
            finished: 2,
            active: 0,
            people: 1,
            success_pct: Some(50.0),
            completed: 1,
            reported: 2,
            eval_mean: Some(90.0),
            eval_scored: 1,
            infra_failures: 1,
            cost_microdollars: 1,
            last_run_at: None,
        }],
        agents: vec![DashboardAgent {
            farm: "f".to_owned(),
            agent: "a".to_owned(),
            runs: 1,
            succeeded: 1,
            finished: 1,
            success_pct: None,
            completed: 1,
            eval_mean: None,
            eval_scored: 0,
            infra_failures: 0,
            cost_microdollars: 0,
            last_run_at: "2026-10-15T00:00:00Z".to_owned(),
        }],
        daily: vec![DashboardDay {
            day: "2026-10-15".to_owned(),
            runs: 1,
            succeeded: 1,
            finished: 1,
            cost_microdollars: 0,
            eval_mean: None,
            eval_scored: 0,
            infra_failures: 0,
        }],
        recent_runs: vec![DashboardRun {
            farm_run_id: "r".to_owned(),
            farm: "f".to_owned(),
            agent: "a".to_owned(),
            client: None,
            client_name: None,
            launched_by: UserId::new(uuid_like()),
            status: "succeeded".to_owned(),
            triggered_via: "mcp".to_owned(),
            completed: None,
            summary: None,
            eval_passed: None,
            eval_score: None,
            cost_microdollars: 0,
            created_at: "2026-10-15T00:00:00Z".to_owned(),
            finished_at: None,
        }],
    };
    serde_json::to_value(output).expect("dashboard output serialises")
}

fn uuid_like() -> String {
    "00000000-0000-4000-8000-000000000001".to_owned()
}

fn keys(value: &Value) -> BTreeSet<String> {
    value
        .as_object()
        .expect("a JSON object")
        .keys()
        .cloned()
        .collect()
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

const TOP: &[&str] = &[
    "scope",
    "window",
    "client",
    "checked_at",
    "totals",
    "clients",
    "agents",
    "daily",
    "recent_runs",
];
const TOTALS: &[&str] = &[
    "runs",
    "succeeded",
    "finished",
    "success_pct",
    "cost_microdollars",
    "eval_mean",
    "eval_scored",
    "infra_failures",
    "completed",
    "reported",
];
const CLIENT: &[&str] = &[
    "client",
    "name",
    "runs",
    "succeeded",
    "finished",
    "active",
    "people",
    "success_pct",
    "completed",
    "reported",
    "eval_mean",
    "eval_scored",
    "infra_failures",
    "cost_microdollars",
    "last_run_at",
];
const AGENT: &[&str] = &[
    "farm",
    "agent",
    "runs",
    "succeeded",
    "finished",
    "success_pct",
    "completed",
    "eval_mean",
    "eval_scored",
    "infra_failures",
    "cost_microdollars",
    "last_run_at",
];
const DAY: &[&str] = &[
    "day",
    "runs",
    "succeeded",
    "finished",
    "cost_microdollars",
    "eval_mean",
    "eval_scored",
    "infra_failures",
];
const RUN: &[&str] = &[
    "farm_run_id",
    "farm",
    "agent",
    "client",
    "client_name",
    "launched_by",
    "status",
    "triggered_via",
    "completed",
    "summary",
    "eval_passed",
    "eval_score",
    "cost_microdollars",
    "created_at",
    "finished_at",
];

#[test]
fn the_dashboard_json_shape_is_pinned() {
    let v = sample();
    assert_eq!(keys(&v), set(TOP));
    assert_eq!(keys(&v["totals"]), set(TOTALS));
    assert_eq!(keys(&v["clients"][0]), set(CLIENT));
    assert_eq!(keys(&v["agents"][0]), set(AGENT));
    assert_eq!(keys(&v["daily"][0]), set(DAY));
    assert_eq!(keys(&v["recent_runs"][0]), set(RUN));
}

fn view_html() -> String {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let candidate = dir.join("services/artifacts/agent-farm/view.html");
        if candidate.is_file() {
            return std::fs::read_to_string(candidate).expect("read view.html");
        }
        assert!(
            dir.pop(),
            "repo root with services/artifacts/agent-farm not found"
        );
    }
}

// Every `<var>.<field>` the artifact's render code reads, by the variable
// it binds each object to: `t` totals, `c` a client, `a` an agent, `r` a
// recent run, `d` the top-level response (or a day in `trend`).
const READS: &[(&str, &[&str], &[&str])] = &[
    (
        "t",
        TOTALS,
        &[
            "runs",
            "finished",
            "success_pct",
            "succeeded",
            "completed",
            "reported",
            "eval_mean",
            "eval_scored",
            "infra_failures",
            "cost_microdollars",
        ],
    ),
    (
        "c",
        CLIENT,
        &[
            "client",
            "name",
            "runs",
            "success_pct",
            "completed",
            "reported",
            "eval_mean",
            "infra_failures",
            "people",
            "cost_microdollars",
            "last_run_at",
        ],
    ),
    (
        "a",
        AGENT,
        &[
            "farm",
            "agent",
            "runs",
            "success_pct",
            "completed",
            "eval_mean",
            "infra_failures",
            "cost_microdollars",
            "last_run_at",
        ],
    ),
    (
        "r",
        RUN,
        &[
            "created_at",
            "farm",
            "agent",
            "client_name",
            "launched_by",
            "status",
            "completed",
            "eval_passed",
            "eval_score",
            "summary",
            "cost_microdollars",
        ],
    ),
];

#[test]
fn the_artifact_reads_only_fields_the_tool_returns() {
    let html = view_html();
    for (var, shape, fields) in READS {
        for field in *fields {
            assert!(
                shape.contains(field),
                "{var}.{field} is read by the artifact but not returned by farm_dashboard"
            );
            assert!(
                html.contains(&format!("{var}.{field}")),
                "the artifact no longer reads {var}.{field}; update READS"
            );
        }
    }
    for top in [
        "scope",
        "window",
        "checked_at",
        "client",
        "totals",
        "daily",
        "recent_runs",
    ] {
        assert!(TOP.contains(&top));
        assert!(
            html.contains(&format!("d.{top}")),
            "the artifact no longer reads d.{top}"
        );
    }
    for day in ["day", "runs", "succeeded", "finished"] {
        assert!(DAY.contains(&day));
    }
}
