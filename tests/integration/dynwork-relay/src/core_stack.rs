//! The relay router inside core's middleware stack, applied in the order
//! `systemprompt_api`'s `apply_global_middleware` applies it, and nested at
//! its base path the way core's extension mount nests a non-root router.
//! The session and analytics layers need a booted `AppContext`; their path
//! rule is checked directly instead.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::get;
use dynwork_relay::{BASE_PATH, relay_router};
use systemprompt::extension::ExtensionRouter;
use systemprompt::identifiers::InstanceId;
use systemprompt::manifest::profile::SecurityHeadersConfig;
use systemprompt_api::services::middleware::load_shed::{LoadShed, shed};
use systemprompt_api::services::middleware::{
    content_negotiation_middleware, inject_security_headers, inject_served_by, inject_trace_header,
    remove_trailing_slash, should_skip_session_tracking, strip_caller_purpose,
};
use tokio::sync::Notify;

use crate::support::{
    Relay, close_code, connect_desktop_at, desktop_token, envelope, peer, recv, send, serve,
};

const SLOW: &str = "/api/test/slow";

fn core_stack(relay: &Relay, max_in_flight: u32, release: Arc<Notify>) -> Router {
    let extension = ExtensionRouter::public(relay_router(relay.state.clone()), BASE_PATH);
    let slow = Router::new().route(
        SLOW,
        get(move || {
            let release = Arc::clone(&release);
            async move {
                release.notified().await;
                "done"
            }
        }),
    );
    let security = SecurityHeadersConfig::default();
    Router::new()
        .nest(extension.base_path, extension.router)
        .merge(slow)
        .layer(from_fn_with_state(
            Arc::new(LoadShed::new(
                NonZeroU32::new(max_in_flight).expect("non-zero"),
            )),
            shed,
        ))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .layer(from_fn(remove_trailing_slash))
        .layer(from_fn(inject_trace_header))
        .layer(from_fn_with_state(
            InstanceId::new("relay-test"),
            inject_served_by,
        ))
        .layer(from_fn(strip_caller_purpose))
        .layer(from_fn(content_negotiation_middleware))
        .layer(from_fn(move |request, next| {
            inject_security_headers(security.clone(), request, next)
        }))
}

async fn get_status(addr: std::net::SocketAddr, path: &str, token: Option<&str>) -> StatusCode {
    let mut request = reqwest::Client::new().get(format!("http://{addr}{path}"));
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let status = request.send().await.expect("a response").status();
    StatusCode::from_u16(status.as_u16()).expect("a status")
}

#[tokio::test]
async fn a_tunnel_passes_cores_middleware_stack_end_to_end() {
    let relay = Relay::start().await;
    let addr = serve(core_stack(&relay, 512, Arc::new(Notify::new()))).await;
    let url = format!("ws://{addr}{BASE_PATH}/desktop");
    let mut desktop = connect_desktop_at(&url, &desktop_token("alice"), Some(peer(1)))
        .await
        .expect("the upgrade survives core's layers");
    let (_, credential) = relay.pair_phone("alice", peer(2)).await;
    let mut phone = relay.phone(&credential).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    send(&mut phone, envelope(peer(1), peer(2), 0, b"through core")).await;
    let got = recv(&mut desktop, Duration::from_secs(5))
        .await
        .expect("delivered");
    assert_eq!(&got[37..], b"through core");
    send(&mut desktop, envelope(peer(2), peer(1), 0, b"and back")).await;
    assert!(recv(&mut phone, Duration::from_secs(5)).await.is_some());
    let refused = connect_desktop_at(&url, "nope", Some(peer(1))).await;
    assert_eq!(crate::support::refused_status(refused), 401);
    assert_eq!(
        close_code(&mut desktop, Duration::from_millis(100)).await,
        None
    );
}

#[tokio::test]
async fn an_open_tunnel_does_not_hold_a_load_shed_permit() {
    let relay = Relay::start().await;
    let release = Arc::new(Notify::new());
    let addr = serve(core_stack(&relay, 1, Arc::clone(&release))).await;
    let url = format!("ws://{addr}{BASE_PATH}/desktop");
    let _first = connect_desktop_at(&url, &desktop_token("alice"), Some(peer(1)))
        .await
        .expect("first tunnel");
    let _second = connect_desktop_at(&url, &desktop_token("alice"), Some(peer(3)))
        .await
        .expect("a second tunnel while the first is open");
    let devices = format!("{BASE_PATH}/devices");
    assert_eq!(
        get_status(addr, &devices, Some(&desktop_token("alice"))).await,
        StatusCode::OK
    );

    let held = tokio::spawn(async move { get_status(addr, SLOW, None).await });
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        get_status(addr, &devices, Some(&desktop_token("alice"))).await,
        StatusCode::SERVICE_UNAVAILABLE,
        "an in-flight plain request does hold the only permit"
    );
    release.notify_one();
    assert_eq!(held.await.expect("the slow request"), StatusCode::OK);
}

#[test]
fn cores_session_middleware_leaves_the_relay_paths_untracked() {
    for path in ["/desktop", "/device", "/pairings/abc/claim", "/devices"] {
        assert!(should_skip_session_tracking(&format!("{BASE_PATH}{path}")));
    }
    assert!(!should_skip_session_tracking("/dynwork/relay/v1/device"));
}
