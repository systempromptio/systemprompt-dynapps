//! Mailbox organisation: mail folders, inbox rules and Outlook categories.
//! A rule without a `sequence` runs last: Graph reads a missing sequence as
//! 0 and refuses it, so the next free position is taken from the existing
//! rules, as the Belgian server does.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: rule conditions and actions are Graph's open `messageRulePredicates`
// and `messageRuleActions`, passed through as the model writes them.
use serde_json::{Map, Value, json};

use super::reply::GraphItem;
use crate::graph::sanitize::validate_id;
use crate::graph::{GraphClient, GraphError, GraphQuery};

const RULES_PATH: &str = "/me/mailFolders/inbox/messageRules";
const MAX_COLOR_PRESET: u8 = 24;

pub const CREATE_MAIL_FOLDER_DESCRIPTION: &str = "Create a mail folder, at the top level or \
    under parent_folder_id.";
pub const DELETE_MAIL_FOLDER_DESCRIPTION: &str = "Delete a mail folder and all its contents \
    (its messages move to Deleted Items).";
pub const CREATE_MAIL_RULE_DESCRIPTION: &str = "Create an inbox message rule. conditions: \
    fromAddresses / sentToAddresses ([{\"emailAddress\": {\"address\": \"x@y.com\"}}]), \
    senderContains, recipientContains, subjectContains, bodyContains (lists of text), \
    hasAttachments, importance. actions: moveToFolder / copyToFolder (folder id), delete, \
    markAsRead, markImportance, forwardTo (recipients), assignCategories (names). sequence \
    (1 or higher) defaults to after the last rule.";
pub const CREATE_OUTLOOK_CATEGORY_DESCRIPTION: &str = "Create a category in the user's Outlook \
    master category list, for mail rules, emails and calendar events. color is \"none\" or \
    \"preset0\" to \"preset24\" (default \"preset0\", red).";

const fn default_enabled() -> bool {
    true
}

fn default_color() -> String {
    "preset0".to_owned()
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateMailFolderInput {
    #[schemars(description = "The name of the new folder.")]
    pub display_name: String,
    #[schemars(
        description = "Parent folder ID for a subfolder; empty creates a top-level folder."
    )]
    #[serde(default)]
    pub parent_folder_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct FolderIdInput {
    #[schemars(description = "The unique ID of the mail folder.")]
    pub folder_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateMailRuleInput {
    #[schemars(description = "A human-readable name for the rule.")]
    pub display_name: String,
    #[schemars(description = "Rule conditions object (Graph messageRulePredicates).")]
    pub conditions: Map<String, Value>,
    #[schemars(description = "Rule actions object (Graph messageRuleActions).")]
    pub actions: Map<String, Value>,
    #[schemars(description = "Execution order, 1 or higher; omitted runs the rule last.")]
    #[serde(default)]
    #[schemars(with = "i64")]
    pub sequence: Option<i64>,
    #[schemars(description = "Whether the rule is active (default true).")]
    #[serde(default = "default_enabled")]
    pub is_enabled: bool,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateCategoryInput {
    #[schemars(description = "The category name, unique in the user's category list.")]
    pub display_name: String,
    #[schemars(description = "\"none\" or \"preset0\" to \"preset24\" (default \"preset0\").")]
    #[serde(default = "default_color")]
    pub color: String,
}

pub async fn create_mail_folder(
    graph: &GraphClient,
    input: CreateMailFolderInput,
) -> Result<GraphItem, GraphError> {
    let path = if input.parent_folder_id.is_empty() {
        "/me/mailFolders".to_owned()
    } else {
        let parent = validate_id(&input.parent_folder_id, "parent_folder_id")?;
        format!("/me/mailFolders/{parent}/childFolders")
    };
    graph
        .post_json(&path, json!({"displayName": input.display_name}))
        .await
        .map(GraphItem::new)
}

pub async fn delete_mail_folder(
    graph: &GraphClient,
    input: FolderIdInput,
) -> Result<GraphItem, GraphError> {
    let id = validate_id(&input.folder_id, "folder_id")?;
    graph.delete(&format!("/me/mailFolders/{id}")).await?;
    Ok(GraphItem::new(json!({"deleted": true, "folder_id": id})))
}

async fn next_rule_sequence(graph: &GraphClient) -> Result<i64, GraphError> {
    let rules = graph.get_json(RULES_PATH, &GraphQuery::new()).await?;
    let highest = rules
        .get("value")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|rule| rule.get("sequence").and_then(Value::as_i64))
        .max()
        .unwrap_or(0);
    Ok(highest + 1)
}

pub async fn create_mail_rule(
    graph: &GraphClient,
    input: CreateMailRuleInput,
) -> Result<GraphItem, GraphError> {
    let sequence = match input.sequence {
        Some(sequence) if sequence < 1 => {
            return Err(GraphError::Invalid(
                "sequence must be 1 or higher".to_owned(),
            ));
        },
        Some(sequence) => sequence,
        None => next_rule_sequence(graph).await?,
    };
    let payload = json!({
        "displayName": input.display_name,
        "sequence": sequence,
        "conditions": input.conditions,
        "actions": input.actions,
        "isEnabled": input.is_enabled,
    });
    graph
        .post_json(RULES_PATH, payload)
        .await
        .map(GraphItem::new)
}

fn valid_color(color: &str) -> bool {
    color == "none"
        || color.strip_prefix("preset").is_some_and(|number| {
            !number.is_empty()
                && number.len() <= 2
                && number.bytes().all(|b| b.is_ascii_digit())
                && !(number.len() == 2 && number.starts_with('0'))
                && number.parse::<u8>().is_ok_and(|n| n <= MAX_COLOR_PRESET)
        })
}

pub async fn create_outlook_category(
    graph: &GraphClient,
    input: CreateCategoryInput,
) -> Result<GraphItem, GraphError> {
    if !valid_color(&input.color) {
        return Err(GraphError::Invalid(
            "color must be 'none' or 'preset0' to 'preset24'".to_owned(),
        ));
    }
    graph
        .post_json(
            "/me/outlook/masterCategories",
            json!({"displayName": input.display_name, "color": input.color}),
        )
        .await
        .map(GraphItem::new)
}
