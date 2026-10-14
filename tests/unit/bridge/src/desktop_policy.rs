use serde_json::json;
use systemprompt_bridge::install::mdm::desktop_policy::{
    DesktopPolicy, resolved_settings, settings_catalog,
};
use systemprompt_bridge::install::mdm::policy::{PolicyValue, plist_body, reg_values};

#[test]
fn catalog_covers_every_audited_setting_without_duplicate_keys() {
    let catalog = settings_catalog().unwrap();
    assert_eq!(catalog.schema_version, 1);
    assert_eq!(catalog.settings.len(), 142);
    let keys: std::collections::BTreeSet<_> = catalog.settings.iter().map(|s| &s.key).collect();
    assert_eq!(keys.len(), 142);
    assert!(
        catalog
            .settings
            .iter()
            .all(|s| !s.disposition.is_empty() && s.min_version.is_some())
    );
}

#[test]
fn desktop_policy_rejects_wrong_types_ranges_unknown_fields_and_reserved_settings() {
    for settings in [
        json!({"modelPrefer1mContext":"true"}),
        json!({"mcpToolTimeoutSec":1}),
        json!({"inferenceAnthropicApiKey":"secret"}),
        json!({"inferenceGatewayBaseUrl":"https://evil.example"}),
        json!({"otlpContentCapture":["rawApiBodies"]}),
        json!({"unknownSetting":true}),
        json!({"defaultModelEffort":"turbo"}),
        json!({"builtinToolPolicy":{"WebFetch":"blocked"}}),
        json!({"banner":{"enabled":"yes"}}),
        json!({"banner":{"unknownField":true}}),
        json!({"inferenceModelPricing":[{"name":"model","inputPerMtok":-1}]}),
    ] {
        assert!(
            serde_json::from_value::<DesktopPolicy>(
                json!({"schema_version":1,"settings":settings})
            )
            .is_err()
        );
    }
}

#[test]
fn fleet_defaults_render_equally_for_macos_and_windows() {
    let entries = resolved_settings(&DesktopPolicy::default()).unwrap();
    assert!(entries.contains(&("modelPrefer1mContext", PolicyValue::Bool(true))));
    assert!(entries.contains(&("alwaysStartWithDefaultModel", PolicyValue::Bool(true))));
    assert!(entries.contains(&("disableAutoUpdates", PolicyValue::Bool(false))));
    let plist = plist_body(&entries, "");
    let registry = reg_values(&entries);
    for key in [
        "modelPrefer1mContext",
        "alwaysStartWithDefaultModel",
        "disableAutoUpdates",
    ] {
        let (_, kind, value) = registry.iter().find(|(name, _, _)| *name == key).unwrap();
        assert_eq!(*kind, "REG_SZ");
        assert!(plist.contains(&format!("<key>{key}</key>\n<string>{value}</string>")));
    }
}

#[test]
fn validated_operator_options_override_the_defaults() {
    let policy: DesktopPolicy = serde_json::from_value(
        json!({"schema_version":1,"settings":{"mcpToolTimeoutSec":600,"disableAutoUpdates":true}}),
    )
    .unwrap();
    let entries = resolved_settings(&policy).unwrap();
    assert!(entries.contains(&("mcpToolTimeoutSec", PolicyValue::Str("600".to_owned()))));
    assert!(entries.contains(&("disableAutoUpdates", PolicyValue::Bool(true))));
}

#[test]
fn public_web_access_is_explicit_and_tools_are_available() {
    temp_env::with_var(
        systemprompt_bridge::brand::brand().env("EGRESS_ALLOWED_HOSTS"),
        None::<&str>,
        || {
            let hosts =
                systemprompt_bridge::install::mdm::cowork_egress_allowed_hosts(None).unwrap();
            assert_eq!(hosts, Some(vec!["*".to_owned()]));
        },
    );
    let entries = resolved_settings(&DesktopPolicy::default()).unwrap();
    assert!(entries.contains(&(
        "builtinToolPolicy",
        PolicyValue::Json(json!({"WebSearch":"allow","WebFetch":"allow"}))
    )));
    assert!(entries.contains(&("disableBundledSkills", PolicyValue::Bool(false))));
}

