//! `PgRelayStore` against a throwaway database with the full extension
//! schema, the relay's tables included. Same rules as the in-memory store the
//! socket suites run on.

use std::sync::Arc;

use chrono::{Duration, Utc};
use dynapps_test_common::db_or_skip;
use dynwork_relay::credential::{CredentialHash, DeviceCredential, PairingToken};
use dynwork_relay::ids::{DeviceId, PairingId, PeerKind};
use dynwork_relay::store::postgres::PgRelayStore;
use dynwork_relay::store::{
    ClaimOutcome, DesktopRegistration, NewPairing, PairingOutcome, PhoneClaim, RegisterOutcome,
    RelayStore, RevokeOutcome,
};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use dynwork_relay as _;

use crate::support::{peer, user};

async fn seed_user(pool: &PgPool, name: &str) -> UserId {
    sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $1, $2)")
        .bind(name)
        .bind(format!("{name}@example.test"))
        .execute(pool)
        .await
        .expect("seed a user");
    user(name)
}

fn claim(token: &CredentialHash, byte: u8, max_devices: u32) -> PhoneClaim {
    PhoneClaim {
        token: token.clone(),
        device: DeviceId::generate(),
        peer: peer(byte),
        name: "Pixel".to_owned(),
        credential: DeviceCredential::mint().1,
        now: Utc::now(),
        max_devices,
    }
}

fn pairing(owner: &UserId, ttl: Duration) -> (NewPairing, CredentialHash) {
    let now = Utc::now();
    let (_, token) = PairingToken::mint();
    (
        NewPairing {
            id: PairingId::generate(),
            user: owner.clone(),
            token: token.clone(),
            created_at: now,
            expires_at: now + ttl,
        },
        token,
    )
}

async fn open_pairing(store: &PgRelayStore, owner: &UserId, ttl: Duration) -> CredentialHash {
    let (new, token) = pairing(owner, ttl);
    assert_eq!(
        store
            .insert_pairing(&new, 100)
            .await
            .expect("insert a pairing"),
        PairingOutcome::Inserted
    );
    token
}

async fn revoke(
    store: &PgRelayStore,
    who: &UserId,
    id: DeviceId,
) -> Result<RevokeOutcome, dynwork_relay::store::StoreError> {
    store.revoke_device(who, id, Utc::now()).await
}


#[tokio::test]
async fn a_desktop_registers_once_per_peer_within_the_device_limit() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let store = PgRelayStore::new(Arc::clone(&db.pool));
    let register = |byte| DesktopRegistration {
        user: alice.clone(),
        peer: peer(byte),
        now: Utc::now(),
        max_devices: 2,
    };
    let RegisterOutcome::Registered(first) = store.register_desktop(&register(1)).await.unwrap()
    else {
        panic!("the first desktop registers");
    };
    let RegisterOutcome::Registered(again) = store.register_desktop(&register(1)).await.unwrap()
    else {
        panic!("the same desktop registers again");
    };
    assert_eq!(first.id, again.id);
    assert_eq!(first.kind, PeerKind::Desktop);
    assert!(matches!(
        store.register_desktop(&register(2)).await.unwrap(),
        RegisterOutcome::Registered(_)
    ));
    assert_eq!(
        store.register_desktop(&register(3)).await.unwrap(),
        RegisterOutcome::DeviceLimit
    );
    let ghost = DesktopRegistration {
        user: user("ghost"),
        ..register(4)
    };
    assert_eq!(
        store.register_desktop(&ghost).await.unwrap(),
        RegisterOutcome::UnknownUser
    );
    db.cleanup().await;
}

#[tokio::test]
async fn two_concurrent_claims_of_one_pairing_yield_exactly_one_device() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let store = Arc::new(PgRelayStore::new(Arc::clone(&db.pool)));
    let pairing = open_pairing(&store, &alice, Duration::minutes(5)).await;
    let (first, second) = (claim(&pairing, 2, 10), claim(&pairing, 3, 10));
    let (a, b) = tokio::join!(store.claim_pairing(&first), store.claim_pairing(&second));
    let outcomes = [a.unwrap(), b.unwrap()];
    assert_eq!(
        outcomes
            .iter()
            .filter(|o| matches!(o, ClaimOutcome::Claimed { .. }))
            .count(),
        1
    );
    assert!(outcomes.contains(&ClaimOutcome::Used));
    assert_eq!(store.list_devices(&alice).await.unwrap().len(), 1);
    assert_eq!(
        store.count_open_pairings(&alice, Utc::now()).await.unwrap(),
        0
    );
    db.cleanup().await;
}

