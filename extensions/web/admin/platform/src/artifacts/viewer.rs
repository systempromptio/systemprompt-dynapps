//! Who is asking: a person, never an agent.
//!
//! Two credentials are taken. The web login's `access_token` cookie carrying
//! a token core signed `browser_session`, as the Drive pages take it
//! (`routes::drive_actor`), and only from this site's own pages. Or the
//! person's `ui` session as the bearer, the session a native client such as
//! `DynWork` desktop holds for its person, checked as the session handoff
//! checks it (`routes::person_session`); a bearer request carries no ambient
//! credential, so no Origin check applies to it. Any other bearer, an
//! agent's, the bridge's, an API key, a Device key or the web session sent as
//! a bearer, is refused `reauth`, cookie or not.
//!
//! Core's MCP proxy admits only the first-party audiences, never `ui`. The
//! cookie's token is presented to it as it came; for a `ui` session the
//! broker presents a web session it mints for the same person (`minted`).
//! Neither is ever answered or shown by `Debug`.

use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, Method};
use systemprompt::analytics::SessionAnalyticsBuilder;
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::{AuthenticatedUser, JwtAudience};
use systemprompt::oauth::validate_jwt_token;
use systemprompt::traits::SessionAnalytics;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;

use super::error::{ArtifactCallCode, ArtifactCallError};
use crate::routes::drive_actor::{cookie_token, is_browser_session, may_act_from, site_origins};
use crate::routes::person_session::{PersonSession, person_session};

pub(crate) struct ArtifactViewer {
    pub(crate) user: AuthenticatedUser,
    credential: Credential,
}

pub(super) enum Credential {
    Browser(String),
    Person(Box<PersonViewer>),
}

// Why: a web session is minted for a `ui` session only when a call reaches
// core's proxy, with the analytics of the request that needed it.
pub(super) struct PersonViewer {
    pub(super) session: PersonSession,
    pub(super) analytics: SessionAnalytics,
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
        if headers.contains_key(AUTHORIZATION) {
            let person = person_session(boot, headers).ok_or_else(sign_in)?;
            return Ok(Self {
                user: person.user.clone(),
                credential: Credential::Person(Box::new(PersonViewer {
                    session: person,
                    analytics: SessionAnalyticsBuilder::new(headers).build(),
                })),
            });
        }
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
            credential: Credential::Browser(token),
        })
    }

    pub(super) const fn credential(&self) -> &Credential {
        &self.credential
    }
}
