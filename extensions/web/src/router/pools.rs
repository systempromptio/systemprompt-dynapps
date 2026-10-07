//! Database handle extraction from the type-erased extension context, plus
//! the storage scratch root the managed-source Git checkouts are staged in.

use std::path::PathBuf;
use std::sync::Arc;

use systemprompt::config::{AppPaths, ProfileBootstrap};

use systemprompt::database::{Database, DbPool};
use systemprompt::extension::prelude::ExtensionContext;
use systemprompt::oauth::SessionCreationService;
use systemprompt::prelude::PgPool;
use systemprompt::users::{SessionRepository, UserService};

pub(crate) struct DbHandles {
    pub owner: systemprompt::identifiers::UserId,
    pub scratch_root: Option<PathBuf>,
    pub read: Arc<PgPool>,
    pub write: Arc<PgPool>,
    pub db: DbPool,
}

impl DbHandles {
    pub(crate) fn from_context(ctx: &dyn ExtensionContext) -> Option<Self> {
        let db_handle = ctx.database();
        let db = db_handle.as_any().downcast_ref::<Database>()?;
        let read = db.pool();
        let write = db.write_pool();
        let db = Arc::new(Database::from_pools(
            Arc::clone(&read),
            Some(Arc::clone(&write)),
        ));
        Some(Self {
            read,
            write,
            db,
            owner: ctx.system_owner_id(),
            scratch_root: scratch_root(),
        })
    }
}

fn scratch_root() -> Option<PathBuf> {
    let profile = ProfileBootstrap::get()
        .map_err(|e| tracing::error!(error = %e, "Profile unavailable for the scratch root"))
        .ok()?;
    let paths = AppPaths::from_profile(
        &profile.paths,
        profile.path_resolution(),
        systemprompt::loader::ServicesRootBootstrap::get().map(|root| root.path.as_path()),
    )
    .map_err(|e| tracing::error!(error = %e, "App paths unavailable for the scratch root"))
    .ok()?;
    Some(paths.storage().scratch().to_path_buf())
}

pub(crate) fn build_session_service(db: &DbHandles) -> Arc<SessionCreationService> {
    let dbpool = Arc::clone(&db.db);
    let user_repo = systemprompt::users::UserRepository::new(&dbpool);
    let user = UserService::new(Arc::new(user_repo));
    let sessions = SessionRepository::new(&dbpool);
    Arc::new(SessionCreationService::new(
        Arc::new(sessions),
        Arc::new(user),
    ))
}
