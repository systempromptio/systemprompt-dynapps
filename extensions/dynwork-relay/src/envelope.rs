//! The relay's view of an envelope (`relay-protocol.md` in
//! `@dynwork/remote`).
//!
//! | offset | size | field        |
//! |--------|------|--------------|
//! | 0      | 1    | `version`    |
//! | 1      | 16   | `to`         |
//! | 17     | 16   | `from`       |
//! | 33     | 4    | `seq`        |
//! | 37     | rest | `ciphertext` |
//!
//! The relay reads `version` and `to`, overwrites `from` with the sender's
//! authenticated peer id, and never parses, logs or keeps `ciphertext` beyond
//! the ring buffer.

use crate::ids::{PEER_ID_LEN, PeerId};

pub const VERSION: u8 = 1;
pub const HEADER_LEN: usize = 37;
pub const MAX_CIPHERTEXT: usize = 65_535;
pub const MAX_ENVELOPE: usize = HEADER_LEN + MAX_CIPHERTEXT;

const TO_AT: usize = 1;
pub const FROM_AT: usize = TO_AT + PEER_ID_LEN;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum EnvelopeError {
    #[error("envelope of {len} bytes exceeds {MAX_ENVELOPE}")]
    TooLarge { len: usize },
    #[error("envelope of {len} bytes has no ciphertext")]
    TooShort { len: usize },
    #[error("envelope version {0} is not supported")]
    UnknownVersion(u8),
}

pub fn recipient(envelope: &[u8], max_len: usize) -> Result<PeerId, EnvelopeError> {
    let len = envelope.len();
    if len > max_len.min(MAX_ENVELOPE) {
        return Err(EnvelopeError::TooLarge { len });
    }
    match envelope.first() {
        None => return Err(EnvelopeError::TooShort { len }),
        Some(&version) if version != VERSION => {
            return Err(EnvelopeError::UnknownVersion(version));
        },
        Some(_) => {},
    }
    if len <= HEADER_LEN {
        return Err(EnvelopeError::TooShort { len });
    }
    PeerId::from_slice(&envelope[TO_AT..FROM_AT]).ok_or(EnvelopeError::TooShort { len })
}

pub fn stamp_sender(envelope: &mut [u8], from: PeerId) {
    if let Some(field) = envelope.get_mut(FROM_AT..FROM_AT + PEER_ID_LEN) {
        field.copy_from_slice(from.as_bytes());
    }
}

pub const CONTROL_VERSION: u8 = 0xFF;
pub const PRESENCE_LEN: usize = FROM_AT + PEER_ID_LEN + 1;

/// A phone's socket came or went, told to the user's desktops.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    Connected = 1,
    Disconnected = 2,
}

pub fn presence(to: PeerId, phone: PeerId, event: Presence) -> Vec<u8> {
    let mut message = Vec::with_capacity(PRESENCE_LEN);
    message.push(CONTROL_VERSION);
    message.extend_from_slice(to.as_bytes());
    message.extend_from_slice(phone.as_bytes());
    message.push(event as u8);
    message
}
