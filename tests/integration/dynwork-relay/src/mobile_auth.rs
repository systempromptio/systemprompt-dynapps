//! The mobile sign-in (path 1) through the extension router: consent behind
//! the instance's browser session, Allow and Deny, the code and PKCE
//! exchange, refresh, the session check, the device socket and revocation.

use axum::http::{Method, StatusCode, header};
use dynwork_relay::audit::AuditEvent;
use dynwork_relay::mobile_auth::params::{challenge_of, percent_encode};
use dynwork_relay::{MOBILE_BASE_PATH, RelayLimits};
use serde_json::Value;
use systemprompt::models::auth::Permission;
use systemprompt_api::services::middleware::should_skip_session_tracking;

use crate::mobile_support::{
    Phone, approved_code, authorize, browser_session, decide, exchange, query_value, request_token,
    send, sign_in, with_bearer,
};
use crate::support::{Relay, desktop_token, peer, refused_status};

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value[key].as_str().unwrap_or_default()
}

#[tokio::test]
async fn discovery_names_every_route_and_the_constant_redirect() {
    let relay = Relay::start().await;
    let answer = send(&relay, Method::GET, "/discovery", &[], None).await;
    assert_eq!(answer.status, StatusCode::OK);
    let body = answer.json();
    assert_eq!(body["redirect_uri"], "dynwork://auth/callback");
    assert_eq!(
        body["authorize_path"],
        format!("{MOBILE_BASE_PATH}/authorize")
    );
    assert_eq!(body["device_socket_path"], "/api/dynwork/relay/v1/device");
    assert_eq!(body["code_challenge_methods"][0], "S256");
    for path in [
        "/authorize",
        "/authorize/decision",
        "/token",
        "/refresh",
        "/session",
    ] {
        assert!(should_skip_session_tracking(&format!(
            "{MOBILE_BASE_PATH}{path}"
        )));
    }
}

#[tokio::test]
async fn a_signed_out_browser_goes_through_the_instance_login_and_back() {
    let relay = Relay::start().await;
    let phone = Phone::new(1);
    let path = phone.authorize_path("Pixel");
    let answer = send(&relay, Method::GET, &path, &[], None).await;
    assert_eq!(answer.status, StatusCode::TEMPORARY_REDIRECT);
    let back = format!("{MOBILE_BASE_PATH}{path}");
    let encoded = query_value(answer.location(), "redirect").expect("a redirect");
    assert!(answer.location().starts_with("/admin/login?redirect="));
    assert!(encoded.starts_with("%2Fapi%2Fdynwork%2Fmobile%2Fv1%2Fauthorize%3Fredirect_uri%3D"));
    assert_eq!(encoded, percent_encode(&back));
}

#[tokio::test]
async fn only_a_browser_session_of_a_person_opens_the_consent_page() {
    let relay = Relay::start().await;
    let path = Phone::new(1).authorize_path("Pixel");
    let bearer = format!("Bearer {}", desktop_token("alice"));
    let not_browser = format!(
        "access_token={}",
        browser_session("alice", vec![Permission::User], false)
    );
    let anonymous = format!(
        "access_token={}",
        browser_session("alice", vec![Permission::Anonymous], true)
    );
    let bridge = format!("access_token={}", desktop_token("alice"));
    for headers in [
        vec![("authorization", bearer.as_str())],
        vec![("cookie", not_browser.as_str())],
        vec![("cookie", anonymous.as_str())],
        vec![("cookie", bridge.as_str())],
        vec![("cookie", "access_token=not-a-jwt")],
    ] {
        let answer = send(&relay, Method::GET, &path, &headers, None).await;
        assert_eq!(answer.status, StatusCode::TEMPORARY_REDIRECT, "{headers:?}");
    }
}

#[tokio::test]
async fn a_malformed_request_is_refused_before_any_login_redirect() {
    let relay = Relay::start().await;
    let phone = Phone::new(1);
    let good = phone.authorize_path("Pixel");
    let cases = [
        good.replace(
            "dynwork%3A%2F%2Fauth%2Fcallback",
            "https%3A%2F%2Fevil.example%2Fcb",
        ),
        good.replace(
            "dynwork%3A%2F%2Fauth%2Fcallback",
            "dynwork%3A%2F%2Fauth%2Fcallback%2Fx",
        ),
        good.replace("S256", "plain"),
        good.replace(&challenge_of(&phone.verifier), "short"),
        good.replace(&phone.state, "too-short"),
        good.replace(&phone.state, &format!("{}%20x", phone.state)),
        good.replace("device_name=Pixel", "device_name=%07bell"),
        good.replace("&state=", "&state=x&state="),
    ];
    for path in cases {
        for headers in [vec![], vec![("cookie", "")]] {
            let answer = send(&relay, Method::GET, &path, &headers, None).await;
            assert_eq!(answer.status, StatusCode::BAD_REQUEST, "{path}");
            assert!(answer.headers.get(header::LOCATION).is_none());
        }
    }
}

