//! An advertised personal-subscription provider the user has no key for
//! becomes the "add your key" badge, with the Settings button as its fix.

use systemprompt_models::bridge::profile::ProviderHealth;
use systemprompt_models::providers::ApiSurface;
use systemprompt_bridge::gateway::model_view::host_model_view;
use systemprompt_bridge::integration::agent_health::AgentReason;

#[test]
fn a_keyless_personal_provider_is_listed_by_display_name() {
    let health = vec![
        ProviderHealth {
            name: "opencode-go".to_owned(),
            surface: ApiSurface::OpenAi,
            configured: false,
            models: vec!["kimi-k2".to_owned()],
            config_issue: None,
            display_name: Some("OpenCode Go".to_owned()),
            personal_key: true,
        },
        ProviderHealth {
            name: "anthropic".to_owned(),
            surface: ApiSurface::OpenAi,
            configured: true,
            models: vec!["claude".to_owned()],
            config_issue: None,
            display_name: None,
            personal_key: false,
        },
    ];
    let view = host_model_view(&health, &[]);
    assert_eq!(view.missing_personal_keys, vec!["OpenCode Go".to_owned()]);
    assert!(view.available);

    let reason = AgentReason::PersonalKeyMissing {
        providers: view.missing_personal_keys.join(", "),
    };
    let json = serde_json::to_value(&reason).expect("serializes");
    assert_eq!(json["code"], "personal-key-missing");
    assert_eq!(json["providers"], "OpenCode Go");
}
