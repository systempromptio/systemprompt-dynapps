use dynwork_relay::RelayExtension;
use systemprompt::extension::Extension;

#[test]
fn the_extension_declares_its_schema_and_depends_on_users() {
    let extension = RelayExtension::default();
    assert_eq!(extension.metadata().id, "dynwork-relay");
    assert_eq!(extension.dependencies(), vec!["users"]);
    let schemas = extension.schemas();
    assert_eq!(schemas.len(), 2);
    assert!(
        schemas[1]
            .sql
            .contains("CREATE TABLE IF NOT EXISTS dynwork_relay_mobile_grants")
    );
    for table in ["dynwork_relay_devices", "dynwork_relay_pairings"] {
        assert!(
            schemas[0]
                .sql
                .contains(&format!("CREATE TABLE IF NOT EXISTS {table}"))
        );
    }
    assert!(
        !schemas[0].sql.contains("public_key"),
        "the relay stores peer ids, never keys"
    );
    assert!(extension.migrations().is_empty());
}
