//! The `practice` subject dimension: a user holds the practices of the
//! projects they are a live member of, their primary project's first, so core
//! attributes a request to the primary's practice by default.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::SubjectAttributeProvider;
use systemprompt_web_admin_core::repositories::scope::defaults::set_scope_defaults;
use systemprompt_web_admin_identity::authz::practice::{
    PracticeAttributeProvider, practice_rule_type,
};

use crate::fixtures::{
    insert_project, insert_project_member, insert_user, unclaimed_email, unique, unique_project,
};
use crate::tempdb::TempDb;

async fn set_practice(pool: &PgPool, project: &str, practice: &str) {
    sqlx::query("UPDATE projects SET practice = $2 WHERE id = $1")
        .bind(project)
        .bind(practice)
        .execute(pool)
        .await
        .expect("set practice");
}

#[tokio::test]
async fn the_primary_projects_practice_leads_and_projects_without_one_add_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("prdim")).await;
    let first = unique_project("proj");
    let primary = unique_project("proj");
    let uncoded = unique_project("proj");
    for project in [&first, &primary, &uncoded] {
        insert_project(&db.pool, project, project.as_str()).await;
        insert_project_member(&db.pool, project, &user, "manual").await;
    }
    set_practice(&db.pool, first.as_str(), "Commerce").await;
    set_practice(&db.pool, primary.as_str(), "Odoo").await;
    set_scope_defaults(&db.pool, &user, None, Some(&primary))
        .await
        .expect("set defaults");
    let provider = PracticeAttributeProvider::new(Arc::new((*db.pool).clone()));

    assert_eq!(provider.dimension().rule_type, practice_rule_type());
    assert_eq!(practice_rule_type().as_str(), "practice");
    assert_eq!(
        provider.values_for(&user).await.expect("practice lookup"),
        vec!["Odoo".to_owned(), "Commerce".to_owned()]
    );
    db.cleanup().await;
}
