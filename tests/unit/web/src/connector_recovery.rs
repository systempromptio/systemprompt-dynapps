//! Regression coverage for connector callback notices.

#[test]
fn callback_notices_name_the_stage_and_only_accept_bounded_fields() {
    use systemprompt_web_admin_platform::connector_oauth::callback_notice::CallbackNotice;
    for stage in ["token", "initialize", "tools", "identity"] {
        let notice: CallbackNotice = serde_json::from_value(serde_json::json!({
            "connection_provider": "dynbrain", "connection_stage": stage,
            "connection_failure": "configuration", "connection_ref": "00000000-0000-0000-0000-000000000001"
        })).unwrap();
        let message = notice
            .message(
                &systemprompt_web_admin_core::extension_services::ExtensionServices::new(
                    crate::boot_fixture::empty_services_view(),
                    std::sync::Arc::new(
                        systemprompt::governance::authz::SubjectProviderSet::from_providers(
                            Vec::new(),
                        ),
                    ),
                ),
            )
            .unwrap();
        assert!(message.contains("failed during") && message.contains("Reference:"));
        assert!(message.contains("app configuration"));
    }
    let notice: CallbackNotice = serde_json::from_value(serde_json::json!({
        "connection_provider": "dynbrain", "connection_stage": "fixture-secret",
        "connection_failure": "configuration", "connection_ref": "00000000-0000-0000-0000-000000000001"
    })).unwrap();
    assert!(
        notice
            .message(
                &systemprompt_web_admin_core::extension_services::ExtensionServices::new(
                    crate::boot_fixture::empty_services_view(),
                    std::sync::Arc::new(
                        systemprompt::governance::authz::SubjectProviderSet::from_providers(
                            Vec::new()
                        )
                    )
                )
            )
            .is_none()
    );
}
