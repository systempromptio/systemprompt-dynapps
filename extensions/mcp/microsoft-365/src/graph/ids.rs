//! Microsoft Graph's own identifiers (messages, users, groups, tasks…) as the
//! tools receive them.
//!
//! They are opaque strings Microsoft mints, not this instance's identifiers,
//! so none of core's typed ids fits and none of their validators applies.
//! The newtype keeps them apart from free text in signatures; on the wire and
//! in a tool's input schema it is a plain string, so the `d365_*` schemas stay
//! those of the Belgian server.

use std::borrow::Cow;
use std::fmt;
use std::ops::Deref;

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GraphId(String);

impl GraphId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for GraphId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for GraphId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for GraphId {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for GraphId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<GraphId> for String {
    fn from(id: GraphId) -> Self {
        id.0
    }
}

impl JsonSchema for GraphId {
    fn inline_schema() -> bool {
        true
    }

    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("GraphId")
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        String::json_schema(generator)
    }
}
