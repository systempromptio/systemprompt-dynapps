//! Database handle extraction from the type-erased extension context, plus
//! the managed-resource services built once over them (their Git checkouts
//! are staged under the profile's storage scratch root) and the agent farm
//! state (run directories under the storage data root).

use std::path::PathBuf;
use std::sync::Arc;

use systemprompt::config::{AppPaths, ProfileBootstrap};
use systemprompt_web_admin_core::state::ManagedState;
use systemprompt_web_admin_farm::state::FarmState;

use systemprompt::database::{Database, DbPool};
use systemprompt::extension::prelude::ExtensionContext;
use systemprompt::oauth::SessionCreationService;
use systemprompt::prelude::PgPool;
use systemprompt::traits::FileStorage;
use systemprompt::users::{SessionRepository, UserService};

pub(crate) struct DbHandles {
    pub managed: Arc<ManagedState>,
    pub read: Arc<PgPool>,
    pub write: Arc<PgPool>,
    pub db: DbPool,
    pub farm: Arc<FarmState>,
    pub files: Arc<dyn FileStorage>,
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
        let managed = Arc::new(ManagedState::new(
            &db,
            ctx.system_owner_id(),
            scratch_root(),
        ));
        let paths = app_paths();
        let files = ctx.file_storage();
        let farm = super::farm_public::build_state(Arc::clone(&write), paths.as_ref());
        super::farm_public::start(&farm, paths.as_ref());
        Some(Self {
            managed,
            read,
            write,
            db,
            farm,
            files,
        })
    }
}

fn app_paths() -> Option<AppPaths> {
    let profile = ProfileBootstrap::get()
        .map_err(|e| tracing::error!(error = %e, "Profile unavailable for the storage paths"))
        .ok()?;
    AppPaths::from_profile(
        &profile.paths,
        profile.path_resolution(),
        systemprompt::loader::ServicesRootBootstrap::get().map(|root| root.path.as_path()),
    )
    .map_err(|e| tracing::error!(error = %e, "App paths unavailable for the storage paths"))
    .ok()
}

fn scratch_root() -> Option<PathBuf> {
    app_paths().map(|paths| paths.storage().scratch().to_path_buf())
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
