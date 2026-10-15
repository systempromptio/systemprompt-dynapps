//! Structural invariants of the service-account pages and the pure rules
//! behind them: which callers may spend a credential, and what the attach
//! form accepts.
//!
//! The secret field is write-only. A template that renders it, or a
//! credential view that carries it, would leak a GitHub App key to every
//! administrator's browser and survive a compile.

use crate::support::repo_root;
use systemprompt_web_admin_platform::machine_credentials::caller::allowed;
use systemprompt_web_admin_platform::machine_credentials::{
    AttachCredential, MachineKind, endpoint_allowed,
};

fn template(name: &str) -> String {
    let path = repo_root().join(format!("storage/files/admin/templates/{name}.hbs"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn no_template_renders_a_secret() {
    for name in ["service-accounts", "service-account-detail"] {
        let source = template(name);
        for leak in [
            "{{secret",
            "{{ciphertext",
            "{{secret_ciphertext",
            "{{config",
        ] {
            assert!(
                !source.contains(leak),
                "{name}.hbs renders `{leak}`; secrets are write-only"
            );
        }
    }
}

#[test]
fn the_create_dialog_sits_inside_the_manage_guard() {
    let source = template("service-accounts");
    let guard = source
        .find("{{#if can_manage}}<button")
        .expect("guarded create button");
    let dialog = source
        .find("data-dialog=\"create-service-account\"")
        .expect("dialog");
    let dialog_guard = source[..dialog]
        .rfind("{{#if can_manage}}")
        .expect("dialog guard");
    assert!(guard < dialog && dialog_guard > guard);
}

#[test]
fn the_listing_keeps_a_slot_for_bound_definitions() {
    assert!(template("service-accounts").contains("data-slot=\"bound-definitions\""));
}

#[test]
fn the_detail_page_offers_every_kind_and_a_write_only_secret() {
    let source = template("service-account-detail");
    for kind in MachineKind::ALL {
        assert!(
            source.contains(&format!("data-kind-fields=\"{}\"", kind.as_str())),
            "no fields for {}",
            kind.as_str()
        );
    }
    let secret = source.find("data-field=\"secret\"").expect("secret field");
    let close = source[secret..].find('>').expect("tag end") + secret;
    assert!(
        source[close..].starts_with("></textarea>"),
        "the secret field must render empty"
    );
    assert!(source.contains("{{#each kinds}}"));
}

#[test]
fn only_workflow_write_backs_and_farms_may_spend_a_credential() {
    for ok in [
        "workflow:release-notes:write",
        "workflow:a:b:write",
        "farm:triage",
    ] {
        assert!(allowed(Some(ok)), "{ok} should be allowed");
    }
    for refused in [
        None,
        Some(""),
        Some("workflow:release-notes"),
        Some("workflow::write"),
        Some("workflow:x:read"),
        Some("farm:"),
        Some("interactive"),
        Some("workflow:x\n:write"),
    ] {
        assert!(!allowed(refused), "{refused:?} should be refused");
    }
}

fn attach(json: serde_json::Value) -> Result<(MachineKind, serde_json::Value, String), String> {
    serde_json::from_value::<AttachCredential>(json)
        .map_err(|e| e.to_string())?
        .validated()
        .map_err(|e| e.to_string())
}

#[test]
fn a_github_app_keeps_its_ids_and_seals_only_the_key() {
    let (kind, config, secret) = attach(serde_json::json!({
        "kind": "github_app", "app_id": "123", "installation_id": "456",
        "secret": "-----BEGIN RSA PRIVATE KEY-----", "email": "ignored@x.io"
    }))
    .expect("valid");
    assert_eq!(kind, MachineKind::GithubApp);
    assert_eq!(
        config,
        serde_json::json!({"app_id": "123", "installation_id": "456"})
    );
    assert!(secret.starts_with("-----BEGIN"));
    assert!(
        !config.to_string().contains("BEGIN"),
        "the secret leaked into config"
    );
}

#[test]
fn the_attach_form_refuses_what_it_cannot_use() {
    let cases = [
        serde_json::json!({"kind": "github_app", "app_id": "12a", "installation_id": "4", "secret": "k"}),
        serde_json::json!({"kind": "github_app", "app_id": "1", "installation_id": "4", "secret": " "}),
        serde_json::json!({"kind": "github_app", "app_id": "1", "installation_id": "4", "api_url": "http://github.example", "secret": "k"}),
        serde_json::json!({"kind": "odoo_api_key", "url": "https://odoo.example", "db": "d", "login": "l", "secret": "k"}),
        serde_json::json!({"kind": "aws_keys", "secret": "t"}),
    ];
    for case in cases {
        assert!(attach(case.clone()).is_err(), "accepted {case}");
    }
}

#[test]
fn provider_endpoints_are_https_or_loopback() {
    assert!(endpoint_allowed("https://github.example.com"));
    assert!(endpoint_allowed("http://127.0.0.1:8123"));
    assert!(!endpoint_allowed("http://github.example.com"));
    assert!(!endpoint_allowed("https://user:pw@github.example.com"));
    assert!(!endpoint_allowed("not a url"));
}
