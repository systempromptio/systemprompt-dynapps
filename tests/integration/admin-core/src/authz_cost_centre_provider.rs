//! The `cost_centre` subject dimension: a user holds the cost centres of the
//! projects they are a live member of, their primary project's first, so core
//! attributes a request to the primary's cost centre by default.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::governance::authz::SubjectAttributeProvider;
use systemprompt_web_admin_core::repositories::scope::defaults::set_scope_defaults;
use systemprompt_web_admin_identity::authz::cost_centre::{
    CostCentreAttributeProvider, cost_centre_rule_type,
};

use crate::fixtures::{
    insert_project, insert_project_member, insert_user, unclaimed_email, unique, unique_project,
};
use crate::tempdb::TempDb;

async fn set_cost_centre(pool: &PgPool, project: &str, cost_centre: &str) {
    sqlx::query("UPDATE projects SET cost_centre = $2 WHERE id = $1")
        .bind(project)
        .bind(cost_centre)
        .execute(pool)
        .await
        .expect("set cost centre");
}

#[tokio::test]
async fn the_primary_projects_cost_centre_leads_and_projects_without_one_add_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ccdim")).await;
    let first = unique_project("proj");
    let primary = unique_project("proj");
    let uncoded = unique_project("proj");
    for project in [&first, &primary, &uncoded] {
        insert_project(&db.pool, project, project.as_str()).await;
        insert_project_member(&db.pool, project, &user, "manual").await;
    }
    set_cost_centre(&db.pool, first.as_str(), "AA-100").await;
    set_cost_centre(&db.pool, primary.as_str(), "ZZ-900").await;
    set_scope_defaults(&db.pool, &user, None, Some(&primary))
        .await
        .expect("set defaults");
    let provider = CostCentreAttributeProvider::new(Arc::new((*db.pool).clone()));

    assert_eq!(provider.dimension().rule_type, cost_centre_rule_type());
    assert_eq!(cost_centre_rule_type().as_str(), "cost_centre");
    assert_eq!(
        provider
            .values_for(&user)
            .await
            .expect("cost centre lookup"),
        vec!["ZZ-900".to_owned(), "AA-100".to_owned()]
    );
    db.cleanup().await;
}
