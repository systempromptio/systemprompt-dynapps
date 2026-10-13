//! Console JSON API for farms under `/api/public/admin/agent-farm`: the farm
//! list and detail (console tier), and the sync action (manage tier).

mod farms;
mod sync;

pub(crate) use farms::{get_farm, list_farms};
pub(crate) use sync::sync_farm;
