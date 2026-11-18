//! Microsoft Entra ID as the connector adapter of the internal `microsoft-365`
//! MCP server.
//!
//! Core refuses a `connector` block on an internal server, so `settings_for`
//! synthesizes one (`adapter: entra`) in code, and the rest of the connector
//! machinery — accounts, sealed grants, locked refresh, revocation and the
//! credential broker — is reused unchanged. Entra publishes no
//! protected-resource metadata and no PKCE method list, so discovery is
//! skipped: the authorize and token endpoints are fixed under the instance's
//! authority ([`super::entra_tenancy`]), and the client posts its secret with
//! the PKCE verifier. The v2.0 token endpoint rejects `resource`, so the form
//! carries `scope` instead. The tenant of every token is checked twice: its ID
//! token on exchange and refresh ([`super::entra_claims`]), and the recorded
//! tenant against the current allow-list each time a grant holding an access
//! or a refresh token is opened. A grant whose tenant is off the list is
//! withdrawn: the broker deletes it, so its sealed refresh token does not
//! outlive the decision.

use super::entra_claims::{EntraRefusal, refuse, wipe};
use super::entra_tenancy::{EntraTenancy, is_tenant_segment};
use super::failure::{FailureKind, FlowFailure, FlowResult};
use super::{ConnectorAccess, Consent, Grant, Provider, Unopened, config};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use systemprompt::manifest::ServicesConfig;
use systemprompt::models::mcp::deployment::{ConnectorConfig, ConnectorGrant};
use systemprompt::models::mcp::{Deployment, McpServerType};
use systemprompt::models::net::{LoopbackPolicy, validate_outbound_url};
use systemprompt::traits::SecretStore;
use systemprompt_web_admin_core::error::{AdminError, AdminResult};
use url::Url;

pub const SERVER_ID: &str = "microsoft-365";
pub const ADAPTER: &str = "entra";
pub const DISPLAY_NAME: &str = "Microsoft 365";
pub const CLIENT_ID_SECRET: &str = "microsoft_365_client_id";
pub const CLIENT_SECRET_SECRET: &str = "microsoft_365_client_secret";
pub const GRAPH_RESOURCE: &str = "https://graph.microsoft.com";
const TOKEN_AUTH_METHOD: &str = "client_secret_post";

// Why: parity with the delegated Microsoft Graph permissions of the Belgian
// `dynapps-365` server (`GRAPH_SCOPES`), so every tool keeps its permission;
// Entra issues a refresh token only with `offline_access` and an ID token only
// with `openid`.
pub const SCOPES: &[&str] = &[
    "Mail.Read",
    "Mail.ReadWrite",
    "Mail.Send",
    "Calendars.Read",
    "Calendars.ReadWrite",
    "Chat.Read",
    "Chat.ReadWrite",
    "ChannelMessage.Read.All",
    "ChannelMessage.Send",
    "Files.Read.All",
    "Files.ReadWrite.All",
    "Sites.Read.All",
    "Sites.ReadWrite.All",
    "Contacts.Read",
    "Contacts.ReadWrite",
    "Tasks.Read",
    "Tasks.ReadWrite",
    "Notes.Read",
    "Notes.ReadWrite",
    "OnlineMeetings.Read",
    "OnlineMeetings.ReadWrite",
    "OnlineMeetingTranscript.Read.All",
    "OnlineMeetingRecording.Read.All",
    "Group.Read.All",
    "Group.ReadWrite.All",
    "User.Read",
    "User.Read.All",
    "Directory.Read.All",
    "Presence.Read",
    "Presence.ReadWrite",
    "Team.ReadBasic.All",
    "Channel.ReadBasic.All",
    "TeamMember.Read.All",
    "ChatMessage.Send",
    "offline_access",
    "openid",
    "profile",
    "email",
];

pub(super) fn settings_for(id: &str, server: &Deployment) -> Option<ConnectorConfig> {
    (id == SERVER_ID && server.server_type == McpServerType::Internal).then(connector)
}

fn connector() -> ConnectorConfig {
    ConnectorConfig {
        adapter: ADAPTER.to_owned(),
        display_name: Some(DISPLAY_NAME.to_owned()),
        scopes: SCOPES.iter().map(|scope| (*scope).to_owned()).collect(),
        authorization_origins: Vec::new(),
        authorization_params: BTreeMap::new(),
        identity: None,
        client_id_secret: Some(CLIENT_ID_SECRET.to_owned()),
        client_secret: Some(CLIENT_SECRET_SECRET.to_owned()),
        grant: ConnectorGrant::AuthorizationCode,
        token_url: None,
        private_key_secret: None,
        installation_id: None,
        app_id: None,
    }
}

pub fn settings(services: &ServicesConfig, provider: &Provider) -> Option<ConnectorConfig> {
    settings_for(provider.slug(), services.mcp_servers.get(provider.slug())?)
}

pub fn is_entra(services: &ServicesConfig, provider: &Provider) -> bool {
    settings(services, provider).is_some()
}

