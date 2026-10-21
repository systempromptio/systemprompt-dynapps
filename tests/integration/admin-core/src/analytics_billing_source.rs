//! `ai_requests.billing_source` in the spend figures: a request billed to a
//! member's own subscription key (`user_key`) is never platform spend. Both the
//! site KPI strip and the requests page's billing split report it apart, as
//! an estimate.

use chrono::{Duration, Utc};
use sqlx::PgPool;
use systemprompt_web_admin_analytics::repositories::analytics::requests::{
    BillingSplit, RequestFilter, find_request_billing_split,
};
use systemprompt_web_admin_analytics::repositories::analytics::site::SiteScope;
use systemprompt_web_admin_analytics::repositories::analytics::site::kpis::get_site_kpis;
use systemprompt_web_admin_core::repositories::scope::{Attribution, SubjectScope};
use systemprompt_web_admin_core::util::time_range::{TimeRange, TimeRangePreset};

use crate::fixtures::{
    RequestSpec, insert_request, insert_user, narrow_window, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

async fn seed(pool: &PgPool, user: &systemprompt::identifiers::UserId) {
    let mut platform = RequestSpec::completed(&unique("req"), user);
    platform.created_at = Utc::now() - Duration::seconds(5);
    platform.cost_microdollars = 700;
    insert_request(pool, &platform).await;

    let member_id = unique("req");
    let mut member = RequestSpec::completed(&member_id, user);
    member.created_at = Utc::now() - Duration::seconds(5);
    member.cost_microdollars = 300;
    insert_request(pool, &member).await;
    sqlx::query("UPDATE ai_requests SET billing_source = 'user_key' WHERE id = $1")
        .bind(&member_id)
        .execute(pool)
        .await
        .expect("mark the member-key request");
}

#[tokio::test]
async fn get_site_kpis_keeps_member_key_cost_out_of_platform_spend() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("billing")).await;
    seed(&db.pool, &user).await;
    let scope = SiteScope {
        user_id: Some(user.clone()),
        scope: SubjectScope::All,
        attribution: Attribution::Exclusive,
    };
    let now = Utc::now();
    let range = TimeRange {
        from: now - Duration::hours(1),
        to: now + Duration::minutes(1),
        preset: TimeRangePreset::Custom,
        rejected_bounds: false,
    };
    let kpis = get_site_kpis(&db.pool, range, &scope)
        .await
        .expect("kpi query succeeds");

    assert_eq!(kpis.total_requests, 2, "both requests are traffic");
    assert_eq!(kpis.total_cost_microdollars, 700, "platform spend only");
    assert_eq!(kpis.user_key_cost_microdollars, 300);
    assert_eq!(kpis.user_key_requests, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn find_request_billing_split_counts_only_member_key_requests() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("billing")).await;
    seed(&db.pool, &user).await;
    let filter = RequestFilter {
        user_id: Some(user.clone()),
        ..RequestFilter::default()
    };
    let split = find_request_billing_split(&db.pool, &filter, narrow_window())
        .await
        .expect("split query succeeds");

    assert_eq!(
        split,
        Some(BillingSplit {
            user_key_cost_microdollars: 300,
            user_key_requests: 1,
        })
    );
    db.cleanup().await;
}

#[tokio::test]
async fn find_request_billing_split_declines_ledger_filters() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let filter = RequestFilter {
        tool: Some("Bash".to_owned()),
        ..RequestFilter::default()
    };
    let split = find_request_billing_split(&db.pool, &filter, narrow_window())
        .await
        .expect("split query succeeds");

    assert_eq!(split, None, "a tool filter needs the ledger, so no split");
    db.cleanup().await;
}
