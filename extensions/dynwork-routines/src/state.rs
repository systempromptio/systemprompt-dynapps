//! What every routines route shares: the write pool, the instance's token
//! authority, and the services view and subject sources the template
//! visibility resolves against.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::models::auth::TokenAuthority;
use systemprompt_web_admin_core::extension_services::ExtensionServices;

#[derive(Debug, Clone)]
pub struct RoutinesState {
    pub pool: Arc<PgPool>,
    pub authority: Arc<dyn TokenAuthority>,
    pub services: ExtensionServices,
}
