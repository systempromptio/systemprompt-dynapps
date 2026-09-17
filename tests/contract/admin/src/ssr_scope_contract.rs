//! The scope and credential pages, driven with rows behind every tab.
//!
//! [`crate::ssr_deep_contract`] pins the activity-trail pages. The pages here
//! are keyed on a *scope* — a group, a project, one account — or on the
//! credentials an account holds, and each renders only the tab it was asked
//! for. A tab whose loader has never run against a row is a tab whose joins
//! and per-row partials have never been executed at all, so every tab is
//! driven once with the seeded record present and once with an id that
//! matches nothing.

use axum::http::StatusCode;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

// The group the non-console principals are provisioned into; see
// `principal::provision`.
const CONTRACT_GROUP: &str = "contract-group";

struct Scope {
    project_id: String,
    project_name: String,
    member: UserId,
    pat_name: String,
    cert_label: String,
}

async fn seed_scope(pool: &PgPool, member: UserId) -> Scope {
    let project_id = seed::unique("scope-project");
    let project_name = format!("Scope project {project_id}");
    sqlx::query("INSERT INTO projects (id, name, description) VALUES ($1, $2, 'Seeded by the contract suite')")
        .bind(&project_id)
        .bind(&project_name)
        .execute(pool)
        .await
        .expect("insert project");
    sqlx::query(
        "INSERT INTO project_members (project_id, user_id, source) VALUES ($1, $2, 'manual')",
    )
    .bind(&project_id)
    .bind(member.as_str())
    .execute(pool)
    .await
    .expect("insert project member");
    sqlx::query("INSERT INTO project_ad_mappings (ad_group, project_id) VALUES ($1, $2)")
        .bind(format!("AD-{project_id}"))
        .bind(&project_id)
        .execute(pool)
        .await
        .expect("insert project mapping");
    sqlx::query("INSERT INTO group_ad_mappings (ad_group, group_id) VALUES ($1, $2)")
        .bind(format!("AD-{CONTRACT_GROUP}"))
        .bind(CONTRACT_GROUP)
        .execute(pool)
        .await
        .expect("insert group mapping");

    let pat_name = seed::unique("scope-pat");
    sqlx::query(
        "INSERT INTO user_api_keys (id, user_id, name, key_prefix, key_hash)
         VALUES ($1, $2, $3, 'sp_pat_ct', 'not-a-real-hash')",
    )
    .bind(seed::unique("scope-key"))
    .bind(member.as_str())
    .bind(&pat_name)
    .execute(pool)
    .await
    .expect("insert api key");

    let cert_label = seed::unique("scope-cert");
    sqlx::query(
        "INSERT INTO user_device_certs (id, user_id, fingerprint, label)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(seed::unique("scope-cert-id"))
    .bind(member.as_str())
    .bind(seed::unique("fp"))
    .bind(&cert_label)
    .execute(pool)
    .await
    .expect("insert device cert");

    sqlx::query(
        "INSERT INTO bridge_exchange_codes (code_hash, user_id, expires_at)
         VALUES ($1, $2, NOW() + INTERVAL '10 minutes')",
    )
    .bind(seed::unique("scope-link"))
    .bind(member.as_str())
    .execute(pool)
    .await
    .expect("insert pending link");

    // One request attributed to the member so the usage tabs have a rollup
    // and the project's activity queries join a real row.
    let session_id = seed::unique("scope-session");
    seed::insert_session(pool, &session_id, &member).await;
    seed::insert_request(
        pool,
        &seed::RequestSpec {
            id: seed::unique("scope-request"),
            user_id: &member,
            session_id: Some(&session_id),
            trace_id: None,
            context_id: None,
            status: "completed",
        },
    )
    .await;

    Scope {
        project_id,
        project_name,
        member,
        pat_name,
        cert_label,
    }
}

// Every tab of the group, project, account and devices pages renders the
// seeded record; a scope id in no table is a 404 that says so.
#[tokio::test(flavor = "multi_thread")]
async fn scope_pages_render_the_seeded_record_on_every_tab_and_miss_cleanly() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping seeded scope suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let member = credentials.non_admin_user_id.clone();
    let app = App::new(&db.pool, credentials);
    let scope = seed_scope(&db.pool, member).await;
    let member = scope.member.as_str();

    // (label, path, a substring the rendered page must carry)
    let mut found: Vec<(String, String, String)> = Vec::new();
    for tab in [
        "usage",
        "members",
        "marketplaces",
        "access",
        "projects",
        "mappings",
    ] {
        found.push((
            format!("the group page, {tab} tab"),
            format!("/admin/groups/{CONTRACT_GROUP}?tab={tab}"),
            "Contract group".to_owned(),
        ));
    }
    found.push((
        "the group members tab lists the provisioned member".to_owned(),
        format!("/admin/groups/{CONTRACT_GROUP}?tab=members"),
        member.to_owned(),
    ));
    found.push((
        "the group mappings tab lists the directory mapping".to_owned(),
        format!("/admin/groups/{CONTRACT_GROUP}?tab=mappings"),
        format!("AD-{CONTRACT_GROUP}"),
    ));
    found.push((
        "the unassigned bucket, members tab".to_owned(),
        "/admin/groups/unassigned?tab=members".to_owned(),
        "unassigned".to_owned(),
    ));
    for tab in ["members", "usage", "settings"] {
        found.push((
            format!("the project page, {tab} tab"),
            format!("/admin/projects/{}?tab={tab}", scope.project_id),
            scope.project_name.clone(),
        ));
    }
    found.push((
        "the project members tab lists the seeded member".to_owned(),
        format!("/admin/projects/{}?tab=members", scope.project_id),
        member.to_owned(),
    ));
    found.push((
        "the project settings tab lists the directory mapping".to_owned(),
        format!("/admin/projects/{}?tab=settings", scope.project_id),
        format!("AD-{}", scope.project_id),
    ));
    for tab in [
        "identity",
        "membership",
        "access",
        "devices",
        "sessions",
        "usage",
        "conversations",
    ] {
        found.push((
            format!("the account page, {tab} tab"),
            format!("/admin/users/{member}?tab={tab}"),
            member.to_owned(),
        ));
    }
    found.push((
        "the account devices tab lists the seeded token".to_owned(),
        format!("/admin/users/{member}?tab=devices"),
        scope.pat_name.clone(),
    ));
    found.push((
        "the account membership tab names the seeded project".to_owned(),
        format!("/admin/users/{member}?tab=membership"),
        scope.project_name.clone(),
    ));
    found.push((
        "the devices page, bridges tab".to_owned(),
        "/admin/devices".to_owned(),
        "Devices".to_owned(),
    ));
    found.push((
        "the devices page, tokens tab".to_owned(),
        "/admin/devices?tab=pats".to_owned(),
        scope.pat_name.clone(),
    ));
    found.push((
        "the devices page, tokens tab filtered to active".to_owned(),
        "/admin/devices?tab=pats&state=active".to_owned(),
        scope.pat_name.clone(),
    ));
    found.push((
        "the devices page, certificates tab".to_owned(),
        "/admin/devices?tab=certs".to_owned(),
        scope.cert_label.clone(),
    ));
    found.push((
        "the devices page, pending links tab".to_owned(),
        "/admin/devices?tab=links&sort=expires&dir=asc".to_owned(),
        member.to_owned(),
    ));

    let mut failures = Vec::new();
    for (label, path, marker) in &found {
        let (status, body) = app.call(Call::get(path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  {label} -> {} (expected 200): {}",
                status.as_u16(),
                body.chars().take(240).collect::<String>()
            ));
            continue;
        }
        if !body.contains(marker.as_str()) {
            failures.push(format!(
                "  {label} -> 200 but the body never carried {marker:?} — the page rendered \
                 without the record it was asked for"
            ));
        }
    }

    let missing: [(&str, String); 3] = [
        (
            "a group id in no table",
            "/admin/groups/no-such-group".to_owned(),
        ),
        (
            "a project id in no table",
            "/admin/projects/no-such-project?tab=usage".to_owned(),
        ),
        (
            "an account id in no table",
            "/admin/users/no-such-user?tab=access".to_owned(),
        ),
    ];
    for (label, path) in missing {
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::NOT_FOUND {
            failures.push(format!(
                "  {label} -> {} (expected 404): {}",
                status.as_u16(),
                body.chars().take(200).collect::<String>()
            ));
        }
    }

    // A non-admin is refused the group and project editors: they carry other
    // people's memberships and directory mappings.
    for path in [
        format!("/admin/groups/{CONTRACT_GROUP}?tab=members"),
        format!("/admin/projects/{}?tab=settings", scope.project_id),
    ] {
        let (status, _) = app.call(Call::get(&path, Principal::NonAdmin)).await;
        if !(status == StatusCode::FORBIDDEN || status.is_redirection()) {
            failures.push(format!(
                "  {path} as a non-admin -> {} (expected a refusal)",
                status.as_u16()
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} seeded scope-page case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
