//! The contract itself: drive every mounted route under every principal and
//! assert the properties that must hold regardless of which status a given
//! route happens to return.
//!
//! One test, not one per route, because the globals in [`crate::globals`] are
//! process-wide `OnceLock`s and the throwaway database is expensive to build;
//! a single pass keeps both to one setup and reports every violation at once
//! rather than the first.

use axum::http::StatusCode;

use crate::app::App;
use crate::principal::Principal;
use crate::route_source::{MountedRoute, mounted_routes};
use crate::tempdb::TempDb;
use crate::{baseline, documents_fixture, globals, principal};

// Routes known to answer 5xx today, each with the reason.
//
// This list exists so the no-5xx invariant can be enforced for everything
// else while the error model is still being adopted. It is checked in both
// directions — an entry whose route stops failing must be deleted — so it can
// only ever shrink.
const KNOWN_5XX: [(&str, &str); 1] = [(
    "POST /api/public/admin/sync/sources/refresh",
    "proxies to core's /api/v1/admin/services/refresh over the loopback listener; the \
     contract harness mounts the routers without a listener, so the by-design 502 for an \
     unreachable refresh is the only answer it can see",
)];

fn known_5xx(key: &str) -> bool {
    KNOWN_5XX.iter().any(|(route, _)| *route == key)
}

// Routes served before authentication, by design. Everything else must
// refuse an anonymous caller.
fn is_public(template: &str) -> bool {
    const PUBLIC: [&str; 1] = ["/admin/login"];
    PUBLIC.contains(&template)
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_routes_honour_their_http_contract() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping admin HTTP contract suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let store = documents_fixture::store(&db.pool);
    let owned = owned_document(&store, &credentials.knowledge_worker_user_id).await;
    let app = App::with_documents(
        &db.pool,
        credentials,
        documents_fixture::mount(&db.pool, &store),
    );

    let routes = mounted_routes();
    assert!(
        routes.len() >= 80,
        "only {} routes discovered — the route-source parser is missing declarations",
        routes.len()
    );

    let mut observed = Vec::with_capacity(routes.len());
    let mut violations = Vec::new();
    let mut still_failing: Vec<String> = Vec::new();

    for route in routes {
        let mut statuses = Vec::with_capacity(Principal::ALL.len());
        for principal in Principal::ALL {
            let (status, body) = app
                .send(&route.method, &route.request_path(), principal)
                .await;
            check(&route, principal, status, body.as_deref(), &mut violations);
            if status.is_server_error() {
                still_failing.push(route.key());
            }
            statuses.push((principal, status));
        }
        observed.push((route, statuses));
    }
    observed.push(owned_document_row(&app, &owned, &mut violations).await);

    db.cleanup().await;

    assert!(
        violations.is_empty(),
        "{} route(s) violate the HTTP contract:\n{}",
        violations.len(),
        violations.join("\n")
    );

    let stale: Vec<&str> = KNOWN_5XX
        .iter()
        .map(|(route, _)| *route)
        .filter(|route| !still_failing.iter().any(|k| k == route))
        .collect();
    assert!(
        stale.is_empty(),
        "KNOWN_5XX lists {} route(s) that no longer return 5xx — delete them so the list keeps \
         reflecting real defects:\n  {}",
        stale.len(),
        stale.join("\n  ")
    );

    baseline::assert_matches(&baseline::render(&observed));
}

// The invariants that hold for every route, whatever its baseline status.
fn check(
    route: &MountedRoute,
    principal: Principal,
    status: StatusCode,
    body: Option<&str>,
    violations: &mut Vec<String>,
) {
    let mut fail = |why: &str| {
        let detail = body.map_or_else(String::new, |b| format!("\n      body: {b}"));
        violations.push(format!(
            "  {} [{}] -> {} : {why}{detail}",
            route.key(),
            principal.label(),
            status.as_u16()
        ));
    };

    // A well-formed request must never produce a server error. This is the
    // property Category E must not regress, and the one that catches an
    // `unwrap_or_default()` rendering a page over a failed query.
    if status.is_server_error() && !known_5xx(&route.key()) {
        fail("well-formed requests must never produce a 5xx");
    }

    // An anonymous caller must be refused or redirected — never served.
    if principal == Principal::Anonymous && !is_public(&route.template) && status.is_success() {
        fail("anonymous callers must not be served a success response");
    }

    // An authenticated admin must never be turned away by the auth layers,
    // except on the platform tier: directory mappings are platform_admin-only
    // by design, so a plain admin is refused there and only there.
    if principal == Principal::Admin
        && !is_platform_only(&route.template)
        && matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN)
    {
        fail("an admin must not be rejected by authentication or authorisation");
    }

    // The platform admin holds every tier, so nothing may refuse them.
    if principal == Principal::PlatformAdmin
        && matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN)
    {
        fail("a platform admin must not be rejected by authentication or authorisation");
    }
}

// The routes mounted on the platform tier (routes/admin_groups.rs): the
// AD-group mappings that feed groups and projects from the directory.
fn is_platform_only(template: &str) -> bool {
    template.contains("/ad-mappings")
        || [
            "/admin/logs",
            "/admin/sql",
            "/admin/sql/query",
            "/admin/data/clear/preview",
            "/admin/data/clear/confirm",
        ]
        .contains(&template)
}

// The one Drive row a well-formed unknown id cannot show: a document that
// exists, owned by the knowledge worker and shared with nobody, so its owner
// opens the page and every other signed-in principal meets a 404.
const OWNED_DOCUMENT_TEMPLATE: &str = "/drive/{owned_document}";

async fn owned_document(
    store: &documents_fixture::Store,
    owner: &systemprompt::identifiers::UserId,
) -> systemprompt::identifiers::DocumentId {
    use systemprompt::files::documents::models::RevisionAttribution;
    use systemprompt::files::documents::service::{Author, CreateRequest, RevisionOrigin};
    use systemprompt::identifiers::{DocumentTemplateId, TraceId};
    use systemprompt::models::auth::{AuthenticatedUser, Permission};
    use systemprompt::models::documents::ActorKind;

    let caller = AuthenticatedUser::new(
        owner.clone(),
        owner.as_str().to_owned(),
        format!("{}@contract.test", owner.as_str()),
        vec![Permission::User],
    );
    let (document, _) = store
        .service
        .create_from_template(
            &caller,
            CreateRequest {
                template_id: DocumentTemplateId::new("dynapps-word"),
                folder: None,
                name: "contract-owned".to_owned(),
                params: systemprompt_provider_contracts::documents::TemplateParams::default(),
                ops: Vec::new(),
                author: Author {
                    actor_kind: ActorKind::User,
                    origin: RevisionOrigin::Api,
                    attribution: RevisionAttribution::default(),
                },
                message: None,
                content: None,
                media: systemprompt::files::documents::model::Media::new(),
            },
            &TraceId::generate(),
        )
        .await
        .expect("seed the knowledge worker's document");
    document.id
}

async fn owned_document_row(
    app: &App,
    document: &systemprompt::identifiers::DocumentId,
    violations: &mut Vec<String>,
) -> (MountedRoute, Vec<(Principal, StatusCode)>) {
    let route = MountedRoute {
        method: "get".to_owned(),
        template: OWNED_DOCUMENT_TEMPLATE.to_owned(),
    };
    let path = format!("/drive/{}", document.as_str());
    let mut statuses = Vec::with_capacity(Principal::ALL.len());
    for principal in Principal::ALL {
        let (status, body) = app.send("get", &path, principal).await;
        check(&route, principal, status, body.as_deref(), violations);
        statuses.push((principal, status));
    }
    (route, statuses)
}
