//! Who is asking: the person signed in to the web, never an agent.
//!
//! The one credential the broker takes is the web login's `access_token`
//! cookie carrying a token core signed `browser_session`, as the Drive pages
//! take it (`routes::drive_actor`), and only from this site's own pages. A
//! request with an `Authorization` header is refused, cookie or not: an
//! agent's or the bridge's bearer must not run a page's tools, and a
//! person's `ui` session cannot either, because core's MCP proxy admits only
//! the first-party audiences and the broker forwards the viewer's own
//! credential to it.
//!
//! The token is kept only to be presented to core's proxy on this one call;
//! its `Debug` never shows it.

use axum::http::{HeaderMap, Method};
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::{AuthenticatedUser, JwtAudience};
use systemprompt::oauth::validate_jwt_token;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use super::error::{ArtifactCallCode, ArtifactCallError};
use crate::routes::drive_actor::{cookie_token, is_browser_session, may_act_from, site_origins};

pub(crate) struct ArtifactViewer {
    pub(crate) user: AuthenticatedUser,
    credential: String,
}

impl std::fmt::Debug for ArtifactViewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactViewer")
            .field("user", &self.user.id)
            .finish_non_exhaustive()
    }
}

impl ArtifactViewer {
    pub(crate) fn from_request(
        boot: &ExtensionBoot,
        headers: &HeaderMap,
    ) -> Result<Self, ArtifactCallError> {
        let sign_in = || {
            ArtifactCallError::new(
                ArtifactCallCode::Reauth,
                "sign in to run this page's live parts",
            )
        };
        let token = cookie_token(headers).ok_or_else(sign_in)?;
        let claims =
            validate_jwt_token(boot.token_authority().as_ref(), &token, &[JwtAudience::Api])
                .map_err(|error| {
                    tracing::debug!(%error, "artifact broker session cookie did not validate");
                    sign_in()
                })?;
        if !is_browser_session(&claims) {
            return Err(sign_in());
        }
        if !may_act_from(&Method::POST, headers, &site_origins(boot)) {
            return Err(ArtifactCallError::new(
                ArtifactCallCode::CrossSite,
                "a browser session runs a page's tools only from this site's pages",
            ));
        }
        let user = AuthenticatedUser::new_with_roles(
            UserId::new(claims.registered.sub),
            claims.custom.username,
            claims.custom.email,
            claims.custom.scope,
            claims.custom.roles,
        );
        Ok(Self {
            user,
            credential: token,
        })
    }

    pub(crate) fn credential(&self) -> &str {
        &self.credential
    }
}