#[tokio::test]
async fn the_consent_page_escapes_what_it_shows_and_forbids_framing_and_scripts() {
    let relay = Relay::start().await;
    let page = authorize(&relay, "alice", &Phone::new(1), "<b>Pixel</b>").await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.body.contains("&lt;b&gt;Pixel&lt;/b&gt;"));
    assert!(!page.body.contains("<b>Pixel"));
    assert!(page.body.contains("alice@example.test"));
    assert!(!page.body.contains("<script"));
    let csp = page.headers[header::CONTENT_SECURITY_POLICY]
        .to_str()
        .unwrap_or_default();
    assert!(csp.contains("frame-ancestors 'none'") && csp.contains("form-action 'self' dynwork:"));
    assert_eq!(page.headers[header::CACHE_CONTROL], "no-store");
}

#[tokio::test]
async fn path_one_round_trip_signs_a_phone_in_and_the_relay_accepts_its_credential() {
    let relay = Relay::start().await;
    let phone = Phone::new(1);
    let page = authorize(&relay, "alice", &phone, "Pixel 9").await;
    let answer = decide(&relay, Some("alice"), &request_token(&page.body), "allow").await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER);
    assert!(
        answer
            .location()
            .starts_with("dynwork://auth/callback?code=dwa1_")
    );
    assert_eq!(
        query_value(answer.location(), "state").as_deref(),
        Some(phone.state.as_str())
    );
    let code = query_value(answer.location(), "code").expect("a code");
    let token = exchange(&relay, &code, &phone.verifier, phone.peer).await;
    assert_eq!(token.status, StatusCode::OK, "{}", token.body);
    assert_eq!(token.headers[header::CACHE_CONTROL], "no-store");
    let body = token.json();
    let credential = text(&body, "credential").to_owned();
    assert!(credential.starts_with("dwm1_"));
    assert_eq!(body["peer_id"], phone.peer.to_hex());
    assert_eq!(body["expires_in"], 3600);
    assert_eq!(body["account"]["email"], "alice@example.test");

    let session = with_bearer(&relay, Method::GET, "/session", text(&body, "token")).await;
    assert_eq!(session.status, StatusCode::OK, "{}", session.body);
    assert_eq!(session.json()["device_id"], body["device_id"]);
    let refreshed = with_bearer(&relay, Method::POST, "/refresh", &credential).await;
    assert_eq!(refreshed.status, StatusCode::OK);
    assert_eq!(refreshed.json()["device_id"], body["device_id"]);

    relay.phone(&credential).await;
    let (status, devices) = relay
        .request(Method::GET, "/devices", Some(&desktop_token("alice")), None)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(devices[0]["name"], "Pixel 9");
    assert_eq!(devices[0]["kind"], "phone");
    let events = relay.audit.events();
    assert!(events.contains(&AuditEvent::MobileConsent {
        user: crate::support::user("alice"),
        approved: true
    }));
    assert!(
        events
            .iter()
            .any(|event| matches!(event, AuditEvent::MobileSignIn { .. }))
    );
    assert!(!format!("{events:?}").contains(&credential));
}

