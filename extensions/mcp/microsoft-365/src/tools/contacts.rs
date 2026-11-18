//! Personal contact tools: list (with a keyword search), read, create, update
//! and delete the signed-in user's contacts.
//!
//! `$search` on contacts needs Graph's `ConsistencyLevel: eventual`. An
//! update sets only the fields given; an email or phone replaces the
//! contact's whole list of them, as in the Belgian server.

use schemars::JsonSchema;
use serde::Deserialize;
// JSON: contact bodies are built as the Graph resource they write, and
// Graph's answer passes through as returned.
use serde_json::json;

use super::graph_tool::graph_tools;
use super::reply::{GraphItem, GraphList};
use crate::graph::sanitize::{
    DEFAULT_MAX_TOP, clamp_skip, clamp_top, sanitize_search, validate_id,
};
use crate::graph::{GraphClient, GraphError, GraphQuery, Window};

pub const CONTACT_FIELDS: &str =
    "id,displayName,emailAddresses,businessPhones,mobilePhone,companyName,jobTitle";

const SEARCH_HEADERS: [(&str, &str); 1] = [("ConsistencyLevel", "eventual")];

const LIST_CONTACTS: &str = "List the signed-in user's personal contacts, optionally narrowed \
    by a keyword `search`. Page with `top` (1-50) and `skip`.";
const GET_CONTACT: &str = "Get the full details of one personal contact.";
const CREATE_CONTACT: &str = "Create a personal contact; only `given_name` is required.";
const UPDATE_CONTACT: &str = "Update a personal contact; only the fields given change. An \
    email or phone replaces the contact's existing ones.";
const DELETE_CONTACT: &str = "Delete a personal contact.";

graph_tools! {
    "d365_list_contacts" => list_contacts, ListContactsInput, GraphList, LIST_CONTACTS;
    "d365_get_contact" => get_contact, ContactIdInput, GraphItem, GET_CONTACT;
    "d365_create_contact" => create_contact, CreateContactInput, GraphItem, CREATE_CONTACT;
    "d365_update_contact" => update_contact, UpdateContactInput, GraphItem, UPDATE_CONTACT;
    "d365_delete_contact" => delete_contact, ContactIdInput, GraphItem, DELETE_CONTACT;
}

const fn default_top() -> i64 {
    25
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ListContactsInput {
    #[schemars(description = "Number of contacts to return (1-50, default 25).")]
    #[serde(default = "default_top")]
    pub top: i64,
    #[schemars(description = "Optional keyword search over the contacts.")]
    #[serde(default)]
    pub search: String,
    #[schemars(description = "Items to skip for paging (default 0).")]
    #[serde(default)]
    pub skip: i64,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ContactIdInput {
    #[schemars(description = "The contact ID.")]
    pub contact_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct CreateContactInput {
    #[schemars(description = "First name (required).")]
    pub given_name: String,
    #[schemars(description = "Last name.")]
    #[serde(default)]
    pub surname: String,
    #[schemars(description = "Email address.")]
    #[serde(default)]
    pub email: String,
    #[schemars(description = "Business phone number.")]
    #[serde(default)]
    pub phone: String,
    #[schemars(description = "Company name.")]
    #[serde(default)]
    pub company: String,
    #[schemars(description = "Job title.")]
    #[serde(default)]
    pub job_title: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct UpdateContactInput {
    #[schemars(description = "The contact ID to update.")]
    pub contact_id: String,
    #[schemars(description = "New first name.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub given_name: Option<String>,
    #[schemars(description = "New last name.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub surname: Option<String>,
    #[schemars(description = "New email address.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub email: Option<String>,
    #[schemars(description = "New business phone number.")]
    #[serde(default)]
    #[schemars(with = "String")]
    pub phone: Option<String>,
}

fn contact_path(contact_id: &str) -> Result<String, GraphError> {
    let contact = validate_id(contact_id, "contact_id")?;
    Ok(format!("/me/contacts/{contact}"))
}

pub async fn list_contacts(
    graph: &GraphClient,
    input: ListContactsInput,
) -> Result<GraphList, GraphError> {
    let mut query = GraphQuery::new().with("$select", CONTACT_FIELDS);
    let headers: &[(&str, &str)] = if input.search.is_empty() {
        &[]
    } else {
        query.set("$search", format!("\"{}\"", sanitize_search(&input.search)));
        &SEARCH_HEADERS
    };
    let window = Window {
        skip: clamp_skip(input.skip),
        top: Some(clamp_top(input.top, DEFAULT_MAX_TOP)),
    };
    let page = graph
        .get_collection_with_headers("/me/contacts", &query, headers, window)
        .await?;
    Ok(GraphList::from(page))
}

pub async fn get_contact(
    graph: &GraphClient,
    input: ContactIdInput,
) -> Result<GraphItem, GraphError> {
    let path = contact_path(&input.contact_id)?;
    let data = graph.get_json(&path, &GraphQuery::new()).await?;
    Ok(GraphItem::new(data))
}

pub async fn create_contact(
    graph: &GraphClient,
    input: CreateContactInput,
) -> Result<GraphItem, GraphError> {
    let mut contact = json!({ "givenName": &input.given_name });
    if !input.surname.is_empty() {
        contact["surname"] = json!(&input.surname);
    }
    if !input.email.is_empty() {
        contact["emailAddresses"] = json!([{ "address": &input.email }]);
    }
    if !input.phone.is_empty() {
        contact["businessPhones"] = json!([&input.phone]);
    }
    if !input.company.is_empty() {
        contact["companyName"] = json!(&input.company);
    }
    if !input.job_title.is_empty() {
        contact["jobTitle"] = json!(&input.job_title);
    }
    let data = graph.post_json("/me/contacts", contact).await?;
    Ok(GraphItem::new(data))
}

pub async fn update_contact(
    graph: &GraphClient,
    input: UpdateContactInput,
) -> Result<GraphItem, GraphError> {
    let path = contact_path(&input.contact_id)?;
    let mut changes = json!({});
    if let Some(given_name) = &input.given_name {
        changes["givenName"] = json!(given_name);
    }
    if let Some(surname) = &input.surname {
        changes["surname"] = json!(surname);
    }
    if let Some(email) = &input.email {
        changes["emailAddresses"] = json!([{ "address": email }]);
    }
    if let Some(phone) = &input.phone {
        changes["businessPhones"] = json!([phone]);
    }
    let data = graph.patch_json(&path, changes).await?;
    Ok(GraphItem::new(data))
}

pub async fn delete_contact(
    graph: &GraphClient,
    input: ContactIdInput,
) -> Result<GraphItem, GraphError> {
    let path = contact_path(&input.contact_id)?;
    graph.delete(&path).await?;
    let done = json!({ "deleted": true, "contact_id": &input.contact_id });
    Ok(GraphItem::new(done))
}
