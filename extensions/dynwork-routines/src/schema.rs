//! Schema and migration inventory. DDL lives in `schema/*.sql`; migrations
//! are discovered from `schema/migrations/` by `build.rs`.

use systemprompt::extension::prelude::{Migration, SchemaDefinition, extension_migrations};

pub fn schema_definitions() -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::sql_only(include_str!("../schema/01_dynwork_routines.sql")),
        SchemaDefinition::sql_only(include_str!("../schema/02_dynwork_routine_templates.sql")),
    ]
}

pub fn migrations() -> Vec<Migration> {
    extension_migrations!()
}
