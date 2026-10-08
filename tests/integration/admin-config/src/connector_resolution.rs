//! End-to-end broker rotation and recovery against a single-use OAuth issuer.

use crate::connector_resolution_fixture::Harness;

#[tokio::test]
async fn concurrent_broker_requests_refresh_once_and_use_the_committed_replacement() {
    let Some(h) = Harness::create_or_skip("rotation").await else {
        return;
    };
    let results: [_; 3] = tokio::join!(h.resolve(false), h.resolve(false), h.resolve(false)).into();
    for (result, _) in results {
        assert_eq!(result.unwrap(), "Bearer access-token");
    }
    assert_eq!(h.requests().await, 1);
    assert_eq!(
        h.grant().await.unwrap().refresh_token.as_deref(),
        Some("rotated-refresh-token")
    );
    assert_eq!(h.account().await.status, "connected");
    h.db.cleanup().await;
}

#[tokio::test]
async fn rotation_survives_a_later_verification_failure() {
    let Some(h) = Harness::create_or_skip("verification-failure").await else {
        return;
    };
    let (result, report) = h.resolve(true).await;
    assert!(result.is_err());
    assert_eq!(
        report.steps.iter().find(|step| !step.ok).unwrap().step,
        "tools"
    );
    let grant = h.grant().await.unwrap();
    assert_eq!(
        grant.refresh_token.as_deref(),
        Some("rotated-refresh-token")
    );
    assert!(grant.expires_at > chrono::Utc::now().timestamp());
    assert_eq!(h.account().await.status, "temporarily_unavailable");
    assert!(h.resolve(true).await.0.is_err());
    assert_eq!(
        h.requests().await,
        1,
        "a verification retry must not replay the old refresh token"
    );
    h.db.cleanup().await;
}

#[tokio::test]
async fn shared_client_configuration_failure_keeps_user_credentials() {
    let Some(h) = Harness::create_or_skip("invalid-client").await else {
        return;
    };
    assert!(h.resolve(false).await.0.is_err());
    assert_eq!(
        h.grant().await.unwrap().refresh_token.as_deref(),
        Some("refresh-token")
    );
    let account = h.account().await;
    assert_eq!(
        account.error_code.as_deref(),
        Some("provider_configuration")
    );
    assert_eq!(account.generation, 0);
    assert!(h.resolve(false).await.0.is_err());
    assert_eq!(
        h.requests().await,
        1,
        "the outage hold prevents repeated refreshes"
    );
    h.db.cleanup().await;
}

#[tokio::test]
async fn a_lost_refresh_response_is_never_replayed_even_by_a_probe() {
    let Some(h) = Harness::create_or_skip("lost-response").await else {
        return;
    };
    assert!(h.resolve(false).await.0.is_err());
    assert_eq!(
        h.account().await.error_code.as_deref(),
        Some("refresh_outcome_unknown")
    );
    assert!(h.grant().await.is_some());
    for probe in [false, true, false] {
        assert!(h.resolve(probe).await.0.is_err());
    }
    assert_eq!(h.requests().await, 1);
    h.db.cleanup().await;
}
