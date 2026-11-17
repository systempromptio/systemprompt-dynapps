//! A scheduled task template exactly as `DynWork` receives it, and what an
//! admin writes to make one.
//!
//! The types follow the wire contract field for field and refuse any field it
//! does not name, so a timezone, a folder, a task id or a typo never reaches
//! the library: those are personal and stay on the person's computer.
//! Publishing checks the rest ([`check_publishable`]).

mod check;
mod state;

use serde::{Deserialize, Serialize};
// JSON: protocol boundary — `recover` and `catchUp` are DynWork's option
// defaults, passed through as the admin wrote them.
use serde_json::Value;

use crate::ids::TemplateId;

pub use check::{Refusal, check_draft, check_publishable};
pub use state::{LinkState, RetireMode, TemplateSource, TemplateState, UnknownValue};

/// What an admin saves or publishes: the template's id, whether every person
/// who sees it gets it, the anchor plugins whose access rules decide who sees
/// it, and the part the desktop receives.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateDraft {
    pub id: TemplateId,
    pub mandatory: bool,
    pub plugins: Vec<String>,
    pub body: TemplateBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateBody {
    pub name: String,
    pub description: String,
    #[serde(rename = "where")]
    pub place: Place,
    pub definition: TemplateDefinition,
    pub locks: Locks,
    pub personal: Personal,
}

/// Where a task from the template runs. Only the person's own computer here;
/// templates for the cloud come later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Place {
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateDefinition {
    pub instructions: Instructions,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub schedule: Schedule,
    pub options: TemplateOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Instructions {
    Prompt { text: String },
    Skill { skill: String, text: String },
}

/// The template's schedule. Never a timezone: that is the person's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schedule {
    pub cron: String,
}

// JSON: protocol boundary — `recover` ("recent" | "none") and `catchUp` (hours)
// are DynWork's to interpret; Governance stores and serves them as sent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TemplateOptions {
    pub permissions: Permissions,
    #[serde(default)]
    pub connectors: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_limit: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recover: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catch_up: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notify: Option<String>,
}

/// `Bypass` is `DynWork`'s "Don't ask": a task that runs without asking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Permissions {
    Ask,
    Bypass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Locks {
    pub model: bool,
    pub schedule: ScheduleLock,
    pub time_limit_max: Option<u32>,
}

/// `Time`: the frequency is fixed (the cron's day, month and weekday fields),
/// the minute and hour are the person's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleLock {
    Free,
    Time,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Personal {
    pub folder: FolderNeed,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FolderNeed {
    None,
    Required,
}

/// A template as the desktop receives it: the published body with its id,
/// revision and whether it is mandatory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TemplateView {
    pub id: TemplateId,
    pub revision: i64,
    pub mandatory: bool,
    #[serde(flatten)]
    pub body: TemplateBody,
}
