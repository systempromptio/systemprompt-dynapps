//! The Entra OIDC side of the harness: a config pointed at a running
//! [`FakeOidcIdp`], a real `start`, and the callback the `IdP` would send.

use dynapps_test_common::{FakeOidcIdp, IdTokenSpec};
use systemprompt::sso::SsoIdentityPolicy;
use systemprompt_web_admin_identity::sso::entra_oidc::EntraOidcConfig;

use super::profile::OIDC_SECRET_KEY;
use super::{
    BINDER, EXTERNAL_URL, OIDC_CALLBACK, OIDC_START, OIDC_TENANT, Reply, SsoApp, form_encode, saml,
};

pub(crate) const CLIENT_ID: &str = "11111111-2222-4333-8444-555555555555";

/// Points discovery at `idp` and admits loopback JWKS; the policy matches
/// [`saml::config`]'s.
pub(crate) fn config(idp: &FakeOidcIdp) -> EntraOidcConfig {
    EntraOidcConfig {
        enabled: true,
        tenant_id: OIDC_TENANT.to_owned(),
        client_id: CLIENT_ID.to_owned(),
        client_secret_secret: OIDC_SECRET_KEY.to_owned(),
        redirect_uri: Some(format!("{EXTERNAL_URL}{OIDC_CALLBACK}")),
        discovery_url: Some(idp.discovery_url()),
        jwks_allowed_hosts: vec!["127.0.0.1".to_owned()],
        policy: SsoIdentityPolicy {
            allowed_email_domains: vec![saml::DOMAIN.to_owned()],
            auto_provision: true,
            deny_without_group: true,
            group_roles: [(saml::USERS_GROUP.to_owned(), vec!["user".to_owned()])].into(),
            ..SsoIdentityPolicy::default()
        },
        ..EntraOidcConfig::default()
    }
}

/// What a real `start` sent the browser to the authorize endpoint with.
#[derive(Debug, Clone)]
pub(crate) struct Started {
    pub state: String,
    pub nonce: String,
    pub code_challenge: String,
    pub binder: String,
    pub redirect: Reply,
}

pub(crate) async fn start(app: &SsoApp, path: &str) -> Started {
    let redirect = app.get(path, None).await;
    assert_eq!(
        redirect.status.as_u16(),
        303,
        "start redirects: {redirect:?}"
    );
    let url = url::Url::parse(&redirect.location).expect("authorize URL");
    let param = |k: &str| {
        url.query_pairs().find(|(n, _)| n == k).map_or_else(
            || panic!("authorize URL lacks {k}: {url}"),
            |(_, v)| v.into_owned(),
        )
    };
    Started {
        state: param("state"),
        nonce: param("nonce"),
        code_challenge: param("code_challenge"),
        binder: redirect
            .cookie(BINDER)
            .expect("start sets the binder cookie"),
        redirect,
    }
}

pub(crate) async fn start_default(app: &SsoApp) -> Started {
    start(app, OIDC_START).await
}

/// A valid ID token for `email` answering `started`, in [`saml::USERS_GROUP`].
pub(crate) fn token_for(started: &Started, email: &str, sub: &str) -> IdTokenSpec {
    IdTokenSpec {
        sub: sub.to_owned(),
        oid: format!("oid-{sub}"),
        email: Some(email.to_owned()),
        preferred_username: Some(email.to_owned()),
        name: "Federated Person".to_owned(),
        groups: vec![saml::USERS_GROUP.to_owned()],
        nonce: started.nonce.clone(),
        aud: CLIENT_ID.to_owned(),
        tid: OIDC_TENANT.to_owned(),
        ..IdTokenSpec::default()
    }
}

pub(crate) fn callback_uri(code: &str, state: &str) -> String {
    format!(
        "{OIDC_CALLBACK}?code={}&state={}",
        form_encode(code),
        form_encode(state)
    )
}

/// Mints a code for `spec` at `idp` and returns it to the callback with
/// `started`'s state and binder.
pub(crate) async fn call_back(
    app: &SsoApp,
    idp: &FakeOidcIdp,
    started: &Started,
    spec: IdTokenSpec,
) -> Reply {
    let code = idp.issue_code(spec, &started.code_challenge);
    app.get(&callback_uri(&code, &started.state), Some(&started.binder))
        .await
}
