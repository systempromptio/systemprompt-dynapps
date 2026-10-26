//! Wire types for the group/project declaration
//! (`services/web/config/groups.yaml`).
//!
//! The file declares the groups and projects an installation ships with, and
//! the AD groups the directory maps into each. It defines *membership shape*,
//! not entitlement and not members: which entity a group or project reaches
//! is declared in `services/access-control/rules.yaml`, and who is in one is
//! the directory's and the console's — people are never written to code.

use serde::Deserialize;

/// One group or project this installation ships.
///
/// Both halves of the file share this shape — a group carries people and
/// marketplace entitlement, a project carries work attribution, and the
/// directory maps AD groups into either the same way.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberSetDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub ad_groups: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupsDoc {
    #[serde(default)]
    pub groups: Vec<MemberSetDef>,
    #[serde(default)]
    pub projects: Vec<MemberSetDef>,
}

#[derive(Debug, thiserror::Error)]
pub enum GroupsDocError {
    #[error(
        "{set}: id '{id}' must be lowercase alphanumeric with '-' or '_', starting with a letter \
         or digit, at most 64 characters"
    )]
    InvalidId { set: &'static str, id: String },
    #[error("{set}: '{id}' must carry a name")]
    MissingName { set: &'static str, id: String },
    #[error("{set}: id '{id}' is declared twice")]
    DuplicateId { set: &'static str, id: String },
}

impl GroupsDoc {
    pub fn validate(&self) -> Result<(), GroupsDocError> {
        validate_set("groups", &self.groups)?;
        validate_set("projects", &self.projects)
    }
}

fn validate_set(set: &'static str, defs: &[MemberSetDef]) -> Result<(), GroupsDocError> {
    let mut seen: Vec<&str> = Vec::new();
    for def in defs {
        if !is_valid_id(&def.id) {
            return Err(GroupsDocError::InvalidId {
                set,
                id: def.id.clone(),
            });
        }
        if def.name.trim().is_empty() {
            return Err(GroupsDocError::MissingName {
                set,
                id: def.id.clone(),
            });
        }
        if seen.contains(&def.id.as_str()) {
            return Err(GroupsDocError::DuplicateId {
                set,
                id: def.id.clone(),
            });
        }
        seen.push(&def.id);
    }
    Ok(())
}

// Why: Postgres `CHECK (id ~ '^[a-z0-9][a-z0-9_-]{0,63}$')` on `groups` and
// `projects` must agree with this, or boot fails on a constraint violation
// instead of a readable validation message.
#[must_use]
pub fn is_valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    id.len() <= 64
        && (first.is_ascii_lowercase() || first.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GroupsLoadReport {
    pub groups: usize,
    pub projects: usize,
    pub mappings: usize,
}
