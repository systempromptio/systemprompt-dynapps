//! `services/web/config/farm.yaml` and the credential it points at: the file
//! names a secrets-store key, and the client reads the token from the store on
//! every call.

use std::sync::Arc;

use systemprompt_web_admin_agent_farm::client::{FarmApi, FarmApiError};
use systemprompt_web_admin_agent_farm::config::{CONFIG_FILE, DEFAULT_TOKEN_SECRET, FarmConfig};

use super::common::{Mode, TestResult, fake_farm};

#[test]
fn an_empty_or_missing_file_is_a_console_without_a_farm() -> TestResult {
    assert_eq!(FarmConfig::parse("")?, FarmConfig::default());
    assert_eq!(
        FarmConfig::parse("# only a comment\n")?,
        FarmConfig::default()
    );
    let dir = tempfile::tempdir()?;
    assert_eq!(FarmConfig::load(dir.path()), FarmConfig::default());
    assert_eq!(FarmConfig::default().token_secret, DEFAULT_TOKEN_SECRET);
    Ok(())
}

#[test]
fn the_file_names_the_url_and_the_secret_key_and_rejects_unknown_keys() -> TestResult {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join(CONFIG_FILE);
    std::fs::create_dir_all(path.parent().ok_or("no parent")?)?;
    std::fs::write(
        &path,
        "api_url: https://farm.example/farm/api/v1\ntoken_secret: other_key\n",
    )?;
    let config = FarmConfig::load(dir.path());
    assert_eq!(config.api_url, "https://farm.example/farm/api/v1");
    assert_eq!(config.token_secret, "other_key");

    assert!(FarmConfig::parse("api_url: x\ntoken: s3cr3t\n").is_err());
    std::fs::write(&path, "api_url: x\ntoken: s3cr3t\n")?;
    assert_eq!(FarmConfig::load(dir.path()), FarmConfig::default());
    Ok(())
}

#[test]
fn the_repository_file_names_no_farm_and_holds_no_token() -> TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../services");
    let text = std::fs::read_to_string(root.join(CONFIG_FILE))?;
    let config = FarmConfig::parse(&text)?;
    assert_eq!(config, FarmConfig::default());
    Ok(())
}

#[tokio::test]
async fn a_missing_secret_is_not_configured_and_a_present_one_is_sent() -> TestResult {
    let farm = fake_farm(Mode::Normal).await?;
    let store = crate::boot_fixture::secret_store();

    let missing = FarmConfig {
        api_url: farm.base.clone(),
        token_secret: "farm_api_token_absent".to_owned(),
    };
    let api = FarmApi::from_config(&missing, Arc::clone(&store));
    assert!(api.is_configured());
    assert!(matches!(
        api.whoami().await,
        Err(FarmApiError::NotConfigured)
    ));

    // A key the fixture store holds: its value is read and sent, and the fake
    // farm-api, which expects another token, answers 401.
    let present = FarmConfig {
        api_url: farm.base.clone(),
        token_secret: "oauth_at_rest_pepper".to_owned(),
    };
    let api = FarmApi::from_config(&present, store);
    match api.whoami().await {
        Err(FarmApiError::Status { status, .. }) => assert_eq!(status, 401),
        other => return Err(format!("expected a 401, got {other:?}").into()),
    }

    let blank = FarmConfig::default();
    assert!(!FarmApi::from_config(&blank, crate::boot_fixture::secret_store()).is_configured());
    Ok(())
}
