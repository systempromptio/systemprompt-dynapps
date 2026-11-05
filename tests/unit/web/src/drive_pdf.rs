//! The PDF export a Drive user downloads: embedded Unicode fonts, kept styling,
//! held images only, and the same bytes on every render.

use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use systemprompt::files::documents::model::RichText;
use systemprompt::files::documents::pdf::{RenderRequest, render_rich};
use systemprompt::identifiers::DocumentRevisionId;

const ONE_PIXEL_PNG: [u8; 67] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae,
    0x42, 0x60, 0x82,
];

fn pdf() -> Vec<u8> {
    let model: RichText = serde_json::from_value(serde_json::json!({"blocks": [
        {"id": "a", "type": "heading", "level": 1, "text": "Ελληνικά Кириллица"},
        {"id": "b", "type": "paragraph", "text": "bold red", "marks": [
            {"start": 0, "end": 4, "mark": "bold"},
            {"start": 5, "end": 8, "mark": "color", "value": "#C00000"}
        ]},
        {"id": "c", "type": "image", "blob": "held", "alt": "held"},
        {"id": "d", "type": "image", "blob": "https://remote.test/x.png", "alt": "remote"}
    ]}))
    .expect("model");
    let media = BTreeMap::from([("held".to_owned(), ONE_PIXEL_PNG.to_vec())]);
    let revision = DocumentRevisionId::new("drev_01HZZZZZZZZZZZZZZZZZZZZZZZ");
    render_rich(
        &model,
        &media,
        &RenderRequest {
            name: "fixture",
            revision: &revision,
            created_at: Utc
                .with_ymd_and_hms(2026, 11, 5, 9, 0, 0)
                .single()
                .expect("date"),
            max_pages: 5,
        },
    )
    .expect("render")
}

fn holds(pdf: &[u8], needle: &str) -> bool {
    pdf.windows(needle.len()).any(|w| w == needle.as_bytes())
}

#[test]
fn the_pdf_embeds_subset_fonts_with_a_unicode_map() {
    let pdf = pdf();
    for needle in ["+DejaVuSans-Bold", "/ToUnicode", "/FontFile2", "/Identity-H"] {
        assert!(holds(&pdf, needle), "{needle} missing");
    }
    assert!(!holds(&pdf, "Helvetica"));
}

#[test]
fn the_pdf_draws_a_held_image_and_only_that() {
    let pdf = pdf();
    // The picture and its soft mask.
    assert_eq!(pdf.windows(14).filter(|w| *w == b"/Subtype /Imag").count(), 2);
    assert!(!holds(&pdf, "remote.test"));
}

#[test]
fn a_revision_renders_to_the_same_bytes() {
    assert_eq!(pdf(), pdf());
}
