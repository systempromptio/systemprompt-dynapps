//! `repositories::projects::crud` — project lifecycle is deliberately
//! independent from group lifecycle: there is no protected system project,
//! and deleting completed work removes its attribution membership.

use systemprompt_web_admin_core::repositories::scope::containers::list_project_options;
use systemprompt_web_admin_core::types::projects::{CreateProjectRequest, UpdateProjectRequest};
use systemprompt_web_admin_identity::repositories::projects::crud::{
    delete_project, find_project, insert_project, update_project,
};

use crate::fixtures::unique_project;
use crate::tempdb::TempDb;

fn create(id: systemprompt_web_shared::ProjectId) -> CreateProjectRequest {
    CreateProjectRequest {
        id,
        name: "Migration programme".to_owned(),
        description: Some("Tracks work during the tenant migration".to_owned()),
        cost_centre: Some("CC-4100".to_owned()),
        practice: Some("Commerce".to_owned()),
    }
}

#[tokio::test]
async fn a_created_project_is_readable_and_keeps_unnamed_fields_on_update() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let request = create(unique_project("project"));
    insert_project(&db.pool, &request, "dashboard")
        .await
        .expect("create project");

    let updated = update_project(
        &db.pool,
        &request.id,
        &UpdateProjectRequest {
            name: Some("Renamed programme".to_owned()),
            description: None,
            cost_centre: None,
            practice: None,
        },
    )
    .await
    .expect("rename project");
    assert_eq!(updated.name, "Renamed programme");
    assert_eq!(
        updated.description, request.description,
        "an omitted description means retain it, rather than erase it"
    );
    assert_eq!(
        updated.cost_centre.as_deref(),
        Some("CC-4100"),
        "an omitted cost centre is retained like the description"
    );
    assert_eq!(
        updated.practice.as_deref(),
        Some("Commerce"),
        "an omitted practice is retained like the description"
    );
    assert_eq!(
        find_project(&db.pool, &request.id)
            .await
            .expect("read project")
            .expect("created row")
            .source,
        "dashboard"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn duplicate_and_missing_project_mutations_return_caller_errors() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let request = create(unique_project("project"));
    insert_project(&db.pool, &request, "dashboard")
        .await
        .expect("initial creation");

    assert_eq!(
        insert_project(&db.pool, &request, "dashboard")
            .await
            .expect_err("same id is refused")
            .status()
            .as_u16(),
        409
    );
    let absent = unique_project("missing");
    assert_eq!(
        update_project(
            &db.pool,
            &absent,
            &UpdateProjectRequest {
                name: Some("Nobody".to_owned()),
                description: None,
                cost_centre: None,
                practice: None,
            },
        )
        .await
        .expect_err("missing update")
        .status()
        .as_u16(),
        404
    );
    assert_eq!(
        delete_project(&db.pool, &absent)
            .await
            .expect_err("missing delete")
            .status()
            .as_u16(),
        404
    );
    db.cleanup().await;
}

#[tokio::test]
async fn deleting_a_project_removes_it_from_the_listing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let request = create(unique_project("project"));
    insert_project(&db.pool, &request, "dashboard")
        .await
        .expect("create project");

    delete_project(&db.pool, &request.id)
        .await
        .expect("delete project");
    assert!(
        find_project(&db.pool, &request.id)
            .await
            .expect("read after delete")
            .is_none()
    );
    assert!(
        !list_project_options(&db.pool)
            .await
            .expect("list projects")
            .iter()
            .any(|project| project.id == request.id)
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_empty_cost_centre_clears_it_and_a_new_one_replaces_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let request = create(unique_project("project"));
    insert_project(&db.pool, &request, "dashboard")
        .await
        .expect("create project");

    let edit = |cost_centre: &str| UpdateProjectRequest {
        name: None,
        description: None,
        cost_centre: Some(cost_centre.to_owned()),
        practice: None,
    };
    let replaced = update_project(&db.pool, &request.id, &edit(" CC-4200 "))
        .await
        .expect("replace cost centre");
    assert_eq!(replaced.cost_centre.as_deref(), Some("CC-4200"));

    let cleared = update_project(&db.pool, &request.id, &edit(""))
        .await
        .expect("clear cost centre");
    assert_eq!(cleared.cost_centre, None);
    db.cleanup().await;
}

#[tokio::test]
async fn an_empty_practice_clears_it_and_a_new_one_replaces_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let request = create(unique_project("project"));
    insert_project(&db.pool, &request, "dashboard")
        .await
        .expect("create project");

    let edit = |practice: &str| UpdateProjectRequest {
        name: None,
        description: None,
        cost_centre: None,
        practice: Some(practice.to_owned()),
    };
    let replaced = update_project(&db.pool, &request.id, &edit(" Odoo "))
        .await
        .expect("replace practice");
    assert_eq!(replaced.practice.as_deref(), Some("Odoo"));
    assert_eq!(
        replaced.cost_centre.as_deref(),
        Some("CC-4100"),
        "editing the practice leaves the cost centre alone"
    );

    let cleared = update_project(&db.pool, &request.id, &edit(""))
        .await
        .expect("clear practice");
    assert_eq!(cleared.practice, None);
    db.cleanup().await;
}
