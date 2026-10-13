//! The item shape both backends return: `op item get --format json` and the
//! Connect `GET /v1/vaults/{id}/items/{id}` body agree on these fields.

use serde::Deserialize;

pub const CONCEALED: &str = "CONCEALED";
pub const STRING: &str = "STRING";

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    #[serde(skip)]
    pub vault: String,
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Field {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub value: Option<String>,
}

impl Field {
    #[must_use]
    pub fn is_readable(&self) -> bool {
        self.kind == CONCEALED || self.kind == STRING
    }

    #[must_use]
    pub fn concealed(name: &str, value: String) -> Self {
        Self {
            id: name.to_owned(),
            label: Some(name.to_owned()),
            kind: CONCEALED.to_owned(),
            value: Some(value),
        }
    }
}
