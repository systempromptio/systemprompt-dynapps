//! The 0.73 data migration over a database the 0.72.1 release left.
//!
//! The 0.72.1 rung is restored and given the rows 0.72.1 wrote in its own
//! shape: personal keys stored under the legacy entry names (`opencode-go`
//! and `opencode-go-anthropic`), audit rows naming those entries, and a
//! console rule on a generated route id. The current installer upgrades it,
//! then `admin config migrate --data` moves the data through the committed
//! `services/ai/legacy_map.yaml`, exactly as a deploy does. Nothing a member
//! stored is lost, every key lands on the credential it now belongs to, audit
//! rows gain their credential and endpoint while `provider` keeps its
//! history, and the rule follows its route to the declared id.

use dynapps_test_common::{empty_db_or_skip, repo_path};
use sqlx::PgPool;
use systemprompt::loader::ConfigLoader;
use systemprompt::manifest::services::catalog::CatalogContext;
use systemprompt_cli::commands::admin::config::migrate::data::{apply_with, read_map};

use crate::upgrade::{ladder, restore_seed_and_upgrade};

const RELEASE: &str = "0.72.1";

const SEED: &str = r"
INSERT INTO user_provider_credentials (user_id, provider, ciphertext, nonce, label) VALUES
    ('seed-1', 'opencode-go', 'ct-1', 'nonce-1', 'Go'),
    ('seed-2', 'opencode-go-anthropic', 'ct-2', 'nonce-2', NULL),
    ('seed-3', 'opencode-go', 'ct-3a', 'nonce-3a', NULL),
    ('seed-3', 'opencode-go-anthropic', 'ct-3b', 'nonce-3b', NULL);
UPDATE ai_requests SET provider = 'opencode-go-anthropic'
 WHERE id IN (SELECT id FROM ai_requests ORDER BY id LIMIT 10);
UPDATE ai_requests SET provider = 'anthropic'
 WHERE id IN (SELECT id FROM ai_requests ORDER BY id OFFSET 10 LIMIT 10);
INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
VALUES ('gateway_route', 'opencode-go-route', true, 'yaml')
ON CONFLICT DO NOTHING;
INSERT INTO access_control_rules
    (entity_type, entity_id, rule_type, rule_value, access, justification, source)
VALUES ('gateway_route', 'opencode-go-route', 'group', 'uk', 'allow', 'go for uk', 'dashboard');
";

async fn keys(pool: &PgPool) -> Vec<(String, String, String, String)> {
    sqlx::query_as(
        "SELECT user_id, credential, ciphertext, nonce FROM user_provider_credentials \
         WHERE user_id IN ('seed-1', 'seed-2', 'seed-3') ORDER BY user_id, ciphertext",
    )
    .fetch_all(pool)
    .await
    .expect("personal keys")
}

async fn audit(pool: &PgPool, provider: &str) -> Vec<(Option<String>, Option<String>)> {
    sqlx::query_as(
        "SELECT credential, endpoint FROM ai_requests WHERE provider = $1 \
         GROUP BY credential, endpoint ORDER BY credential, endpoint",
    )
    .bind(provider)
    .fetch_all(pool)
    .await
    .expect("audit rows")
}

#[tokio::test]
async fn release_0_72_1_keys_audit_and_rules_move_to_the_catalog() {
    let rung = ladder()
        .into_iter()
        .find(|r| r.version == RELEASE)
        .expect("the 0.72.1 rung is on the ladder");
    let db = empty_db_or_skip!();
    restore_seed_and_upgrade(&db, &rung, SEED).await;

    let map = read_map(&repo_path("services")).expect("committed legacy map");
    let report = apply_with(&db.db_pool(), &map)
        .await
        .expect("migrate --data");
    assert_eq!(report.personal_keys_moved, 1, "{report:?}");
    assert_eq!(report.audit_rows_backfilled, 20, "{report:?}");
    assert_eq!(report.route_ids_rewritten, 1, "{report:?}");

    let owned = |user: &str, credential: &str, ct: &str, nonce: &str| {
        (user.to_owned(), credential.to_owned(), ct.to_owned(), nonce.to_owned())
    };
    assert_eq!(
        keys(&db.pool).await,
        vec![
            owned("seed-1", "opencode-go", "ct-1", "nonce-1"),
            owned("seed-2", "opencode-go", "ct-2", "nonce-2"),
            owned("seed-3", "opencode-go", "ct-3a", "nonce-3a"),
            owned("seed-3", "opencode-go-anthropic", "ct-3b", "nonce-3b"),
        ],
        "every sealed key is kept; a member holding both keeps the one already \
         on the credential and the other is left, not overwritten"
    );

    let catalog = ConfigLoader::load_from_path(&repo_path("services/config/config.yaml"))
        .expect("committed services tree loads")
        .resolve_catalog(&CatalogContext::default())
        .expect("committed catalog resolves");
    let go = catalog.credential("opencode-go").expect("opencode-go credential");
    assert!(go.is_personal(), "OpenCode Go is a member's own key");

    assert_eq!(
        audit(&db.pool, "opencode-go-anthropic").await,
        vec![(Some("opencode-go".into()), Some("opencode-go/messages".into()))]
    );
    assert_eq!(
        audit(&db.pool, "anthropic").await,
        vec![(Some("anthropic".into()), Some("anthropic/api".into()))]
    );

    let rules: Vec<(String, String)> = sqlx::query_as(
        "SELECT entity_id, source FROM access_control_rules \
         WHERE entity_type = 'gateway_route' AND rule_type = 'group' AND rule_value = 'uk'",
    )
    .fetch_all(&*db.pool)
    .await
    .expect("route rules");
    assert_eq!(rules.len(), 1, "{rules:?}");
    assert_eq!(rules[0].0, "star-bf70e4", "the rule follows its route");
    let closed = ["code:", "bundle:", "console:"];
    assert!(
        rules[0].1 == "migration" || closed.iter().any(|p| rules[0].1.starts_with(p)),
        "the moved rule carries a closed-vocabulary source: {rules:?}"
    );

    let again = apply_with(&db.db_pool(), &map)
        .await
        .expect("second migrate --data");
    assert_eq!(
        (
            again.personal_keys_moved,
            again.audit_rows_backfilled,
            again.route_ids_rewritten
        ),
        (0, 0, 0),
        "the data migration is idempotent"
    );
    db.cleanup().await;
}
