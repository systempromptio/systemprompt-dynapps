//! The shared document store as the runtime composes it, over a throwaway
//! database: the install's templates, its own group subject source and a
//! blob directory that lives as long as the store. The status contract and
//! docs-smoke mount the user-facing `/drive` pages and core's bridge Drive
//! feed over it.

use std::sync::Arc;

use systemprompt::database::{Database, DbPool};
use systemprompt::events::{EventBroadcasters, EventRouter};
use systemprompt::files::DocumentsConfig;
use systemprompt::files::documents::DocumentService;
use systemprompt::files::documents::templates::load_catalog;
use systemprompt::governance::authz::{AccessControlRepository, AllowAllHook};
use systemprompt::runtime::documents::{DocumentsRuntime, DocumentsRuntimeParts};
use systemprompt_web_admin_platform::routes::drive_user::DriveDocuments;

use crate::app::DocumentsMount;
use crate::globals;

pub(crate) struct Store {
    pub(crate) service: Arc<DocumentService>,
    _blobs: tempfile::TempDir,
}

pub(crate) fn store(pool: &Arc<sqlx::PgPool>) -> Store {
    let blobs = tempfile::tempdir().expect("blob directory");
    let database: DbPool = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let catalog = load_catalog(&dynapps_test_common::repo_path("services"))
        .expect("the install's document templates load");
    let runtime = DocumentsRuntime::new(DocumentsRuntimeParts {
        file_storage: Arc::new(systemprompt::storage::LocalFileStorage::new(
            blobs.path().to_path_buf(),
        )),
        access_entities: Arc::new(AccessControlRepository::new(&database)),
        authz_hook: Arc::new(AllowAllHook::null()),
        subjects: dynapps_test_common::subject_source(
            Arc::clone(pool),
            globals::services().clone(),
            Arc::clone(globals::boot().secrets()),
        ),
        event_router: EventRouter::local_only(Arc::new(EventBroadcasters::new())),
        catalog,
        config: DocumentsConfig::default(),
        database,
    });
    Store {
        service: Arc::clone(runtime.service()),
        _blobs: blobs,
    }
}

fn bridge_router(pool: &Arc<sqlx::PgPool>, service: &Arc<DocumentService>) -> axum::Router {
    use systemprompt::api::services::middleware::jwt::{JtiRevocationChecker, JwtContextExtractor};
    let database: DbPool = Arc::new(Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let users = systemprompt::users::UserService::new(Arc::new(
        systemprompt::users::UserRepository::new(&database),
    ));
    let jwt = JwtContextExtractor::new(
        Arc::new(systemprompt::users::SessionRepository::new(&database)),
        Arc::new(users),
        JtiRevocationChecker::from_repository(
            systemprompt::oauth::OAuthRepository::new(&database, Arc::from("docs-smoke")),
            systemprompt::traits::CacheEpochs::new(),
        ),
        globals::jwt_issuer(),
        globals::signing_keys().clone(),
    );
    systemprompt::api::routes::documents::drive::router(
        systemprompt::api::routes::documents::drive::DriveState::new(
            Arc::clone(service),
            Arc::new(jwt),
        ),
    )
}

pub(crate) fn mount(pool: &Arc<sqlx::PgPool>, store: &Store) -> DocumentsMount {
    DocumentsMount {
        drive: DriveDocuments(Arc::clone(&store.service)),
        bridge: bridge_router(pool, &store.service),
    }
}
