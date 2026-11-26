//! The session handoff: a person's `ui` session opens a web session in the
//! view of a client that embeds the viewer shell.
//!
//! `DynWork` desktop holds the person's `ui` session, which the web pages
//! never take, and shows a web page (the shell, the editor) in a view of its
//! own. It asks `POST /api/v1/drive/handoff {path}` (`issue`) with that
//! session as the bearer, and nothing else: a web cookie, an API key, a
//! Device key or an agent's bearer is refused. The answer is a URL with a
//! one-shot code: 32 random bytes, stored only as their SHA-256, bound to
//! the person, the `/drive/` path and the `ui` session's own sign-in time,
//! good for 60 s and one redeem. The view loads it (`redeem`): the code is
//! spent, core's `mint_browser_session` mints a web session for the person
//! carrying that sign-in time (so the web's freshness and step-up rules see the
//! real sign-in, never the redeem), the web login's cookie is set as core's SSO
//! callback sets it, the login is recorded as a `DynWork` handoff, and the view
//! goes on to the path. Every failure is the same trip to the web login, which
//! comes back to the path. [`handoff_router`] mounts both on every profile.

mod issue;
mod redeem;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::oauth::SessionCreationService;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

pub const ISSUE_PATH: &str = "/api/v1/drive/handoff";
pub const REDEEM_PATH: &str = "/drive/handoff";
const DRIVE_PREFIX: &str = "/drive/";
const MAX_PATH_BYTES: usize = 2048;

/// What the handoff is built from.
///
/// The boot values (the token authority that checks the `ui` session, the
/// site's external URL and the configuration a web session is minted under),
/// the pool the codes live in, and core's session service the web session is
/// minted with.
pub struct HandoffParts {
    pub boot: ExtensionBoot,
    pub pool: Arc<PgPool>,
    pub session_service: Arc<SessionCreationService>,
}

impl std::fmt::Debug for HandoffParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HandoffParts").finish_non_exhaustive()
    }
}

#[derive(Clone)]
struct Handoff(Arc<HandoffParts>);

impl std::fmt::Debug for Handoff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handoff").finish_non_exhaustive()
    }
}

pub fn handoff_router(parts: HandoffParts) -> Router {
    Router::new()
        .route(ISSUE_PATH, post(issue::issue))
        .route(REDEEM_PATH, get(redeem::redeem))
        .with_state(Handoff(Arc::new(parts)))
}

fn hash_code(code: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(code.as_bytes()))
}

// Why: browsers resolve dot segments and their percent forms before asking,
// so a path is checked as resolved on this site, never as written.
fn drive_path(site: &str, raw: &str) -> Option<String> {
    let plain = raw.len() <= MAX_PATH_BYTES
        && raw.starts_with(DRIVE_PREFIX)
        && !raw.contains(['\\', '#'])
        && !raw.chars().any(|c| c.is_control() || c.is_whitespace());
    if !plain {
        return None;
    }
    let base = url::Url::parse(site).ok()?;
    let resolved = base.join(raw).ok()?;
    let same_site = resolved.origin() == base.origin();
    (same_site && resolved.path().starts_with(DRIVE_PREFIX))
        .then(|| resolved[url::Position::BeforePath..url::Position::AfterQuery].to_owned())
}
