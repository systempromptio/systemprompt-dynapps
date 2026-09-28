use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use std::path::Path;
use systemprompt_bridge::cli::doctor::Status;
use systemprompt_bridge::cli::doctor::claude_code::check_file;
use systemprompt_bridge::install::mdm::claude_code_settings::windows_helper_command;

#[test]
fn windows_helper_quotes_paths_and_survives_outer_shell_expansion() {
    let path = r"C:\Users\O'Brien $USER %TEMP% & café\Astound Bridge.exe";
    let command = windows_helper_command(Path::new(path));
    let encoded = command
        .strip_prefix("powershell.exe -NoProfile -NonInteractive -EncodedCommand ")
        .unwrap();
    let bytes = STANDARD.decode(encoded).unwrap();
    let utf16: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    let script = String::from_utf16(&utf16).unwrap();
    assert_eq!(
        script,
        format!(
            "$ErrorActionPreference = 'Stop'; & '{}' credential-helper --host claude-code; exit $LASTEXITCODE",
            path.replace('\'', "''")
        )
    );
    assert!(!command.contains("$USER"));
    assert!(!command.contains("%TEMP%"));
}

fn check(body: &str) -> systemprompt_bridge::cli::doctor::Check {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(&path, body).unwrap();
    check_file(&path, "http://127.0.0.1:48218")
}

#[test]
fn doctor_detects_stale_port_and_never_prints_credentials() {
    let result = check(
        r#"{"env":{"ANTHROPIC_BASE_URL":"http://127.0.0.1:48217","ANTHROPIC_AUTH_TOKEN":"do-not-print"},"apiKeyHelper":"secret-command"}"#,
    );
    assert_eq!(result.status, Status::Fail);
    assert!(!result.detail.contains("do-not-print"));
    assert!(!result.detail.contains("secret-command"));
}

#[test]
fn doctor_checks_routing_helper_and_conflicting_policy_separately() {
    assert_eq!(
        check(r#"{"env":{"ANTHROPIC_BASE_URL":"http://127.0.0.1:48218"},"apiKeyHelper":"helper"}"#)
            .status,
        Status::Ok
    );
    assert_eq!(
        check(r#"{"env":{"ANTHROPIC_BASE_URL":"http://127.0.0.1:48218"}}"#).status,
        Status::Fail
    );
    assert_eq!(
        check(r#"{"forceLoginMethod":"claudeai"}"#).status,
        Status::Fail
    );
    assert_eq!(check("{broken-json").status, Status::Fail);
    assert_eq!(
        check(r#"{"permissions":{"allow":[]}}"#).status,
        Status::Warn
    );
}

fn sandbox(f: impl FnOnce(&Path)) {
    let dir = tempfile::tempdir().unwrap();
    let home = dir.path().join("home");
    let config = dir.path().join("config");
    let state = dir.path().join("state");
    std::fs::create_dir_all(&home).unwrap();
    temp_env::with_vars(
        [
            ("HOME", Some(home.to_str().unwrap())),
            ("XDG_CONFIG_HOME", Some(config.to_str().unwrap())),
            ("XDG_STATE_HOME", Some(state.to_str().unwrap())),
            ("XDG_CACHE_HOME", Some(state.to_str().unwrap())),
        ],
        || f(&home),
    );
}

#[test]
fn enrolment_and_disconnect_preserve_user_settings_and_standalone_profile() {
    use systemprompt_bridge::context::{BridgeContext, ProxyMode};
    use systemprompt_bridge::install::mdm::claude_code_settings::{
        managed_settings_path, standalone_settings_path,
    };
    use systemprompt_bridge::integration::enrol::{self, Outcome, Selection};
    sandbox(|home| {
        let settings = managed_settings_path().unwrap();
        assert_eq!(
            settings,
            home.join(".claude/settings.json"),
            "run this fixture as a standard user, without writable machine policy"
        );
        std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
        std::fs::write(
            &settings,
            r#"{"model":"user-choice","permissions":{"allow":["Read"]},"env":{"KEEP":"yes"}}"#,
        )
        .unwrap();
        let ctx = BridgeContext::start(ProxyMode::Attach).unwrap();
        let selection = Selection::Ids(vec!["claude-code".into()]);
        for _ in 0..2 {
            let reports = ctx
                .block_on(enrol::enrol_hosts(
                    &ctx,
                    &selection,
                    &std::collections::BTreeMap::default(),
                    None,
                ))
                .unwrap();
            assert!(matches!(reports[0].outcome, Outcome::Installed));
        }
        let profile = standalone_settings_path().unwrap();
        let before: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert_eq!(before["model"], "user-choice");
        assert_eq!(
            before["env"]["ANTHROPIC_BASE_URL"],
            ctx.proxy.loopback().origin()
        );
        #[cfg(target_os = "windows")]
        assert!(
            before["apiKeyHelper"]
                .as_str()
                .unwrap()
                .starts_with("powershell.exe ")
        );
        for _ in 0..2 {
            let reports = enrol::remove_host_profiles(&selection).unwrap();
            assert!(!reports[0].is_failure());
        }
        let after: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&settings).unwrap()).unwrap();
        assert_eq!(
            after,
            serde_json::json!({"model":"user-choice","permissions":{"allow":["Read"]},"env":{"KEEP":"yes"}})
        );
        assert!(profile.is_file());
    });
}

#[test]
fn enrolment_reports_invalid_settings_and_forced_login_without_overwriting_them() {
    use systemprompt_bridge::context::{BridgeContext, ProxyMode};
    use systemprompt_bridge::install::mdm::claude_code_settings::managed_settings_path;
    use systemprompt_bridge::integration::enrol::{self, Selection};
    sandbox(|home| {
        let settings = managed_settings_path().unwrap();
        assert_eq!(
            settings,
            home.join(".claude/settings.json"),
            "requires a standard-user test runner"
        );
        std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
        let ctx = BridgeContext::start(ProxyMode::Attach).unwrap();
        let selection = Selection::Ids(vec!["claude-code".into()]);
        for body in [
            "{invalid",
            r#"{"forceLoginMethod":"claudeai"}"#,
            r#"{"env":"invalid"}"#,
        ] {
            std::fs::write(&settings, body).unwrap();
            let reports = ctx
                .block_on(enrol::enrol_hosts(
                    &ctx,
                    &selection,
                    &std::collections::BTreeMap::default(),
                    None,
                ))
                .unwrap();
            assert!(reports[0].is_failure());
            assert_eq!(std::fs::read_to_string(&settings).unwrap(), body);
        }
    });
}
