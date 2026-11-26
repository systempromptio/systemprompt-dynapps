//! The credential the broker presents to core's MCP proxy for a viewer.
//!
//! The web login's cookie token goes as it came. Core's proxy admits only
//! the first-party audiences, never `ui`, so for a person's `ui` session the
//! broker mints a web session for that person with core's
//! `mint_browser_session`, as the session handoff does, carrying the `ui`
//! session's own sign-in time, never the time of the mint. One is minted per
//! `ui` session and kept until shortly before it expires (`RENEW_BEFORE`),
//! so a page that makes many calls opens one web session, not one per call;
//! entries past that point are dropped whenever one is stored. The minted
//! credential is presented only to the proxy: it is never answered, logged or
//! shown by `Debug`.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use systemprompt::identifiers::{SessionId, UserId};
use systemprompt::sso::{BrowserSessionSubject, mint_browser_session};
use tokio::sync::Mutex;

use super::ArtifactsBroker;
use super::error::{ArtifactCallCode, ArtifactCallError};
use super::viewer::{ArtifactViewer, Credential, PersonViewer};

const RENEW_BEFORE: Duration = Duration::from_secs(120);

#[derive(Default)]
pub(super) struct MintedSessions {
    kept: Mutex<HashMap<(UserId, SessionId), Kept>>,
}

struct Kept {
    token: String,
    renew_at: Instant,
}

pub(super) struct ProxyBearer(String);

impl std::fmt::Debug for ProxyBearer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProxyBearer(..)")
    }
}

impl ProxyBearer {
    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

impl ArtifactsBroker {
    pub(super) async fn proxy_bearer(
        &self,
        viewer: &ArtifactViewer,
    ) -> Result<ProxyBearer, ArtifactCallError> {
        match viewer.credential() {
            Credential::Browser(token) => Ok(ProxyBearer(token.clone())),
            Credential::Person(person) => self.minted_for(person).await,
        }
    }

    // Why: the lock is held across the mint so two calls of one `ui` session
    // arriving together still open a single web session.
    async fn minted_for(&self, person: &PersonViewer) -> Result<ProxyBearer, ArtifactCallError> {
        let user = &person.session.user;
        let key = (user.id.clone(), person.session.session.clone());
        let mut kept = self.inner.minted.kept.lock().await;
        let now = Instant::now();
        if let Some(entry) = kept.get(&key).filter(|entry| entry.renew_at > now) {
            return Ok(ProxyBearer(entry.token.clone()));
        }
        let subject = BrowserSessionSubject {
            user_id: &user.id,
            display_name: &user.username,
            email: &user.email,
            roles: &user.roles,
        };
        let fresh = mint_browser_session(
            &self.inner.parts.session_service,
            self.inner.parts.boot.config(),
            &subject,
            person.session.signed_in,
            &person.analytics,
        )
        .await
        .map_err(|error| {
            tracing::error!(%error, user_id = %user.id, "the broker could not open a web session for a ui session");
            ArtifactCallError::new(
                ArtifactCallCode::Internal,
                "the page's tools could not be reached as you",
            )
        })?;
        let lifetime = Duration::from_secs(u64::try_from(fresh.max_age_secs).unwrap_or(0));
        kept.retain(|_, entry| entry.renew_at > now);
        kept.insert(
            key,
            Kept {
                token: fresh.access_token.clone(),
                renew_at: now + lifetime.saturating_sub(RENEW_BEFORE),
            },
        );
        drop(kept);
        Ok(ProxyBearer(fresh.access_token))
    }
}
