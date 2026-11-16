//! Planner rosters: group-less containers of plans with their own member
//! list. Graph adds the caller as the first member of a new roster.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: rosters and their members are the Graph resources Planner returns.
use serde_json::json;

use super::graph_tool::graph_tools;
use super::planner::delete_versioned;
use super::reply::{GraphItem, GraphList};
use super::teams::list_page;
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError, GraphId, GraphQuery};

const CREATE_ROSTER: &str = "Create a Planner roster. Rosters hold group-less plans and carry \
    their own member list; the signed-in user becomes its first member.";
const DELETE_ROSTER: &str = "Delete a Planner roster. Needs the `@odata.etag` from a prior get.";
const LIST_MEMBERS: &str = "List the members of a Planner roster. Page with `skip`.";
const ADD_MEMBER: &str = "Add a user to a Planner roster.";
const DELETE_MEMBER: &str = "Remove a member from a Planner roster.";

graph_tools! {
    "d365_create_planner_roster" => create_planner_roster, NewRosterInput, GraphItem,
        CREATE_ROSTER;
    "d365_delete_planner_roster" => delete_planner_roster, DeleteRosterInput, GraphItem,
        DELETE_ROSTER;
    "d365_list_planner_roster_members" => list_planner_roster_members, RosterMembersInput,
        GraphList, LIST_MEMBERS;
    "d365_add_planner_roster_member" => add_planner_roster_member, AddRosterMemberInput,
        GraphItem, ADD_MEMBER;
    "d365_delete_planner_roster_member" => delete_planner_roster_member,
        DeleteRosterMemberInput, GraphItem, DELETE_MEMBER;
}

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct NewRosterInput {}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DeleteRosterInput {
    #[schemars(description = "The roster ID.")]
    pub roster_id: String,
    #[schemars(description = "`@odata.etag` from a prior get (sent as If-Match).")]
    pub etag: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RosterMembersInput {
    #[schemars(description = "The roster ID.")]
    pub roster_id: String,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct AddRosterMemberInput {
    #[schemars(description = "The roster ID.")]
    pub roster_id: String,
    #[schemars(description = "Entra object ID of the user to add.")]
    pub user_id: GraphId,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DeleteRosterMemberInput {
    #[schemars(description = "The roster ID.")]
    pub roster_id: String,
    #[schemars(
        description = "The roster member ID, from d365_list_planner_roster_members (not the \
        user's object ID)."
    )]
    pub member_id: String,
}

fn roster_path(roster_id: &str) -> Result<String, GraphError> {
    let roster = validate_id(roster_id, "roster_id")?;
    Ok(format!("/planner/rosters/{roster}"))
}

pub async fn create_planner_roster(
    graph: &GraphClient,
    _input: NewRosterInput,
) -> Result<GraphItem, GraphError> {
    let data = graph.post_json("/planner/rosters", json!({})).await?;
    Ok(GraphItem::new(data))
}

pub async fn delete_planner_roster(
    graph: &GraphClient,
    input: DeleteRosterInput,
) -> Result<GraphItem, GraphError> {
    let path = roster_path(&input.roster_id)?;
    let deleted = json!({"deleted": true, "roster_id": input.roster_id});
    delete_versioned(graph, &path, &input.etag, deleted).await
}

pub async fn list_planner_roster_members(
    graph: &GraphClient,
    input: RosterMembersInput,
) -> Result<GraphList, GraphError> {
    let path = format!("{}/members", roster_path(&input.roster_id)?);
    list_page(graph, &path, GraphQuery::new(), input.skip, None).await
}

pub async fn add_planner_roster_member(
    graph: &GraphClient,
    input: AddRosterMemberInput,
) -> Result<GraphItem, GraphError> {
    let path = format!("{}/members", roster_path(&input.roster_id)?);
    let user = validate_id(&input.user_id, "user_id")?;
    let data = graph.post_json(&path, json!({"userId": user})).await?;
    Ok(GraphItem::new(data))
}

pub async fn delete_planner_roster_member(
    graph: &GraphClient,
    input: DeleteRosterMemberInput,
) -> Result<GraphItem, GraphError> {
    let roster = roster_path(&input.roster_id)?;
    let member = validate_id(&input.member_id, "member_id")?;
    graph.delete(&format!("{roster}/members/{member}")).await?;
    Ok(GraphItem::new(json!({
        "deleted": true, "roster_id": input.roster_id, "member_id": member
    })))
}
