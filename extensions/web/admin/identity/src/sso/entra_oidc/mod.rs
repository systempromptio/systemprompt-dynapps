//! Microsoft Entra ID as an OpenID Connect relying party: the authorization
//! code flow with PKCE, validated against the tenant's discovery document and
//! JWKS.
//!
//! This is the alternative to the SAML trust in `entra_saml` — an App
//! Registration with a client secret instead of an Enterprise Application with
//! federation metadata. State correlation, replay, the identity policy and the
//! session are the core SSO driver's; this module only builds the redirect and
//! turns the callback into an [`ExternalIdentity`].
//!
//! The `redirect_uri` sent at authorize and at code redemption is the same
//! value — the configured one, else the driver's callback URL — since Entra
//! refuses a redemption whose URI differs. `SsoError` has no IdP-refusal
//! variant, so a user cancelling at Entra (`access_denied`) surfaces as an
//! invalid assertion.

mod claims;
mod config;
mod discovery;
mod token;
mod verify;

use std::sync::Arc;

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use sha2::{Digest, Sha256};
use systemprompt::security::keys::JwksClient;
use systemprompt::sso::{
    CookieSameSite, ExternalIdentity, SsoError, SsoHookContext, SsoIdentityPolicy, SsoPaths,
    SsoProtocol, SsoProvider, SsoRedirect, SsoReturn, SsoStart, SsoStateContext, SsoStateSeed,
};

pub use claims::IdTokenClaims;
pub use config::{EntraOidcConfig, ID, install_config};
pub use discovery::OidcDiscovery;
pub use token::ClientSecretSource;
pub use verify::identity_from_claims;

use discovery::DiscoverySource;
use token::{CodeRedemption, redeem_code};

const PATHS: SsoPaths = SsoPaths {
    start: "/admin/auth/entra/start",
    callback: "/admin/auth/entra/callback",
    metadata: None,
};

/// The parameters of one authorization request.
#[derive(Debug, Clone, Copy)]
pub struct AuthorizeRequest<'a> {
    pub authorization_endpoint: &'a str,
    pub client_id: &'a str,
    pub redirect_uri: &'a str,
    pub scopes: &'a [String],
    pub state: &'a str,
    pub nonce: &'a str,
    pub code_challenge: &'a str,
}

#[must_use]
pub fn build_authorize_url(req: &AuthorizeRequest<'_>) -> String {
    let scope = req.scopes.join(" ");
    let params = [
        ("client_id", req.client_id),
        ("response_type", "code"),
        ("redirect_uri", req.redirect_uri),
        ("response_mode", "query"),
        ("scope", scope.as_str()),
        ("state", req.state),
        ("nonce", req.nonce),
        ("code_challenge", req.code_challenge),
        ("code_challenge_method", "S256"),
        ("prompt", "select_account"),
    ];
    let query = params
        .iter()
        .map(|(key, value)| format!("{key}={}", urlencoding::encode(value)))
        .collect::<Vec<_>>()
        .join("&");
    let separator = if req.authorization_endpoint.contains('?') {
        '&'
    } else {
        '?'
    };
    format!("{}{separator}{query}", req.authorization_endpoint)
}