#[tokio::test]
async fn expired_and_unknown_pairings_are_not_claimable() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let store = PgRelayStore::new(Arc::clone(&db.pool));
    let expired = open_pairing(&store, &alice, Duration::seconds(-1)).await;
    assert_eq!(
        store.claim_pairing(&claim(&expired, 2, 10)).await.unwrap(),
        ClaimOutcome::Expired
    );
    let (_, unknown) = PairingToken::mint();
    assert_eq!(
        store.claim_pairing(&claim(&unknown, 2, 10)).await.unwrap(),
        ClaimOutcome::Unknown
    );
    let live = open_pairing(&store, &alice, Duration::minutes(5)).await;
    let outcome = store.claim_pairing(&claim(&live, 2, 10)).await.unwrap();
    let ClaimOutcome::Claimed { device, .. } = outcome else {
        panic!("claimed");
    };
    assert_eq!(device.user, alice);
    db.cleanup().await;
}

#[tokio::test]
async fn open_pairings_are_counted_and_inserted_atomically() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let store = Arc::new(PgRelayStore::new(Arc::clone(&db.pool)));
    let (first, _) = pairing(&alice, Duration::minutes(5));
    let (second, _) = pairing(&alice, Duration::minutes(5));
    let (a, b) = tokio::join!(
        store.insert_pairing(&first, 1),
        store.insert_pairing(&second, 1)
    );
    let mut outcomes = [a.unwrap(), b.unwrap()];
    outcomes.sort_by_key(|o| *o == PairingOutcome::TooMany);
    assert_eq!(
        outcomes,
        [PairingOutcome::Inserted, PairingOutcome::TooMany]
    );
    let (ghost, _) = pairing(&user("ghost"), Duration::minutes(5));
    assert_eq!(
        store.insert_pairing(&ghost, 1).await.unwrap(),
        PairingOutcome::UnknownUser
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_refused_claim_leaves_the_pairing_claimable() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let store = PgRelayStore::new(Arc::clone(&db.pool));
    let first = open_pairing(&store, &alice, Duration::minutes(5)).await;
    assert!(matches!(
        store.claim_pairing(&claim(&first, 2, 1)).await.unwrap(),
        ClaimOutcome::Claimed { .. }
    ));
    let second = open_pairing(&store, &alice, Duration::minutes(5)).await;
    assert_eq!(
        store.claim_pairing(&claim(&second, 3, 1)).await.unwrap(),
        ClaimOutcome::DeviceLimit
    );
    assert_eq!(
        store.claim_pairing(&claim(&second, 2, 10)).await.unwrap(),
        ClaimOutcome::PeerTaken
    );
    assert!(matches!(
        store.claim_pairing(&claim(&second, 3, 10)).await.unwrap(),
        ClaimOutcome::Claimed { .. }
    ));
    db.cleanup().await;
}

#[tokio::test]
async fn a_credential_finds_its_phone_until_the_phone_is_revoked() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let mallory = seed_user(&db.pool, "mallory").await;
    let store = PgRelayStore::new(Arc::clone(&db.pool));
    let pairing = open_pairing(&store, &alice, Duration::minutes(5)).await;
    let request = claim(&pairing, 2, 10);
    let ClaimOutcome::Claimed { device: phone, .. } = store.claim_pairing(&request).await.unwrap()
    else {
        panic!("claimed");
    };
    let found = store
        .find_phone_by_credential(&request.credential)
        .await
        .unwrap();
    assert_eq!(found.map(|d| d.id), Some(phone.id));
    let before = Utc::now() - Duration::seconds(1);
    assert_eq!(
        revoke(&store, &mallory, phone.id).await.unwrap(),
        RevokeOutcome::NotFound
    );
    assert_eq!(
        revoke(&store, &alice, phone.id).await.unwrap(),
        RevokeOutcome::Revoked
    );
    assert_eq!(
        revoke(&store, &alice, phone.id).await.unwrap(),
        RevokeOutcome::NotFound
    );
    let registration = DesktopRegistration {
        user: alice.clone(),
        peer: peer(1),
        now: Utc::now(),
        max_devices: 10,
    };
    let RegisterOutcome::Registered(desktop) = store.register_desktop(&registration).await.unwrap()
    else {
        panic!("registered");
    };
    assert_eq!(
        revoke(&store, &alice, desktop.id).await.unwrap(),
        RevokeOutcome::Desktop
    );
    assert_eq!(
        store
            .find_phone_by_credential(&request.credential)
            .await
            .unwrap(),
        None
    );
    let revoked = store.list_revoked_since(before).await.unwrap();
    assert_eq!(
        revoked.iter().map(|r| r.device).collect::<Vec<_>>(),
        vec![phone.id]
    );
    let live = store.list_devices(&alice).await.unwrap();
    assert_eq!(
        live.iter().map(|d| d.kind).collect::<Vec<_>>(),
        vec![PeerKind::Desktop]
    );
    store.touch_device(phone.id, Utc::now()).await.unwrap();
    db.cleanup().await;
}

#[tokio::test]
async fn expired_pairings_are_purged() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let store = PgRelayStore::new(Arc::clone(&db.pool));
    open_pairing(&store, &alice, Duration::days(-2)).await;
    open_pairing(&store, &alice, Duration::minutes(5)).await;
    assert_eq!(
        store
            .delete_expired_pairings(Utc::now() - Duration::days(1))
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        store.count_open_pairings(&alice, Utc::now()).await.unwrap(),
        1
    );
    db.cleanup().await;
}
