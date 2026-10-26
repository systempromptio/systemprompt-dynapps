use std::error::Error;

use systemprompt_web_admin_core::crypto::secret_crypto::{
    SecretCryptoError, decrypt, encrypt, fixed_key_material, generate_dek, generate_nonce,
    master_key_from_secret,
};

const KEY: [u8; 32] = [7u8; 32];
const NONCE: [u8; 12] = [3u8; 12];

#[test]
fn round_trip_recovers_the_plaintext() {
    let plaintext = b"sk-ant-api03-not-a-real-key";
    let sealed = encrypt(&KEY, &NONCE, plaintext).expect("encrypt");
    assert_ne!(sealed.as_slice(), plaintext.as_slice());
    let opened = decrypt(&KEY, &NONCE, &sealed).expect("decrypt");
    assert_eq!(opened, plaintext.to_vec());
}

#[test]
fn empty_plaintext_round_trips() {
    let sealed = encrypt(&KEY, &NONCE, b"").expect("encrypt");
    assert_eq!(sealed.len(), 16, "an empty body is just the auth tag");
    assert_eq!(
        decrypt(&KEY, &NONCE, &sealed).expect("decrypt"),
        Vec::<u8>::new()
    );
}

#[test]
fn ciphertext_is_the_plaintext_length_plus_a_tag() {
    let sealed = encrypt(&KEY, &NONCE, b"0123456789").expect("encrypt");
    assert_eq!(sealed.len(), 10 + 16);
}

#[test]
fn the_same_key_and_nonce_are_deterministic() {
    let a = encrypt(&KEY, &NONCE, b"same input").expect("encrypt");
    let b = encrypt(&KEY, &NONCE, b"same input").expect("encrypt");
    assert_eq!(a, b);
}

#[test]
fn a_different_nonce_produces_different_ciphertext() {
    let other = [9u8; 12];
    let a = encrypt(&KEY, &NONCE, b"same input").expect("encrypt");
    let b = encrypt(&KEY, &other, b"same input").expect("encrypt");
    assert_ne!(a, b);
}

#[test]
fn decrypting_with_the_wrong_key_fails() {
    let sealed = encrypt(&KEY, &NONCE, b"secret").expect("encrypt");
    let wrong = [8u8; 32];
    assert!(matches!(
        decrypt(&wrong, &NONCE, &sealed),
        Err(SecretCryptoError::DecryptionFailed(_))
    ));
}

#[test]
fn decrypting_with_the_wrong_nonce_fails() {
    let sealed = encrypt(&KEY, &NONCE, b"secret").expect("encrypt");
    assert!(matches!(
        decrypt(&KEY, &[4u8; 12], &sealed),
        Err(SecretCryptoError::DecryptionFailed(_))
    ));
}

#[test]
fn a_tampered_ciphertext_is_rejected() {
    let mut sealed = encrypt(&KEY, &NONCE, b"secret").expect("encrypt");
    sealed[0] ^= 0xff;
    assert!(matches!(
        decrypt(&KEY, &NONCE, &sealed),
        Err(SecretCryptoError::DecryptionFailed(_))
    ));
}

#[test]
fn a_truncated_ciphertext_is_rejected() {
    assert!(decrypt(&KEY, &NONCE, b"too short").is_err());
    assert!(decrypt(&KEY, &NONCE, b"").is_err());
}

#[test]
fn generated_keys_and_nonces_are_not_constant() {
    assert_ne!(generate_dek(), generate_dek());
    assert_ne!(generate_dek(), [0u8; 32]);
    assert_ne!(generate_nonce(), generate_nonce());
    assert_ne!(generate_nonce(), [0u8; 12]);
}

#[test]
fn a_generated_dek_seals_and_opens_its_own_traffic() {
    let dek = generate_dek();
    let nonce = generate_nonce();
    let sealed = encrypt(&dek, &nonce, b"per-user secret").expect("encrypt");
    assert_eq!(
        decrypt(&dek, &nonce, &sealed).expect("decrypt"),
        b"per-user secret".to_vec()
    );
}

#[test]
fn error_messages_name_the_failing_stage() {
    assert!(
        SecretCryptoError::MasterKeyMissing
            .to_string()
            .starts_with("Master key not configured")
    );
    assert!(
        SecretCryptoError::Database(sqlx::Error::PoolClosed)
            .to_string()
            .starts_with("Database error:")
    );
    let decryption = decrypt(&KEY, &NONCE, b"not a ciphertext").expect_err("refused");
    assert!(
        decryption.to_string().starts_with("Decryption failed:"),
        "{decryption}"
    );
}

#[test]
fn a_missing_master_key_is_a_typed_refusal() {
    assert!(matches!(
        master_key_from_secret(None),
        Err(SecretCryptoError::MasterKeyMissing)
    ));
    assert!(matches!(
        master_key_from_secret(Some("   ")),
        Err(SecretCryptoError::MasterKeyMissing)
    ));
}

#[test]
fn short_or_non_hex_key_material_keeps_cores_cause() {
    let sixty_three_hex = "0".repeat(63);
    let error = master_key_from_secret(Some(&sixty_three_hex)).expect_err("63 hex chars refused");
    assert!(matches!(error, SecretCryptoError::InvalidKeyMaterial(_)));
    assert!(
        error
            .source()
            .is_some_and(<dyn Error>::is::<systemprompt::config::SecretsBootstrapError>)
    );
    let error = master_key_from_secret(Some(&"zz".repeat(32))).expect_err("non-hex refused");
    assert!(matches!(error, SecretCryptoError::InvalidKeyMaterial(_)));
    assert!(error.source().is_some());
}

#[test]
fn a_valid_master_key_decodes_to_thirty_two_bytes() {
    let key = master_key_from_secret(Some(&format!(" {} ", "ab".repeat(32)))).expect("valid key");
    assert_eq!(key, [0xab; 32]);
}

#[test]
fn stored_key_material_of_the_wrong_length_names_both_lengths() {
    let error = fixed_key_material::<12>(vec![0; 11]).expect_err("11 bytes is not a nonce");
    assert!(matches!(
        error,
        SecretCryptoError::StoredKeyLength {
            expected: 12,
            actual: 11
        }
    ));
    assert_eq!(
        fixed_key_material::<12>(vec![7; 12]).expect("nonce"),
        [7; 12]
    );
}
