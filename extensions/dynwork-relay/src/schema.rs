//! Schema and migration inventory. DDL lives in `schema/*.sql`; migrations
//! are discovered from `schema/migrations/` by `build.rs`.

use systemprompt::extension::prelude::{Migration, SchemaDefinition, extension_migrations};

pub fn schema_definitions() -> Vec<SchemaDefinition> {
    vec![SchemaDefinition::sql_only(include_str!(
        "../schema/01_dynwork_relay.sql"
    ))]
}

#[expect(
    clippy::missing_const_for_fn,
    reason = "the body is generated from schema/migrations/ and stops being const with the first migration"
)]
pub fn migrations() -> Vec<Migration> {
    extension_migrations!()
}
