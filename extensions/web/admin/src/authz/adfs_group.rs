//! The `adfs_group` subject dimension.
//!
//! The values are derived, not assigned: a user holds each AD group name
//! recorded in `adfs_group_memberships` — written by the ADFS callback from
//! the `id_token`'s group claim, replaced on every sign-in. Rules at
//! `rule_type = 'adfs_group'` confine an entity to members of a named
//! `Systemprompt-*` group; a user outside the group holds no matching value,
//! matches no rule, and the resolver's default closes the gated entity.
//!
//! Resolved by lookup, like the department dimension, so membership binds on
//! the next request rather than at token issue time. Sign-in is an explicit
//! event with a handler of its own, so it calls [`invalidate`] and a changed
//! membership does not wait out the TTL.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{RuleType, SubjectAttributeProvider, SubjectDimension};
use tokio::sync::RwLock;

const ADFS_GROUP_SLUG: &str = "adfs_group";

// Why: between `department` (100) and core's `ROLE` (200) — a group rule must
// out-rank the role band that grants the marketplace to every user, while an
// operator's department rule can still override it.
const ADFS_GROUP_PRECEDENCE: u16 = 150;

const ADFS_GROUP_TTL: Duration = Duration::from_secs(60);

type GroupCache = HashMap<String, (Vec<String>, Instant)>;

static GROUP_CACHE: LazyLock<RwLock<GroupCache>> = LazyLock::new(|| RwLock::new(HashMap::new()));

#[must_use]
pub fn adfs_group_rule_type() -> RuleType {
    RuleType::extension(ADFS_GROUP_SLUG)
        .unwrap_or_else(|e| unreachable!("`{ADFS_GROUP_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn adfs_group_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: adfs_group_rule_type(),
        label: "AD group",
        precedence: ADFS_GROUP_PRECEDENCE,
    }
}

// Why: Drop the cached values for one user, so a sign-in with changed groups
// takes effect on the next request instead of after the TTL.
pub async fn invalidate(user_id: &UserId) {
    GROUP_CACHE.write().await.remove(user_id.as_str());
}

#[derive(Debug)]
pub struct AdfsGroupAttributeProvider {
    pool: Arc<PgPool>,
}

impl AdfsGroupAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn cached(user_id: &UserId) -> Option<Vec<String>> {
        let cache = GROUP_CACHE.read().await;
        cache
            .get(user_id.as_str())
            .filter(|(_, at)| at.elapsed() < ADFS_GROUP_TTL)
            .map(|(values, _)| values.clone())
    }

    async fn store(user_id: &UserId, values: &[String]) {
        let mut cache = GROUP_CACHE.write().await;
        cache.insert(
            user_id.as_str().to_owned(),
            (values.to_vec(), Instant::now()),
        );
    }
}

#[async_trait]
impl SubjectAttributeProvider for AdfsGroupAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        adfs_group_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Vec<String> {
        if let Some(values) = Self::cached(user_id).await {
            return values;
        }
        let values =
            match crate::repositories::users::adfs_groups::list_groups(self.pool.as_ref(), user_id)
                .await
            {
                Ok(groups) => groups,
                Err(e) => {
                    tracing::warn!(
                        error = %e, user_id = %user_id,
                        "adfs group lookup failed; resolving with no group attribute",
                    );
                    Vec::new()
                },
            };
        Self::store(user_id, &values).await;
        values
    }
}
