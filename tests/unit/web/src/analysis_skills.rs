//! The pure halves of the retained skill drill-down: window-bound parsing and
//! rendering, query URL building, the portfolio drill-down link, and the
//! skill and conversation view shaping.

use chrono::{DateTime, TimeZone, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use systemprompt::identifiers::{ManagedResourceId, NativeSessionId, PluginId, UserId};
use systemprompt_web_admin::repositories::analysis::inventory_index::{
    InventoryIndex, MarketplaceRef,
};
use systemprompt_web_admin::repositories::analysis::retained::{
    RetainedConversationRow, RetainedSkillRow,
};
use systemprompt_web_admin::test_support::{
    AnalysisQuery, Drilldown, RetainedConversationView, RetainedSkillView, drilldown_url,
    parse_window_bound, render_window_bound,
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
        plugin: None,
        skill: None,
        page: Some(3),
    };
    assert_eq!(
        query.url(1, None),
        "/admin/analysis/skills?start=2026-08-16&end=2026-09-16"
    );
    assert_eq!(
        query.url(2, Some(&Drilldown::Resource("res/1".to_owned()))),
        "/admin/analysis/skills?start=2026-08-16&end=2026-09-16&resource=res%2F1&page=2"
    );
    assert_eq!(
        AnalysisQuery::default().url(1, None),
        "/admin/analysis/skills"
    );
    assert_eq!(
        drilldown_url(
            &ManagedResourceId::new("res-1"),
            ("2026-08-16", "2026-09-16")
        ),
        "/admin/analysis/skills?resource=res-1&start=2026-08-16&end=2026-09-16"
    );
}

fn skill_row(plugin: &str, key: &str, managed: bool) -> RetainedSkillRow {
    RetainedSkillRow {
        plugin_id: PluginId::new(plugin),
        skill: format!("{plugin}:{}", key.replace('_', "-")),
        marketplace_id: Some(systemprompt::identifiers::MarketplaceId::new("india")),
        source: Some("bundle:india".to_owned()),
        source_hash: Some("abcdef0123456789".to_owned()),
        resource_id: managed.then(|| ManagedResourceId::new(format!("res-{key}"))),
        resource_key: managed.then(|| key.to_owned()),
        kind: managed.then(|| "skill".to_owned()),
        invocations: 4,
        revision_verified: if managed { 3 } else { 0 },
        conversations: 2,
        requests: 3,
        failed: 1,
        tokens: 1_200,
        cost: 2_500_000,
        artifacts: 5,
        structured_artifacts: 2,
        total_rows: 1,
    }
}

fn index() -> InventoryIndex {
    let mut plugins_by_skill = HashMap::new();
    plugins_by_skill.insert(
        "who_am_i".to_owned(),
        vec![Arc::new((
            "astound-commons".to_owned(),
            "Astound Commons".to_owned(),
        ))],
    );
    let mut marketplaces_by_plugin = HashMap::new();
    marketplaces_by_plugin.insert(
        "astound-commons".to_owned(),
        vec![
            MarketplaceRef {
                id: "commons".to_owned(),
                name: "Commons".to_owned(),
            },
            MarketplaceRef {
                id: "india".to_owned(),
                name: "India".to_owned(),
            },
        ],
    );
    InventoryIndex {
        marketplaces: Vec::new(),
        plugins_by_skill,
        marketplaces_by_plugin,
    }
}

#[test]
fn a_skill_view_resolves_its_plugin_marketplaces_source_and_inventory_link() {
    let view = RetainedSkillView::new(
        skill_row("astound-commons", "who_am_i", true),
        "/x".to_owned(),
        &index(),
    );
    assert_eq!(view.skill_name, "who-am-i");
    assert_eq!(view.plugin_name, "Astound Commons");
    assert_eq!(view.marketplaces, ["Commons", "India"]);
    assert_eq!(view.source_label, "bundle:india");
    assert_eq!(view.source_hash_short.as_deref(), Some("abcdef012345"));
    assert_eq!(
        view.inventory_href,
        "/admin/analysis/resources?search=who-am-i"
    );
    assert_eq!(view.cost, "$2.50");
    assert_eq!(view.href, "/x");

    // Why: a kit's plugin this tree does not know still names its plugin id
    // and the marketplace the fact carried — nothing about it is blank.
    let bundled = RetainedSkillView::new(
        skill_row("astound-india-ba", "ba_bug_logging", false),
        "/y".to_owned(),
        &index(),
    );
    assert_eq!(bundled.plugin_name, "astound-india-ba");
    assert_eq!(bundled.marketplaces, ["india"]);
    assert!(bundled.row.resource_id.is_none());
    assert_eq!(bundled.row.revision_verified, 0);
}

#[test]
fn a_skill_drilldown_url_carries_plugin_and_skill() {
    use systemprompt_web_admin::repositories::analysis::retained::RetainedSkillRef;
    let query = AnalysisQuery {
        start: at(2026, 8, 16, 0, 0).into(),
        end: at(2026, 9, 16, 0, 0).into(),
        ..AnalysisQuery::default()
    };
    let target = Drilldown::Skill(RetainedSkillRef {
        plugin_id: PluginId::new("astound-india-ba"),
        skill: "astound-india-ba:ba-bug-logging".to_owned(),
    });
    assert_eq!(
        query.url(1, Some(&target)),
        "/admin/analysis/skills?start=2026-08-16&end=2026-09-16&plugin=astound-india-ba&skill=astound-india-ba%3Aba-bug-logging"
    );
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
