//! Typed properties: the definitions a space declares and the values an
//! article variant carries, checked on every write.
//!
//! A definition is `{name, string?, type, selection?, tags?}`, the shape
//! dynbrain uses. `selection` and `tags` list `[value, label]` pairs. A value
//! may be `null` (unset); anything else must match its property's type, and a
//! key the space does not declare is refused. The free front matter is not
//! checked: it is kept as written.

use std::collections::BTreeSet;

use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Serialize};
// JSON: definitions and values are the JSON documents the schema stores.
use serde_json::{Map, Value};
use systemprompt::identifiers::UserId;
use url::Url;

/// The value types a property may declare.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyType {
    Boolean,
    Integer,
    Float,
    Text,
    Char,
    Date,
    Datetime,
    Selection,
    Tags,
    User,
    Link,
}

/// One property of a space.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PropertyDefinition {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub string: Option<String>,
    #[serde(rename = "type")]
    pub kind: PropertyType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selection: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<(String, String)>,
}

/// The validated property definitions of a space.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PropertySchema(pub Vec<PropertyDefinition>);

#[derive(Debug, thiserror::Error)]
pub enum PropertyError {
    #[error("the property schema is not a list of definitions")]
    Shape(#[from] serde_json::Error),
    #[error("property name `{0}` must be lowercase letters, digits or `_`, starting with a letter")]
    InvalidName(String),
    #[error("property `{0}` is declared twice")]
    DuplicateName(String),
    #[error("property `{0}` needs a non-empty list of options")]
    MissingOptions(String),
    #[error("property `{0}` declares options its type does not take")]
    UnexpectedOptions(String),
    #[error("property `{property}` repeats the option `{value}`")]
    DuplicateOption { property: String, value: String },
    #[error("property `{0}` is not declared by the space")]
    Unknown(String),
    #[error("'{name}' is not a metadata field of this documentation. Declared: {declared}.")]
    NotDeclared { name: String, declared: String },
    #[error("property `{property}` must be a {expected:?} value")]
    WrongType {
        property: String,
        expected: PropertyType,
    },
}

impl PropertySchema {
    pub fn names(&self) -> BTreeSet<String> {
        self.0
            .iter()
            .map(|definition| definition.name.clone())
            .collect()
    }

    // JSON: the schema is stored and exchanged as the JSON document dynbrain uses.
    pub fn parse(document: &Value) -> Result<Self, PropertyError> {
        let schema = Self(serde_json::from_value(document.clone())?);
        schema.check()?;
        Ok(schema)
    }

    // JSON: the `property_schema` JSONB column.
    pub fn to_document(&self) -> Result<Value, PropertyError> {
        Ok(serde_json::to_value(self)?)
    }

    // JSON: property values are free-typed JSON until checked here.
    pub fn validate(&self, values: &Map<String, Value>) -> Result<(), PropertyError> {
        values.iter().try_for_each(|(name, value)| {
            let definition = self
                .0
                .iter()
                .find(|definition| &definition.name == name)
                .ok_or_else(|| PropertyError::Unknown(name.clone()))?;
            if value.is_null() || definition.accepts(value) {
                Ok(())
            } else {
                Err(PropertyError::WrongType {
                    property: name.clone(),
                    expected: definition.kind,
                })
            }
        })
    }

    fn check(&self) -> Result<(), PropertyError> {
        let mut seen = Vec::new();
        for definition in &self.0 {
            if !is_property_name(&definition.name) {
                return Err(PropertyError::InvalidName(definition.name.clone()));
            }
            if seen.contains(&definition.name.as_str()) {
                return Err(PropertyError::DuplicateName(definition.name.clone()));
            }
            seen.push(definition.name.as_str());
            definition.check_options()?;
        }
        Ok(())
    }
}

impl PropertyDefinition {
    fn check_options(&self) -> Result<(), PropertyError> {
        let (wanted, unwanted) = match self.kind {
            PropertyType::Selection => (Some(&self.selection), &self.tags),
            PropertyType::Tags => (Some(&self.tags), &self.selection),
            _ if self.selection.is_empty() => (None, &self.tags),
            _ => (None, &self.selection),
        };
        if !unwanted.is_empty() {
            return Err(PropertyError::UnexpectedOptions(self.name.clone()));
        }
        let Some(options) = wanted else {
            return Ok(());
        };
        if options.is_empty() {
            return Err(PropertyError::MissingOptions(self.name.clone()));
        }
        let mut values: Vec<&str> = Vec::new();
        for (value, _) in options {
            if values.contains(&value.as_str()) {
                return Err(PropertyError::DuplicateOption {
                    property: self.name.clone(),
                    value: value.clone(),
                });
            }
            values.push(value);
        }
        Ok(())
    }

    // JSON: one property value, checked against its declared type.
    fn accepts(&self, value: &Value) -> bool {
        match self.kind {
            PropertyType::Boolean => value.is_boolean(),
            PropertyType::Integer => value.is_i64() || value.is_u64(),
            PropertyType::Float => value.is_number(),
            PropertyType::Text => value.is_string(),
            PropertyType::Char => value
                .as_str()
                .is_some_and(|text| !text.contains(['\n', '\r'])),
            PropertyType::Date => value
                .as_str()
                .is_some_and(|text| NaiveDate::parse_from_str(text, "%Y-%m-%d").is_ok()),
            PropertyType::Datetime => value
                .as_str()
                .is_some_and(|text| DateTime::parse_from_rfc3339(text).is_ok()),
            PropertyType::Selection => value
                .as_str()
                .is_some_and(|text| self.selection.iter().any(|(option, _)| option == text)),
            PropertyType::Tags => self.accepts_tags(value),
            PropertyType::User => value
                .as_str()
                .is_some_and(|text| UserId::try_new(text).is_ok()),
            PropertyType::Link => value.as_str().is_some_and(is_web_link),
        }
    }

    // JSON: a `tags` value is a JSON array of declared option values.
    fn accepts_tags(&self, value: &Value) -> bool {
        let Some(items) = value.as_array() else {
            return false;
        };
        let mut picked: Vec<&str> = Vec::new();
        items.iter().all(|item| {
            item.as_str().is_some_and(|text| {
                let declared = self.tags.iter().any(|(option, _)| option == text);
                let fresh = !picked.contains(&text);
                picked.push(text);
                declared && fresh
            })
        })
    }
}

fn is_property_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|first| first.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        && name.len() <= 64
}

fn is_web_link(text: &str) -> bool {
    Url::parse(text).is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.has_host())
}
