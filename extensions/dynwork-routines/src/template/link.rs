//! A task's link to a template and what the template locks on it.
//!
//! The task's definition is `DynWork`'s and stays opaque here: only the paths
//! the contract locks are read from it, exactly the set `DynWork`'s
//! `RoutinesTemplates.locked()` names: `instructions`, `agent`,
//! `options.connectors` and `options.permissions` always, `model` when the
//! template locks it, the cron fields its schedule lock fixes, and
//! `options.timeLimit` when it has a ceiling. Permissions and the time limit
//! are ceilings: a task may stay under them.

use serde::Deserialize;
// JSON: protocol boundary — the definition is DynWork's; only the locked
// paths are read, and the defaults are written in its shape.
use serde_json::{Value, json};

use super::{Permissions, ScheduleLock, TemplateBody};
use crate::ids::TemplateId;

/// `definition.template`: the template a task was activated from and the
/// revision it follows.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Link {
    pub id: TemplateId,
    pub revision: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("definition.template is {{id, revision}} with a positive revision")]
pub struct BadLink;

pub fn link(definition: &Value) -> Result<Option<Link>, BadLink> {
    let Some(raw) = definition.get("template").filter(|raw| !raw.is_null()) else {
        return Ok(None);
    };
    let link = Link::deserialize(raw).map_err(|error| {
        tracing::debug!(%error, "definition.template is not a link");
        BadLink
    })?;
    if link.revision < 1 {
        return Err(BadLink);
    }
    Ok(Some(link))
}

// JSON: protocol boundary — reads the locked paths of DynWork's definition.
pub fn keeps_locks(body: &TemplateBody, definition: &Value) -> bool {
    let template = &body.definition;
    let option = |key: &str| {
        definition
            .get("options")
            .and_then(|options| options.get(key))
    };
    let instructions = serde_json::to_value(&template.instructions)
        .is_ok_and(|own| definition.get("instructions") == Some(&own));
    let connectors = option("connectors").map_or(template.options.connectors.is_empty(), |names| {
        *names == json!(template.options.connectors)
    });
    let permissions = template.options.permissions == Permissions::Bypass
        || option("permissions").and_then(Value::as_str) != Some("bypass");
    let model = !body.locks.model || same_text(definition.get("model"), template.model.as_deref());
    let time_limit = body.locks.time_limit_max.is_none_or(|max| {
        option("timeLimit")
            .and_then(Value::as_f64)
            .is_some_and(|limit| limit <= f64::from(max))
    });
    instructions
        && same_text(definition.get("agent"), template.agent.as_deref())
        && connectors
        && permissions
        && model
        && time_limit
        && cron_kept(body.locks.schedule, &template.schedule.cron, definition)
}

// JSON: protocol boundary — writes a definition in DynWork's shape.
// Why: a mandatory template's routine starts as the template's defaults,
// linked to its revision, in no time zone: the desktop that claims it adds
// its own. A time limit with a ceiling and no default starts at the ceiling,
// as DynWork's `defaults()` makes it.
pub fn task_defaults(
    id: &TemplateId,
    revision: i64,
    body: &TemplateBody,
) -> Result<Value, serde_json::Error> {
    let template = &body.definition;
    let mut options = serde_json::to_value(&template.options)?;
    if let (None, Some(max), Some(options)) = (
        template.options.time_limit,
        body.locks.time_limit_max,
        options.as_object_mut(),
    ) {
        options.insert("timeLimit".to_owned(), json!(max));
    }
    let instructions = serde_json::to_value(&template.instructions)?;
    let mut task = json!({
        "name": body.name,
        "enabled": true,
        "instructions": instructions,
        "schedule": { "cron": template.schedule.cron },
        "options": options,
        "template": { "id": id.as_str(), "revision": revision },
    });
    if let Some(agent) = &template.agent {
        task["agent"] = json!(agent);
    }
    if let Some(model) = &template.model {
        task["model"] = json!(model);
    }
    Ok(task)
}

// JSON: protocol boundary — one text field of DynWork's definition.
fn same_text(value: Option<&Value>, own: Option<&str>) -> bool {
    match (value, own) {
        (None | Some(Value::Null), None) => true,
        (Some(Value::String(text)), Some(own)) => text == own,
        _ => false,
    }
}

// JSON: protocol boundary — the cron of DynWork's definition.
// Why: the cron fields are minute, hour, day, month and weekday; a time lock
// fixes the last three and leaves the time of day to the person.
fn cron_kept(lock: ScheduleLock, own: &str, definition: &Value) -> bool {
    let first_fixed = match lock {
        ScheduleLock::Free => return true,
        ScheduleLock::Time => 2,
        ScheduleLock::Locked => 0,
    };
    let Some(cron) = definition
        .get("schedule")
        .and_then(|schedule| schedule.get("cron"))
        .and_then(Value::as_str)
    else {
        return false;
    };
    let mine: Vec<&str> = cron.split_whitespace().collect();
    let theirs: Vec<&str> = own.split_whitespace().collect();
    (first_fixed..5).all(|field| mine.get(field) == theirs.get(field))
}
