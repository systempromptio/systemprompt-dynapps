use systemprompt::evaluation::capabilities::{
    CAPABILITY_REGISTRY_VERSION, CapabilityAvailability, EvaluatorClient,
    UnsupportedCapabilityReason, evaluator_capabilities,
};

#[test]
fn capability_registry_is_versioned_and_fails_closed() {
    let capabilities = evaluator_capabilities();

    assert_eq!(capabilities.len(), 5);
    assert!(capabilities.iter().all(|capability| {
        capability.registry_version == CAPABILITY_REGISTRY_VERSION
            && capability.installation_supported
            && capability.automated_evaluation == CapabilityAvailability::Unsupported
    }));
    assert!(capabilities.iter().any(|capability| {
        capability.client == EvaluatorClient::ClaudeDesktop
            && capability.reason == UnsupportedCapabilityReason::InteractiveHost
    }));
    assert!(capabilities.iter().any(|capability| {
        capability.client == EvaluatorClient::Codex
            && capability.reason == UnsupportedCapabilityReason::AdapterNotVerified
    }));
}
