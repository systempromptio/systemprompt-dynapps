//! The gateway-route catalog plane owns row existence, never access: its
//! upsert conflicts with `DO NOTHING`, so a restart cannot rewrite the
//! `default_included` that `rules.yaml` declared for a route.

use crate::support::repo_root;

const CATALOG: &str = "extensions/web/shared/admin-core/src/repositories/sync/route_catalog.rs";

fn catalog_source() -> String {
    std::fs::read_to_string(repo_root().join(CATALOG)).expect("route catalog source")
}

fn insert_sql(src: &str) -> &str {
    let start = src
        .find("INSERT INTO access_control_entities")
        .expect("catalog inserts route entities");
    let end = src[start..].find('"').expect("end of the insert literal");
    &src[start..start + end]
}

#[test]
fn catalog_upsert_does_nothing_on_conflict() {
    let src = catalog_source();
    let sql = insert_sql(&src);
    assert!(
        sql.contains("ON CONFLICT (entity_type, entity_id) DO NOTHING"),
        "the catalog must leave an existing row alone: {sql}"
    );
    assert!(
        !sql.contains("DO UPDATE"),
        "the catalog rewrites a row it does not own: {sql}"
    );
}

#[test]
fn catalog_never_updates_an_access_column() {
    let src = catalog_source();
    assert!(
        !src.contains("UPDATE access_control_entities"),
        "the catalog plane updates access_control_entities"
    );
    assert!(
        !src.contains("SET default_included"),
        "the catalog plane sets default_included on an existing row"
    );
}
