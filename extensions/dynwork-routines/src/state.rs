//! What every routines route shares: the write pool and the instance's token
//! authority.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::models::auth::TokenAuthority;

#[derive(Debug, Clone)]
pub struct RoutinesState {
    pub pool: Arc<PgPool>,
    pub authority: Arc<dyn TokenAuthority>,
}
