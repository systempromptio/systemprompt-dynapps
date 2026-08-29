//! The calling identity, and the subjects a document may be granted to.
//!
//! REQ-052 (P0) requires retrieval-time filtering by identity, project, role,
//! and permission, with unauthorized documents never returned. The Node
//! "project-context" system this crate ports has no equivalent — it is a
//! single-user local tool whose config states plainly that all storage stays
//! local — so this module is net-new rather than a migration.
//!
//! Subject kinds deliberately reuse the names of the instance's existing
//! authz precedence bands (`user`, `department`, `adfs_group`, `role`,
//! `organization`) so document grants and `access_control_rules` describe
//! principals the same way, rather than inventing a second scheme.

use std::collections::BTreeMap;
use systemprompt::identifiers::UserId;
use systemprompt::models::execution::context::RequestContext;

pub const SUBJECT_USER: &str = "user";
pub const SUBJECT_ROLE: &str = "role";
pub const ATTRIBUTE_SUBJECT_KINDS: [&str; 3] = ["department", "adfs_group", "organization"];

/// One `(kind, value)` pair a document grant can name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Subject {
    pub kind: String,
    pub value: String,
}

/// The authenticated caller, reduced to what retrieval filtering needs.
#[derive(Debug, Clone)]
pub struct Principal {
    pub user_id: UserId,
    pub roles: Vec<String>,
    pub attributes: BTreeMap<String, String>,
}

impl Principal {
    pub fn from_request_context(ctx: &RequestContext) -> Self {
        let user_id = ctx.auth.actor.user_id.clone();
        let (roles, attributes) = ctx.user.as_ref().map_or_else(
            || (Vec::new(), BTreeMap::new()),
            |user| {
                let attributes = user
                    .attributes
                    .iter()
                    .filter_map(|(key, value)| value.as_str().map(|s| (key.clone(), s.to_owned())))
                    .collect();
                (user.roles.clone(), attributes)
            },
        );
        Self {
            user_id,
            roles,
            attributes,
        }
    }

    pub fn subjects(&self) -> Vec<Subject> {
        let mut subjects = vec![Subject {
            kind: SUBJECT_USER.to_owned(),
            value: self.user_id.to_string(),
        }];
        subjects.extend(self.roles.iter().map(|role| Subject {
            kind: SUBJECT_ROLE.to_owned(),
            value: role.clone(),
        }));
        for kind in ATTRIBUTE_SUBJECT_KINDS {
            if let Some(value) = self.attributes.get(kind) {
                subjects.push(Subject {
                    kind: kind.to_owned(),
                    value: value.clone(),
                });
            }
        }
        subjects
    }
}
