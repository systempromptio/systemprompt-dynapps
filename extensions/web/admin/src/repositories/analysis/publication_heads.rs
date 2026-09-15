//! The published head of every managed resource: which generation the bridge
//! serves today, so the inventory table can say whether the latest captured
//! revision is the one consumers actually receive.

use sqlx::PgPool;
use systemprompt::identifiers::{ManagedResourceId, ResourceRevisionId, UserId};

/// The most recent publication row of one resource.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PublicationHead {
    pub resource_id: ManagedResourceId,
    pub generation: i64,
    pub action: String,
    pub revision_id: Option<ResourceRevisionId>,
}

impl PublicationHead {
    // Why: a withdrawal is a head with nothing to serve.
    #[must_use]
    pub fn published_revision(&self) -> Option<&ResourceRevisionId> {
        (self.action != "withdraw")
            .then_some(self.revision_id.as_ref())
            .flatten()
    }
}

// Why: `DISTINCT ON (resource_id)` by generation — one newest publication
// per resource, keyed for the inventory rows to join on.
pub async fn list_publication_heads(
    pool: &PgPool,
    owner: &UserId,
) -> Result<Vec<PublicationHead>, sqlx::Error> {
    sqlx::query_as::<_, PublicationHead>(
        "SELECT DISTINCT ON (resource_id) resource_id, generation, action, revision_id \
         FROM managed_publications WHERE owner_id = $1 \
         ORDER BY resource_id, generation DESC",
    )
    .bind(owner.as_str())
    .fetch_all(pool)
    .await
}
