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
