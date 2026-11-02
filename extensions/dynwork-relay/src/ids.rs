//! What the relay routes and stores by: peer ids, device ids, pairing ids,
//! mobile sign-in grant ids and the kind of end a socket belongs to.
//!
//! A peer id is the first 16 bytes of SHA-256 of a peer's static X25519
//! public key. The relay never sees the key, only the id, which travels as 32
//! lowercase hex digits outside the envelope.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

pub const PEER_ID_LEN: usize = 16;

#[derive(Debug, Clone, Copy, thiserror::Error, PartialEq, Eq)]
pub enum IdError {
    #[error("a peer id is 32 hex digits")]
    PeerId,
    #[error("a device id is a UUID")]
    DeviceId,
    #[error("a pairing id is a UUID")]
    PairingId,
    #[error("a mobile sign-in grant id is a UUID")]
    GrantId,
    #[error("the device kind is `desktop` or `phone`")]
    Kind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PeerId([u8; PEER_ID_LEN]);

impl PeerId {
    pub const fn from_bytes(bytes: [u8; PEER_ID_LEN]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; PEER_ID_LEN] {
        &self.0
    }

    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        <[u8; PEER_ID_LEN]>::try_from(bytes).ok().map(Self)
    }

    pub fn parse_hex(text: &str) -> Result<Self, IdError> {
        if text.len() != PEER_ID_LEN * 2 {
            return Err(IdError::PeerId);
        }
        let mut bytes = [0u8; PEER_ID_LEN];
        hex::decode_to_slice(text, &mut bytes).map_err(|source| {
            tracing::debug!(error = %source, "peer id is not hex");
            IdError::PeerId
        })?;
        Ok(Self(bytes))
    }

    pub fn to_hex(self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl Serialize for PeerId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_hex())
    }
}

impl<'de> Deserialize<'de> for PeerId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse_hex(&text).map_err(serde::de::Error::custom)
    }
}

macro_rules! uuid_id {
    ($name:ident, $error:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            pub fn generate() -> Self {
                Self(Uuid::new_v4())
            }

            pub const fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(text: &str) -> Result<Self, IdError> {
                Uuid::parse_str(text).map(Self).map_err(|source| {
                    tracing::debug!(error = %source, "identifier is not a UUID");
                    $error
                })
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

uuid_id!(DeviceId, IdError::DeviceId);
uuid_id!(PairingId, IdError::PairingId);
uuid_id!(GrantId, IdError::GrantId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeerKind {
    Desktop,
    Phone,
}

impl PeerKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Desktop => "desktop",
            Self::Phone => "phone",
        }
    }
}

impl FromStr for PeerKind {
    type Err = IdError;

    fn from_str(text: &str) -> Result<Self, IdError> {
        match text {
            "desktop" => Ok(Self::Desktop),
            "phone" => Ok(Self::Phone),
            _ => Err(IdError::Kind),
        }
    }
}

impl fmt::Display for PeerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
