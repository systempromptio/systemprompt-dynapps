//! Template context for `/admin/access-control`.
//!
//! The page is a three-pane editor whose centre is rendered client-side, so
//! most of what the server supplies is the material the controller needs
//! embedded once: the entity catalogue, the role list, and the group/project
//! trees. Everything editable is guarded by `can_write`, which is the
//! MANAGE tier — a project manager reads this page and cannot change it.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcUserView {
    pub id: String,
    pub display_name: String,
    pub email: String,
    pub roles: Vec<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcGroupView {
    pub id: String,
    pub name: String,
    pub is_system: bool,
    pub user_count: usize,
    pub users: Vec<AcUserView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcProjectView {
    pub id: String,
    pub name: String,
    pub member_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcStatsView {
    pub groups: usize,
    pub projects: usize,
    pub users: usize,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcEntityView {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct AcEntityCatalogue {
    pub gateway_routes: Vec<AcEntityView>,
    pub mcp_servers: Vec<AcEntityView>,
    pub plugins: Vec<AcEntityView>,
    pub agents: Vec<AcEntityView>,
    pub marketplaces: Vec<AcEntityView>,
    pub skills: Vec<AcEntityView>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AccessControlPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub can_write: bool,
    pub stats: AcStatsView,
    pub groups: Vec<AcGroupView>,
    pub projects: Vec<AcProjectView>,
    pub entity_catalogue: AcEntityCatalogue,
    pub known_roles: Vec<String>,
    pub group_ids: Vec<String>,
    pub project_ids: Vec<String>,
}
