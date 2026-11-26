//! What every route shares, built once per process by the extension, or by a
//! suite with its own authority.

use std::sync::Arc;

use dynwork_relay::store::postgres::PgRelayStore;
use systemprompt::models::auth::TokenAuthority;

use crate::brain::Brain;

#[derive(Debug, Clone)]
pub struct BrainState {
    pub brain: Arc<dyn Brain>,
    pub authority: Arc<dyn TokenAuthority>,
    pub devices: Arc<PgRelayStore>,
}
