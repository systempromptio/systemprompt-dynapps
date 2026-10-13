//! `op://<vault>/<item>/<field>` secret references, as 1Password prints them.

use std::fmt;
use std::str::FromStr;

use crate::error::OnePasswordError;

const SCHEME: &str = "op://";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct OpReference {
    pub vault: String,
    pub item: String,
    pub field: String,
}

impl OpReference {
    pub fn parse(raw: &str) -> Result<Self, OnePasswordError> {
        let invalid = |why: &str| {
            OnePasswordError::Config(format!(
                "reference '{raw}' {why}; expected op://<vault>/<item>/<field>"
            ))
        };
        let rest = raw
            .strip_prefix(SCHEME)
            .ok_or_else(|| invalid("does not start with op://"))?;
        let parts: Vec<&str> = rest.split('/').collect();
        let [vault, item, field] = parts.as_slice() else {
            return Err(invalid("does not have exactly three segments"));
        };
        if [vault, item, field].iter().any(|p| p.trim().is_empty()) {
            return Err(invalid("has an empty segment"));
        }
        Ok(Self {
            vault: (*vault).to_owned(),
            item: (*item).to_owned(),
            field: (*field).to_owned(),
        })
    }
}

impl FromStr for OpReference {
    type Err = OnePasswordError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

impl fmt::Display for OpReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{SCHEME}{}/{}/{}", self.vault, self.item, self.field)
    }
}
