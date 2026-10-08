//! The pure half of connector token resolution: who may use a saved grant,
//! and when a grant is re-minted, refreshed, verified or re-sealed.

use systemprompt_web_admin_core::error::AdminError;
use systemprompt_web_admin_platform::connector_oauth::token_plan::{
    AccessMode, Admission, REFRESH_LEEWAY_SECONDS, TokenPlan, needs_refresh, needs_remint,
};
use systemprompt_web_admin_platform::connector_oauth::{Grant, Provider};

const NOW: i64 = 1_800_000_000;

fn grant(auth_method: &str, expires_at: i64) -> Grant {
    Grant {
        configuration_binding: String::new(),
        authorization_issuer: String::new(),
        token_auth_method: String::new(),
        user: "fixture-user".into(),
        provider: Provider::Atlassian,
        client: "fixture-app".into(),
        client_secret: "fixture-secret".into(),
        verifier: String::new(),
        access_token: "token".into(),
        refresh_token: None,
        expires_at,
        token_endpoint: String::new(),
        generation: 0,
        session: None,
        auth_method: auth_method.into(),
        account_id: String::new(),
        account_name: String::new(),
        resource_id: String::new(),
        resource_name: String::new(),
        authorization_scheme: "Bearer".into(),
    }
}

const USE: AccessMode = AccessMode {
    probe: false,
    preauthorize: false,
};
const PROBE: AccessMode = AccessMode {
    probe: true,
    preauthorize: false,
};
const PREAUTHORIZE: AccessMode = AccessMode {
    probe: false,
    preauthorize: true,
};

#[test]
fn admission_with_probe_returns_allowed_even_in_outage_hold() {
    assert_eq!(PROBE.admission(true, false), Admission::Allowed);
}

#[test]
fn admission_with_outage_hold_returns_outage_hold_before_verification() {
    assert_eq!(USE.admission(true, false), Admission::OutageHold);
    assert_eq!(PREAUTHORIZE.admission(true, true), Admission::OutageHold);
}

#[test]
fn admission_with_unverified_grant_returns_unverified() {
    assert_eq!(USE.admission(false, false), Admission::Unverified);
}

#[test]
fn admission_with_verified_or_preauthorized_returns_allowed() {
    assert_eq!(USE.admission(false, true), Admission::Allowed);
    assert_eq!(PREAUTHORIZE.admission(false, false), Admission::Allowed);
}

#[test]
fn require_with_each_admission_returns_the_matching_error() {
    assert!(Admission::Allowed.require().is_ok());
    assert!(matches!(
        Admission::OutageHold.require(),
        Err(AdminError::Unavailable(message))
            if message == "Connector provider recently unavailable; retry shortly"
    ));
    assert!(matches!(
        Admission::Unverified.require(),
        Err(AdminError::Forbidden(message))
            if message == "Test the connection on the Connectors page before using this MCP"
    ));
}

#[test]
fn needs_remint_with_expired_jwt_bearer_returns_true() {
    assert!(needs_remint(&grant("jwt_bearer", NOW), NOW));
    assert!(!needs_remint(&grant("jwt_bearer", NOW + 1), NOW));
    assert!(!needs_remint(&grant("oauth", 0), NOW));
}

#[test]
fn needs_refresh_with_oauth_inside_leeway_returns_true() {
    assert!(needs_refresh(
        &grant("oauth", NOW + REFRESH_LEEWAY_SECONDS),
        NOW
    ));
    assert!(!needs_refresh(
        &grant("oauth", NOW + REFRESH_LEEWAY_SECONDS + 1),
        NOW
    ));
    assert!(!needs_refresh(&grant("personal_token", 0), NOW));
}

#[test]
fn token_plan_with_plain_use_of_fresh_grant_returns_no_work() {
    let plan = TokenPlan::for_grant(&grant("oauth", i64::MAX), USE, false, NOW);
    assert_eq!(
        plan,
        TokenPlan {
            refresh: false,
            verify: false,
            reminted: false,
        }
    );
    assert!(!plan.reseals());
}

#[test]
fn token_plan_with_refresh_due_returns_reseal_without_verify() {
    let plan = TokenPlan::for_grant(&grant("oauth", NOW), USE, false, NOW);
    assert!(plan.refresh);
    assert!(!plan.verify);
    assert!(plan.reseals());
}

#[test]
fn token_plan_with_probe_preauthorize_or_remint_returns_verify() {
    let fresh = grant("oauth", i64::MAX);
    assert!(TokenPlan::for_grant(&fresh, PROBE, false, NOW).verify);
    assert!(TokenPlan::for_grant(&fresh, PREAUTHORIZE, false, NOW).verify);
    let reminted = TokenPlan::for_grant(&fresh, USE, true, NOW);
    assert!(reminted.verify && reminted.reminted && reminted.reseals());
}
