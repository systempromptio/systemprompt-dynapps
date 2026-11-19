//! Planner buckets: the columns of a plan. Updates and deletes carry the
//! bucket's `@odata.etag` as `If-Match`, like every Planner write.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: buckets and their writes are the Graph resources Planner returns.
use serde_json::{Map, Value, json};

use super::graph_tool::graph_tools;
use super::planner::{delete_versioned, patch_versioned};
use super::reply::{GraphItem, GraphList};
use super::teams::{list_page, select};
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError};

pub const PLANNER_BUCKET_FIELDS: &str = "id,name,planId,orderHint";

const LIST_BUCKETS: &str = "List all buckets in a Planner plan. Page with `skip`.";
const GET_BUCKET: &str = "Get a Planner bucket by ID.";
const CREATE_BUCKET: &str = "Create a bucket in a Planner plan.";
const UPDATE_BUCKET: &str = "Update a Planner bucket. Needs the `@odata.etag` from a prior get.";
const DELETE_BUCKET: &str = "Delete a Planner bucket. Needs the `@odata.etag` from a prior get.";

graph_tools! {
    "d365_list_planner_buckets" => list_planner_buckets, PlanBucketsInput, GraphList,
        LIST_BUCKETS;
    "d365_get_planner_bucket" => get_planner_bucket, BucketIdInput, GraphItem, GET_BUCKET;
    "d365_create_planner_bucket" => create_planner_bucket, CreateBucketInput, GraphItem,
        CREATE_BUCKET;
    "d365_update_planner_bucket" => update_planner_bucket, UpdateBucketInput, GraphItem,
        UPDATE_BUCKET;
    "d365_delete_planner_bucket" => delete_planner_bucket, DeleteBucketInput, GraphItem,
        DELETE_BUCKET;
}

fn append_hint() -> String {
    " !".to_owned()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct PlanBucketsInput {
    #[schemars(description = "The plan ID.")]
    pub plan_id: String,
    #[schemars(description = "Number of items to skip for pagination (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct BucketIdInput {
    #[schemars(description = "The bucket ID.")]
    pub bucket_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateBucketInput {
    #[schemars(description = "The plan ID to add the bucket to.")]
    pub plan_id: String,
    #[schemars(description = "Bucket name.")]
    pub name: String,
    #[schemars(
        description = "Planner order hint; the default \" !\" appends to the end of the plan."
    )]
    #[serde(default = "append_hint")]
    pub order_hint: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateBucketInput {
    #[schemars(description = "The bucket ID.")]
    pub bucket_id: String,
    #[schemars(description = "`@odata.etag` from a prior get (sent as If-Match).")]
    pub etag: String,
    #[schemars(description = "New bucket name.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub name: Option<String>,
    #[schemars(description = "New order hint (repositions the bucket).")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub order_hint: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DeleteBucketInput {
    #[schemars(description = "The bucket ID.")]
    pub bucket_id: String,
    #[schemars(description = "`@odata.etag` from a prior get (sent as If-Match).")]
    pub etag: String,
}

fn bucket_path(bucket_id: &str) -> Result<String, GraphError> {
    let bucket = validate_id(bucket_id, "bucket_id")?;
    Ok(format!("/planner/buckets/{bucket}"))
}

pub async fn list_planner_buckets(
    graph: &GraphClient,
    input: PlanBucketsInput,
) -> Result<GraphList, GraphError> {
    let plan = validate_id(&input.plan_id, "plan_id")?;
    let path = format!("/planner/plans/{plan}/buckets");
    list_page(
        graph,
        &path,
        select(PLANNER_BUCKET_FIELDS),
        input.skip,
        None,
    )
    .await
}

pub async fn get_planner_bucket(
    graph: &GraphClient,
    input: BucketIdInput,
) -> Result<GraphItem, GraphError> {
    let path = bucket_path(&input.bucket_id)?;
    let data = graph
        .get_json(&path, &select(PLANNER_BUCKET_FIELDS))
        .await?;
    Ok(GraphItem::new(data))
}

pub async fn create_planner_bucket(
    graph: &GraphClient,
    input: CreateBucketInput,
) -> Result<GraphItem, GraphError> {
    let plan = validate_id(&input.plan_id, "plan_id")?;
    let body = json!({"name": input.name, "planId": plan, "orderHint": input.order_hint});
    let data = graph.post_json("/planner/buckets", body).await?;
    Ok(GraphItem::new(data))
}

pub async fn update_planner_bucket(
    graph: &GraphClient,
    input: UpdateBucketInput,
) -> Result<GraphItem, GraphError> {
    let path = bucket_path(&input.bucket_id)?;
    let mut changes = Map::new();
    if let Some(name) = input.name {
        changes.insert("name".to_owned(), Value::String(name));
    }
    if let Some(hint) = input.order_hint {
        changes.insert("orderHint".to_owned(), Value::String(hint));
    }
    patch_versioned(graph, &path, &input.etag, changes).await
}

pub async fn delete_planner_bucket(
    graph: &GraphClient,
    input: DeleteBucketInput,
) -> Result<GraphItem, GraphError> {
    let path = bucket_path(&input.bucket_id)?;
    let deleted = json!({"deleted": true, "bucket_id": input.bucket_id});
    delete_versioned(graph, &path, &input.etag, deleted).await
}
