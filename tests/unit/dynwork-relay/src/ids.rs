use std::str::FromStr;

use dynwork_relay::ids::{DeviceId, IdError, PeerId, PeerKind};

#[test]
fn a_peer_id_round_trips_through_hex_and_json() {
    let peer = PeerId::from_bytes([0xAB; 16]);
    assert_eq!(peer.to_hex(), "ab".repeat(16));
    assert_eq!(PeerId::parse_hex(&peer.to_hex()), Ok(peer));
    let json = serde_json::to_string(&peer).expect("serialise a peer id");
    assert_eq!(json, format!("\"{}\"", "ab".repeat(16)));
    let back: PeerId = serde_json::from_str(&json).expect("parse a peer id");
    assert_eq!(back, peer);
    assert!(serde_json::from_str::<PeerId>("\"abc\"").is_err());
}

#[test]
fn a_peer_id_of_the_wrong_length_or_alphabet_is_refused() {
    assert_eq!(PeerId::parse_hex("ab"), Err(IdError::PeerId));
    assert_eq!(PeerId::parse_hex(&"zz".repeat(16)), Err(IdError::PeerId));
    assert_eq!(PeerId::parse_hex(&"a".repeat(33)), Err(IdError::PeerId));
    assert!(PeerId::from_slice(&[0; 15]).is_none());
}

#[test]
fn device_ids_parse_only_uuids() {
    let id = DeviceId::generate();
    assert_eq!(DeviceId::from_str(&id.to_string()), Ok(id));
    assert_eq!(DeviceId::from_str("not-a-uuid"), Err(IdError::DeviceId));
}

#[test]
fn the_device_kind_has_two_names() {
    assert_eq!(PeerKind::from_str("desktop"), Ok(PeerKind::Desktop));
    assert_eq!(PeerKind::from_str("phone"), Ok(PeerKind::Phone));
    assert_eq!(PeerKind::Phone.as_str(), "phone");
    assert_eq!(PeerKind::from_str("tablet"), Err(IdError::Kind));
}
