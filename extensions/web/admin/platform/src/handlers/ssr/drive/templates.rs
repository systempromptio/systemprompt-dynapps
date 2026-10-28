//! The install's document templates, read from
//! `<services>/documents/templates/*.yaml` for the create dialog. A catalog
//! file the server loaded at boot always parses here, so a failure is a
//! fault in the page, not a template left out of the dialog.

use std::path::Path;

use serde::{Deserialize, Serialize};
use systemprompt_web_admin_core::error::AdminError;

#[derive(Debug, Deserialize)]
struct TemplateFile {
    id: String,
    format: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    placeholders: Vec<PlaceholderView>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct PlaceholderView {
    key: String,
    label: String,
    #[serde(default)]
    default: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct TemplateView {
    id: String,
    format: String,
    name: String,
    description: String,
    placeholders: Vec<PlaceholderView>,
}

pub(super) fn install_templates(services_root: &Path) -> Result<Vec<TemplateView>, AdminError> {
    let dir = services_root.join("documents").join("templates");
    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(AdminError::internal(error)),
    };
    let mut templates = Vec::new();
    for entry in entries {
        let path = entry.map_err(AdminError::internal)?.path();
        if path.extension().is_some_and(|ext| ext == "yaml") {
            templates.push(read_template(&path)?);
        }
    }
    templates.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(templates)
}

fn read_template(path: &Path) -> Result<TemplateView, AdminError> {
    let text = std::fs::read_to_string(path).map_err(AdminError::internal)?;
    let file: TemplateFile = serde_yaml::from_str(&text).map_err(AdminError::internal)?;
    Ok(TemplateView {
        id: file.id,
        format: file.format,
        name: file.name,
        description: file.description,
        placeholders: file.placeholders,
    })
}
