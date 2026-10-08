//! A console save keeps the safety settings the form has no fields for.

use systemprompt::gateway::{SafetyConfig, ScannerFailMode, ScannerSettings};
use systemprompt_web_admin_governance::repositories::gateway_policies::unedited::carry_unedited_safety;

fn stored() -> SafetyConfig {
    SafetyConfig {
        scanners: vec!["secrets".to_owned(), "pii_extended".to_owned()],
        redact_categories: vec!["secret".to_owned()],
        scanner_settings: [
            (
                "secrets".to_owned(),
                ScannerSettings {
                    fail_mode: ScannerFailMode::Open,
                    timeout_ms: 2000,
                    ..ScannerSettings::default()
                },
            ),
            ("pii_extended".to_owned(), ScannerSettings::default()),
        ]
        .into_iter()
        .collect(),
        ..SafetyConfig::default()
    }
}

#[test]
fn a_form_save_keeps_stored_scanner_settings_and_redaction() {
    let mut edited = SafetyConfig {
        scanners: vec!["secrets".to_owned(), "pii_extended".to_owned()],
        ..SafetyConfig::default()
    };
    carry_unedited_safety(&mut edited, &stored());
    assert_eq!(edited.redact_categories, vec!["secret".to_owned()]);
    let secrets = edited.scanner_settings.get("secrets").expect("kept");
    assert_eq!(secrets.fail_mode, ScannerFailMode::Open);
    assert_eq!(secrets.timeout_ms, 2000);
}

#[test]
fn settings_for_a_scanner_the_form_unlisted_are_dropped() {
    let mut edited = SafetyConfig {
        scanners: vec!["secrets".to_owned()],
        ..SafetyConfig::default()
    };
    carry_unedited_safety(&mut edited, &stored());
    assert!(
        !edited.scanner_settings.contains_key("pii_extended"),
        "core refuses settings for a scanner not in `scanners`"
    );
    assert!(edited.scanner_settings.contains_key("secrets"));
}