pub fn authorize(
    access: ConnectorAccess<'_>,
    consent: Consent<'_>,
    provider: Provider,
    callback: &str,
) -> AdminResult<(String, Grant)> {
    let settings = settings(&access.services.config(), &provider)
        .ok_or_else(|| AdminError::Unavailable("OAuth setup required".into()))?;
    let tenancy = EntraTenancy::load(access.secrets)?;
    let client = client_id(access.secrets)?;
    let client_secret = config::secret(access.secrets, CLIENT_SECRET_SECRET)?;
    let Consent {
        user,
        state,
        verifier,
    } = consent;
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let scope = settings.scopes.join(" ");
    let mut url = tenancy.authorize_url();
    url.query_pairs_mut().extend_pairs([
        ("client_id", client.as_str()),
        ("response_type", "code"),
        ("redirect_uri", callback),
        ("scope", scope.as_str()),
        ("state", state),
        ("code_challenge_method", "S256"),
        ("code_challenge", challenge.as_str()),
    ]);
    let grant = Grant {
        user: user.into(),
        configuration_binding: super::generic::binding(access.services, &provider)?,
        authorization_issuer: String::new(),
        token_auth_method: TOKEN_AUTH_METHOD.into(),
        provider,
        client,
        client_secret,
        verifier,
        access_token: String::new(),
        refresh_token: None,
        expires_at: 0,
        token_endpoint: tenancy.token_url().into(),
        generation: 0,
        session: None,
        auth_method: "oauth".into(),
        account_id: String::new(),
        account_name: String::new(),
        resource_id: String::new(),
        resource_name: String::new(),
        authorization_scheme: "Bearer".into(),
    };
    Ok((url.into(), grant))
}

pub async fn exchange_with_client(
    access: ConnectorAccess<'_>,
    grant: &mut Grant,
    code: &str,
    callback: &str,
    http: &reqwest::Client,
) -> AdminResult<()> {
    super::tokens::exchange_request(
        access.services,
        Some(access.secrets),
        grant,
        super::AuthorizationCode { code, callback },
        http,
    )
    .await
    .map_err(|failure| *failure.error)
}

pub(super) fn trusted_token_endpoint(value: &str) -> AdminResult<Url> {
    let url = validate_outbound_url(value, LoopbackPolicy::Allow)?;
    let segment = url
        .path()
        .strip_prefix('/')
        .and_then(|path| path.strip_suffix("/oauth2/v2.0/token"));
    let shaped = url.scheme() == "https"
        && url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none()
        && segment.is_some_and(is_tenant_segment);
    if !shaped {
        return Err(AdminError::Unavailable(
            "OAuth endpoint is outside configured trusted origins".into(),
        ));
    }
    Ok(url)
}

pub(super) fn token_form(
    settings: &ConnectorConfig,
    grant: &Grant,
    fields: &[(&str, &str)],
) -> String {
    let mut form = url::form_urlencoded::Serializer::new(String::new());
    form.extend_pairs(fields)
        .append_pair("client_id", &grant.client)
        .append_pair("scope", &settings.scopes.join(" "));
    if !grant.client_secret.is_empty() {
        form.append_pair("client_secret", &grant.client_secret);
    }
    form.finish()
}

fn client_id(secrets: &dyn SecretStore) -> AdminResult<String> {
    config::secret(secrets, CLIENT_ID_SECRET).map(|id| id.trim().to_ascii_lowercase())
}

pub(super) fn reconcile(secrets: &dyn SecretStore, grant: &mut Grant) -> Result<(), Unopened> {
    let tenancy = EntraTenancy::load(secrets)?;
    let client = client_id(secrets)?;
    if grant.token_endpoint != tenancy.token_url().as_str() || grant.client != client {
        return Err(AdminError::Unauthorized(
            "Connector configuration changed; reconnect required".into(),
        )
        .into());
    }
    let holds_tokens = !grant.access_token.is_empty() || grant.refresh_token.is_some();
    if holds_tokens && !tenancy.admits(&grant.resource_id) {
        return Err(withdraw(grant));
    }
    grant.client_secret = config::secret(secrets, CLIENT_SECRET_SECRET)?;
    Ok(())
}

fn withdraw(grant: &mut Grant) -> Unopened {
    tracing::warn!(target: "connector_oauth", provider = grant.provider.slug(), user_id = %grant.user,
        stage = "open", reason = EntraRefusal::NotAllowed.code(), tenant = grant.resource_id.as_str(),
        "Microsoft 365 grant withdrawn: its tenant is off the allow-list");
    wipe(grant);
    Unopened {
        error: AdminError::Unauthorized(
            "Microsoft 365 tenant is not allowed on this instance; reconnect required".into(),
        ),
        withdrawn: true,
    }
}

pub(super) fn allow_list_unavailable() -> FlowFailure {
    FlowFailure {
        kind: FailureKind::Configuration,
        error: Box::new(AdminError::Unavailable(
            "Microsoft 365 tenants are admitted only with the secrets store".into(),
        )),
    }
}

pub(super) fn admit_tenant(secrets: &dyn SecretStore, grant: &mut Grant) -> FlowResult<()> {
    match EntraTenancy::load(secrets) {
        Ok(tenancy) if tenancy.admits(&grant.resource_id) => Ok(()),
        Ok(_) => {
            let tenant = grant.resource_id.clone();
            Err(refuse(
                grant,
                false,
                EntraRefusal::NotAllowed,
                Some(&tenant),
            ))
        },
        Err(error) => {
            wipe(grant);
            Err(FlowFailure {
                kind: FailureKind::Configuration,
                error: Box::new(error),
            })
        },
    }
}
