//! In-process cache of minted provider tokens, keyed by account, provider and
//! the credential's rotation stamp so a rotation never serves the old token.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use chrono::{DateTime, Duration, Utc};
use systemprompt::identifiers::UserId;

use super::exchange::Minted;

// Why: a token handed to core must outlive the MCP call it authorises, so one
// that expires inside this margin is minted again rather than served.
const REFRESH_MARGIN_SECS: i64 = 60;

#[derive(Clone)]
struct Entry {
    stamp: i64,
    header_value: String,
    expires_at: DateTime<Utc>,
}

static CACHE: LazyLock<Mutex<HashMap<(String, String), Entry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn key(user: &UserId, provider: &str) -> (String, String) {
    (user.as_str().to_owned(), provider.to_owned())
}

pub(super) fn get(user: &UserId, provider: &str, stamp: i64) -> Option<String> {
    let entry = CACHE.lock().ok()?.get(&key(user, provider)).cloned()?;
    let fresh = Utc::now() + Duration::seconds(REFRESH_MARGIN_SECS) < entry.expires_at;
    (entry.stamp == stamp && fresh).then_some(entry.header_value)
}

pub(super) fn put(user: &UserId, provider: &str, stamp: i64, minted: &Minted) {
    if let Ok(mut cache) = CACHE.lock() {
        cache.insert(
            key(user, provider),
            Entry {
                stamp,
                header_value: minted.header_value.clone(),
                expires_at: minted.expires_at,
            },
        );
    }
}

pub fn forget(user: &UserId, provider: &str) {
    if let Ok(mut cache) = CACHE.lock() {
        cache.remove(&key(user, provider));
    }
}
