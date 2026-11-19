use chrono::Utc;
use dynwork_chats::ChatsExtension;
use dynwork_chats::clock::{Clock, SystemClock};
use systemprompt::extension::Extension;

#[test]
fn the_extension_declares_its_schema_and_depends_on_users() {
    let extension = ChatsExtension::default();
    let metadata = extension.metadata();
    assert_eq!(metadata.id, "dynwork-chats");
    assert_eq!(metadata.name, "DynWork Chats");
    assert_eq!(extension.dependencies(), vec!["users"]);
    let schemas = extension.schemas();
    assert_eq!(schemas.len(), 1);
    for table in [
        "dynwork_chat_threads",
        "dynwork_chat_messages",
        "dynwork_chat_events",
    ] {
        assert!(
            schemas[0]
                .sql
                .contains(&format!("CREATE TABLE IF NOT EXISTS {table}")),
            "{table}"
        );
    }
    assert!(
        schemas[0]
            .sql
            .contains("CREATE SEQUENCE IF NOT EXISTS dynwork_chat_change_seq")
    );
    assert!(extension.migrations().is_empty());
}

#[test]
fn the_system_clock_reads_the_wall_clock() {
    let before = Utc::now();
    let read = SystemClock.now();
    assert!(read >= before && read <= Utc::now());
}

#[test]
fn the_extension_reads_its_config_block_and_registers_the_retention_job() {
    let extension = ChatsExtension::default();
    assert_eq!(extension.config_prefix(), Some("dynwork-chats"));
    assert!(extension.validate_config(&serde_json::json!({})).is_ok());
    assert!(
        extension
            .validate_config(&serde_json::json!({ "inactive_days": 15 }))
            .is_ok()
    );
    for bad in [
        serde_json::json!({ "inactive_days": 0 }),
        serde_json::json!({ "inactive_days": "soon" }),
        serde_json::json!({ "other": 1 }),
    ] {
        assert!(extension.validate_config(&bad).is_err(), "{bad}");
    }
    let jobs = extension.jobs();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].name(), "dynwork_chats_retention");
    assert_eq!(jobs[0].schedule(), "0 30 3 * * *");
}

#[test]
fn production_limits_are_a_15_s_heartbeat_a_2_minute_idle_model_and_3_replies() {
    let limits = dynwork_chats::Limits::default();
    assert_eq!(limits.heartbeat, std::time::Duration::from_secs(15));
    assert_eq!(limits.reply_idle, std::time::Duration::from_secs(120));
    assert_eq!(limits.max_running_replies, 3);
}
