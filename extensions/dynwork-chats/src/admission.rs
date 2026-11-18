//! Who may use which model.
//!
//! [`Admission`] is the list `GET /models` answers and the check
//! `POST /threads/{id}/prompt` and `PUT /threads/{id}` make before they accept
//! a model, from one place so they can never disagree.
//!
//! The instance's authz hook is the decision (see [`catalog::listing`]). Core
//! builds one at boot (`AppContext::authz_hook`) but gives it to its own
//! routes, not to an extension's: `ExtensionContext` carries the profile, the
//! services and the secrets it is built from. So [`LazyHook`] builds one, on
//! the first use and once, with core's own `build_authz_hook` from those same
//! inputs, the way core's boot does (`authz_stage`). Building it only
//! constructs objects: no task, listener, migration or write.
//!
//! Its rule cache rechecks `config_version` every 5 s (a committed rule or
//! entity edit shows within that) and reloads every 60 s whatever happens
//! (`parent_chain/cache.rs`). The installation hook behind it keeps its rules,
//! roles and parent chains for 30 s in process-wide caches keyed by database,
//! which are core's own (the same entries the gateway uses) and which a console
//! edit or a sync apply on this replica drops at once. So a rule edit shows in
//! `/models` within 5 s when made in the console, and within 35 s (5 s plus 30
//! s) at the latest when written any other way. What this hook lacks next to
//! core's is core's change epoch, which would reload at once.
//!
//! Everything fails closed: a hook that cannot be built is the deny-all hook
//! (an empty list, every model refused), and a roles lookup that fails is an
//! error, never the full catalog.

use std::collections::BTreeSet;
use std::fmt;
use std::sync::{Arc, OnceLock};

use dynwork_relay::auth::AuthError;
use sqlx::PgPool;
use systemprompt::api::routes::gateway::bridge_admission::AdmissionPrincipal;
use systemprompt::governance::authz::{
    AuthzHookContext, AuthzWiring, ChainSources, DenyAllHook, NullAuditSink, SharedAuthzHook,
    SubjectProviderSet, build_authz_hook,
};
use systemprompt::manifest::Profile;
use systemprompt::manifest::services::ServicesView;
use systemprompt::manifest::services::catalog::ResolvedCatalog;
use systemprompt::models::authz::SubjectSourceHandle;
use systemprompt::traits::{CacheEpochs, SecretStore};

use crate::auth::Caller;
use crate::catalog::{self, Listing};
use crate::error::ChatsError;
use crate::roles::Roles;

type Build = Box<dyn Fn() -> SharedAuthzHook + Send + Sync>;

/// A hook built the first time it is asked for, then kept.
pub struct LazyHook {
    build: Build,
    hook: OnceLock<SharedAuthzHook>,
}

impl fmt::Debug for LazyHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LazyHook")
            .field("built", &self.hook.get().is_some())
            .finish_non_exhaustive()
    }
}

impl LazyHook {
    pub fn new(build: impl Fn() -> SharedAuthzHook + Send + Sync + 'static) -> Self {
        Self {
            build: Box::new(build),
            hook: OnceLock::new(),
        }
    }

    pub fn get(&self) -> &SharedAuthzHook {
        self.hook.get_or_init(|| (self.build)())
    }

    pub fn instance(
        profile: Arc<Profile>,
        services: ServicesView,
        secrets: Arc<dyn SecretStore>,
        pool: Arc<PgPool>,
    ) -> Self {
        Self::new(move || instance_hook(&profile, &services, &secrets, &pool))
    }
}

fn instance_hook(
    profile: &Profile,
    services: &ServicesView,
    secrets: &Arc<dyn SecretStore>,
    pool: &Arc<PgPool>,
) -> SharedAuthzHook {
    let subjects = SubjectSourceHandle::default();
    let providers = SubjectProviderSet::discover(&AuthzHookContext {
        pool: Arc::clone(pool),
        sink: Arc::new(NullAuditSink),
        services: services.clone(),
        subjects: subjects.clone(),
        secrets: Arc::clone(secrets),
    });
    if subjects.bind(Arc::new(providers.clone())).is_err() {
        tracing::error!("dynwork chats subject source was already bound; no model is served");
        return Arc::new(DenyAllHook::null());
    }
    let wiring = AuthzWiring {
        sources: ChainSources::from_services(&services.config()),
        epochs: CacheEpochs::new(),
        services: services.clone(),
        subjects,
        providers,
        secrets: Arc::clone(secrets),
    };
    build_authz_hook(profile.governance.as_ref(), Arc::clone(pool), None, wiring).unwrap_or_else(
        |error| {
            tracing::error!(%error, "dynwork chats could not build the authz hook; no model is served");
            Arc::new(DenyAllHook::null())
        },
    )
}

/// The names of the secrets the instance holds, lowercase.
pub type SecretNames = Arc<dyn Fn() -> BTreeSet<String> + Send + Sync>;

pub fn names_of(secrets: Arc<dyn SecretStore>) -> SecretNames {
    Arc::new(move || match secrets.key_names() {
        Ok(names) => names.into_iter().map(|name| name.to_lowercase()).collect(),
        Err(error) => {
            tracing::error!(error = %error, "dynwork chats could not list the instance secrets; no model is served");
            BTreeSet::new()
        },
    })
}

/// The models a caller may use, by the instance's rules.
pub struct Admission {
    secrets: SecretNames,
    hook: LazyHook,
    roles: Arc<dyn Roles>,
}

impl fmt::Debug for Admission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Admission")
            .field("hook", &self.hook)
            .finish_non_exhaustive()
    }
}

impl Admission {
    pub fn new(secrets: SecretNames, hook: LazyHook, roles: Arc<dyn Roles>) -> Self {
        Self {
            secrets,
            hook,
            roles,
        }
    }

    pub async fn listing(
        &self,
        catalog: &ResolvedCatalog,
        caller: &Caller,
    ) -> Result<Listing, ChatsError> {
        let roles = self
            .roles
            .of(&caller.user)
            .await?
            .ok_or(AuthError::UnknownUser)?;
        let principal = AdmissionPrincipal {
            user_id: caller.user.clone(),
            roles,
            client_id: caller.client_id.clone(),
            session_id: caller.admission_session(),
        };
        let names = (self.secrets)();
        Ok(catalog::listing(
            catalog,
            &|name| names.contains(&name.to_lowercase()),
            &BTreeSet::new(),
            self.hook.get(),
            &principal,
        )
        .await)
    }
}
