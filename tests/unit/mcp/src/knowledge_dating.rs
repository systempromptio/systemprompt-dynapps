//! Dating precedence and its guard rails: a source-stated date beats the
//! model, the model beats mtime, an impossible calendar date is never
//! accepted, and the provenance label always says which step produced the
//! date — the agent policy trusts `frontmatter > llm > mtime`.

use crate::knowledge_fake_model::FakeModel;
use chrono::{NaiveDate, TimeZone, Utc};
use systemprompt_mcp_knowledge_bank::ingest::dating::{
    DateInputs, SOURCE_FRONTMATTER, SOURCE_LLM, SOURCE_MTIME, SOURCE_NONE, frontmatter_date,
    parse_iso_date, resolve,
};

fn date(s: &str) -> NaiveDate {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("valid date")
}

#[test]
fn frontmatter_prefers_updated_then_created_then_date() {
    let body = "---\ndate: 2026-01-01\ncreated: 2026-02-02\nupdated: 2026-03-03\n---\n# Doc";
    assert_eq!(frontmatter_date(body), Some(date("2026-03-03")));
    let body = "---\ndate: 2026-01-01\ncreated: 2026-02-02\n---\n# Doc";
    assert_eq!(frontmatter_date(body), Some(date("2026-02-02")));
    assert_eq!(frontmatter_date("# No frontmatter"), None);
}

#[test]
fn impossible_calendar_dates_are_rejected() {
    assert_eq!(parse_iso_date("2026-13-40"), None);
    assert_eq!(parse_iso_date("the date is 2026-02-29"), None);
    assert_eq!(
        parse_iso_date("the date is 2024-02-29."),
        Some(date("2024-02-29"))
    );
}

#[tokio::test]
async fn a_source_stated_date_wins_without_calling_the_model() {
    let model = FakeModel::replying(&["2020-01-01"]);
    let (resolved, source) = resolve(
        DateInputs {
            body: "no frontmatter",
            authored: Some(date("2026-05-05")),
            mtime: None,
            extract_via_llm: true,
        },
        Some(&model),
    )
    .await;
    assert_eq!(resolved, Some(date("2026-05-05")));
    assert_eq!(source, SOURCE_FRONTMATTER);
    assert_eq!(*model.converse_calls.lock().expect("lock"), 0);
}

#[tokio::test]
async fn the_model_dates_when_enabled_and_mtime_is_the_last_resort() {
    let mtime = Utc.with_ymd_and_hms(2026, 7, 7, 12, 0, 0).single();
    let model = FakeModel::replying(&["The date is 2026-06-06"]);
    let (resolved, source) = resolve(
        DateInputs {
            body: "Meeting notes",
            authored: None,
            mtime,
            extract_via_llm: true,
        },
        Some(&model),
    )
    .await;
    assert_eq!((resolved, source), (Some(date("2026-06-06")), SOURCE_LLM));

    let (resolved, source) = resolve(
        DateInputs {
            body: "Meeting notes",
            authored: None,
            mtime,
            extract_via_llm: false,
        },
        Some(&model),
    )
    .await;
    assert_eq!((resolved, source), (Some(date("2026-07-07")), SOURCE_MTIME));

    let none_model = FakeModel::replying(&["NONE"]);
    let (resolved, source) = resolve(
        DateInputs {
            body: "Meeting notes",
            authored: None,
            mtime: None,
            extract_via_llm: true,
        },
        Some(&none_model),
    )
    .await;
    assert_eq!((resolved, source), (None, SOURCE_NONE));
}
