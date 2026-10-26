//! The shipped gateway policy under warn mode, through the real ingestion and
//! resolution path: `services/gateway/policies.yaml` is parsed by the loader
//! the `gateway_policies` sync plane uses, written to `ai_gateway_policies`
//! in a throwaway database by that plane's row writer, and read back through
//! the gateway's `PolicyResolver` — the merge that decides what the dispatch
//! path actually enforces. `safety.mode` has to come out the far side as `warn`
//! and so does `quota_mode`, or the file says one thing and the gateway does
//! another.
//!
//! The quota plane gets its own proof: the resolved windows are driven to
//! exhaustion with the real reservation code, which must report the breach
//! that warn mode records and enforce mode would turn into a refusal.

use std::sync::Arc;

use systemprompt::ai::repository::{AiGatewayPolicyRepository, AiQuotaBucketRepository};
use systemprompt::database::{Database, DbPool};
use systemprompt::gateway::policies::PolicyResolver;
use systemprompt::gateway::quota::{
    QuotaEstimate, QuotaSubjects, ReserveOutcome, ReserveParams, precheck_and_reserve,
};
use systemprompt::gateway::{GatewayPolicyConfig, QuotaWindow};
use systemprompt::governance::authz::{AuthzHookContext, NullAuditSink, SubjectProviderSet};
use systemprompt::identifiers::UserId;
use systemprompt::manifest::services::EnforcementMode;
use systemprompt::models::attribution::RequestAttribution;
use systemprompt_web_admin_governance::repositories::gateway_policies::rows::{
    PolicyWrite, upsert_policy,
};

use crate::fixtures::unique;
use crate::tempdb::TempDb;

fn shipped_policies() -> GatewayPolicyConfig {
    let path = dynapps_test_common::repo_path("services/gateway/policies.yaml");
    let yaml =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

fn db_pool(db: &TempDb) -> DbPool {
    Arc::new(Database::from_pools(
        Arc::clone(&db.pool),
        Some(Arc::clone(&db.pool)),
    ))
}

async fn ingest_shipped(db: &TempDb) -> DbPool {
    let pool = db_pool(db);
    let shipped = shipped_policies();
    shipped.validate().expect("the shipped file validates");
    assert!(
        !shipped.policies.is_empty(),
        "at least the default-quotas policy is declared"
    );
    for entry in &shipped.policies {
        upsert_policy(
            &db.pool,
            &PolicyWrite {
                name: &entry.name,
                spec: &entry.spec,
                enabled: entry.enabled,
                priority: entry.priority,
            },
        )
        .await
        .expect("the shipped policy ingests");
    }
    pool
}

#[test]
fn the_shipped_file_warns_on_quotas_and_safety() {
    let cfg = shipped_policies();
    cfg.validate().expect("the shipped file validates");
    let spec = &cfg
        .policies
        .iter()
        .find(|p| p.enabled)
        .expect("an enabled policy")
        .spec;
    assert_eq!(spec.quota_mode, EnforcementMode::Warn);
    assert_eq!(spec.safety.mode, EnforcementMode::Warn);
    assert!(
        !spec.quota_windows.is_empty(),
        "the windows stay declared so warn has thresholds to report"
    );
    assert!(
        !spec.safety.block_categories.is_empty()
            && !spec.safety.block_response_categories.is_empty(),
        "the block lists stay declared under warn for the same reason"
    );
}

#[tokio::test]
async fn the_resolver_keeps_both_mode_switches_after_ingestion() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = ingest_shipped(&db).await;

    let repo = AiGatewayPolicyRepository::new(&pool);
    let resolved = PolicyResolver::from_repository(
        repo,
        systemprompt::traits::CacheEpochs::new(),
        systemprompt::manifest::profile::UnappliedPlaneMode::Warn,
    )
    .resolve(systemprompt::manifest::services::QuotaFaultMode::default())
    .await
    .expect("policy row resolves");

    assert!(
        resolved.quota_mode.is_warn(),
        "quota_mode: warn survived the row merge"
    );
    assert!(
        resolved.safety.mode.is_warn(),
        "safety.mode survived the row merge; the scanners would otherwise refuse"
    );
    assert!(
        !resolved.quota_windows.is_empty() && !resolved.safety.scanners.is_empty(),
        "warn did not empty the windows or the scanner list: {resolved:?}"
    );

    db.cleanup().await;
}

// Why: the quota stage is `precheck_and_reserve`; under `quota_mode: warn`
// core records the breach instead of answering 429, and that warning must
// carry its reason, so the exhausted state is driven with the real windows
// and bucket table.
#[tokio::test]
async fn an_exhausted_window_is_detected_so_the_refusal_carries_a_reason() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = ingest_shipped(&db).await;
    let repo = AiGatewayPolicyRepository::new(&pool);
    let resolved = PolicyResolver::from_repository(
        repo,
        systemprompt::traits::CacheEpochs::new(),
        systemprompt::manifest::profile::UnappliedPlaneMode::Warn,
    )
    .resolve(systemprompt::manifest::services::QuotaFaultMode::default())
    .await
    .expect("policy row resolves");
    assert!(resolved.quota_mode.is_warn());

    let user_window = resolved
        .quota_windows
        .iter()
        .find(|w| w.subject == "user")
        .expect("the shipped per-user window");
    let max_requests = user_window
        .max_requests
        .expect("the per-user window caps requests");
    // Why: a one-request window on the same subject kind and seconds, so the
    // test exhausts a real user bucket in two calls rather than six hundred.
    let tiny = QuotaWindow {
        max_requests: Some(1),
        ..user_window.clone()
    };
    let buckets = AiQuotaBucketRepository::new(&pool);
    let user = UserId::new(unique("quota-user"));

    let providers = SubjectProviderSet::discover(&AuthzHookContext {
        pool: pool.pool(),
        sink: Arc::new(NullAuditSink),
        services: dynapps_test_common::empty_services_view(),
        subjects: systemprompt::models::authz::SubjectSourceHandle::default(),
        secrets: dynapps_test_common::fixture_secret_store(),
    });
    let attribution = RequestAttribution::none();
    let params = ReserveParams {
        providers: &providers,
        subjects: QuotaSubjects {
            user_id: &user,
            api_key_id: None,
            attribution: &attribution,
        },
        windows: std::slice::from_ref(&tiny),
        fault_mode: systemprompt::manifest::services::QuotaFaultMode::default(),
        estimate: QuotaEstimate::default(),
        sharding: systemprompt::manifest::profile::QuotaConfig::default(),
    };

    let first = precheck_and_reserve(&buckets, params)
        .await
        .expect("first reservation");
    assert!(
        matches!(first, ReserveOutcome::Admitted(_)),
        "the first request is within the window"
    );

    let ReserveOutcome::Denied {
        decision: second, ..
    } = precheck_and_reserve(&buckets, params)
        .await
        .expect("second reservation")
    else {
        panic!("the second request breaches a one-request window");
    };
    assert!(
        !second.allow,
        "the breach is detected exactly as under enforce"
    );
    // Why: core words this "<label> ceiling exceeded for <subject> window <n>s
    // (used x/y)" — `request`, `input token` or `output token` for the label —
    // so the assertion pins the subject and the ceiling clause rather than the
    // label, which varies with whichever limit the window tripped.
    assert!(
        second.message.contains("ceiling exceeded") && second.message.contains("user"),
        "the reason names the subject and the ceiling: {}",
        second.message
    );
    assert_eq!(second.window_seconds, user_window.window_seconds);
    assert!(
        max_requests > 1,
        "the shipped ceiling is a real backstop, not this test's 1"
    );

    db.cleanup().await;
}
