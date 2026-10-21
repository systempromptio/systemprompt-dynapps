//! Server-rendered identity pages.
//!
//! The people roster and detail, groups, projects and their reports, roles,
//! the access-control matrix and the viewer's own profile.
//!
//! Each module owns one page: it builds a typed template context and renders a
//! `.hbs` template from `storage/files/admin/templates/` at request time.

pub(crate) mod people_chart;
pub(crate) mod people_view;
mod roles;
mod ssr_access_control;
mod ssr_group_detail;
mod ssr_groups;
mod ssr_member_subscriptions;
mod ssr_profile;
mod ssr_projects;
mod ssr_users;
pub(crate) mod types;

pub(crate) use roles::roles_page;
pub(crate) use ssr_access_control::access_control_page;
pub(crate) use ssr_group_detail::group_detail_page;
pub(crate) use ssr_groups::groups_page;
pub(crate) use ssr_member_subscriptions::member_subscriptions_page;
pub(crate) use ssr_profile::{issue_bridge_code, profile_page};
pub(crate) use ssr_projects::{project_detail_page, project_report_page, projects_page};
pub(crate) use ssr_users::{user_detail_by_id_page, user_detail_page, users_page};
