//! Shared row builders. Kept deliberately minimal: every test owns its own
//! database, so fixtures only need to satisfy NOT NULL columns and leave the
//! interesting fields to the caller.

use chrono::{TimeZone, Utc};
use systemprompt::identifiers::SourceId;
use systemprompt_web_shared::models::{ContentSeed, CreateContentParams, CreateLinkParams};

pub(crate) fn source_id() -> SourceId {
    SourceId::new("test-source".to_owned())
}

pub(crate) fn content_params(slug: &str, source: &SourceId) -> CreateContentParams {
    CreateContentParams::new(ContentSeed {
        slug: slug.to_owned(),
        title: format!("Title for {slug}"),
        description: format!("Description for {slug}"),
        body: format!("Body for {slug}"),
        author: "Test Author".to_owned(),
        published_at: Utc
            .with_ymd_and_hms(2026, 1, 1, 12, 0, 0)
            .single()
            .expect("fixed timestamp is unambiguous"),
        source_id: source.clone(),
    })
    .with_keywords("alpha,beta".to_owned())
    .with_version_hash("hash-v1".to_owned())
}

pub(crate) fn link_params(short_code: &str) -> CreateLinkParams {
    CreateLinkParams::new(
        short_code.to_owned(),
        format!("https://example.com/{short_code}"),
        "cta",
    )
}
