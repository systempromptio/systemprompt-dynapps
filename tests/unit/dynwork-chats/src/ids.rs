use chrono::{Duration, Utc};
use dynwork_chats::ids::{Cursor, IdError, MessageId, ThreadId};

#[test]
fn minted_ids_carry_their_prefix_and_a_26_character_ulid() {
    let thread = ThreadId::generate();
    let message = MessageId::generate();
    assert!(thread.as_str().starts_with("ses_"));
    assert!(message.as_str().starts_with("msg_"));
    assert_eq!(thread.as_str().len(), 4 + 26);
    assert_eq!(message.as_str().len(), 4 + 26);
    assert_ne!(ThreadId::generate(), ThreadId::generate());
    assert!(
        thread.as_str().parse::<ThreadId>().is_ok(),
        "a minted id parses back"
    );
}

#[test]
fn minted_ids_sort_by_creation_time() {
    let now = Utc::now();
    let earlier = ThreadId::generate_at(now);
    let later = ThreadId::generate_at(now + Duration::seconds(1));
    assert!(earlier < later);
    assert!(earlier.as_str() < later.as_str());
}

#[test]
fn a_client_may_supply_a_well_formed_id() {
    assert!("msg_0123456789abcdefABCDEF".parse::<MessageId>().is_ok());
    assert!("ses_abcd1234".parse::<ThreadId>().is_ok());
}

#[test]
fn malformed_ids_are_refused() {
    for bad in [
        "",
        "ses_",
        "ses_short",
        "msg_0123456789abcdef",
        "ses_has space1234",
        "ses_dash-1234567",
    ] {
        assert_eq!(bad.parse::<ThreadId>(), Err(IdError::Thread), "{bad:?}");
    }
    let too_long = format!("msg_{}", "a".repeat(41));
    assert_eq!(too_long.parse::<MessageId>(), Err(IdError::Message));
    assert_eq!(
        "ses_0123456789abcdef".parse::<MessageId>(),
        Err(IdError::Message)
    );
}

#[test]
fn ids_serialize_as_plain_strings_and_validate_on_the_way_in() {
    let id: ThreadId = "ses_0123456789abcdef".parse().expect("parse");
    let json = serde_json::to_string(&id).expect("serialize");
    assert_eq!(json, "\"ses_0123456789abcdef\"");
    assert_eq!(
        serde_json::from_str::<ThreadId>(&json).expect("deserialize"),
        id
    );
    assert!(serde_json::from_str::<ThreadId>("\"nope\"").is_err());
}

#[test]
fn a_cursor_round_trips_through_its_opaque_text() {
    for seq in [0, 1, 255, 1 << 40, i64::MAX] {
        let cursor = Cursor::from_seq(seq).expect("non-negative");
        let text = cursor.to_string();
        assert!(
            text.bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'),
            "base64url, no padding: {text}"
        );
        assert_eq!(text.parse::<Cursor>(), Ok(cursor));
        assert_eq!(cursor.seq(), seq);
    }
    assert_eq!(Cursor::ZERO.seq(), 0);
}

#[test]
fn a_cursor_refuses_what_the_server_never_issued() {
    assert_eq!(Cursor::from_seq(-1), None);
    for bad in [
        "",
        "not a cursor",
        "AAAA",
        "AAAAAAAAAAAAAAAAAA",
        "//////////8",
    ] {
        assert_eq!(bad.parse::<Cursor>(), Err(IdError::Cursor), "{bad:?}");
    }
    // eight 0xFF bytes decode to -1.
    assert_eq!("__________8".parse::<Cursor>(), Err(IdError::Cursor));
}

#[test]
fn a_cursor_serializes_as_its_text() {
    let cursor = Cursor::from_seq(42).expect("cursor");
    let json = serde_json::to_string(&cursor).expect("serialize");
    assert_eq!(json, format!("\"{cursor}\""));
    assert_eq!(
        serde_json::from_str::<Cursor>(&json).expect("deserialize"),
        cursor
    );
}
