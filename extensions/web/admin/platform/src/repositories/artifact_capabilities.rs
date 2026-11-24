//! A live Drive page's manifest: the capabilities the agent declared for an
//! HTML document and the runtime contract it declared them against. One row
//! per document; a new declaration replaces it whole.

use sqlx::PgPool;
// JSON: the declared capabilities, stored as the JSONB the manifest is.
use serde_json::Value;
use systemprompt::identifiers::{DocumentId, UserId};

// JSON: the manifest as declared, validated by the caller before it is stored.
#[derive(Debug)]
pub struct DeclaredCapabilities {
    pub capabilities: Value,
    pub contract_version: String,
}

// JSON: the manifest, already validated against the instance and its policy.
pub async fn set_artifact_capabilities(
    pool: &PgPool,
    document: &DocumentId,
    capabilities: &Value,
    contract_version: &str,
    declared_by: &UserId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO artifact_capabilities
                (document_id, capabilities, contract_version, declared_by)
           VALUES ($1::text::uuid, $2, $3, $4)
           ON CONFLICT (document_id) DO UPDATE
              SET capabilities = EXCLUDED.capabilities,
                  contract_version = EXCLUDED.contract_version,
                  declared_by = EXCLUDED.declared_by,
                  declared_at = CURRENT_TIMESTAMP"#,
        document.as_str(),
        capabilities,
        contract_version,
        declared_by.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_artifact_capabilities(
    pool: &PgPool,
    document: &DocumentId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM artifact_capabilities WHERE document_id = $1::text::uuid",
        document.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn find_artifact_capabilities(
    pool: &PgPool,
    document: &DocumentId,
) -> Result<Option<DeclaredCapabilities>, sqlx::Error> {
    sqlx::query_as!(
        DeclaredCapabilities,
        r#"SELECT capabilities, contract_version
             FROM artifact_capabilities
            WHERE document_id = $1::text::uuid"#,
        document.as_str(),
    )
    .fetch_optional(pool)
    .await
}
