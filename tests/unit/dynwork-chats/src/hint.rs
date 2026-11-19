use axum::http::{HeaderMap, HeaderValue};
use dynwork_chats::hint::{KIND, caller_peer};
use dynwork_relay::ids::PeerId;
use dynwork_relay::ws::PEER_HEADER;

const PEER: &str = "0123456789abcdef0123456789abcdef";

fn with_peer(value: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        PEER_HEADER,
        HeaderValue::from_str(value).expect("a header value"),
    );
    headers
}

#[test]
fn the_hint_kind_is_the_one_the_desktop_listens_for() {
    assert_eq!(KIND, "chats");
}

#[test]
fn a_peer_header_names_the_desktop_to_skip() {
    assert_eq!(
        caller_peer(&with_peer(PEER)),
        Some(PeerId::parse_hex(PEER).expect("a peer id"))
    );
}

#[test]
fn no_header_or_a_bad_one_names_nobody() {
    assert_eq!(caller_peer(&HeaderMap::new()), None);
    assert_eq!(caller_peer(&with_peer("")), None);
    assert_eq!(caller_peer(&with_peer("0123")), None);
    assert_eq!(caller_peer(&with_peer(&"z".repeat(32))), None);
    let mut raw = HeaderMap::new();
    raw.insert(
        PEER_HEADER,
        HeaderValue::from_bytes(&[0xE9; 32]).expect("opaque bytes"),
    );
    assert_eq!(caller_peer(&raw), None);
}
