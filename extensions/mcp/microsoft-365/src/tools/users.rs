//! The directory's people: the signed-in user's profile and manager, a user
//! by ID or UPN, a display-name search, and a user's direct reports.

use schemars::JsonSchema;
use serde::Deserialize;

use super::graph_tool::graph_tools;
use super::groups::search_directory;
use super::reply::{GraphItem, GraphList};
use super::teams::{list_page, select};
use crate::graph::sanitize::{DEFAULT_MAX_TOP, clamp_skip, clamp_top, validate_id};
use crate::graph::{GraphClient, GraphError, GraphId, Window};

pub const USER_FIELDS: &str =
    "id,displayName,mail,userPrincipalName,jobTitle,department,officeLocation";
pub const USER_DETAIL_FIELDS: &str = "id,displayName,mail,userPrincipalName,jobTitle,\
    department,officeLocation,businessPhones,mobilePhone,companyName";

const GET_MY_PROFILE: &str = "Get the signed-in user's full profile.";
const SEARCH_USERS: &str = "Search for users by display name in the directory. Page with \
    `top` (1-50) and `skip`.";
const GET_USER: &str = "Get detailed profile information for a specific user.";
const GET_MY_MANAGER: &str = "Get the signed-in user's manager.";
const LIST_DIRECT_REPORTS: &str = "List the direct reports of a user, or of the signed-in \
    user when `user_id` is empty. Page with `skip`.";

graph_tools! {
    "d365_get_my_profile" => get_my_profile, MyProfileInput, GraphItem, GET_MY_PROFILE;
    "d365_search_users" => search_users, SearchUsersInput, GraphList, SEARCH_USERS;
    "d365_get_user" => get_user, UserIdInput, GraphItem, GET_USER;
    "d365_get_my_manager" => get_my_manager, MyProfileInput, GraphItem, GET_MY_MANAGER;
    "d365_list_direct_reports" => list_direct_reports, DirectReportsInput, GraphList,
        LIST_DIRECT_REPORTS;
}

const fn default_top() -> i64 {
    10
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct MyProfileInput {}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchUsersInput {
    #[schemars(description = "Display name search term.")]
    pub query: String,
    #[schemars(description = "Maximum number of results to return (1-50, default 10).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UserIdInput {
    #[schemars(description = "The user ID or userPrincipalName.")]
    pub user_id: GraphId,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DirectReportsInput {
    #[schemars(
        description = "The user ID or UPN. Empty lists the signed-in user's direct reports."
    )]
    #[serde(default)]
    #[schemars(with = "String")]
    pub user_id: Option<GraphId>,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

pub async fn get_my_profile(
    graph: &GraphClient,
    _input: MyProfileInput,
) -> Result<GraphItem, GraphError> {
    let data = graph.get_json("/me", &select(USER_DETAIL_FIELDS)).await?;
    Ok(GraphItem::new(data))
}

pub async fn search_users(
    graph: &GraphClient,
    input: SearchUsersInput,
) -> Result<GraphList, GraphError> {
    let window = Window {
        skip: clamp_skip(input.skip),
        top: Some(clamp_top(input.top, DEFAULT_MAX_TOP)),
    };
    search_directory(graph, "/users", select(USER_FIELDS), &input.query, window).await
}

pub async fn get_user(graph: &GraphClient, input: UserIdInput) -> Result<GraphItem, GraphError> {
    let user = validate_id(&input.user_id, "user_id")?;
    let path = format!("/users/{user}");
    let data = graph.get_json(&path, &select(USER_DETAIL_FIELDS)).await?;
    Ok(GraphItem::new(data))
}

pub async fn get_my_manager(
    graph: &GraphClient,
    _input: MyProfileInput,
) -> Result<GraphItem, GraphError> {
    let data = graph.get_json("/me/manager", &select(USER_FIELDS)).await?;
    Ok(GraphItem::new(data))
}

pub async fn list_direct_reports(
    graph: &GraphClient,
    input: DirectReportsInput,
) -> Result<GraphList, GraphError> {
    let path = match input.user_id.as_deref().filter(|user| !user.is_empty()) {
        Some(user) => format!("/users/{}/directReports", validate_id(user, "user_id")?),
        None => "/me/directReports".to_owned(),
    };
    list_page(graph, &path, select(USER_FIELDS), input.skip, None).await
}
