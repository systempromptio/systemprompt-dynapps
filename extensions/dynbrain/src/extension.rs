//! The `Extension` registration: the schema and its migrations.
//!
//! The HTTP API and the MCP server mount on top of this in later phases; for
//! now the extension only owns its tables.

use systemprompt::extension::prelude::{Extension, ExtensionMetadata, Migration, SchemaDefinition};

use crate::schema;

#[derive(Debug, Default, Clone, Copy)]
pub struct DynbrainExtension;

impl DynbrainExtension {
    pub const ID: &'static str = "dynbrain";
}

impl Extension for DynbrainExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: Self::ID,
            name: "Brain",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        schema::schema_definitions()
    }

    fn migrations(&self) -> Vec<Migration> {
        schema::migrations()
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec!["users", "web"]
    }
}
