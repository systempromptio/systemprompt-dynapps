//! Schema DDL and migrations are the only things that run against a live
//! database before any request is served, so a mistake here is a boot failure
//! rather than a page bug. `include_str!` makes an empty or missing `.sql` file
//! compile fine, and migration versions are discovered by `build.rs` — both are
//! silent, so emptiness and version uniqueness are asserted directly.

use std::collections::HashSet;
use systemprompt_web_extension::schemas::{migrations, schema_definitions};

#[test]
fn removed_organization_tables_are_not_altered_by_historical_migrations() {
    let migrations = migrations();
    for version in [28, 31, 35] {
        let migration = migrations.iter().find(|m| m.version == version).unwrap();
        assert!(
            migration.tombstone,
            "retired slot {version} must stay spent"
        );
        assert!(migration.sql.is_empty());
    }
}

#[test]
fn every_schema_definition_carries_non_empty_ddl() {
    let definitions = schema_definitions();
    assert!(!definitions.is_empty(), "no schema DDL is registered");

    for definition in &definitions {
        assert!(
            !definition.sql.trim().is_empty(),
            "a schema definition embedded an empty .sql file"
        );
    }
}

#[test]
fn schema_ddl_is_not_registered_twice() {
    let definitions = schema_definitions();
    let unique: HashSet<&str> = definitions.iter().map(|d| d.sql.as_str()).collect();

    assert_eq!(
        unique.len(),
        definitions.len(),
        "the same DDL file is registered under two entries"
    );
}

#[test]
fn migrations_have_unique_increasing_versions_and_non_empty_sql() {
    let migrations = migrations();

    let versions: Vec<u32> = migrations.iter().map(|m| m.version).collect();
    let unique: HashSet<u32> = versions.iter().copied().collect();
    assert_eq!(
        unique.len(),
        versions.len(),
        "two migrations share a version: {versions:?}"
    );

    let mut sorted = versions.clone();
    sorted.sort_unstable();
    assert_eq!(
        versions, sorted,
        "migrations must be handed to the runner in version order"
    );

    // Tombstones are excluded from both checks below, and deliberately so. A
    // `NNN-MMM_<name>.tombstone` marks a span of spent slots and `build.rs`
    // expands it to one entry per covered version, every one carrying the same
    // name and no SQL at all -- so a tombstoned range legitimately repeats a
    // name and legitimately has nothing to execute. Asserting over the whole
    // list would therefore fail on correct input, which is why this test began
    // failing when spent slots stopped being invisible. The properties still
    // matter for real migrations, so they are asserted over those.
    let real: Vec<_> = migrations.iter().filter(|m| !m.tombstone).collect();

    let names: HashSet<&str> = real.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names.len(), real.len(), "two migrations share a name");

    for migration in &real {
        assert!(
            !migration.sql.trim().is_empty(),
            "migration {} ({}) has no SQL",
            migration.version,
            migration.name
        );
    }

    // The converse, so a tombstone cannot quietly acquire SQL that would never
    // be run: the runner skips tombstoned slots entirely.
    for migration in migrations.iter().filter(|m| m.tombstone) {
        assert!(
            migration.sql.is_empty(),
            "tombstone {} ({}) carries SQL, which the runner will never execute",
            migration.version,
            migration.name
        );
    }
}
