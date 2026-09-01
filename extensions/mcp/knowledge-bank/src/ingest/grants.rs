//! Translation of source-system permissions into systemprompt subjects.
//!
//! Grants are written in the *systemprompt* vocabulary the principal module
//! defines — `user`, `role`, and the attribute bands — never as raw Atlassian
//! identifiers, because the retrieval join compares them against what a
//! [`crate::principal::Principal`] produces. The rules:
//!
//! - content with no restriction of its own receives the source's configured
//!   grants, which default to `role: user` (everyone signed in);
//! - a Confluence user restriction maps to `user:<systemprompt id>` through the
//!   linked-account table, then by e-mail; an unmapped account is dropped;
//! - a Confluence group restriction maps to `adfs_group:<value>` only through
//!   the configured `atlassian.group_map`; an unmapped group is dropped;
//! - a restricted document whose restrictions all failed to map keeps NO grants
//!   and is reachable by nobody — the fail-closed direction.

use crate::config::{AtlassianSettings, SourceConfig};
use crate::principal::Subject;
use sqlx::PgPool;
use std::collections::HashMap;
use systemprompt::identifiers::UserId;

/// Read restrictions as the source system states them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RawRestrictions {
    pub account_ids: Vec<String>,
    pub emails: Vec<String>,
    pub group_names: Vec<String>,
}

impl RawRestrictions {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.account_ids.is_empty() && self.emails.is_empty() && self.group_names.is_empty()
    }
}

/// Linked Atlassian accounts and e-mails, resolved to systemprompt users.
#[derive(Debug, Clone, Default)]
pub struct IdentityMap {
    by_account_id: HashMap<String, UserId>,
    by_email: HashMap<String, UserId>,
}

impl IdentityMap {
    pub async fn load(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let linked =
            sqlx::query!(r#"SELECT user_id, atlassian_account_id FROM atlassian_user_credentials"#)
                .fetch_all(pool)
                .await?;
        let users = sqlx::query!(r#"SELECT id, email FROM users"#)
            .fetch_all(pool)
            .await?;
        Ok(Self {
            by_account_id: linked
                .into_iter()
                .map(|row| (row.atlassian_account_id, UserId::new(row.user_id)))
                .collect(),
            by_email: users
                .into_iter()
                .map(|row| (row.email.to_lowercase(), UserId::new(row.id)))
                .collect(),
        })
    }

    #[must_use]
    pub fn with_account(mut self, account_id: &str, user_id: UserId) -> Self {
        self.by_account_id.insert(account_id.to_owned(), user_id);
        self
    }

    #[must_use]
    pub fn with_email(mut self, email: &str, user_id: UserId) -> Self {
        self.by_email.insert(email.to_lowercase(), user_id);
        self
    }
}

/// Resolves grants for one source category.
#[derive(Debug, Clone)]
pub struct GrantResolver {
    defaults: Vec<Subject>,
    identities: IdentityMap,
    group_map: HashMap<String, String>,
}

impl GrantResolver {
    #[must_use]
    pub fn new(
        source: &SourceConfig,
        settings: &AtlassianSettings,
        identities: IdentityMap,
    ) -> Self {
        Self {
            defaults: source
                .default_grants()
                .into_iter()
                .map(|g| Subject {
                    kind: g.kind,
                    value: g.value,
                })
                .collect(),
            identities,
            group_map: settings.group_map.clone().into_iter().collect(),
        }
    }

    #[must_use]
    pub fn unrestricted(&self) -> Vec<Subject> {
        self.defaults.clone()
    }

    #[must_use]
    pub fn resolve(&self, restrictions: &RawRestrictions) -> Vec<Subject> {
        if restrictions.is_empty() {
            return self.unrestricted();
        }
        let mut subjects: Vec<Subject> = Vec::new();
        for account in &restrictions.account_ids {
            if let Some(user) = self.identities.by_account_id.get(account) {
                subjects.push(user_subject(user));
            } else {
                tracing::debug!(account, "confluence account has no linked user; dropped");
            }
        }
        for email in &restrictions.emails {
            if let Some(user) = self.identities.by_email.get(&email.to_lowercase()) {
                subjects.push(user_subject(user));
            }
        }
        for group in &restrictions.group_names {
            if let Some(mapped) = self.group_map.get(group) {
                subjects.push(Subject {
                    kind: "adfs_group".to_owned(),
                    value: mapped.clone(),
                });
            } else {
                tracing::debug!(group, "atlassian group is not in group_map; dropped");
            }
        }
        subjects.sort();
        subjects.dedup();
        subjects
    }
}

fn user_subject(user: &UserId) -> Subject {
    Subject {
        kind: "user".to_owned(),
        value: user.to_string(),
    }
}
