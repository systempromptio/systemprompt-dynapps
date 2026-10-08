//! Desktop model policies use only gateway-confirmed context capacities.

use std::collections::BTreeMap;
use systemprompt_bridge::install::mdm::default_inference_models;
use systemprompt_bridge::integration::claude_desktop::reg_profile::with_context_variants;

#[test]
fn desktop_defaults_never_offer_fable_or_short_context_models() {
    let defaults = default_inference_models();
    assert!(!defaults.is_empty());
    assert!(
        defaults
            .iter()
            .all(|id| id.ends_with("[1m]") && !id.contains("fable"))
    );
}

#[test]
fn desktop_does_not_invent_context_capacity_without_gateway_limits() {
    let models = vec![
        "claude-sonnet-5".to_owned(),
        "claude-sonnet-5[1m]".to_owned(),
        "claude-fable-5-1".to_owned(),
    ];
    assert!(with_context_variants(&models, &BTreeMap::new()).is_empty());
}

#[test]
fn desktop_lists_one_million_context_variant_per_supported_model() {
    let limits = serde_json::from_value(serde_json::json!({
        "claude-sonnet-5": {"context_window": 1_000_000, "max_output_tokens": 128_000},
        "claude-haiku-4-5": {"context_window": 200_000, "max_output_tokens": 64_000},
        "claude-fable-5-1": {"context_window": 1_000_000, "max_output_tokens": 128_000}
    }))
    .unwrap();
    let models = [
        "claude-sonnet-5",
        "claude-sonnet-5[1m]",
        "claude-haiku-4-5",
        "claude-fable-5-1",
        "anthropic-claude-sonnet-5",
    ]
    .map(str::to_owned);
    assert_eq!(
        with_context_variants(&models, &limits),
        vec!["claude-sonnet-5[1m]"]
    );
}
