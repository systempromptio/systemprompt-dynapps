//! The `Extension` registration: schema and migrations. Nothing is mounted on
//! HTTP yet.

use systemprompt::extension::prelude::*;

use crate::schema;

#[derive(Debug, Default, Clone, Copy)]
pub struct ChatsExtension;

impl ChatsExtension {
    pub const ID: &'static str = "dynwork-chats";
}

impl Extension for ChatsExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: Self::ID,
            name: "DynWork Chats",
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
        vec!["users"]
    }
}
