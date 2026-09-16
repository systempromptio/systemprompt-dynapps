//! The skill drill-down reads retained data only: an attributed invocation
//! reaches the totals, its resource row and its conversation row only through
//! the snapshot pipeline, and re-running that pipeline changes nothing.
use super::feedback_attribution::{attribute, authenticate_event, enrol_device, publish_skill};
use super::feedback_ingestion::{drain, seed};
use crate::tempdb::TempDb;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use systemprompt::analytics::feedback::FeedbackFactsRepository;
use systemprompt::analytics::snapshots::FeedbackSnapshotsRepository;
use systemprompt::identifiers::{AnalyticsWorkerId, UserId};
use systemprompt::marketplace::managed::ManagedRepository;
use systemprompt_web_admin::repositories::analysis::retained::{
    RetainedSkillRef, RetainedWindow, find_retained_resource, get_retained_totals,
    list_retained_conversations, list_retained_skill_conversations, list_retained_skills,
};

async fn snapshot(pool: &PgPool, owner: &UserId) {
    let snapshots =
        FeedbackSnapshotsRepository::new(pool.clone(), FeedbackFactsRepository::new(pool.clone()));
    let now = Utc::now();
    while snapshots
        .process(owner, &AnalyticsWorkerId::generate(), now)
        .await
        .unwrap()
        > 0
    {}
    snapshots.refresh(owner, &[], now).await.unwrap();
}

