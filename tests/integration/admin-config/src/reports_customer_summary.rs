//! `repositories::reports::customer` — the header figures on a month, for
//! the whole instance or one project.

use systemprompt_web_admin_analytics::repositories::reports::customer::get_customer_month_summary;

use systemprompt_web_admin_core::repositories::scope::SubjectScope;

use crate::fixtures::{
    RequestSeed, ancient_window, at, insert_request, insert_user, project_scope, set_project,
    unique,
};
use crate::tempdb::TempDb;

// Two users on different projects, both of whom made a request inside the
// historical window, plus one request outside it.
async fn seed(pool: &sqlx::PgPool) {
    let heavy = unique("u-heavy");
    let light = unique("u-light");
    for user in [&heavy, &light] {
        insert_user(pool, user).await;
    }
    set_project(pool, &heavy, Some("commerce")).await;
    set_project(pool, &light, Some("core")).await;

    let mut seed = RequestSeed::new("r-heavy-1", &heavy, at(2001, 3, 5, 9));
    seed.input_tokens = 1_000;
    seed.output_tokens = 200;
    seed.cache_read_tokens = 50;
    seed.cost_microdollars = 900_000;
    insert_request(pool, &seed).await;

    let mut seed = RequestSeed::new("r-heavy-2", &heavy, at(2001, 3, 6, 9));
    seed.model = Some("claude-opus-5-5");
    seed.input_tokens = 500;
    seed.output_tokens = 100;
    seed.status = "failed";
    insert_request(pool, &seed).await;

    let mut seed = RequestSeed::new("r-light-1", &light, at(2001, 3, 7, 9));
    seed.input_tokens = 10;
    seed.output_tokens = 2;
    insert_request(pool, &seed).await;

    let mut seed = RequestSeed::new("r-outside", &light, at(2001, 5, 1, 9));
    seed.input_tokens = 999_999;
    insert_request(pool, &seed).await;
}

#[tokio::test]
async fn get_customer_month_summary_counts_only_requests_inside_the_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed(&db.pool).await;
    let (from, to) = ancient_window();

    let summary = get_customer_month_summary(&db.pool, &SubjectScope::All, from, to)
        .await
        .expect("summary");

    assert_eq!(summary.requests, 3);
    assert_eq!(summary.input_tokens, 1_510);
    assert_eq!(summary.output_tokens, 302);
    assert_eq!(summary.cache_read_tokens, 50);
    // Why: the billable total counts the cached slice -- 1,812 visible tokens
    // plus the 50 read from cache -- and is read from `tokens_used` rather
    // than re-summed here.
    assert_eq!(summary.total_tokens, 1_862);

    db.cleanup().await;
}

#[tokio::test]
async fn get_customer_month_summary_counts_non_success_statuses_as_errors() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed(&db.pool).await;
    let (from, to) = ancient_window();

    let summary = get_customer_month_summary(&db.pool, &SubjectScope::All, from, to)
        .await
        .expect("summary");

    assert_eq!(summary.error_count, 1, "one seeded request failed");
    assert_eq!(summary.active_users, 2);

    db.cleanup().await;
}

#[tokio::test]
async fn get_customer_month_summary_narrows_to_one_project() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed(&db.pool).await;
    let (from, to) = ancient_window();

    let summary = get_customer_month_summary(
        &db.pool,
        &project_scope(&db.pool, "commerce").await,
        from,
        to,
    )
    .await
    .expect("summary");

    assert_eq!(summary.requests, 2, "only the commerce user's requests");
    assert_eq!(summary.active_users, 1);
    assert_eq!(summary.error_count, 1);

    db.cleanup().await;
}

#[tokio::test]
async fn get_customer_month_summary_reports_zeroes_under_the_nothing_scope() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed(&db.pool).await;
    let (from, to) = ancient_window();

    let summary = get_customer_month_summary(&db.pool, &SubjectScope::Users(Vec::new()), from, to)
        .await
        .expect("summary");

    assert_eq!(summary.requests, 0);
    assert_eq!(summary.active_users, 0);

    db.cleanup().await;
}

#[tokio::test]
async fn get_customer_month_summary_reports_reasoning_without_re_summing_the_total() {
    let db = dynapps_test_common::db_or_skip!();
    let user = unique("u-think");
    insert_user(&db.pool, &user).await;

    // Why: a total no re-summation of input, output and cache could produce.
    // A reader that recomputes instead of aggregating `tokens_used` fails here.
    let mut seed = RequestSeed::new("r-think", &user, at(2001, 3, 5, 9));
    seed.input_tokens = 100;
    seed.output_tokens = 40;
    seed.cache_read_tokens = 70;
    seed.reasoning_tokens = 25;
    seed.tokens_used = Some(9_999);
    insert_request(&db.pool, &seed).await;

    let (from, to) = ancient_window();
    let summary = get_customer_month_summary(&db.pool, &SubjectScope::All, from, to)
        .await
        .expect("summary");

    assert_eq!(
        summary.total_tokens, 9_999,
        "the stored billable total is read as stored"
    );
    assert_eq!(summary.reasoning_tokens, 25);
    assert_eq!(summary.output_tokens, 40);
    assert!(
        summary.reasoning_tokens <= summary.output_tokens,
        "reasoning is billed inside output, never alongside it"
    );

    db.cleanup().await;
}
