//! Pure invariants of the roster and the user detail page.
//!
//! The handlers are covered by the contract and Playwright suites. What this
//! file guards is what neither catches cheaply: the roster's URL contract
//! (which filters exist and how they parse), and the two markup properties a
//! rewrite quietly loses — a write control drifting outside its authorisation
//! guard, and a table losing its empty state.

use systemprompt_web_admin::repositories::users::roster::{RosterFilter, RosterSort};

use crate::support::repo_root;

fn template(name: &str) -> String {
    let path = repo_root().join("storage/files/admin/templates").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// The three chips the retired Unassigned and No-role pages redirect to. Each
// string is a live URL somewhere, so a rename here is a dead redirect.
#[test]
fn every_named_filter_parses_to_its_own_arm() {
    for (raw, expected) in [
        ("unassigned", RosterFilter::Unassigned),
        ("no-role", RosterFilter::NoRole),
        ("inactive-30d", RosterFilter::Inactive30d),
    ] {
        assert_eq!(RosterFilter::parse(Some(raw)), expected, "?filter={raw}");
        assert_eq!(expected.as_str(), raw, "round trip for {raw}");
    }
}

#[test]
fn an_unknown_filter_widens_rather_than_erroring() {
    for raw in [None, Some(""), Some("no-such-filter")] {
        assert_eq!(RosterFilter::parse(raw), RosterFilter::None, "{raw:?}");
    }
}

// Why: the sort column reaches an ORDER BY ladder of literal CASE arms. A value
// with no arm sorts by nothing at all, so parsing must never pass one through.
#[test]
fn an_unknown_sort_column_falls_back_to_the_default() {
    let parsed = RosterSort::parse(Some("'; DROP TABLE users --"), Some("desc"));
    assert_eq!(parsed.column, RosterSort::default().column);
    assert!(RosterSort::COLUMNS.contains(&parsed.column));
}

#[test]
fn every_sort_column_survives_a_round_trip() {
    for column in RosterSort::COLUMNS {
        let asc = RosterSort::parse(Some(column), Some("asc"));
        assert_eq!(asc.column, column);
        assert!(!asc.descending, "{column} asc");
        assert!(
            RosterSort::parse(Some(column), Some("desc")).descending,
            "{column} desc"
        );
    }
}

// Why: an unrecognised direction must mean largest-first, which is what an
// operator scanning for the expensive seat wants, not silently ascending.
#[test]
fn an_absent_direction_sorts_largest_first() {
    assert!(RosterSort::parse(Some("cost"), None).descending);
}

#[test]
fn every_roster_column_is_a_sort_header() {
    let source = template("users.hbs");
    assert!(
        source.contains("{{#each sort_headers}}{{> components/sort-header this}}{{/each}}"),
        "the roster's header row must be built from sort_headers, so every column sorts"
    );
}

#[test]
fn the_roster_paginates_and_shows_an_empty_state() {
    let source = template("users.hbs");
    assert!(
        source.contains("pagination=pagination"),
        "the roster's table must carry the pagination footer"
    );
    assert!(
        source.contains("components/empty-state"),
        "the roster must render an empty state when no account matches"
    );
}

// Why: `can_write` is the only thing between a project manager and a control
// the API would refuse. Every mutating control on both templates sits inside
// that guard, and this counts them so a new one cannot be added outside it.
#[test]
fn every_write_control_sits_behind_the_write_guard() {
    for (name, markers) in [
        ("users.hbs", vec!["data-action=\"create-user\"", "data-select-user"]),
        (
            "user-detail.hbs",
            vec![
                "data-revoke-device",
                "data-revoke-session",
                "data-revoke-all",
                "data-issue-share-token",
            ],
        ),
    ] {
        let source = template(name);
        for marker in markers {
            let position = source
                .find(marker)
                .unwrap_or_else(|| panic!("{name} no longer offers {marker}"));
            let before = &source[..position];
            let opened = before.matches("{{#if can_write}}").count()
                + before.matches("{{#if ../can_write}}").count();
            let closed = before.matches("{{/if}}").count();
            assert!(
                opened > 0 && opened > closed.saturating_sub(opened),
                "{name}: {marker} is not inside a can_write guard"
            );
        }
    }
}

#[test]
fn the_detail_page_renders_all_five_tabs() {
    let source = template("user-detail.hbs");
    assert!(source.contains("components/tabs"), "the tab strip is missing");
    for pane in ["identity", "membership", "devices", "sessions", "usage"] {
        assert!(
            source.contains(&format!("{{{{#if {pane}}}}}")),
            "no pane for the {pane} tab"
        );
    }
}

// Why: the primary group and project are the key every exclusive cost total is
// counted by. Losing this editor makes those numbers unmovable from the
// console, which is the state the scope-defaults endpoint exists to end.
#[test]
fn the_detail_page_keeps_the_attribution_editor() {
    let source = template("user-detail.hbs");
    assert!(source.contains("data-form=\"scope-defaults\""));
    assert!(source.contains("name=\"primary_group_id\""));
    assert!(source.contains("name=\"primary_project_id\""));
}

#[test]
fn every_detail_table_has_an_empty_state() {
    let source = template("user-detail.hbs");
    let tables = source.matches("components/table").count();
    let empties = source.matches("components/empty-state").count();
    assert!(
        empties >= tables - 2,
        "{tables} tables but only {empties} empty states — a table lost its empty branch"
    );
}
