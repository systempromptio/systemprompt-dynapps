//! A user's linked Salesforce Usernames.
//!
//! One per org, as the user detail page lists them.
//! The rows live in the web-owned `salesforce_user_identities` side table. The
//! platform domain owns every write and the JWT-bearer lookup; this is the one
//! read another domain needs.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone)]
pub struct SalesforceIdentity {
    pub provider: String,
    pub sf_username: String,
}

pub async fn list_identities(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<SalesforceIdentity>, sqlx::Error> {
    sqlx::query_as!(
        SalesforceIdentity,
        "SELECT provider, sf_username FROM salesforce_user_identities \
         WHERE user_id = $1 ORDER BY provider",
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}
