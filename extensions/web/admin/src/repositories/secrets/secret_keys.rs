//! Per-user data encryption key issue and rotation.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::secrets::secret_crypto::{self, SecretCryptoError};

pub async fn get_or_create_user_dek(
    pool: &PgPool,
    user_id: &UserId,
    master_key: &[u8; 32],
) -> Result<[u8; 32], SecretCryptoError> {
    let row = sqlx::query!(
        "SELECT encrypted_dek, dek_nonce FROM user_encryption_keys WHERE user_id = $1",
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    if let Some(row) = row {
        let nonce: [u8; 12] = row.dek_nonce.try_into().map_err(|v: Vec<u8>| {
            tracing::warn!(len = v.len(), "Stored DEK nonce is not 12 bytes");
            SecretCryptoError::InvalidKeyMaterial
        })?;
        let plaintext = secret_crypto::decrypt(master_key, &nonce, &row.encrypted_dek)?;
        let dek: [u8; 32] = plaintext.try_into().map_err(|v: Vec<u8>| {
            tracing::warn!(len = v.len(), "Decrypted DEK is not 32 bytes");
            SecretCryptoError::InvalidKeyMaterial
        })?;
        return Ok(dek);
    }

    let dek = secret_crypto::generate_dek();
    let nonce = secret_crypto::generate_nonce();
    let encrypted_dek = secret_crypto::encrypt(master_key, &nonce, &dek)?;
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO user_encryption_keys (id, user_id, encrypted_dek, dek_nonce) \
         VALUES ($1, $2, $3, $4)",
        id,
        user_id.as_str(),
        encrypted_dek.as_slice(),
        nonce.as_slice(),
    )
    .execute(pool)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    tracing::info!(user_id = %user_id, "Created new encryption key for user");

    Ok(dek)
}

pub async fn rotate_user_dek(
    pool: &PgPool,
    user_id: &UserId,
    master_key: &[u8; 32],
) -> Result<(), SecretCryptoError> {
    let old_dek = get_or_create_user_dek(pool, user_id, master_key).await?;

    let new_dek = secret_crypto::generate_dek();
    let new_dek_nonce = secret_crypto::generate_nonce();
    let encrypted_new_dek = secret_crypto::encrypt(master_key, &new_dek_nonce, &new_dek)?;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    let secret_rows = sqlx::query!(
        "SELECT id, encrypted_value, value_nonce FROM plugin_env_vars \
         WHERE user_id = $1 AND is_secret = true AND encrypted_value IS NOT NULL",
        user_id.as_str(),
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    for row in &secret_rows {
        let old_nonce: [u8; 12] = row
            .value_nonce
            .as_deref()
            .unwrap_or(&[])
            .try_into()
            .map_err(|e: std::array::TryFromSliceError| {
                tracing::warn!(error = %e, "Stored secret nonce is not 12 bytes");
                SecretCryptoError::InvalidKeyMaterial
            })?;
        let encrypted_value = row.encrypted_value.as_deref().unwrap_or(&[]);
        let plaintext = secret_crypto::decrypt(&old_dek, &old_nonce, encrypted_value)?;

        let new_value_nonce = secret_crypto::generate_nonce();
        let new_encrypted = secret_crypto::encrypt(&new_dek, &new_value_nonce, &plaintext)?;

        sqlx::query!(
            "UPDATE plugin_env_vars SET encrypted_value = $1, value_nonce = $2, \
             key_version = key_version + 1 WHERE id = $3",
            new_encrypted.as_slice(),
            new_value_nonce.as_slice(),
            row.id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| SecretCryptoError::Database(e.to_string()))?;
    }

    // Why: the Atlassian refresh token is sealed under the same DEK. Skipping it
    // here would not fail loudly — the row would simply stop decrypting, and the
    // user would be silently signed out of Jira and Confluence with no event to
    // trace it back to. Rewrap it in the same transaction as everything else.
    rewrap_atlassian_credential(&mut tx, user_id, &old_dek, &new_dek).await?;

    sqlx::query!(
        "UPDATE user_encryption_keys SET encrypted_dek = $1, dek_nonce = $2, \
         key_version = key_version + 1, rotated_at = NOW() WHERE user_id = $3",
        encrypted_new_dek.as_slice(),
        new_dek_nonce.as_slice(),
        user_id.as_str(),
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    tx.commit()
        .await
        .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    tracing::info!(
        user_id = %user_id,
        secrets_rotated = %secret_rows.len(),
        "Rotated user encryption key"
    );

    Ok(())
}

// Why: split out of `rotate_user_dek` purely to keep that function inside the
// file-size and complexity gates. Both token columns move together — the access
// token is a cache and could be dropped instead, but rewrapping it costs one
// more seal and avoids a forced refresh immediately after every rotation.
async fn rewrap_atlassian_credential(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: &UserId,
    old_dek: &[u8; 32],
    new_dek: &[u8; 32],
) -> Result<(), SecretCryptoError> {
    let row = sqlx::query!(
        "SELECT encrypted_refresh_token, refresh_token_nonce, encrypted_access_token, \
         access_token_nonce FROM atlassian_user_credentials WHERE user_id = $1 FOR UPDATE",
        user_id.as_str(),
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    let Some(row) = row else {
        return Ok(());
    };

    let refresh = reseal(old_dek, new_dek, &row.refresh_token_nonce, &row.encrypted_refresh_token)?;

    // Why: the access token is nullable — a row written by the callback before
    // its first mint has none. Absent is normal, not a corruption signal.
    let access = match (row.encrypted_access_token, row.access_token_nonce) {
        (Some(ct), Some(nonce)) => Some(reseal(old_dek, new_dek, &nonce, &ct)?),
        _ => None,
    };

    sqlx::query!(
        "UPDATE atlassian_user_credentials SET encrypted_refresh_token = $1, \
         refresh_token_nonce = $2, encrypted_access_token = $3, access_token_nonce = $4, \
         key_version = key_version + 1, updated_at = NOW() WHERE user_id = $5",
        refresh.0.as_slice(),
        refresh.1.as_slice(),
        access.as_ref().map(|a| a.0.as_slice()),
        access.as_ref().map(|a| a.1.as_slice()),
        user_id.as_str(),
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| SecretCryptoError::Database(e.to_string()))?;

    Ok(())
}

// Why: decrypt under the old key and seal under the new one with a FRESH nonce.
// Reusing the stored nonce against a new key would be safe, but reusing it
// against the same key never is, and a helper that always generates one removes
// the chance of a future caller getting that distinction wrong.
fn reseal(
    old_dek: &[u8; 32],
    new_dek: &[u8; 32],
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<(Vec<u8>, [u8; 12]), SecretCryptoError> {
    let old_nonce: [u8; 12] = nonce.try_into().map_err(|_| {
        tracing::warn!("Stored Atlassian nonce is not 12 bytes");
        SecretCryptoError::InvalidKeyMaterial
    })?;
    let plaintext = secret_crypto::decrypt(old_dek, &old_nonce, ciphertext)?;
    let new_nonce = secret_crypto::generate_nonce();
    let sealed = secret_crypto::encrypt(new_dek, &new_nonce, &plaintext)?;
    Ok((sealed, new_nonce))
}