#[test]
fn deferred_tools_and_server_web_history_survive_anthropic_rebuilds() {
    let tools = json!([
        {"type":"tool_search_tool_regex_20251119","name":"tool_search_tool_regex"},
        {"name":"salesforce_query","description":"CRM query","defer_loading":true,"input_schema":{"type":"object"}},
        {"type":"web_search_20250305","name":"web_search","max_uses":3}
    ]);
    let messages = json!([
        {"role":"user","content":"Find current company news"},
        {"role":"assistant","content":[
            {"type":"server_tool_use","id":"srvtoolu_search","name":"web_search","input":{"query":"example news"}},
            {"type":"web_search_tool_result","tool_use_id":"srvtoolu_search","content":[{"type":"web_search_result","url":"https://example.com/news","title":"News","encrypted_content":"opaque"}]}
        ]},
        {"role":"user","content":[{"type":"tool_result","tool_use_id":"toolu_lookup","content":[{"type":"tool_reference","tool_name":"salesforce_query"}]}]}
    ]);
    let request = systemprompt_gateway::protocol::inbound::anthropic_messages::parse::parse(
        &json!({"model":"claude-sonnet-5-5","max_tokens":128,"tools":tools,"messages":messages}),
    )
    .unwrap();
    let rebuilt =
        systemprompt_wire::anthropic::build_request_body(&request, "claude-sonnet-5-5", None);
    assert_eq!(rebuilt["tools"], tools);
    assert_eq!(rebuilt["messages"][1], messages[1]);
    assert_eq!(
        rebuilt["messages"][2]["content"][0]["content"],
        messages[2]["content"][0]["content"]
    );
    assert_eq!(
        rebuilt["messages"][2]["content"][0]["tool_use_id"],
        messages[2]["content"][0]["tool_use_id"]
    );
}

#[test]
fn desktop_version_checks_use_numeric_components() {
    use systemprompt_bridge::install::mdm::desktop_policy::supports_version;
    assert!(supports_version("2.19675.0", "2.110.0"));
    assert!(!supports_version("1.99999.0", "2.110.0"));
    assert!(!supports_version("2.9.0", "2.110.0"));
    assert!(!supports_version("unknown", "2.110.0"));
}

#[test]
fn desktop_telemetry_credential_is_scoped_to_supported_routes() {
    use systemprompt_bridge::ids::{LoopbackSecret, ProxySecret};
    use systemprompt_bridge::proxy::credential::{Rejection, RouteClass, authenticate};
    let secret = ProxySecret::new("test-loopback-secret");
    let token = systemprompt_bridge::install::mdm::policy::desktop_host_token(
        &LoopbackSecret::new("test-loopback-secret"),
    );
    assert!(authenticate(token.as_str(), &secret, &RouteClass::Otel).is_ok());
    assert_eq!(
        authenticate(token.as_str(), &secret, &RouteClass::Other),
        Err(Rejection::ScopeMismatch)
    );
    assert!(authenticate("wrong-secret", &secret, &RouteClass::Otel).is_err());
}

#[test]
fn telemetry_keeps_metadata_and_drops_content_and_credentials() {
    use opentelemetry_proto::tonic::common::v1::{AnyValue, KeyValue, any_value};
    let attribute = |key: &str, value: &str| KeyValue {
        key: key.to_owned(),
        value: Some(AnyValue {
            value: Some(any_value::Value::StringValue(value.to_owned())),
        }),
        ..Default::default()
    };
    let sanitized = systemprompt_api::routes::gateway::otel::convert::attrs_to_json(&[
        attribute("gen_ai.request.model", "claude-sonnet-5-5"),
        attribute("gen_ai.prompt", "private prompt"),
        attribute("http.request.header.authorization", "Bearer secret"),
        attribute("gen_ai.response.model", "sk-secret"),
    ]);
    assert_eq!(
        sanitized,
        json!({"gen_ai.request.model":"claude-sonnet-5-5"})
    );
}

