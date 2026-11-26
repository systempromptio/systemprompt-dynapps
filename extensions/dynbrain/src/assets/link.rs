//! Signed links for the files of the Brain's articles: the one-time upload
//! URL and the short-lived download URL.
//!
//! Core's signed upload and download links (`SignedUploadLinks` and the
//! document links the MCP file layer uses) cannot serve these files: they
//! are user JWTs minted by core's API for its own document store, open only
//! core's `/api/v1/uploads/{id}` and document revision routes, and check
//! access to a core document, not to a Brain article. A Brain file is no core
//! document, and the token authority an extension holds may only verify. So
//! a link here is an HMAC-SHA256 (core's `hmac_sha256`) under a key derived
//! from the instance secret `dynbrain_links`, over the purpose, the target and
//! the expiry. The API and the `brain` MCP server both hold that secret, so a
//! link signed by either opens the API's download route. The expiry is inside
//! what is signed, the link carries nothing about a person, and a link for one
//! purpose or target never opens another.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::security::hmac_sha256;

pub const UPLOAD_LINK_TTL: Duration = Duration::minutes(15);
pub const DOWNLOAD_LINK_TTL: Duration = Duration::minutes(10);
const KEY_LABEL: &[u8] = b"dynbrain asset links v1";
pub const LINKS_SECRET: &str = "dynbrain_links";

/// What a link opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkPurpose {
    Upload,
    Download,
}

impl LinkPurpose {
    const fn label(self) -> &'static str {
        match self {
            Self::Upload => "upload",
            Self::Download => "download",
        }
    }
}

/// The `exp` (unix seconds) and `sig` (hex) query parameters of a link.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SignedLink {
    pub exp: i64,
    pub sig: String,
}

impl SignedLink {
    pub fn query(&self) -> String {
        format!("exp={}&sig={}", self.exp, self.sig)
    }
}

/// Signs and checks links with the instance's key.
#[derive(Clone, Copy)]
pub struct LinkSigner {
    key: [u8; 32],
}

impl std::fmt::Debug for LinkSigner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LinkSigner").finish_non_exhaustive()
    }
}

impl LinkSigner {
    pub fn new(seed: &[u8]) -> Option<Self> {
        hmac_sha256(seed, KEY_LABEL).ok().map(|key| Self { key })
    }

    pub fn from_secret(secret: &str) -> Option<Self> {
        if secret.trim().is_empty() {
            return None;
        }
        Self::new(secret.as_bytes())
    }

    pub fn sign(
        &self,
        purpose: LinkPurpose,
        target: &str,
        expires: DateTime<Utc>,
    ) -> Option<SignedLink> {
        let exp = expires.timestamp();
        let mac = hmac_sha256(&self.key, payload(purpose, target, exp).as_bytes()).ok()?;
        Some(SignedLink {
            exp,
            sig: hex::encode(mac),
        })
    }

    pub fn verify(
        &self,
        purpose: LinkPurpose,
        target: &str,
        link: &SignedLink,
        now: DateTime<Utc>,
    ) -> bool {
        let Some(expected) = self.sign_at(purpose, target, link.exp) else {
            return false;
        };
        same(expected.sig.as_bytes(), link.sig.as_bytes()) && link.exp > now.timestamp()
    }

    fn sign_at(&self, purpose: LinkPurpose, target: &str, exp: i64) -> Option<SignedLink> {
        let expires = DateTime::<Utc>::from_timestamp(exp, 0)?;
        self.sign(purpose, target, expires)
    }
}

fn payload(purpose: LinkPurpose, target: &str, exp: i64) -> String {
    format!("{}\n{target}\n{exp}", purpose.label())
}

fn same(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}
