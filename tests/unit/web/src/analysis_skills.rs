//! The pure halves of the skill page: window-bound parsing and rendering,
//! query URL building on the `plugin:skill` path, the skill-key parser, the
//! drill-down link other pages use, and the conversation view shaping.

use chrono::{DateTime, TimeZone, Utc};
use systemprompt::identifiers::{ManagedResourceId, NativeSessionId, UserId};
use systemprompt_web_admin::repositories::analysis::retained::RetainedConversationRow;
use systemprompt_web_admin::test_support::{
    AnalysisQuery, RetainedConversationView, drilldown_url, parse_skill_key, parse_window_bound,
    render_window_bound,
};

fn at(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, h, min, 0)
        .single()
        .unwrap_or_default()
}

#[test]
fn window_bounds_accept_dates_datetimes_and_rfc3339() {
    assert_eq!(parse_window_bound("2026-09-01"), Some(at(2026, 9, 1, 0, 0)));
    assert_eq!(
        parse_window_bound("2026-09-01T10:30"),
        Some(at(2026, 9, 1, 10, 30))
    );
    assert_eq!(
        parse_window_bound("2026-09-01T10:30:00"),
        Some(at(2026, 9, 1, 10, 30))
    );
    assert_eq!(
        parse_window_bound("2026-09-01T12:30:00+02:00"),
        Some(at(2026, 9, 1, 10, 30))
    );
    assert_eq!(parse_window_bound("yesterday"), None);
    assert_eq!(parse_window_bound("2026-13-01"), None);
}

#[test]
fn window_bounds_render_as_dates_on_midnight_and_datetimes_otherwise() {
    assert_eq!(render_window_bound(at(2026, 9, 1, 0, 0)), "2026-09-01");
    assert_eq!(
        render_window_bound(at(2026, 9, 1, 10, 30)),
        "2026-09-01T10:30:00"
    );
    let round_trip = parse_window_bound(&render_window_bound(at(2026, 9, 1, 0, 0)));
    assert_eq!(round_trip, Some(at(2026, 9, 1, 0, 0)));
}

#[test]
fn the_default_window_covers_the_last_30_days_up_to_the_next_midnight() {
    let window = AnalysisQuery::default()
        .window(at(2026, 9, 15, 13, 45))
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(window.end, at(2026, 9, 16, 0, 0));
    assert_eq!(window.start, at(2026, 8, 17, 0, 0));
}

#[test]
fn an_inverted_or_oversized_window_is_rejected() {
    let inverted = AnalysisQuery {
        start: at(2026, 9, 10, 0, 0).into(),
        end: at(2026, 9, 1, 0, 0).into(),
        ..AnalysisQuery::default()
    };
    assert!(inverted.window(at(2026, 9, 15, 0, 0)).is_err());
    let oversized = AnalysisQuery {
        start: at(2025, 1, 1, 0, 0).into(),
        end: at(2026, 9, 1, 0, 0).into(),
        ..AnalysisQuery::default()
    };
    assert!(oversized.window(at(2026, 9, 15, 0, 0)).is_err());
}

#[test]
fn the_resource_filter_is_trimmed_and_validated() {
    let blank = AnalysisQuery {
        resource: Some("   ".to_owned()),
        ..AnalysisQuery::default()
    };
    assert!(
        blank
            .resource()
            .unwrap_or_else(|error| panic!("{error}"))
            .is_none()
    );
    let bad = AnalysisQuery {
        resource: Some("res 1".to_owned()),
        ..AnalysisQuery::default()
    };
    assert!(bad.resource().is_err());
    let ok = AnalysisQuery {
        resource: Some(" res-1 ".to_owned()),
        ..AnalysisQuery::default()
    };
    assert_eq!(
        ok.resource()
            .unwrap_or_else(|error| panic!("{error}"))
            .map(|id| id.as_str().to_owned()),
        Some("res-1".to_owned())
    );
}

#[test]
fn urls_carry_the_window_the_resource_and_only_a_non_first_page() {
    let query = AnalysisQuery {
        start: at(2026, 8, 16, 0, 0).into(),
        end: at(2026, 9, 16, 0, 0).into(),
        resource: None,
        page: Some(3),
    };
    assert_eq!(
        query.url(1, "a:who-am-i"),
        "/admin/analysis/skills/a%3Awho-am-i?start=2026-08-16&end=2026-09-16"
    );
    let narrowed = AnalysisQuery {
        resource: Some("res/1".to_owned()),
        ..query
    };
    assert_eq!(
        narrowed.url(2, "a:who-am-i"),
        "/admin/analysis/skills/a%3Awho-am-i?start=2026-08-16&end=2026-09-16&resource=res%2F1&page=2"
    );
    assert_eq!(
        AnalysisQuery::default().url(1, "a:b"),
        "/admin/analysis/skills/a%3Ab"
    );
    assert_eq!(
        drilldown_url("a:who-am-i", None, ("2026-08-16", "2026-09-16")),
        "/admin/analysis/skills/a%3Awho-am-i?start=2026-08-16&end=2026-09-16"
    );
    assert_eq!(
        drilldown_url(
            "a:who-am-i",
            Some(&ManagedResourceId::new("res-1")),
            ("2026-08-16", "2026-09-16")
        ),
        "/admin/analysis/skills/a%3Awho-am-i?start=2026-08-16&end=2026-09-16&resource=res-1"
    );
}

#[test]
fn a_skill_key_is_plugin_colon_skill_and_nothing_else() {
    let key = parse_skill_key("astound-india-ba:ba-bug-logging").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(key.plugin_id.as_str(), "astound-india-ba");
    assert_eq!(key.skill, "astound-india-ba:ba-bug-logging");
    assert!(parse_skill_key("who-am-i").is_err(), "no plugin prefix");
    assert!(parse_skill_key(":who-am-i").is_err(), "empty plugin");
    assert!(parse_skill_key("a:").is_err(), "empty skill");
    assert!(parse_skill_key("a:b c").is_err(), "whitespace");
}

fn conversation_row(status: Option<&str>, score: Option<i64>) -> RetainedConversationRow {
    RetainedConversationRow {
        session_id: NativeSessionId::new("ses/1"),
        user_id: UserId::new("consumer"),
        host: Some("codex".to_owned()),
        first_at: at(2026, 9, 1, 9, 5),
        last_at: at(2026, 9, 1, 9, 45),
        invocations: 2,
        requests: 3,
        failed: 0,
        tokens: 900,
        cost: 125,
        assessment_status: status.map(str::to_owned),
        score_millionths: score,
        artifacts: 1,
        total_rows: 1,
    }
}

#[test]
fn a_conversation_view_links_the_session_and_words_the_assessment() {
    let scored = RetainedConversationView::new(conversation_row(Some("scored"), Some(850_000)));
    assert_eq!(scored.href, "/admin/sessions/ses%2F1");
    assert_eq!(scored.host, "codex");
    assert_eq!(scored.first, "2026-09-01 09:05");
    assert_eq!(scored.last, "2026-09-01 09:45");
    assert_eq!(scored.assessment, "scored 85%");
    assert_eq!(
        RetainedConversationView::new(conversation_row(Some("failed"), None)).assessment,
        "failed"
    );
    assert_eq!(
        RetainedConversationView::new(conversation_row(None, None)).assessment,
        "not assessed"
    );
    let mut unknown = conversation_row(None, None);
    unknown.host = None;
    assert_eq!(RetainedConversationView::new(unknown).host, "unknown host");
}
