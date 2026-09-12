//! Identity boundaries and non-additive conversation spend.
use crate::fixtures::{ancient_window, insert_user, unique};
use crate::tempdb::TempDb;
use systemprompt_web_admin::repositories::analysis::{AnalysisFilter, get_totals, list_skills};

#[tokio::test]
async fn repeated_skill_signals_do_not_multiply_request_spend() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("owner");
    insert_user(&db.pool, &user).await;
    let session = unique("native");
    let request = unique("request");
    for skill in ["alpha", "alpha", "beta"] {
        sqlx::query("INSERT INTO plugin_usage_events(id,user_id,session_id,plugin_id,event_type,prompt_preview,dedup_key,created_at) VALUES($1,$2,$3,'test','UserPromptSubmit',$4,$1,'2001-03-15')")
            .bind(unique("event")).bind(&user).bind(&session).bind(format!("/test:{skill}")).execute(&*db.pool).await.expect("insert signal");
    }
    sqlx::query("INSERT INTO ai_requests(id,request_id,user_id,context_id,client_session_id,provider,model,actor_kind,actor_id,status,input_tokens,output_tokens,cache_read_tokens,cache_creation_tokens,cost_microdollars,created_at) VALUES($1,$1,$2,$3,$4,'test','test','user',$2,'completed',100,20,30,10,1000000,'2001-03-15')")
        .bind(&request).bind(&user).bind(unique("context")).bind(&session).execute(&*db.pool).await.expect("insert request");
    let (start, end) = ancient_window();
    let filter = AnalysisFilter {
        start,
        end,
        skill: None,
        user: Some(user),
        model: None,
    };
    let totals = get_totals(&db.pool, &filter).await.expect("totals");
    assert_eq!(totals.uses, 3);
    assert_eq!(totals.conversations, 1);
    assert_eq!(totals.requests, 1);
    assert_eq!(totals.cost, 1_000_000);
    assert_eq!(totals.tokens, 160);
    let skills = list_skills(&db.pool, &filter, 0).await.expect("skills");
    assert_eq!(skills.len(), 2);
    assert_eq!(skills[0].uses, 2);
    assert_eq!(skills[0].cost, 1_000_000);
}

#[tokio::test]
async fn ingestion_rejects_owner_changes_and_conflicting_retries() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let owner = unique("owner");
    let other = unique("other");
    let session = unique("session");
    insert_user(&db.pool, &owner).await;
    insert_user(&db.pool, &other).await;
    let sql = "INSERT INTO plugin_usage_events(id,user_id,session_id,plugin_id,event_type,dedup_key,metadata) VALUES($1,$2,$3,'test','SessionStart',$1,jsonb_build_object('_ingestion_digest',$4::text)) ON CONFLICT(dedup_key) WHERE dedup_key IS NOT NULL DO NOTHING";
    let event = unique("event");
    let write = |user: String, digest: String| {
        sqlx::query(sql)
            .bind(event.clone())
            .bind(user)
            .bind(session.clone())
            .bind(digest)
    };
    assert_eq!(
        write(owner.clone(), "one".into())
            .execute(&*db.pool)
            .await
            .expect("first delivery")
            .rows_affected(),
        1
    );
    assert_eq!(
        write(owner.clone(), "one".into())
            .execute(&*db.pool)
            .await
            .expect("same delivery")
            .rows_affected(),
        0
    );
    assert!(
        write(owner, "changed".into())
            .execute(&*db.pool)
            .await
            .is_err()
    );
    assert!(write(other, "one".into()).execute(&*db.pool).await.is_err());
    let first: i64 = sqlx::query_scalar("SELECT drain_ingestion_outbox(100)")
        .fetch_one(&*db.pool)
        .await
        .expect("drain");
    let second: i64 = sqlx::query_scalar("SELECT drain_ingestion_outbox(100)")
        .fetch_one(&*db.pool)
        .await
        .expect("redrain");
    assert!(first >= 1);
    assert_eq!(second, 0);
    let count: i64 =
        sqlx::query_scalar("SELECT total_events FROM plugin_session_summaries WHERE session_id=$1")
            .bind(session)
            .fetch_one(&*db.pool)
            .await
            .expect("summary");
    assert_eq!(count, 1);
}

#[tokio::test]
async fn ambiguous_tool_ids_do_not_guess_a_conversation() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let owner = unique("owner");
    insert_user(&db.pool, &owner).await;
    let request = unique("request");
    let tool = unique("tool");
    sqlx::query("INSERT INTO ai_requests(id,request_id,user_id,context_id,provider,model,actor_kind,actor_id,status,input_tokens,output_tokens,cost_microdollars,created_at) VALUES($1,$1,$2,$3,'test','test','user',$2,'completed',10,5,500000,'2001-03-15')")
        .bind(&request).bind(&owner).bind(unique("context")).execute(&*db.pool).await.expect("request");
    sqlx::query("INSERT INTO ai_request_tool_calls(request_id,ai_tool_call_id,tool_name,tool_input,sequence_number) VALUES($1,$2,'Skill','{}',1)")
        .bind(request).bind(&tool).execute(&*db.pool).await.expect("tool");
    let (start, end) = ancient_window();
    let filter = AnalysisFilter {
        start,
        end,
        skill: None,
        user: Some(owner.clone()),
        model: None,
    };
    for index in 0..2 {
        sqlx::query("INSERT INTO plugin_usage_events(id,user_id,session_id,plugin_id,event_type,tool_name,metadata,dedup_key,created_at) VALUES($1,$2,$3,'test','PostToolUse','Skill',$4,$1,'2001-03-15')")
            .bind(unique("event")).bind(&owner).bind(unique("session"))
            .bind(serde_json::json!({"tool_use_id":tool,"tool_input":{"skill":"test:alpha"}}))
            .execute(&*db.pool).await.expect("skill evidence");
        let totals = get_totals(&db.pool, &filter).await.expect("totals");
        assert_eq!(totals.conversations, index + 1);
        assert_eq!(totals.linked, if index == 0 { 1 } else { 0 });
        assert_eq!(totals.cost, if index == 0 { 500_000 } else { 0 });
    }
}