#[test]
fn desktop_json_logs_decode_without_exporting_content() {
    let fixture = json!({"resourceLogs":[{"resource":{"attributes":[
        {"key":"service.name","value":{"stringValue":"claude-desktop"}}
    ]},"scopeLogs":[{"logRecords":[{
        "timeUnixNano":"123456789", "severityNumber":9, "body":{"stringValue":"private prompt"},
        "traceId":"00112233445566778899aabbccddeeff", "spanId":"0011223344556677"
    }]}]}]});
    let decoded = systemprompt_api::routes::gateway::otel::json::decode_logs(
        &serde_json::to_vec(&fixture).unwrap(),
    )
    .unwrap();
    let record = &decoded.resource_logs[0].scope_logs[0].log_records[0];
    assert_eq!(record.time_unix_nano, 123_456_789);
    assert_eq!(record.trace_id.len(), 16);
    assert!(record.body.is_none());
}

#[test]
fn native_tool_route_does_not_capture_ordinary_inference() {
    use systemprompt_manifest::services::RouteMatch;
    let condition: RouteMatch =
        serde_json::from_value(json!({"tool_names_any":["web_search","tool_search_tool_regex"]}))
            .unwrap();
    let ordinary = systemprompt_gateway::protocol::inbound::anthropic_messages::parse::parse(
        &json!({"model":"claude-sonnet-5-5","max_tokens":16,"messages":[{"role":"user","content":"Hi"}]})
    ).unwrap();
    assert!(!condition.matches_request(&ordinary));
    let searched = systemprompt_gateway::protocol::inbound::anthropic_messages::parse::parse(
        &json!({"model":"claude-sonnet-5-5","max_tokens":16,"messages":[{"role":"user","content":"Hi"}],"tools":[{"type":"web_search_20250305","name":"web_search"}]})
    ).unwrap();
    assert!(condition.matches_request(&searched));
}

#[tokio::test]
async fn large_inference_body_is_accepted_and_oversize_is_reported_as_bytes() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use systemprompt_api::routes::gateway::messages::extract::RejectionPartial;
    use systemprompt_api::routes::gateway::messages::extract::headers::read_gateway_body;
    use systemprompt_gateway::protocol::inbound::InboundAdapter;
    use systemprompt_gateway::protocol::inbound::anthropic_messages::AnthropicMessagesInbound;
    use systemprompt_models::origin::{
        ClientAttestation, ClientKind, InboundWireProtocol, RequestOrigin,
    };
    let adapter: Arc<dyn InboundAdapter> = Arc::new(AnthropicMessagesInbound);
    let partial = || {
        RejectionPartial::new(RequestOrigin::gateway(
            ClientKind::ClaudeDesktop,
            InboundWireProtocol::AnthropicMessages,
            ClientAttestation::None,
        ))
    };
    let text = "x".repeat(9 * 1024 * 1024);
    let body = serde_json::to_vec(&json!({"model":"claude-sonnet-5-5","max_tokens":16,"messages":[{"role":"user","content":text}]})).unwrap();
    let (_, parsed) = read_gateway_body(&adapter, Request::new(Body::from(body)), &mut partial())
        .await
        .unwrap();
    assert_eq!(parsed.messages.len(), 1);
    let rejected = read_gateway_body(
        &adapter,
        Request::new(Body::from(vec![b'x'; 32 * 1024 * 1024 + 1])),
        &mut partial(),
    )
    .await
    .unwrap_err();
    assert_eq!(rejected.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert!(
        rejected
            .public_message()
            .contains("serialized inference request")
    );
}

#[test]
fn tool_search_requires_an_explicit_compatible_fleet_policy() {
    let legacy = resolved_settings(&DesktopPolicy::default()).unwrap();
    assert!(!legacy.iter().any(|(key, _)| *key == "toolSearchEnabled"));
    let policy: DesktopPolicy = serde_json::from_value(json!({
        "schema_version":1,
        "settings":{"toolSearchEnabled":true,"deploymentDisplayName":"Dynapps"}
    }))
    .unwrap();
    let configured = resolved_settings(&policy).unwrap();
    assert!(configured.contains(&("toolSearchEnabled", PolicyValue::Bool(true))));
    assert!(configured.contains(&(
        "deploymentDisplayName",
        PolicyValue::Str("Dynapps".to_owned())
    )));
}
