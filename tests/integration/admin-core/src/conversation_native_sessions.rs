//! Native-session read models group threads without rewriting audit identity.

use systemprompt::identifiers::ContextId;
use systemprompt_web_admin_analytics::repositories::analytics::context_detail::find_context_header;

use crate::fixtures::{
    RequestSpec, insert_request, insert_user, new_context_id, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn native_sessions_group_threads_and_preserve_audit_contexts_and_user_isolation() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = insert_user(&db.pool, &unique("alice"), &unclaimed_email("alice")).await;
    let bob = insert_user(&db.pool, &unique("bob"), &unclaimed_email("bob")).await;
    let session = unique("native-session");
    let other_session = unique("other-session");
    let contexts: Vec<String> = (0..4).map(|_| new_context_id()).collect();
    let mut request_ids = Vec::new();
    for (index, (user, native_session)) in [
        (&alice, &session),
        (&alice, &session),
        (&bob, &session),
        (&alice, &other_session),
    ]
    .into_iter()
    .enumerate()
    {
        let request_id = unique("request");
        let mut request = RequestSpec::completed(&request_id, user);
        request.context_id = Some(&contexts[index]);
        request.client_session_id = Some(native_session);
        insert_request(&db.pool, &request).await;
        request_ids.push(request_id);
    }

    let raw_contexts: Vec<String> = sqlx::query_scalar(
        "SELECT context_id::text FROM ai_requests WHERE id = ANY($1) ORDER BY id",
    )
    .bind(&request_ids)
    .fetch_all(&*db.pool)
    .await
    .expect("read original audit contexts");
    let mut expected = contexts.clone();
    let mut actual = raw_contexts;
    expected.sort();
    actual.sort();
    assert_eq!(actual, expected);

    let canonical: String = sqlx::query_scalar("SELECT canonical_conversation_context($1)")
        .bind(&contexts[0])
        .fetch_one(&*db.pool)
        .await
        .expect("resolve canonical context");
    let grouped: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM conversation_requests WHERE context_id = $1")
            .bind(&canonical)
            .fetch_one(&*db.pool)
            .await
            .expect("count grouped threads");
    assert_eq!(grouped, 2);
    let distinct_groups: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT context_id) FROM conversation_requests WHERE id = ANY($1)",
    )
    .bind(&request_ids)
    .fetch_one(&*db.pool)
    .await
    .expect("count isolated sessions");
    assert_eq!(distinct_groups, 3);

    for original in &contexts[..2] {
        let header = find_context_header(
            &db.pool,
            &ContextId::try_new(original.clone()).expect("fixture context"),
        )
        .await
        .expect("resolve original link")
        .expect("native conversation header");
        assert_eq!(header.user_id.as_ref(), Some(&alice));
    }
    db.cleanup().await;
}

#[tokio::test]
async fn ambiguous_legacy_context_never_selects_an_arbitrary_native_session() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ambiguous")).await;
    let original = new_context_id();
    for _ in 0..2 {
        let session = unique("native-session");
        let mut request = RequestSpec::completed(&unique("request"), &user);
        request.context_id = Some(&original);
        request.client_session_id = Some(&session);
        insert_request(&db.pool, &request).await;
    }
    let resolved: String = sqlx::query_scalar("SELECT canonical_conversation_context($1)")
        .bind(&original)
        .fetch_one(&*db.pool)
        .await
        .expect("resolve ambiguous context");
    assert_eq!(resolved, original);
    let aliases: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM conversation_context_aliases WHERE original_context_id = $1",
    )
    .bind(&original)
    .fetch_one(&*db.pool)
    .await
    .expect("count distinct native sessions");
    assert_eq!(aliases, 2);
    db.cleanup().await;
}
