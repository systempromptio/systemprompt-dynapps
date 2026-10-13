//! "Sign in with Dynapps SSO (Microsoft Entra ID)" through core's SSO driver —
//! the branches reachable without a live tenant.
//!
//! SSO reports failure by *redirecting*, not by returning an error status: a
//! `500` on the callback would strand the browser on a dead end instead of
//! returning it to a usable login page. Every case here therefore asserts on
//! the `Location` header — `?sso=<reason>` — because the status is `303` on
//! success and failure alike and proves nothing on its own.
//!
//! The suite runs the router twice. Once with no provider installed, which is
//! how [`crate::app::App::new`] builds it and which pins the "unavailable"
//! response every sign-in route owes. Once with an Entra ID-shaped federation
//! metadata fixture (placeholder tenant, throwaway self-signed certificate),
//! so the `AuthnRequest`, the driver's binder cookie, and the callback's
//! validation ladder — including a genuine XML-DSig check against the pinned
//! certificate — run for real, offline. Core's generic
//! `/api/v1/core/sso/{id}/*` routes are mounted by core, not this router, and
//! are not driven here.

use systemprompt::sso::SsoIdentityPolicy;
use systemprompt_web_admin_identity::EntraSamlConfig;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const START: &str = "/admin/auth/adfs/start";
const ACS_PATH: &str = "/admin/auth/adfs/acs";
const ACS: &str = "https://dynapps.systemprompt.io/admin/auth/adfs/acs";
const OIDC_START: &str = "/admin/auth/entra/start";
const OIDC_CALLBACK: &str = "/admin/auth/entra/callback?code=abc&state=xyz";
const METADATA: &str = "/saml/metadata";
const ENTRA_SSO: &str =
    "https://login.microsoftonline.com/00000000-0000-4000-8000-000000000e17/saml2";
// A well-formed but unsigned Response — base64 of a minimal document.
const UNSIGNED: &str = "PHNhbWxwOlJlc3BvbnNlIHhtbG5zOnNhbWxwPSJ1cm46b2FzaXM6bmFtZXM6dGM6U0FNTDoyLjA6cHJvdG9jb2wiIElEPSJfeCIgVmVyc2lvbj0iMi4wIiBJc3N1ZUluc3RhbnQ9IjIwMjYtMDEtMDFUMDA6MDA6MDBaIi8+";
const PRINCIPALS: [Principal; 3] = [Principal::Anonymous, Principal::NonAdmin, Principal::Admin];

fn configured() -> EntraSamlConfig {
    let metadata =
        globals::repo_root().join("tests/contract/admin/fixtures/entra-federation-metadata.xml");
    let mut cfg = EntraSamlConfig::disabled();
    cfg.enabled = true;
    cfg.entity_id = "https://dynapps.systemprompt.io/saml/metadata".to_owned();
    cfg.acs_url = ACS.to_owned();
    cfg.idp_metadata_path = "entra-federation-metadata.xml".to_owned();
    cfg.idp_metadata_xml = std::fs::read_to_string(metadata).expect("Entra ID metadata fixture");
    cfg.policy = SsoIdentityPolicy {
        allowed_email_domains: vec!["contract.test".to_owned()],
        auto_provision: true,
        deny_without_group: true,
        group_roles: [("Systemprompt-Users".to_owned(), vec!["user".to_owned()])].into(),
        ..SsoIdentityPolicy::default()
    };
    cfg
}

fn acs_post(body: &str) -> Call<'_> {
    Call {
        method: "post",
        path: ACS_PATH,
        principal: Principal::Anonymous,
        content_type: Some("application/x-www-form-urlencoded"),
        body: Some(body),
    }
}

fn report(kind: &str, failures: &[String]) {
    assert!(
        failures.is_empty(),
        "{} {kind} case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// With no provider installed, every sign-in route reports unavailable rather
// than pretending the flow can start, and there is no metadata to publish.
#[tokio::test(flavor = "multi_thread")]
async fn sso_routes_report_unavailable_when_sso_is_not_configured() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping SSO suite");
        return;
    };
    let app = App::new(&db.pool, principal::provision(&db.pool).await);

    let mut failures = Vec::new();
    for principal in PRINCIPALS {
        for call in [
            Call::get(START, principal),
            Call::get("/admin/auth/adfs/start?redirect=/admin/profile", principal),
            Call {
                principal,
                ..acs_post("SAMLResponse=abc")
            },
            Call::get(OIDC_START, principal),
            Call::get(OIDC_CALLBACK, principal),
        ] {
            let path = call.path;
            let (status, location) = app.redirect_of(call).await;
            if !status.is_redirection() || !location.contains("/admin/login?sso=unavailable") {
                failures.push(format!(
                    "  {path} as {principal:?} -> {} {location:?}, expected a redirect to ?sso=unavailable",
                    status.as_u16()
                ));
            }
        }
        let (status, _) = app.response_headers(Call::get(METADATA, principal)).await;
        if status.as_u16() != 404 {
            failures.push(format!(
                "  {METADATA} as {principal:?} -> {status}, expected 404"
            ));
        }
    }

    db.cleanup().await;
    report("disabled-SSO", &failures);
}

