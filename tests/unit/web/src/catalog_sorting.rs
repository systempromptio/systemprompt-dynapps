#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]
//! The pure half of the platform list pages: sort-link construction, the
//! server-side text filter, and the MCP status a declaration plus a heartbeat
//! resolve to.

use systemprompt_web_admin::repositories::mcp::runtime::McpServerLiveness;
use systemprompt_web_admin::test_support::{
    SortColumn, direction, matches, preserved_search, sort_headers, status_of,
};

fn columns() -> Vec<SortColumn> {
    vec![
        SortColumn {
            key: "name",
            label: "Name",
            class: "",
            hint: "the id",
        },
        SortColumn {
            key: "calls",
            label: "Calls",
            class: "sp-table__cell--num",
            hint: "call volume",
        },
    ]
}

fn liveness(open: i64, alive: i64) -> McpServerLiveness {
    McpServerLiveness {
        server_id: "s".to_owned(),
        sessions_open: open,
        sessions_alive: alive,
        distinct_users: 0,
        last_activity_at: None,
        proxy_identities: 0,
    }
}

#[test]
fn direction_normalises_to_two_values() {
    assert_eq!(direction(Some("asc")), "asc");
    assert_eq!(direction(Some("desc")), "desc");
    assert_eq!(direction(None), "desc");
    assert_eq!(direction(Some("sideways")), "desc");
}

#[test]
fn an_inactive_column_opens_descending_and_the_active_one_flips() {
    let headers = sort_headers("/admin/mcp", &columns(), "calls", "desc", "");
    let name = &headers[0];
    let calls = &headers[1];

    assert!(!name.active);
    assert_eq!(name.aria_sort, "none");
    assert!(name.url.ends_with("sort=name&dir=desc"));

    assert!(calls.active);
    assert_eq!(calls.aria_sort, "descending");
    assert!(calls.url.ends_with("sort=calls&dir=asc"));
}

#[test]
fn an_ascending_active_column_reports_ascending() {
    let headers = sort_headers("/admin/mcp", &columns(), "name", "asc", "");
    assert_eq!(headers[0].aria_sort, "ascending");
    assert_eq!(headers[0].indicator, "\u{25b2}");
}

// Why: a sort link that dropped the filter would silently widen the listing
// the operator is looking at, which is the one thing a sort must never do.
#[test]
fn sort_links_carry_the_active_filter() {
    let headers = sort_headers("/admin/mcp", &columns(), "calls", "desc", "q=jira");
    assert!(headers[0].url.starts_with("/admin/mcp?q=jira&sort="));
}

#[test]
fn preserved_search_encodes_and_omits_the_empty_filter() {
    assert_eq!(preserved_search(""), "");
    assert_eq!(preserved_search("knowledge bank"), "q=knowledge%20bank");
}

#[test]
fn the_filter_is_case_insensitive_and_matches_any_field() {
    assert!(matches(&["knowledge-bank", "Atlassian search"], "ATLAS"));
    assert!(matches(&["jira"], ""));
    assert!(!matches(&["jira", "issues"], "confluence"));
}

#[test]
fn an_undeclared_server_is_flagged_however_it_is_behaving() {
    assert_eq!(
        status_of(false, true, Some(&liveness(3, 3))).0,
        "Unconfigured"
    );
    assert_eq!(status_of(false, false, None).0, "Unconfigured");
}

#[test]
fn a_declared_server_reads_alive_stale_or_never_connected() {
    assert_eq!(status_of(true, true, Some(&liveness(2, 1))).0, "Alive");
    assert_eq!(status_of(true, true, Some(&liveness(2, 0))).0, "Stale");
    assert_eq!(status_of(true, true, None).0, "Never connected");
    assert_eq!(status_of(true, false, Some(&liveness(2, 2))).0, "Disabled");
}

#[test]
fn the_status_tone_matches_the_status() {
    assert_eq!(status_of(true, true, Some(&liveness(1, 1))).1, "ok");
    assert_eq!(status_of(true, true, Some(&liveness(1, 0))).1, "warn");
    assert_eq!(status_of(false, true, None).1, "warn");
    assert_eq!(status_of(true, false, None).1, "muted");
}
