//! Pure halves of the Farm console's `credentials` module.

use systemprompt_web_admin_agent_farm::credentials::credential_row;
use systemprompt_web_admin_agent_farm::models::CredentialOut;

// The view model of a credential has no field that could hold a value.
#[test]
fn a_credential_row_has_no_value_like_field() {
    let row = credential_row(
        &CredentialOut {
            name: "n".to_owned(),
            ..CredentialOut::default()
        },
        "t",
    );
    let json = serde_json::to_value(&row).unwrap_or_default();
    let keys: Vec<&str> = json
        .as_object()
        .map(|o| o.keys().map(String::as_str).collect())
        .unwrap_or_default();
    for banned in [
        "value",
        "secret",
        "fingerprint",
        "metadata",
        "token",
        "password",
    ] {
        assert!(!keys.contains(&banned), "{banned} in {keys:?}");
    }
}

#[test]
fn farm_api_extras_are_dropped_on_deserialisation() {
    let parsed: Result<CredentialOut, _> = serde_json::from_str(
        r#"{"name":"n","type":"git_token","value":"S3","secret":"S3","fingerprint":"F","metadata":{"a":"S3"}}"#,
    );
    let c = parsed.unwrap_or_default();
    let shown = format!("{c:?}");
    assert!(!shown.contains("S3"), "{shown}");
    assert_eq!(c.kind.as_deref(), Some("git_token"));
}
