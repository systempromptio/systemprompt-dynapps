//! The content hash of a variant: what the indexer reads, so an unchanged
//! hash means there is nothing to re-index.

use sha2::{Digest, Sha256};

pub fn content_hash(title: &str, description: &str, body: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(b"\0");
    hasher.update(description.as_bytes());
    hasher.update(b"\0");
    hasher.update(body.as_bytes());
    hex::encode(hasher.finalize())
}