// With SSO configured, `start` redirects to Entra ID with an AuthnRequest and
// a RelayState, and the driver binds the flow to the browser with a cookie
// scoped to the ACS.
#[tokio::test(flavor = "multi_thread")]
async fn sso_start_redirects_to_entra_id_with_a_request_and_a_binder_cookie() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let app = App::with_sso(&db.pool, principal::provision(&db.pool).await, configured());

    let mut failures = Vec::new();
    for path in [
        START,
        "/admin/auth/adfs/start?redirect=/admin/profile",
        // An off-site redirect target is replaced by the default rather than
        // honoured — this is the open-redirect gate.
        "/admin/auth/adfs/start?redirect=https://evil.example/steal",
    ] {
        let (status, headers) = app
            .response_headers(Call::get(path, Principal::Anonymous))
            .await;
        if !status.is_redirection() {
            failures.push(format!(
                "  {path} -> {} (expected a redirect)",
                status.as_u16()
            ));
            continue;
        }
        let target = headers.location.clone().unwrap_or_default();
        for marker in [ENTRA_SSO, "SAMLRequest=", "RelayState="] {
            if !target.contains(marker) {
                failures.push(format!("  {path} -> SSO URL {target:?} lacks {marker:?}"));
            }
        }
        let cookie = headers.set_cookie.join(" ");
        for marker in [
            "sp_sso_bind=",
            "HttpOnly",
            "Path=/admin/auth/adfs/acs",
            "SameSite=",
        ] {
            if !cookie.contains(marker) {
                failures.push(format!(
                    "  {path} -> binder cookie lacks {marker:?}: {cookie:?}"
                ));
            }
        }
        if cookie.contains("evil.example") || target.contains("evil.example") {
            failures.push(format!("  an off-site redirect target leaked: {cookie:?}"));
        }
    }

    let (status, _) = app
        .response_headers(Call::get(METADATA, Principal::Anonymous))
        .await;
    if status.as_u16() != 200 {
        failures.push(format!(
            "  {METADATA} -> {status}, expected the SP descriptor"
        ));
    }

    db.cleanup().await;
    report("SSO start", &failures);
}

// The RelayState and binder cookie a real `start` issued, so the ladder can
// be entered above the driver's correlation check.
async fn started_flow(app: &App) -> (String, String) {
    let (_, headers) = app
        .response_headers(Call::get(START, Principal::Anonymous))
        .await;
    let target = headers.location.unwrap_or_default();
    let relay = target
        .split(['?', '&'])
        .find_map(|kv| kv.strip_prefix("RelayState="))
        .expect("start redirect carries a RelayState")
        .to_owned();
    let binder = headers
        .set_cookie
        .iter()
        .find_map(|c| {
            c.split(';')
                .next()
                .filter(|kv| kv.starts_with("sp_sso_bind="))
        })
        .expect("start sets the binder cookie")
        .to_owned();
    (relay, binder)
}

// The callback's validation ladder, each rung reached by a request that is
// wrong in exactly one way.
#[tokio::test(flavor = "multi_thread")]
async fn sso_callback_rejects_every_malformed_return() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let app = App::with_sso(&db.pool, principal::provision(&db.pool).await, configured());
    let (relay, binder) = started_flow(&app).await;
    let (other_relay, _) = started_flow(&app).await;

    let solicited = format!("SAMLResponse={UNSIGNED}&RelayState={relay}");
    let cases: Vec<(&str, String, Option<String>, &str)> = vec![
        // IdP-initiated is allowed, so an empty post reaches the provider and
        // fails there for want of a SAMLResponse.
        (
            "no form body at all",
            String::new(),
            None,
            "sso=invalid_assertion",
        ),
        // With IdP-initiated sign-in allowed, a RelayState the driver never
        // issued is treated as unsolicited and verified as such.
        (
            "a RelayState this instance never issued",
            "SAMLResponse=abc&RelayState=someone-elses-state".to_owned(),
            Some(binder.clone()),
            "sso=invalid_assertion",
        ),
        (
            "a real RelayState with another browser's binder",
            format!("SAMLResponse={UNSIGNED}&RelayState={other_relay}"),
            Some("sp_sso_bind=not-the-binder".to_owned()),
            "sso=correlation_lost",
        ),
        (
            "a response that is not base64",
            "SAMLResponse=%21%21%21".to_owned(),
            None,
            "sso=invalid_assertion",
        ),
        (
            "an unsigned response with a matching RelayState and binder",
            solicited,
            Some(binder),
            "sso=invalid_assertion",
        ),
        (
            "an unsolicited unsigned response",
            format!("SAMLResponse={UNSIGNED}"),
            None,
            "sso=invalid_assertion",
        ),
    ];

    let mut failures = Vec::new();
    for (label, body, cookie, marker) in &cases {
        let headers: Vec<(&str, &str)> = cookie
            .as_deref()
            .map(|c| vec![("cookie", c)])
            .unwrap_or_default();
        let (status, location) = app.redirect_with_headers(acs_post(body), &headers).await;
        if !status.is_redirection() || !location.contains(marker) {
            failures.push(format!(
                "  {label} -> {} {location:?}, expected a redirect to {marker:?}",
                status.as_u16()
            ));
        }
    }

    db.cleanup().await;
    report("SSO callback", &failures);
}
