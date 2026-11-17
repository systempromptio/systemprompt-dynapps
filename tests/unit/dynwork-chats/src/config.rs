//! The chats settings: the optional YAML file and core's JSON block.

use std::path::PathBuf;

use chrono::Duration;
use dynwork_chats::config::{CONFIG_PREFIX, ChatsConfig, ConfigError};
use dynwork_chats::ids::ThreadId;
use serde_json::json;

fn services_root() -> PathBuf {
    std::env::temp_dir().join(format!("dynwork-chats-config-{}", ThreadId::generate()))
}

#[test]
fn nothing_set_keeps_threads_and_sets_no_search_limit() {
    for text in ["", "# only a comment\n", "{}\n"] {
        let config = ChatsConfig::parse(text).expect("an empty config parses");
        assert_eq!(config, ChatsConfig::default(), "{text:?}");
        assert_eq!(config.inactivity(), None);
        assert_eq!(config.web_search_per_thread, None);
    }
}

#[test]
fn both_settings_are_read() {
    let config = ChatsConfig::parse("inactive_days: 15\nweb_search_per_thread: 100\n")
        .expect("a valid config");
    assert_eq!(config.inactive_days, Some(15));
    assert_eq!(config.web_search_per_thread, Some(100));
    assert_eq!(config.inactivity(), Some(Duration::days(15)));
}

#[test]
fn zero_days_is_refused_so_a_typo_cannot_delete_everything() {
    assert!(matches!(
        ChatsConfig::parse("inactive_days: 0\n"),
        Err(ConfigError::ZeroDays)
    ));
    assert!(matches!(
        ChatsConfig::from_json(&json!({ "inactive_days": 0 })),
        Err(ConfigError::ZeroDays)
    ));
}

#[test]
fn unknown_keys_and_wrong_types_are_refused() {
    assert!(matches!(
        ChatsConfig::parse("inactive_day: 15\n"),
        Err(ConfigError::Yaml(_))
    ));
    assert!(matches!(
        ChatsConfig::parse("inactive_days: soon\n"),
        Err(ConfigError::Yaml(_))
    ));
    assert!(matches!(
        ChatsConfig::parse("inactive_days: -3\n"),
        Err(ConfigError::Yaml(_))
    ));
}

#[test]
fn core_hands_the_block_as_json() {
    assert_eq!(
        ChatsConfig::from_json(&json!({})).expect("an empty block"),
        ChatsConfig::default()
    );
    assert_eq!(
        ChatsConfig::from_json(&json!(null)).expect("no block"),
        ChatsConfig::default()
    );
    let config = ChatsConfig::from_json(&json!({ "inactive_days": 30 })).expect("a block");
    assert_eq!(config.inactive_days, Some(30));
    assert!(matches!(
        ChatsConfig::from_json(&json!({ "surprise": true })),
        Err(ConfigError::Json(_))
    ));
}

#[test]
fn the_block_core_hands_at_mount_time_is_accepted_and_its_typos_still_refused() {
    // Core validates an extension twice: with the config file's block at startup, and with `{"paths": ...}` when it
    // mounts the router. The second one made the server refuse to boot (found by the phase 11 run on a real instance).
    let mount = json!({ "paths": { "system": "/app", "services": "/app/services" } });
    assert_eq!(
        ChatsConfig::from_json(&mount).expect("the mount-time block"),
        ChatsConfig::default()
    );
    let with_setting = json!({ "paths": {}, "inactive_days": 15 });
    assert_eq!(
        ChatsConfig::from_json(&with_setting)
            .expect("a block with a setting")
            .inactive_days,
        Some(15)
    );
    assert!(matches!(
        ChatsConfig::from_json(&json!({ "paths": {}, "surprise": true })),
        Err(ConfigError::Json(_))
    ));
}

#[tokio::test]
async fn load_reads_the_file_and_a_missing_file_is_the_default() {
    let root = services_root();
    assert_eq!(
        ChatsConfig::load(&root).await.expect("no config dir"),
        ChatsConfig::default()
    );
    let dir = root.join("config");
    std::fs::create_dir_all(&dir).expect("make the config dir");
    let file = dir.join(format!("{CONFIG_PREFIX}.yaml"));
    std::fs::write(&file, "inactive_days: 7\n").expect("write the config");
    let loaded = ChatsConfig::load(&root).await;
    std::fs::write(&file, "inactive_days: 0\n").expect("rewrite the config");
    let refused = ChatsConfig::load(&root).await;
    std::fs::remove_dir_all(&root).expect("remove the temp dir");
    assert_eq!(loaded.expect("a valid file").inactive_days, Some(7));
    assert!(matches!(refused, Err(ConfigError::ZeroDays)));
}

#[tokio::test]
async fn a_config_path_that_is_not_a_file_is_a_read_error() {
    let root = services_root();
    let dir = root.join("config").join(format!("{CONFIG_PREFIX}.yaml"));
    std::fs::create_dir_all(&dir).expect("make a directory where the file should be");
    let outcome = ChatsConfig::load(&root).await;
    std::fs::remove_dir_all(&root).expect("remove the temp dir");
    assert!(
        matches!(outcome, Err(ConfigError::Read { .. })),
        "{outcome:?}"
    );
}
