use dynwork_relay::envelope::{
    EnvelopeError, HEADER_LEN, MAX_CIPHERTEXT, MAX_ENVELOPE, recipient, stamp_sender,
};

use crate::support::{envelope, peer};

#[test]
fn reads_the_recipient_of_a_minimal_envelope() {
    let bytes = envelope(peer(2), peer(1), &[0xAA]);
    assert_eq!(recipient(&bytes, MAX_ENVELOPE), Ok(peer(2)));
}

#[test]
fn an_envelope_without_ciphertext_is_too_short() {
    let bytes = envelope(peer(2), peer(1), &[]);
    assert_eq!(
        recipient(&bytes, MAX_ENVELOPE),
        Err(EnvelopeError::TooShort { len: HEADER_LEN })
    );
    assert!(matches!(
        recipient(&[1, 2, 3], MAX_ENVELOPE),
        Err(EnvelopeError::TooShort { .. })
    ));
}

#[test]
fn the_largest_envelope_is_accepted_and_one_byte_more_is_not() {
    let largest = envelope(peer(2), peer(1), &vec![0; MAX_CIPHERTEXT]);
    assert_eq!(largest.len(), MAX_ENVELOPE);
    assert_eq!(recipient(&largest, MAX_ENVELOPE), Ok(peer(2)));
    let over = envelope(peer(2), peer(1), &vec![0; MAX_CIPHERTEXT + 1]);
    assert_eq!(
        recipient(&over, MAX_ENVELOPE),
        Err(EnvelopeError::TooLarge {
            len: MAX_ENVELOPE + 1
        })
    );
}

#[test]
fn a_configured_limit_below_the_protocol_maximum_applies() {
    let bytes = envelope(peer(2), peer(1), &[0; 100]);
    assert!(matches!(
        recipient(&bytes, 120),
        Err(EnvelopeError::TooLarge { .. })
    ));
}

#[test]
fn an_unknown_version_is_reported() {
    let mut bytes = envelope(peer(2), peer(1), &[0xAA]);
    bytes[0] = 2;
    assert_eq!(
        recipient(&bytes, MAX_ENVELOPE),
        Err(EnvelopeError::UnknownVersion(2))
    );
}

#[test]
fn stamping_overwrites_only_the_sender() {
    let original = envelope(peer(2), peer(9), &[1, 2, 3, 4]);
    let mut stamped = original.clone();
    stamp_sender(&mut stamped, peer(5));
    assert_eq!(&stamped[17..33], peer(5).as_bytes());
    assert_eq!(stamped[..17], original[..17]);
    assert_eq!(stamped[33..], original[33..]);
}

#[test]
fn a_presence_message_uses_the_envelope_offsets_and_a_reserved_version() {
    use dynwork_relay::envelope::{CONTROL_VERSION, PRESENCE_LEN, Presence, presence};
    let message = presence(peer(1), peer(2), Presence::Disconnected);
    assert_eq!(message.len(), PRESENCE_LEN);
    assert_eq!(PRESENCE_LEN, 34);
    assert_eq!(message[0], CONTROL_VERSION);
    assert_eq!(&message[1..17], peer(1).as_bytes());
    assert_eq!(&message[17..33], peer(2).as_bytes());
    assert_eq!(message[33], 2);
    assert_eq!(
        recipient(&message, MAX_ENVELOPE),
        Err(EnvelopeError::UnknownVersion(0xFF))
    );
}
