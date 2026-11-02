//! `PgRelayStore`'s mobile sign-in against a throwaway database with the
//! full extension schema: the grant's steps, single use under concurrency
//! (the losing redemption is a replay and revokes the winner's device),
//! the PKCE challenge, replay, replacement and inactive accounts.

use std::sync::Arc;

use chrono::{Duration, Utc};
use dynapps_test_common::db_or_skip;
use dynwork_relay::credential::{
    AuthorizationCode, ConsentRequest, CredentialHash, MobileCredential,
};
use dynwork_relay::ids::{DeviceId, GrantId};
use dynwork_relay::store::postgres::PgRelayStore;
use dynwork_relay::store::{
    CodeRedemption, DecisionOutcome, GrantDecision, GrantOutcome, MobileAuthStore, NewGrant,
    RedeemOutcome, RelayStore,
};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::support::{peer, user};

const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

async fn seed_user(pool: &PgPool, name: &str) -> UserId {
    sqlx::query("INSERT INTO users (id, name, email) VALUES ($1, $1, $2)")
        .bind(name)
        .bind(format!("{name}@example.test"))
        .execute(pool)
        .await
        .expect("seed a user");
    user(name)
}

// A grant opened and approved for `owner`; the code's hash.
async fn approved(store: &PgRelayStore, owner: &UserId) -> CredentialHash {
    let now = Utc::now();
    let (_, request) = ConsentRequest::mint();
    let grant = NewGrant {
        id: GrantId::generate(),
        user: owner.clone(),
        request: request.clone(),
        challenge: CHALLENGE.to_owned(),
        state: "s".repeat(43),
        device_name: "Pixel".to_owned(),
        now,
        expires_at: now + Duration::minutes(10),
        max_open: 10,
    };
    assert_eq!(
        store.insert_grant(&grant).await.unwrap(),
        GrantOutcome::Inserted
    );
    let (_, code) = AuthorizationCode::mint();
    let decision = GrantDecision {
        request,
        user: owner.clone(),
        code: Some((code.clone(), now + Duration::minutes(10))),
        now,
    };
    assert!(matches!(
        store.decide_grant(&decision).await.unwrap(),
        DecisionOutcome::Approved { .. }
    ));
    assert_eq!(
        store.decide_grant(&decision).await.unwrap(),
        DecisionOutcome::Decided
    );
    code
}

fn redemption(code: &CredentialHash, byte: u8, challenge: &str) -> CodeRedemption {
    CodeRedemption {
        code: code.clone(),
        challenge: challenge.to_owned(),
        device: DeviceId::generate(),
        peer: peer(byte),
        credential: MobileCredential::mint().1,
        now: Utc::now(),
        max_devices: 10,
    }
}

#[tokio::test]
async fn a_code_redeems_once_under_concurrency_and_its_replay_revokes_the_device() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let store = Arc::new(PgRelayStore::new(Arc::clone(&db.pool)));
    let code = approved(&store, &alice).await;
    let (first, second) = (
        redemption(&code, 1, CHALLENGE),
        redemption(&code, 2, CHALLENGE),
    );
    let (a, b) = tokio::join!(store.redeem_code(&first), store.redeem_code(&second));
    let outcomes = [a.unwrap(), b.unwrap()];
    let signed: Vec<_> = outcomes
        .iter()
        .filter_map(|outcome| match outcome {
            RedeemOutcome::SignedIn {
                device, account, ..
            } => Some((device.id, account.email.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(signed.len(), 1);
    assert_eq!(signed[0].1, "alice@example.test");
    assert!(outcomes.contains(&RedeemOutcome::Used {
        user: alice.clone(),
        revoked: Some(signed[0].0),
    }));
    assert!(store.list_devices(&alice).await.unwrap().is_empty());
    let replay = store
        .redeem_code(&redemption(&code, 3, CHALLENGE))
        .await
        .unwrap();
    assert_eq!(
        replay,
        RedeemOutcome::Used {
            user: alice.clone(),
            revoked: None
        }
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_wrong_challenge_burns_the_code_and_a_new_sign_in_replaces_the_peer() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let store = PgRelayStore::new(Arc::clone(&db.pool));
    let code = approved(&store, &alice).await;
    assert_eq!(
        store
            .redeem_code(&redemption(&code, 1, "x".repeat(43).as_str()))
            .await
            .unwrap(),
        RedeemOutcome::Mismatch
    );
    assert!(matches!(
        store
            .redeem_code(&redemption(&code, 1, CHALLENGE))
            .await
            .unwrap(),
        RedeemOutcome::Used { revoked: None, .. }
    ));
    let first = store
        .redeem_code(&redemption(&approved(&store, &alice).await, 1, CHALLENGE))
        .await
        .unwrap();
    let RedeemOutcome::SignedIn {
        device: old,
        replaced: None,
        ..
    } = first
    else {
        panic!("the first sign-in creates a device: {first:?}");
    };
    let second = store
        .redeem_code(&redemption(&approved(&store, &alice).await, 1, CHALLENGE))
        .await
        .unwrap();
    let RedeemOutcome::SignedIn {
        device: new,
        replaced,
        ..
    } = second
    else {
        panic!("the second sign-in creates a device: {second:?}");
    };
    assert_eq!(replaced, Some(old.id));
    let live = store.list_devices(&alice).await.unwrap();
    assert_eq!(
        live.iter().map(|device| device.id).collect::<Vec<_>>(),
        vec![new.id]
    );
    assert_eq!(
        store
            .redeem_code(&redemption(&AuthorizationCode::mint().1, 1, CHALLENGE))
            .await
            .unwrap(),
        RedeemOutcome::Unknown
    );
    db.cleanup().await;
}

#[tokio::test]
async fn grants_are_capped_bound_to_their_owner_and_purged() {
    let db = db_or_skip!();
    let alice = seed_user(&db.pool, "alice").await;
    let mallory = seed_user(&db.pool, "mallory").await;
    let store = PgRelayStore::new(Arc::clone(&db.pool));
    let now = Utc::now();
    let (_, request) = ConsentRequest::mint();
    let grant = |request: CredentialHash, max_open| NewGrant {
        id: GrantId::generate(),
        user: alice.clone(),
        request,
        challenge: CHALLENGE.to_owned(),
        state: "s".repeat(43),
        device_name: "Pixel".to_owned(),
        now,
        expires_at: now + Duration::minutes(10),
        max_open,
    };
    assert_eq!(
        store
            .insert_grant(&grant(request.clone(), 1))
            .await
            .unwrap(),
        GrantOutcome::Inserted
    );
    assert_eq!(
        store
            .insert_grant(&grant(ConsentRequest::mint().1, 1))
            .await
            .unwrap(),
        GrantOutcome::TooMany
    );
    let mut decision = GrantDecision {
        request,
        user: mallory,
        code: None,
        now,
    };
    assert_eq!(
        store.decide_grant(&decision).await.unwrap(),
        DecisionOutcome::Unknown
    );
    decision.user = alice.clone();
    assert_eq!(
        store.decide_grant(&decision).await.unwrap(),
        DecisionOutcome::Denied {
            state: "s".repeat(43)
        }
    );
    assert_eq!(
        store
            .delete_expired_grants(now + Duration::minutes(11))
            .await
            .unwrap(),
        1
    );
    assert!(store.find_account(&alice).await.unwrap().is_some());
    sqlx::query("UPDATE users SET status = 'suspended' WHERE id = $1")
        .bind(alice.as_str())
        .execute(&*db.pool)
        .await
        .expect("suspend alice");
    assert!(store.find_account(&alice).await.unwrap().is_none());
    db.cleanup().await;
}
