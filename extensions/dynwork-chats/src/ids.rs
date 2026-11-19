//! What the store addresses by: thread ids (`ses_…`), message ids (`msg_…`)
//! and the opaque cursor of the changes feed.
//!
//! The server mints ids as a prefix and a 26-character ULID (48-bit
//! millisecond time and 80 random bits, Crockford base32), so they sort by
//! creation. A client may supply its own (a prompt's `messageID` makes a retry
//! idempotent), so parsing accepts any prefix plus 8 to 40 ASCII
//! alphanumerics, which is also what the schema's `CHECK` enforces. A reply's
//! id is derived from its prompt instead ([`MessageId::derived_at`]): the
//! prompt's time and a hash of a seed take the place of the random bits, so
//! the same prompt at the same time always names the same reply.

use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const MIN_SUFFIX: usize = 8;
const MAX_SUFFIX: usize = 40;
const RANDOM_BITS: u32 = 80;

#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub enum IdError {
    #[error("a thread id is `ses_` and 8 to 40 letters or digits")]
    Thread,
    #[error("a message id is `msg_` and 8 to 40 letters or digits")]
    Message,
    #[error("the cursor is not one this server issued")]
    Cursor,
}

fn ulid(now: DateTime<Utc>) -> String {
    encode(now, rand::random::<u128>())
}

fn seeded_ulid(now: DateTime<Utc>, seed: &str) -> String {
    let mut high = DefaultHasher::new();
    seed.hash(&mut high);
    let mut low = DefaultHasher::new();
    (seed, RANDOM_BITS).hash(&mut low);
    encode(
        now,
        (u128::from(high.finish()) << 64) | u128::from(low.finish()),
    )
}

fn encode(now: DateTime<Utc>, random: u128) -> String {
    let millis = u128::try_from(now.timestamp_millis()).unwrap_or(0) & ((1 << 48) - 1);
    let random = random & ((1 << RANDOM_BITS) - 1);
    let mut value = (millis << RANDOM_BITS) | random;
    let mut text = [0u8; 26];
    for slot in text.iter_mut().rev() {
        *slot = CROCKFORD[usize::try_from(value & 31).unwrap_or(0)];
        value >>= 5;
    }
    text.iter().map(|byte| char::from(*byte)).collect()
}

fn well_formed(text: &str, prefix: &str) -> bool {
    text.strip_prefix(prefix).is_some_and(|suffix| {
        (MIN_SUFFIX..=MAX_SUFFIX).contains(&suffix.len())
            && suffix.bytes().all(|byte| byte.is_ascii_alphanumeric())
    })
}

macro_rules! chat_id {
    ($name:ident, $prefix:literal, $error:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(String);

        impl $name {
            pub const PREFIX: &'static str = $prefix;

            pub fn generate() -> Self {
                Self::generate_at(Utc::now())
            }

            pub fn generate_at(now: DateTime<Utc>) -> Self {
                Self(format!("{}{}", $prefix, ulid(now)))
            }

            pub fn derived_at(now: DateTime<Utc>, seed: &str) -> Self {
                Self(format!("{}{}", $prefix, seeded_ulid(now, seed)))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(text: &str) -> Result<Self, IdError> {
                if well_formed(text, $prefix) {
                    Ok(Self(text.to_owned()))
                } else {
                    Err($error)
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                text.parse().map_err(serde::de::Error::custom)
            }
        }
    };
}

chat_id!(ThreadId, "ses_", IdError::Thread);
chat_id!(MessageId, "msg_", IdError::Message);

/// Where a client resumes the changes feed. It wraps a `change_seq` as the
/// base64url of its 8 big-endian bytes; clients treat it as opaque and never
/// parse it. [`Cursor::ZERO`] precedes every change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cursor(i64);

impl Cursor {
    pub const ZERO: Self = Self(0);

    pub const fn from_seq(seq: i64) -> Option<Self> {
        if seq < 0 { None } else { Some(Self(seq)) }
    }

    pub const fn seq(self) -> i64 {
        self.0
    }
}

impl fmt::Display for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&URL_SAFE_NO_PAD.encode(self.0.to_be_bytes()))
    }
}

impl FromStr for Cursor {
    type Err = IdError;

    fn from_str(text: &str) -> Result<Self, IdError> {
        let bytes = URL_SAFE_NO_PAD.decode(text).map_err(|source| {
            tracing::debug!(error = %source, "cursor is not base64url");
            IdError::Cursor
        })?;
        let bytes = <[u8; 8]>::try_from(bytes.as_slice()).map_err(|source| {
            tracing::debug!(error = %source, "cursor is not 8 bytes");
            IdError::Cursor
        })?;
        Self::from_seq(i64::from_be_bytes(bytes)).ok_or(IdError::Cursor)
    }
}

impl Serialize for Cursor {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Cursor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}
