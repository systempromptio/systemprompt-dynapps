//! The upload sessions this server opened, each bound to the user it opened
//! it for.
//!
//! Every tenant shares Microsoft's upload hosts (`*.sharepoint.com`,
//! `outlook.office*.com`), so a URL on one of them is not tied to the
//! caller: a session opened in another tenant would take the caller's file.
//! `d365_create_mail_attachment_upload_session` records the `uploadUrl` it
//! returns against the caller, and `d365_upload_attachment_chunk` admits only
//! a URL recorded for the same caller and not yet expired. The expiry is
//! Graph's `expirationDateTime`, capped at [`UPLOAD_SESSION_MAX_MINUTES`].
//!
//! The record lives in this process's memory, bounded per user and overall,
//! and every call's Graph client shares it. A session opened by another
//! process (a second replica, or before a restart) is refused, and the model
//! is told to open a new one. The URLs carry Graph's pre-authenticated
//! token, so nothing here logs or prints them.

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use chrono::{DateTime, TimeDelta, Utc};
use systemprompt::identifiers::UserId;
use url::Url;

use super::GraphError;

// Why: Outlook takes an attachment of up to 150 MB, about forty 4 MiB
// chunks; an hour covers that upload and bounds how long a recorded URL
// lives when Graph sends a later expiry or none.
pub const UPLOAD_SESSION_MAX_MINUTES: i64 = 60;
pub const MAX_UPLOAD_SESSIONS: usize = 1024;
pub const MAX_UPLOAD_SESSIONS_PER_USER: usize = 16;

const NOT_OPENED: &str = "upload_url is not an upload session this server opened for you: \
    open one with d365_create_mail_attachment_upload_session and pass its uploadUrl";
const EXPIRED: &str = "the upload session has expired: open a new one with \
    d365_create_mail_attachment_upload_session and send the file again from its first byte";

#[derive(Debug, Clone)]
struct Opened {
    owner: UserId,
    expires_at: DateTime<Utc>,
}

/// The upload sessions this process opened, by `uploadUrl`. Clones share
/// one record.
#[derive(Clone, Default)]
pub struct UploadSessions {
    opened: Arc<Mutex<HashMap<String, Opened>>>,
}

impl fmt::Debug for UploadSessions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UploadSessions")
            .field("open", &self.len())
            .finish()
    }
}

impl UploadSessions {
    pub fn issue(
        &self,
        upload_url: &str,
        owner: &UserId,
        graph_expiry: Option<&str>,
        now: DateTime<Utc>,
    ) {
        let Some(key) = session_key(upload_url) else {
            return;
        };
        let session = Opened {
            owner: owner.clone(),
            expires_at: expiry(graph_expiry, now),
        };
        let mut opened = self.lock();
        opened.retain(|_, live| live.expires_at > now);
        let held = opened.values().filter(|live| live.owner == *owner).count();
        if held >= MAX_UPLOAD_SESSIONS_PER_USER {
            evict_soonest(&mut opened, Some(owner));
        }
        if opened.len() >= MAX_UPLOAD_SESSIONS {
            evict_soonest(&mut opened, None);
        }
        opened.insert(key, session);
    }

    pub fn admit(
        &self,
        upload_url: &str,
        caller: &UserId,
        now: DateTime<Utc>,
    ) -> Result<(), GraphError> {
        let key = session_key(upload_url).ok_or_else(not_opened)?;
        let mut opened = self.lock();
        let expires_at = opened
            .get(&key)
            .filter(|session| session.owner == *caller)
            .map(|session| session.expires_at)
            .ok_or_else(not_opened)?;
        let live = expires_at > now;
        if !live {
            opened.remove(&key);
        }
        drop(opened);
        if live {
            Ok(())
        } else {
            Err(GraphError::Invalid(EXPIRED.to_owned()))
        }
    }

    pub fn len(&self) -> usize {
        self.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, Opened>> {
        self.opened.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn not_opened() -> GraphError {
    GraphError::Invalid(NOT_OPENED.to_owned())
}

fn session_key(upload_url: &str) -> Option<String> {
    Url::parse(upload_url.trim()).ok().map(String::from)
}

fn expiry(graph_expiry: Option<&str>, now: DateTime<Utc>) -> DateTime<Utc> {
    let cap = now + TimeDelta::minutes(UPLOAD_SESSION_MAX_MINUTES);
    graph_expiry
        .and_then(|raw| DateTime::parse_from_rfc3339(raw.trim()).ok())
        .map_or(cap, |at| at.with_timezone(&Utc).min(cap))
}

fn evict_soonest(opened: &mut HashMap<String, Opened>, owner: Option<&UserId>) {
    let soonest = opened
        .iter()
        .filter(|(_, session)| owner.is_none_or(|wanted| session.owner == *wanted))
        .min_by_key(|(_, session)| session.expires_at)
        .map(|(key, _)| key.clone());
    if let Some(key) = soonest {
        opened.remove(&key);
    }
}
