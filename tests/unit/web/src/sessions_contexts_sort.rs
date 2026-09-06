//! The sort contracts of the sessions and contexts lists.
//!
//! Both pages bind their sort as two text parameters that a `CASE` in the
//! `ORDER BY` selects on, so the string a column maps to is load-bearing SQL,
//! not a label. A rename that the query does not match does not fail to
//! compile: it silently falls through to the default ordering and the page
//! quietly stops honouring the header the user clicked.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_web_admin::repositories::analytics::contexts_list::{
    ContextSort, ContextSortColumn, ContextSortDir,
};
use systemprompt_web_admin::repositories::analytics::sessions_list::{
    SessionSort, SessionSortColumn, SessionSortDir,
};

#[test]
fn session_sort_columns_keep_the_keys_the_query_binds() {
    let cases = [
        (SessionSortColumn::StartedAt, "started_at"),
        (SessionSortColumn::Duration, "duration"),
        (SessionSortColumn::Requests, "requests"),
        (SessionSortColumn::Tokens, "tokens"),
        (SessionSortColumn::Cost, "cost"),
    ];
    for (column, expected) in cases {
        assert_eq!(column.as_str(), expected);
        assert_eq!(
            SessionSortColumn::parse_session_column(Some(expected)),
            column,
            "the key a header emits must round-trip back to its column"
        );
    }
}

#[test]
fn an_unknown_session_sort_key_falls_back_to_newest_first() {
    for key in [None, Some(""), Some("nonsense"), Some("started_atX")] {
        assert_eq!(
            SessionSortColumn::parse_session_column(key),
            SessionSortColumn::StartedAt
        );
    }
    assert_eq!(
        SessionSortDir::parse_session_dir(Some("sideways")),
        SessionSortDir::Desc
    );
    assert_eq!(
        SessionSortDir::parse_session_dir(Some("asc")),
        SessionSortDir::Asc
    );
    assert_eq!(
        SessionSortDir::parse_session_dir(None),
        SessionSortDir::Desc
    );
}

#[test]
fn the_default_session_sort_is_newest_first() {
    let sort = SessionSort::default();
    assert_eq!(sort.column, SessionSortColumn::StartedAt);
    assert_eq!(sort.dir, SessionSortDir::Desc);
    assert_eq!(sort.column.as_str(), "started_at");
    assert_eq!(sort.dir.as_str(), "desc");
}

#[test]
fn context_sort_columns_keep_the_keys_the_query_binds() {
    let cases = [
        (ContextSortColumn::Activity, "activity"),
        (ContextSortColumn::Requests, "requests"),
        (ContextSortColumn::Messages, "messages"),
        (ContextSortColumn::Tokens, "tokens"),
        (ContextSortColumn::Cost, "cost"),
    ];
    for (column, expected) in cases {
        assert_eq!(column.as_str(), expected);
        assert_eq!(
            ContextSortColumn::parse_context_column(Some(expected)),
            column
        );
    }
}

#[test]
fn an_unknown_context_sort_key_falls_back_to_most_recent_activity() {
    for key in [None, Some(""), Some("cost ; DROP TABLE users")] {
        assert_eq!(
            ContextSortColumn::parse_context_column(key),
            ContextSortColumn::Activity
        );
    }
    let sort = ContextSort::default();
    assert_eq!(sort.column, ContextSortColumn::Activity);
    assert_eq!(sort.dir, ContextSortDir::Desc);
}
