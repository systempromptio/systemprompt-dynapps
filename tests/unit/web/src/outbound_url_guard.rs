//! Every outbound destination passes core's SSRF guard, and the refusal
//! reaches the caller as a typed error with its cause attached.
//!
//! The connector OAuth endpoint check, the Fly-aware mesh validator used by
//! the OTLP probe and the farm worker, and the admin error type are covered
//! here; the 1Password Connect URL is covered in the secrets unit crate.

use std::error::Error;

use systemprompt::models::net::OutboundUrlError;
use systemprompt_web_admin_core::error::AdminError;
use systemprompt_web_admin_core::util::outbound::validate_mesh_url;
use systemprompt_web_admin_platform::connector_oauth::generic::validate_endpoint;

const METADATA_SERVICE: &str = "http://169.254.169.254/latest/meta-data/";

#[test]
fn the_cloud_metadata_address_is_refused_everywhere() {
    assert!(matches!(
        validate_mesh_url(METADATA_SERVICE),
        Err(OutboundUrlError::NonLoopbackHttp)
    ));
    assert!(matches!(
        validate_mesh_url("https://169.254.169.254/"),
        Err(OutboundUrlError::BlockedHost(_))
    ));
    let refused = validate_endpoint("https://169.254.169.254/", &[], "https://169.254.169.254/");
    assert!(matches!(refused, Err(AdminError::OutboundUrl(_))));
}

#[test]
fn a_refused_connector_endpoint_keeps_its_cause() {
    let error = validate_endpoint("https://10.0.0.9/mcp", &[], "https://10.0.0.9/mcp")
        .expect_err("private address refused");
    let source = error.source().expect("the outbound refusal is the cause");
    assert!(source.is::<OutboundUrlError>());
    assert_eq!(error.status(), axum::http::StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn loopback_is_allowed_over_plain_http() {
    assert!(validate_mesh_url("http://localhost:4318/v1/traces").is_ok());
    assert!(validate_mesh_url("http://127.0.0.1:8080/").is_ok());
    assert!(validate_endpoint("https://example.com/mcp", &[], "https://example.com/mcp").is_ok());
}

#[test]
fn a_fly_private_collector_is_trusted_and_other_private_hosts_are_not() {
    assert!(validate_mesh_url("http://otel-collector.internal:4318/v1/traces").is_ok());
    assert!(validate_mesh_url("http://3d8d9e2b.vm.dynapps.internal:8080/").is_ok());
    assert!(matches!(
        validate_mesh_url("http://otel-collector:4318/v1/traces"),
        Err(OutboundUrlError::NonLoopbackHttp)
    ));
    assert!(matches!(
        validate_mesh_url("http://10.0.0.5:4318/v1/traces"),
        Err(OutboundUrlError::NonLoopbackHttp)
    ));
    assert!(matches!(
        validate_mesh_url("https://10.0.0.5:4318/v1/traces"),
        Err(OutboundUrlError::BlockedHost(_))
    ));
}

#[test]
fn an_unparseable_url_is_a_parse_error_not_a_panic() {
    assert!(matches!(
        validate_mesh_url("not a url"),
        Err(OutboundUrlError::Parse(_))
    ));
}