async fn shadow_rows(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM analytics_snapshot_shadow")
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn an_attributed_invocation_reaches_totals_resource_and_conversation_rows_idempotently() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let pool = db.pool.as_ref();
    seed(pool).await;
    sqlx::query(
        "UPDATE plugin_usage_events SET prompt_preview='/plugin:my-skill' WHERE id='event1'",
    )
    .execute(pool)
    .await
    .unwrap();
    // Why: the hook always names the plugin; the seed's bare rows do not, and
    // an invocation without a plugin has no skill identity to attribute.
    sqlx::query("UPDATE plugin_usage_events SET plugin_id='plugin'")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO service_sources(name,kind,content_hash,provenance) VALUES('bundle:kit','bundle','hash-of-kit','fetched')")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO service_owned_ids(kind,id,source,marketplace_id) VALUES('plugin','plugin','bundle:kit','kit-marketplace')")
        .execute(pool)
        .await
        .unwrap();
    let owner = UserId::new("publisher");
    let consumer = UserId::new("consumer");
    let repo = ManagedRepository::new(&db.db_pool()).expect("build the managed repository");
    let device = enrol_device(pool, &repo, &consumer).await;
    let published = publish_skill(&repo, &owner, "skill", "my_skill", b"# Skill").await;
    authenticate_event(pool, "event1", &consumer, &device, "native-session").await;
    drain(pool).await;
    attribute(&repo, &published, &consumer, &device, "native-session").await;
    drain(pool).await;
    let window = RetainedWindow {
        start: Utc::now() - Duration::days(2),
        end: Utc::now() + Duration::days(1),
    };
    let unsnapshotted = get_retained_totals(pool, &owner, window).await.unwrap();
    assert_eq!(
        (unsnapshotted.invocations, unsnapshotted.requests),
        (0, 0),
        "nothing is read before the snapshot pipeline drains the facts"
    );
    snapshot(pool, &owner).await;
    let totals = get_retained_totals(pool, &owner, window).await.unwrap();
    assert_eq!(totals.invocations, 2);
    assert_eq!(
        totals.attributed, 2,
        "every invocation the hook named a plugin for"
    );
    assert_eq!(
        totals.revision_verified, 1,
        "only the one proven against the published resource"
    );
    assert_eq!(totals.conversations, 1);
    assert_eq!(totals.requests, 2);
    assert_eq!(totals.failed, 1);
    assert_eq!(totals.tokens, 15);
    assert_eq!(totals.cost, 125);
    let (skills, total) = list_retained_skills(pool, &owner, window, 1).await.unwrap();
    assert_eq!(total, 2, "both skills the hook reported, managed or not");
    assert_eq!(skills.len(), 2);
    let row = skills
        .iter()
        .find(|r| r.skill == "plugin:my-skill")
        .expect("the managed skill's row");
    assert_eq!(row.plugin_id.as_str(), "plugin");
    assert_eq!(row.resource_id.as_ref(), Some(&published.resource));
    assert_eq!(row.resource_key.as_deref(), Some("my_skill"));
    assert_eq!(row.kind.as_deref(), Some("skill"));
    assert_eq!(row.source.as_deref(), Some("bundle:kit"));
    assert_eq!(row.source_hash.as_deref(), Some("hash-of-kit"));
    assert_eq!(
        row.marketplace_id.as_ref().map(|m| m.as_str()),
        Some("kit-marketplace")
    );
    assert_eq!(row.invocations, 1);
    assert_eq!(row.revision_verified, 1);
    assert_eq!(row.conversations, 1);
    assert_eq!(row.requests, 2);
    assert_eq!(row.failed, 1);
    assert_eq!(row.tokens, 15);
    assert_eq!(row.cost, 125);
    let other = skills
        .iter()
        .find(|r| r.skill == "plugin:other")
        .expect("the unmanaged skill's row");
    assert!(
        other.resource_id.is_none(),
        "no revision proof, still attributed"
    );
    assert_eq!(other.revision_verified, 0);
    assert_eq!((other.invocations, other.requests, other.cost), (1, 2, 125));
    let (by_skill, total) = list_retained_skill_conversations(
        pool,
        &owner,
        &RetainedSkillRef {
            plugin_id: systemprompt::identifiers::PluginId::new("plugin"),
            skill: "plugin:other".to_owned(),
        },
        window,
        1,
    )
    .await
    .unwrap();
    assert_eq!((by_skill.len(), total), (1, 1));
    assert_eq!(by_skill[0].requests, 2);
    let selected = find_retained_resource(pool, &owner, &published.resource)
        .await
        .unwrap()
        .expect("the bound resource resolves for its owner");
    assert_eq!(selected.resource_key, "my_skill");
    assert!(
        find_retained_resource(pool, &consumer, &published.resource)
            .await
            .unwrap()
            .is_none(),
        "resources resolve only for their owner"
    );
    let (conversations, total) =
        list_retained_conversations(pool, &owner, &published.resource, window, 1)
            .await
            .unwrap();
    assert_eq!(total, 1);
    assert_eq!(conversations.len(), 1);
    let conversation = &conversations[0];
    assert_eq!(conversation.session_id.as_str(), "native-session");
    assert_eq!(conversation.user_id, consumer);
    assert_eq!(conversation.host.as_deref(), Some("codex"));
    assert!(conversation.first_at <= conversation.last_at);
    assert_eq!(conversation.invocations, 1);
    assert_eq!(conversation.requests, 2);
    assert_eq!(conversation.failed, 1);
    assert_eq!(conversation.tokens, 15);
    assert_eq!(conversation.cost, 125);
    assert_eq!(conversation.assessment_status.as_deref(), Some("scored"));
    assert_eq!(conversation.score_millionths, Some(850_000));
    let (_, empty) = list_retained_conversations(pool, &owner, &published.resource, window, 2)
        .await
        .unwrap();
    assert_eq!(empty, 0, "a page past the end carries no total");
    let retained_before = shadow_rows(pool).await;
    drain(pool).await;
    snapshot(pool, &owner).await;
    assert_eq!(shadow_rows(pool).await, retained_before);
    let again = get_retained_totals(pool, &owner, window).await.unwrap();
    assert_eq!(
        (
            again.invocations,
            again.attributed,
            again.revision_verified,
            again.requests,
            again.cost
        ),
        (2, 2, 1, 2, 125)
    );
    let (skills, total) = list_retained_skills(pool, &owner, window, 1).await.unwrap();
    assert_eq!((skills.len(), total), (2, 2));
    assert!(skills.iter().all(|r| r.invocations == 1));
    let (conversations, total) =
        list_retained_conversations(pool, &owner, &published.resource, window, 1)
            .await
            .unwrap();
    assert_eq!((conversations.len(), total), (1, 1));
    assert_eq!(conversations[0].requests, 2);
    db.cleanup().await;
}
