//! Database handle extraction from the type-erased extension context, plus
//! the managed-resource services built once over them (their Git checkouts
//! are staged under the profile's storage scratch root) and the agent farm
//! state (run directories under the storage data root).

use std::sync::Arc;

use systemprompt::config::AppPaths;
use systemprompt_web_admin_core::extension_boot::ExtensionBoot;
use systemprompt_web_admin_core::state::ManagedState;
use systemprompt_web_admin_farm::services::runner::fly::registry::RemoteRuns;
use systemprompt_web_admin_farm::state::FarmState;

use systemprompt::ai::repository::AiRepositories;
use systemprompt::database::{Database, DbPool};
use systemprompt::extension::prelude::ExtensionContext;
use systemprompt::oauth::SessionCreationService;
use systemprompt::oauth::repository::OAuthRepositories;
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
    pub services: systemprompt_web_admin_core::extension_services::ExtensionServices,
    pub boot: ExtensionBoot,
    pub oauth: Arc<OAuthRepositories>,
    pub ai: Arc<AiRepositories>,
}

impl DbHandles {
    pub(crate) fn from_context(ctx: &dyn ExtensionContext, runs: Arc<RemoteRuns>) -> Option<Self> {
        let db_handle = ctx.database();
        let db = db_handle.as_any().downcast_ref::<Database>()?;
        let read = db.pool();
        let write = db.write_pool();
        let db = Arc::new(Database::from_pools(
            Arc::clone(&read),
            Some(Arc::clone(&write)),
        ));
        let boot = ExtensionBoot::from_context(ctx);
        let dependencies = ctx.dependencies();
        let oauth = dependencies
            .get::<Arc<OAuthRepositories>>()
            .inspect_err(|error| tracing::error!(%error, "web routes not mounted"))
            .ok()
            .map(Arc::clone)?;
        let ai = dependencies
            .get::<Arc<AiRepositories>>()
            .inspect_err(|error| tracing::error!(%error, "web routes not mounted"))
            .ok()
            .map(Arc::clone)?;
        let paths = app_paths(&boot);
        let managed = Arc::new(ManagedState::new(
            &db,
            ctx.system_owner_id(),
            paths
                .as_ref()
                .map(|paths| paths.storage().scratch().to_path_buf()),
            boot.services_root(),
        ));
        let files = ctx.file_storage();
        let services =
            systemprompt_web_admin_core::extension_services::ExtensionServices::from_context(ctx);
        let farm = super::farm_public::build_state(
            Arc::clone(&write),
            paths.as_ref(),
            systemprompt_web_admin_farm::state::FarmHost {
                artefacts: Arc::clone(&files),
                instance: ctx.instance_id(),
                runs,
                tasks: ctx.background_tasks(),
                services: services.clone(),
                boot: boot.clone(),
            },
        );
        super::farm_public::start(&farm, paths.as_ref());
        Some(Self {
            managed,
            read,
            write,
            db,
            farm,
            files,
            services,
            boot,
            oauth,
            ai,
        })
    }
}

pub(crate) fn app_paths(boot: &ExtensionBoot) -> Option<AppPaths> {
    let profile = boot.profile();
    AppPaths::from_profile(
        &profile.paths,
        profile.path_resolution(),
        Some(boot.services_root()),
    )
    .map_err(|e| tracing::error!(error = %e, "App paths unavailable for the storage paths"))
    .ok()
}

pub(crate) fn build_session_service(db: &DbHandles) -> Arc<SessionCreationService> {
    let dbpool = Arc::clone(&db.db);
    let user_repo = systemprompt::users::UserRepository::new(&dbpool);
    let user = UserService::new(Arc::new(user_repo));
    let sessions = SessionRepository::new(&dbpool);
    Arc::new(SessionCreationService::new(
        Arc::new(sessions),
        Arc::new(user),
        Arc::clone(db.boot.token_authority()),
        Arc::clone(db.boot.config_arc()),
    ))
}
