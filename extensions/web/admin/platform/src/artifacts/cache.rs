//! A read's cache: per viewer, document and call identity, with the page's
//! runtime freshness rules.
//!
//! The identity is the server, the tool and the input with its object keys
//! sorted, hashed, so inputs that differ only in key order are one call (an
//! approval binds to the same hash). Only a tool that declares
//! `readOnlyHint: true` is ever stored or served; a write never is, whatever
//! the page asks. Omitted options store a read for five minutes and never
//! serve it to a call (`staleTime` 0); `{staleTime, gcTime}` serves an entry
//! younger than `staleTime` without running the tool and keeps it for
//! `gcTime` (both at most 7 days); `refresh` runs the tool and overwrites the
//! entry; `false` neither serves nor stores. A served result carries
//! `cache: {storedAt, revalidating: false}`; any `cache` member a tool sent
//! itself is dropped before storing and before answering.

use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
// JSON: MCP-protocol boundary, a tool's input and result are schema-less.
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

const MAX_LIFETIME_MS: i64 = 7 * 24 * 60 * 60 * 1000;
const DEFAULT_GC_MS: i64 = 5 * 60 * 1000;
const MAX_STORED_BYTES: usize = 1024 * 1024;
const MARKER: &str = "cache";

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub(super) enum CacheOption {
    Switch(bool),
    Tuned(Tuning),
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Tuning {
    #[serde(default)]
    stale_time: Option<i64>,
    #[serde(default)]
    gc_time: Option<i64>,
    #[serde(default)]
    refresh: bool,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Freshness {
    pub(super) serve_within_ms: i64,
    pub(super) keep_ms: i64,
}

impl Freshness {
    pub(super) fn of(option: Option<&CacheOption>) -> Self {
        let tuning = match option {
            Some(CacheOption::Switch(false)) => {
                return Self {
                    serve_within_ms: 0,
                    keep_ms: 0,
                };
            },
            None | Some(CacheOption::Switch(true)) => Tuning::default(),
            Some(CacheOption::Tuned(tuning)) => tuning.clone(),
        };
        let keep_ms = tuning
            .gc_time
            .unwrap_or(DEFAULT_GC_MS)
            .clamp(0, MAX_LIFETIME_MS);
        let stale = tuning.stale_time.unwrap_or(0).clamp(0, keep_ms);
        Self {
            serve_within_ms: if tuning.refresh { 0 } else { stale },
            keep_ms,
        }
    }

    pub(super) fn serves(self, stored_at: DateTime<Utc>) -> bool {
        self.serve_within_ms > 0
            && Utc::now() - stored_at < Duration::milliseconds(self.serve_within_ms)
    }

    pub(super) fn expiry(self) -> Option<DateTime<Utc>> {
        (self.keep_ms > 0).then(|| Utc::now() + Duration::milliseconds(self.keep_ms))
    }
}

// JSON: MCP-protocol boundary, the call's input as the page sent it.
pub(super) fn input_hash(input: &Value) -> String {
    let canonical = sorted(input).to_string();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(canonical.as_bytes()))
}

// JSON: rebuilds a schema-less value with sorted keys.
fn sorted(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(&String, &Value)> = map.iter().collect();
            entries.sort_by(|left, right| left.0.cmp(right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, inner)| (key.clone(), sorted(inner)))
                    .collect::<Map<_, _>>(),
            )
        },
        Value::Array(items) => Value::Array(items.iter().map(sorted).collect()),
        other => other.clone(),
    }
}

// JSON: MCP-protocol boundary, a tool result.
pub(super) fn stripped(mut result: Value) -> Value {
    if let Some(object) = result.as_object_mut() {
        object.remove(MARKER);
    }
    result
}

// JSON: MCP-protocol boundary, a tool result.
pub(super) fn storable(result: &Value) -> bool {
    result.get("isError").and_then(Value::as_bool) != Some(true)
        && result.to_string().len() <= MAX_STORED_BYTES
}

// JSON: MCP-protocol boundary, a stored tool result served again.
pub(super) fn served(mut result: Value, stored_at: DateTime<Utc>) -> Value {
    if let Some(object) = result.as_object_mut() {
        object.insert(
            MARKER.to_owned(),
            json!({ "storedAt": stored_at.timestamp_millis(), "revalidating": false }),
        );
    }
    result
}
