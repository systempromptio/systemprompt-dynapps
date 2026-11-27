//! The `where` argument: typed property values a document must carry.
//!
//! A map, never a query: each key must be a property some space declares, or
//! the call is refused naming the declared ones. `false` or `null` asks for
//! "not set", as dynbrain's filter does; any other value must be carried as
//! written. The search applies it inside its SQL and a list applies the same
//! rule with [`PropertyFilter::admits`].

use std::collections::BTreeSet;

// JSON: property values are free JSON, as the `docs_*` tools pass them.
use serde_json::{Map, Value};

use crate::properties::PropertyError;

/// The property values a document must carry, and the ones it must not set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PropertyFilter {
    pub carried: Map<String, Value>,
    pub unset: Vec<String>,
}

impl PropertyFilter {
    pub fn parse(
        filter: Option<&Map<String, Value>>,
        declared: &BTreeSet<String>,
    ) -> Result<Self, PropertyError> {
        let mut parsed = Self::default();
        for (key, value) in filter.into_iter().flatten() {
            if !declared.contains(key) {
                let known: Vec<&str> = declared.iter().map(String::as_str).collect();
                return Err(PropertyError::NotDeclared {
                    name: key.clone(),
                    declared: if known.is_empty() {
                        "none".to_owned()
                    } else {
                        known.join(", ")
                    },
                });
            }
            if is_blank(value) {
                parsed.unset.push(key.clone());
            } else {
                parsed.carried.insert(key.clone(), value.clone());
            }
        }
        Ok(parsed)
    }

    pub fn admits(&self, properties: &Map<String, Value>) -> bool {
        let carries = self
            .carried
            .iter()
            .all(|(key, value)| properties.get(key) == Some(value));
        let sets_one = self
            .unset
            .iter()
            .any(|key| properties.get(key).is_some_and(|value| !is_blank(value)));
        carries && !sets_one
    }

    // JSON: the carried values cross into the SQL as one JSONB object.
    pub fn carried_document(&self) -> Value {
        Value::Object(self.carried.clone())
    }
}

fn is_blank(value: &Value) -> bool {
    matches!(value, Value::Null | Value::Bool(false)) || value.as_str() == Some("")
}
