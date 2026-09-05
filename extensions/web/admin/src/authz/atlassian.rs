//! The `atlassian` subject dimension.
//!
//! One derived value, `linked`, held by any user with a Atlassian credential
//! on file. Rules at `rule_type = 'atlassian'` confine the Jira and Confluence
//! MCP servers to those users, so an unlinked user never sees a server whose
//! every tool call could only answer "connect your account first".
//!
//! Resolved by lookup rather than at token-issue time, so linking takes effect
//! on the next request. Link and unlink are explicit events with handlers of
//! their own, so they call [`invalidate`] and the change does not wait out
//! the TTL.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{RuleType, SubjectAttributeProvider, SubjectDimension};
use tokio::sync::RwLock;

const ATLASSIAN_SLUG: &str = "atlassian";

// Why: above `group` (150) and below core's `ROLE` (200). A link rule
// must out-rank the role band that grants the marketplace to every user, while
// remaining a narrower scope than a role.
const ATLASSIAN_PRECEDENCE: u16 = 160;

const ATLASSIAN_TTL: Duration = Duration::from_secs(60);

// Why: the value a linked user holds. Absence of a value closes the gate.
pub const ATLASSIAN_LINKED_VALUE: &str = "linked";

type LinkCache = HashMap<String, (Vec<String>, Instant)>;

static LINK_CACHE: LazyLock<RwLock<LinkCache>> = LazyLock::new(|| RwLock::new(HashMap::new()));

#[must_use]
pub fn atlassian_rule_type() -> RuleType {
    RuleType::extension(ATLASSIAN_SLUG)
        .unwrap_or_else(|e| unreachable!("`{ATLASSIAN_SLUG}` is a well-formed slug: {e}"))
}

#[must_use]
pub fn atlassian_dimension() -> SubjectDimension {
    SubjectDimension {
        rule_type: atlassian_rule_type(),
        label: "Atlassian link",
        precedence: ATLASSIAN_PRECEDENCE,
    }
}

// Why: drop the cached value for one user so a link or unlink takes effect on
// the next request instead of after the TTL.
pub async fn invalidate(user_id: &UserId) {
    LINK_CACHE.write().await.remove(user_id.as_str());
}

#[derive(Debug)]
pub struct AtlassianAttributeProvider {
    pool: Arc<PgPool>,
}

impl AtlassianAttributeProvider {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    async fn cached(user_id: &UserId) -> Option<Vec<String>> {
        let cache = LINK_CACHE.read().await;
        cache
            .get(user_id.as_str())
            .filter(|(_, at)| at.elapsed() < ATLASSIAN_TTL)
            .map(|(values, _)| values.clone())
    }

    async fn store(user_id: &UserId, values: &[String]) {
        let mut cache = LINK_CACHE.write().await;
        cache.insert(
            user_id.as_str().to_owned(),
            (values.to_vec(), Instant::now()),
        );
    }
}

#[async_trait]
impl SubjectAttributeProvider for AtlassianAttributeProvider {
    fn dimension(&self) -> SubjectDimension {
        atlassian_dimension()
    }

    async fn values_for(&self, user_id: &UserId) -> Vec<String> {
        if let Some(values) = Self::cached(user_id).await {
            return values;
        }
        // Why: a lookup failure resolves to no value, which closes the gate.
        // Failing open would expose a server that cannot work anyway.
        let linked = match crate::repositories::users::atlassian_credentials::is_atlassian_linked(
            self.pool.as_ref(),
            user_id,
        )
        .await
        {
            Ok(linked) => linked,
            Err(e) => {
                tracing::warn!(
                    error = %e, user_id = %user_id,
                    "atlassian link lookup failed; resolving with no link attribute",
                );
                false
            },
        };
        let values = if linked {
            vec![ATLASSIAN_LINKED_VALUE.to_owned()]
        } else {
            Vec::new()
        };
        Self::store(user_id, &values).await;
        values
    }
}
