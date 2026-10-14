//! The Entra SAML side of the harness: a config trusting the fake `IdP`'s
//! metadata, a real `start`, and the ACS form a browser would post back.

use dynapps_test_common::{SamlIdpFixture, SamlResponseSpec, parse_authn_request_redirect};
use systemprompt::sso::SsoIdentityPolicy;
use systemprompt_web_admin_identity::EntraSamlConfig;

use super::{BINDER, EXTERNAL_URL, Reply, SAML_ACS, SAML_START, SsoApp, form_encode};

pub(crate) const DOMAIN: &str = "sso.test";
pub(crate) const USERS_GROUP: &str = "Dynapps-Users";

pub(crate) fn acs_url() -> String {
    format!("{EXTERNAL_URL}{SAML_ACS}")
}

pub(crate) fn entity_id() -> String {
    format!("{EXTERNAL_URL}/saml/metadata")
}

// Trusts [`SamlIdpFixture::default`]; one allowed domain, auto-provisioning
// on, and one directory group granting `user`.
pub(crate) fn config(idp: &SamlIdpFixture) -> EntraSamlConfig {
    let mut cfg = EntraSamlConfig::disabled();
    cfg.enabled = true;
    cfg.entity_id = entity_id();
    cfg.acs_url = acs_url();
    cfg.idp_metadata_path = "fake-idp-metadata.xml".to_owned();
    cfg.idp_metadata_xml = idp.idp_metadata_xml();
    cfg.name_attribute = dynapps_test_common::sso::saml_idp::NAME_ATTR.to_owned();
    cfg.policy = SsoIdentityPolicy {
        allowed_email_domains: vec![DOMAIN.to_owned()],
        auto_provision: true,
        deny_without_group: true,
        group_roles: [(USERS_GROUP.to_owned(), vec!["user".to_owned()])].into(),
        ..SsoIdentityPolicy::default()
    };
    cfg
}

// What a real `start` handed the browser.
#[derive(Debug, Clone)]
pub(crate) struct Started {
    pub request_id: String,
    pub relay_state: String,
    // `sp_sso_bind=<value>`, ready for a `Cookie` header.
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
    let parsed = parse_authn_request_redirect(&redirect.location);
    Started {
        request_id: parsed.request_id,
        relay_state: parsed.relay_state.expect("start sets a RelayState"),
        binder: redirect
            .cookie(BINDER)
            .expect("start sets the binder cookie"),
        redirect,
    }
}

pub(crate) async fn start_default(app: &SsoApp) -> Started {
    start(app, SAML_START).await
}

// A valid response to `started` for `email`, in group [`USERS_GROUP`].
pub(crate) fn response_for(started: &Started, email: &str) -> SamlResponseSpec {
    let mut spec =
        SamlResponseSpec::new(&acs_url(), &entity_id(), Some(&started.request_id), email);
    spec.display_name = Some("Federated Person".to_owned());
    spec.groups = vec![USERS_GROUP.to_owned()];
    spec
}

pub(crate) fn acs_body(saml_response: &str, relay_state: Option<&str>) -> String {
    let mut body = format!("SAMLResponse={}", form_encode(saml_response));
    if let Some(relay) = relay_state {
        body.push_str(&format!("&RelayState={}", form_encode(relay)));
    }
    body
}

// Signs `spec` and posts it to the ACS with `started`'s `RelayState` and
// binder.
pub(crate) async fn post_back(
    app: &SsoApp,
    idp: &SamlIdpFixture,
    started: &Started,
    spec: &SamlResponseSpec,
) -> Reply {
    let body = acs_body(&idp.issue_response(spec), Some(&started.relay_state));
    app.post_form(SAML_ACS, &body, Some(&started.binder)).await
}
