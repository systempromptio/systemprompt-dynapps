//! `OpReference::parse` and its `Display` round trip.

use systemprompt_secrets_onepassword::{OnePasswordError, OpReference};

#[test]
fn parses_three_segments() {
    let r = OpReference::parse("op://dynapps-prod/platform/anthropic").unwrap();
    assert_eq!(r.vault, "dynapps-prod");
    assert_eq!(r.item, "platform");
    assert_eq!(r.field, "anthropic");
}

#[test]
fn display_round_trips() {
    let raw = "op://v/Item Title/field_1";
    assert_eq!(OpReference::parse(raw).unwrap().to_string(), raw);
}

#[test]
fn from_str_matches_parse() {
    let r: OpReference = "op://a/b/c".parse().unwrap();
    assert_eq!(r, OpReference::parse("op://a/b/c").unwrap());
}

#[test]
fn rejects_wrong_scheme() {
    assert!(matches!(
        OpReference::parse("https://a/b/c"),
        Err(OnePasswordError::Config(_))
    ));
}

#[test]
fn rejects_wrong_segment_count() {
    for raw in ["op://a/b", "op://a/b/c/d", "op://"] {
        assert!(OpReference::parse(raw).is_err(), "{raw} should be rejected");
    }
}

#[test]
fn rejects_empty_segment() {
    for raw in ["op:///b/c", "op://a//c", "op://a/b/", "op://a/ /c"] {
        assert!(OpReference::parse(raw).is_err(), "{raw} should be rejected");
    }
}