#[must_use]
pub fn pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn random_b64url<const N: usize>() -> String {
    let mut bytes = [0u8; N];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

#[derive(Debug)]
pub struct EntraOidcProvider {
    cfg: Arc<EntraOidcConfig>,
    client_secret: ClientSecretSource,
    issuer: String,
    discovery: DiscoverySource,
    http: reqwest::Client,
    jwks: JwksClient,
}

impl EntraOidcProvider {
    pub fn new(
        cfg: Arc<EntraOidcConfig>,
        client_secret: ClientSecretSource,
    ) -> Result<Self, SsoError> {
        let http = discovery::http_client()?;
        let discovery = DiscoverySource::new(http.clone(), cfg.discovery_url());
        discovery.spawn_refresh();
        Ok(Self {
            issuer: cfg.issuer(),
            jwks: JwksClient::new(cfg.jwks_allowed_hosts.clone()),
            client_secret,
            discovery,
            http,
            cfg,
        })
    }
}

impl EntraOidcProvider {
    fn redirect_uri<'a>(&'a self, callback_url: &'a str) -> &'a str {
        self.cfg.redirect_uri.as_deref().unwrap_or(callback_url)
    }
}

// Why: core calls this hook once at discovery; only the client secret's
// presence is checked, its value is re-read at every code redemption.
pub fn factory(_ctx: &SsoHookContext) -> Option<Arc<dyn SsoProvider>> {
    let cfg = config::installed_config()?;
    if !cfg.is_usable() {
        if cfg.enabled {
            tracing::warn!(
                "Entra OIDC enabled but tenant_id, client_id or redirect_uri is unset; provider off"
            );
        }
        return None;
    }
    let secret = ClientSecretSource::Store(cfg.client_secret_secret.clone());
    if secret.current().is_none() {
        tracing::warn!(
            key = %cfg.client_secret_secret,
            "Entra OIDC client secret missing from the secrets store; provider off"
        );
        return None;
    }
    match EntraOidcProvider::new(cfg, secret) {
        Ok(provider) => Some(Arc::new(provider)),
        Err(e) => {
            tracing::warn!(error = %e, "Entra OIDC provider could not be built; provider off");
            None
        },
    }
}

#[async_trait]
impl SsoProvider for EntraOidcProvider {
    fn id(&self) -> &str {
        ID
    }

    fn issuer(&self) -> &str {
        &self.issuer
    }

    fn protocol(&self) -> SsoProtocol {
        SsoProtocol::Oidc
    }

    fn policy(&self) -> &SsoIdentityPolicy {
        &self.cfg.policy
    }

    fn paths(&self) -> SsoPaths {
        PATHS
    }

    fn login_path(&self) -> &str {
        "/admin/login"
    }

    fn default_return_to(&self) -> &str {
        "/admin"
    }

    fn return_cookie_same_site(&self) -> CookieSameSite {
        CookieSameSite::Lax
    }

    async fn start(&self, input: SsoStart<'_>) -> Result<SsoRedirect, SsoError> {
        let discovery = self.discovery.current().await?;
        let state = random_b64url::<32>();
        let nonce = random_b64url::<32>();
        let verifier = random_b64url::<64>();
        let url = build_authorize_url(&AuthorizeRequest {
            authorization_endpoint: &discovery.authorization_endpoint,
            client_id: &self.cfg.entra_client_id,
            redirect_uri: self.redirect_uri(input.callback_url),
            scopes: &self.cfg.scopes,
            state: &state,
            nonce: &nonce,
            code_challenge: &pkce_challenge(&verifier),
        });
        Ok(SsoRedirect {
            url,
            seed: SsoStateSeed {
                state,
                request_id: None,
                nonce: Some(nonce),
                pkce_verifier: Some(verifier),
            },
        })
    }

    async fn finish(
        &self,
        ret: SsoReturn,
        state: Option<SsoStateContext>,
    ) -> Result<ExternalIdentity, SsoError> {
        let state = state.ok_or(SsoError::StateLost)?;
        if let Some(error) = ret.param("error") {
            return Err(SsoError::InvalidAssertion(if error == "access_denied" {
                "access_denied".to_owned()
            } else {
                format!("idp error: {error}")
            }));
        }
        let code = ret
            .param("code")
            .ok_or_else(|| SsoError::InvalidAssertion("missing code".to_owned()))?;
        let (Some(nonce), Some(verifier)) = (state.nonce, state.pkce_verifier) else {
            return Err(SsoError::StateLost);
        };
        let redirect_uri = self.redirect_uri("");
        let discovery = self.discovery.current().await?;
        let client_secret = self.client_secret.current().ok_or_else(|| {
            SsoError::Unavailable(
                "Entra OIDC client secret missing from the secrets store".to_owned(),
            )
        })?;
        let id_token = redeem_code(
            &self.http,
            &CodeRedemption {
                token_endpoint: &discovery.token_endpoint,
                code,
                redirect_uri,
                client_id: &self.cfg.entra_client_id,
                client_secret: &client_secret,
                code_verifier: &verifier,
            },
        )
        .await?;
        let identity =
            verify::validate_id_token(&id_token, &self.cfg, &discovery, &self.jwks, &nonce).await?;
        Ok(identity)
    }
}