#[tokio::test]
async fn revoking_the_device_ends_refresh_session_and_socket() {
    let relay = Relay::start().await;
    let body = sign_in(&relay, "alice", &Phone::new(1)).await;
    let credential = text(&body, "credential");
    let (status, _) = relay
        .request(
            Method::DELETE,
            &format!("/devices/{}", text(&body, "device_id")),
            Some(&desktop_token("alice")),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        with_bearer(&relay, Method::POST, "/refresh", credential)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        with_bearer(&relay, Method::GET, "/session", text(&body, "token"))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(refused_status(relay.connect_phone(credential).await), 401);
}

#[tokio::test]
async fn the_phone_signs_itself_out() {
    let relay = Relay::start().await;
    let body = sign_in(&relay, "alice", &Phone::new(1)).await;
    let credential = text(&body, "credential");
    assert_eq!(
        with_bearer(&relay, Method::POST, "/revoke", credential)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        with_bearer(&relay, Method::POST, "/refresh", credential)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        with_bearer(&relay, Method::POST, "/revoke", credential)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn a_code_works_once_and_its_replay_revokes_what_it_issued() {
    let relay = Relay::start().await;
    let phone = Phone::new(1);
    let code = approved_code(&relay, "alice", &phone).await;
    let first = exchange(&relay, &code, &phone.verifier, phone.peer).await;
    assert_eq!(first.status, StatusCode::OK);
    let again = exchange(&relay, &code, &phone.verifier, phone.peer).await;
    assert_eq!(again.status, StatusCode::BAD_REQUEST);
    assert_eq!(again.json()["error"], "invalid_grant");
    let credential = text(&first.json(), "credential").to_owned();
    assert_eq!(
        with_bearer(&relay, Method::POST, "/refresh", &credential)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn a_wrong_verifier_burns_the_code() {
    let relay = Relay::start().await;
    let phone = Phone::new(1);
    let code = approved_code(&relay, "alice", &phone).await;
    let wrong = exchange(&relay, &code, &Phone::new(2).verifier, phone.peer).await;
    assert_eq!(
        (wrong.status, wrong.json()["error"].clone()),
        (StatusCode::BAD_REQUEST, "invalid_grant".into())
    );
    let right = exchange(&relay, &code, &phone.verifier, phone.peer).await;
    assert_eq!(right.json()["error"], "invalid_grant");
    let malformed = exchange(&relay, &code, "short", phone.peer).await;
    assert_eq!(malformed.json()["error"], "bad_request");
    let made_up = exchange(&relay, "dwa1_made-up", &phone.verifier, phone.peer).await;
    assert_eq!(made_up.json()["error"], "invalid_grant");
}

#[tokio::test]
async fn a_code_lives_ten_minutes() {
    let relay = Relay::start().await;
    let phone = Phone::new(1);
    let code = approved_code(&relay, "alice", &phone).await;
    relay.clock.advance(chrono::Duration::minutes(10));
    let late = exchange(&relay, &code, &phone.verifier, phone.peer).await;
    assert_eq!(late.json()["error"], "invalid_grant");
}

#[tokio::test]
async fn deny_comes_back_with_access_denied_and_the_state() {
    let relay = Relay::start().await;
    let phone = Phone::new(1);
    let page = authorize(&relay, "alice", &phone, "Pixel").await;
    let request = request_token(&page.body);
    let answer = decide(&relay, Some("alice"), &request, "deny").await;
    assert_eq!(answer.status, StatusCode::SEE_OTHER);
    assert_eq!(
        answer.location(),
        format!(
            "dynwork://auth/callback?error=access_denied&state={}",
            phone.state
        )
    );
    assert_eq!(
        decide(&relay, Some("alice"), &request, "allow")
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn a_decision_needs_the_same_signed_in_person_in_time() {
    let relay = Relay::start().await;
    let page = authorize(&relay, "alice", &Phone::new(1), "Pixel").await;
    let request = request_token(&page.body);
    assert_eq!(
        decide(&relay, None, &request, "allow").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        decide(&relay, Some("mallory"), &request, "allow")
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        decide(&relay, Some("alice"), &request, "maybe")
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        decide(&relay, Some("alice"), "dwq1_forged", "allow")
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    relay.clock.advance(chrono::Duration::minutes(10));
    assert_eq!(
        decide(&relay, Some("alice"), &request, "allow")
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn open_consents_are_capped_per_person() {
    let relay = Relay::start().await;
    for _ in 0..10 {
        assert_eq!(
            authorize(&relay, "alice", &Phone::new(1), "Pixel")
                .await
                .status,
            StatusCode::OK
        );
    }
    let capped = authorize(&relay, "alice", &Phone::new(1), "Pixel").await;
    assert_eq!(capped.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        authorize(&relay, "mallory", &Phone::new(1), "Pixel")
            .await
            .status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn signing_in_again_replaces_the_phones_previous_device() {
    let relay = Relay::start().await;
    let phone = Phone::new(1);
    let first = sign_in(&relay, "alice", &phone).await;
    let second = sign_in(&relay, "alice", &phone).await;
    assert_ne!(first["device_id"], second["device_id"]);
    assert_eq!(
        with_bearer(&relay, Method::POST, "/refresh", text(&first, "credential"))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    let (_, devices) = relay
        .request(Method::GET, "/devices", Some(&desktop_token("alice")), None)
        .await;
    assert_eq!(devices.as_array().map(Vec::len), Some(1));
}

#[tokio::test]
async fn a_mobile_token_is_not_a_desktop_credential_and_a_desktop_token_is_not_a_mobile_one() {
    let relay = Relay::start().await;
    let body = sign_in(&relay, "alice", &Phone::new(1)).await;
    let jwt = text(&body, "token");
    assert_eq!(
        refused_status(relay.connect_desktop(jwt, Some(peer(9))).await),
        401
    );
    let (status, _) = relay
        .request(Method::POST, "/pairings", Some(jwt), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(refused_status(relay.connect_phone(jwt).await), 401);
    let desktop = desktop_token("alice");
    assert_eq!(
        with_bearer(&relay, Method::GET, "/session", &desktop)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        with_bearer(&relay, Method::POST, "/refresh", &desktop)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn an_inactive_account_cannot_refresh_and_a_full_account_cannot_sign_in() {
    let relay = Relay::with_limits(RelayLimits {
        devices_per_user: 1,
        ..RelayLimits::default()
    })
    .await;
    let body = sign_in(&relay, "alice", &Phone::new(1)).await;
    let other = Phone::new(2);
    let code = approved_code(&relay, "alice", &other).await;
    let full = exchange(&relay, &code, &other.verifier, other.peer).await;
    assert_eq!(
        (full.status, full.json()["error"].clone()),
        (StatusCode::CONFLICT, "device_limit".into())
    );
    relay.store.deactivate(&crate::support::user("alice"));
    assert_eq!(
        with_bearer(&relay, Method::POST, "/refresh", text(&body, "credential"))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}
