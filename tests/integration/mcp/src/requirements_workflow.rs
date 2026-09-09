use systemprompt::identifiers::UserId;
use systemprompt_mcp_knowledge_bank::{export, schema};
use systemprompt_requirements_workflow::model::{Evidence, Workflow};
use systemprompt_requirements_workflow::store;

#[tokio::test]
async fn requirements_workflow_compare_and_swap_rejects_stale_writers() {
    let db = astound_test_common::TempDb::create_empty()
        .await
        .expect("PostgreSQL required");
    let pool = &*db.pool;
    sqlx::raw_sql("CREATE TABLE users(id TEXT PRIMARY KEY,roles TEXT[]); CREATE TABLE projects(id TEXT PRIMARY KEY); CREATE TABLE project_members(user_id TEXT,project_id TEXT); CREATE TABLE group_members(user_id TEXT,group_id TEXT); INSERT INTO projects VALUES('storefront')").execute(pool).await.unwrap();
    let user = UserId::new("10000000-0000-4000-8000-000000000001");
    sqlx::query("INSERT INTO users VALUES($1,ARRAY['admin'])")
        .bind(user.as_str())
        .execute(pool)
        .await
        .unwrap();
    use systemprompt::security::authz::SubjectAttributeProvider;
    let memberships = systemprompt_requirements_workflow::authz::MembershipProvider::groups(
        std::sync::Arc::new(pool.clone()),
    );
    sqlx::query("INSERT INTO group_members VALUES($1,'europe-devs')")
        .bind(user.as_str())
        .execute(pool)
        .await
        .unwrap();
    assert_eq!(memberships.values_for(&user).await, vec!["europe-devs"]);
    sqlx::query("DELETE FROM group_members WHERE user_id=$1")
        .bind(user.as_str())
        .execute(pool)
        .await
        .unwrap();
    assert!(
        memberships.values_for(&user).await.is_empty(),
        "Membership revocation must affect the next decision"
    );
    store::migrate(pool).await.unwrap();
    store::migrate(pool).await.unwrap();
    let workflow = Workflow::new(
        "linen-polos".into(),
        "Preserve category browsing behavior".into(),
        vec![Evidence {
            source_ref: "EAICOMM-1".into(),
            source_type: "jira".into(),
            uri: "https://example.test/1".into(),
            title: "Baseline".into(),
            body: "requirement".into(),
            hash: "test".into(),
            indexed_hash: "test".into(),
            retrieval_method: "fixture".into(),
            version: "1".into(),
        }],
    )
    .unwrap();
    let stored = store::create(pool, &user, "storefront", workflow)
        .await
        .unwrap();
    let (a, b) = tokio::join!(
        store::save(pool, &user, 0, &stored, "writer_a"),
        store::save(pool, &user, 0, &stored, "writer_b")
    );
    assert_ne!(
        a.is_ok(),
        b.is_ok(),
        "Only one concurrent revision may commit"
    );
    let current = store::get(pool, &user, stored.workflow.id).await.unwrap();
    assert_eq!(current.revision, 1);
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM requirements_workflow_events")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 2);
    let mut submitted = current.clone();
    submitted
        .workflow
        .submit(
            systemprompt_requirements_workflow::model::Phase::Context,
            std::collections::BTreeMap::from([("context.md".to_owned(), "# Draft".to_owned())]),
        )
        .unwrap();
    submitted.revision = store::save(pool, &user, current.revision, &submitted, "submitted")
        .await
        .unwrap();
    let result = systemprompt_requirements_workflow::validation::validate(
        &submitted.workflow,
        systemprompt_requirements_workflow::model::Phase::Context,
    )
    .unwrap();
    assert!(
        !result.passed,
        "An uncited short draft must fail validation"
    );
    let stranger = UserId::new("10000000-0000-4000-8000-000000000002");
    assert!(
        store::get(pool, &stranger, stored.workflow.id)
            .await
            .is_err()
    );
    db.cleanup().await;
}

#[tokio::test]
async fn requirements_workflow_export_requires_current_content_and_caller_grant() {
    let db = astound_test_common::TempDb::create_empty()
        .await
        .expect("PostgreSQL required");
    let pool = &*db.pool;
    schema::apply(pool).await.unwrap();
    sqlx::raw_sql("INSERT INTO knowledge_documents(doc_id,source_type,source,source_ref,title,uri,content_hash) VALUES('jira:ONE','jira','jira','ONE','One','https://example.test/one','h1'),('jira:TWO','jira','jira','TWO','Two','https://example.test/two','h2'); INSERT INTO knowledge_source_text VALUES('jira:ONE','Complete first source','h1'),('jira:TWO','Other reader source','h2')").execute(pool).await.unwrap();
    let user = UserId::new("10000000-0000-4000-8000-000000000001");
    sqlx::query("INSERT INTO knowledge_document_grants(doc_id,subject_kind,subject_value) VALUES('jira:ONE','user',$1)").bind(user.as_str()).execute(pool).await.unwrap();
    let rows = export::export_sources(pool, &user, "", 1).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].body, "Complete first source");
    assert!(
        export::export_sources(pool, &user, "jira:ONE", 100)
            .await
            .unwrap()
            .is_empty()
    );
    sqlx::query("UPDATE knowledge_documents SET content_hash='changed' WHERE doc_id='jira:ONE'")
        .execute(pool)
        .await
        .unwrap();
    assert!(
        export::export_sources(pool, &user, "", 100)
            .await
            .unwrap()
            .is_empty(),
        "Stale raw source must not be exported with a new hash"
    );
    db.cleanup().await;
}
