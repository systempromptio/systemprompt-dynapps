use dynwork_relay::credential::{CREDENTIAL_PREFIX, CredentialHash, DeviceCredential};

#[test]
fn a_minted_credential_hashes_to_its_stored_hash() {
    let (credential, hash) = DeviceCredential::mint();
    assert!(credential.expose().starts_with(CREDENTIAL_PREFIX));
    assert_eq!(
        CredentialHash::of_presented(credential.expose()),
        Some(hash.clone())
    );
    assert_eq!(hash.as_str().len(), 64);
    assert!(!hash.as_str().contains(credential.expose()));
}

#[test]
fn two_credentials_differ() {
    let (first, first_hash) = DeviceCredential::mint();
    let (second, second_hash) = DeviceCredential::mint();
    assert_ne!(first, second);
    assert_ne!(first_hash, second_hash);
}

#[test]
fn tokens_that_are_not_relay_credentials_are_not_hashed() {
    let (credential, _) = DeviceCredential::mint();
    let token = credential.expose();
    assert_eq!(
        CredentialHash::of_presented("eyJhbGciOiJSUzI1NiJ9.e30.sig"),
        None
    );
    assert_eq!(
        CredentialHash::of_presented(&token[..token.len() - 1]),
        None
    );
    assert_eq!(
        CredentialHash::of_presented(&format!("{}!", &token[..token.len() - 1])),
        None
    );
}

#[test]
fn the_debug_form_redacts_the_secret() {
    let (credential, _) = DeviceCredential::mint();
    let printed = format!("{credential:?}");
    assert!(!printed.contains(&credential.expose()[CREDENTIAL_PREFIX.len()..]));
    assert!(printed.contains("redacted"));
}
