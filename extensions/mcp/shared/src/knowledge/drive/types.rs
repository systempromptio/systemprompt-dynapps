//! Google Drive wire models and stable project assignment.

use super::config::DriveSource;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A Google Drive file identifier as the Drive API and the inventory tables
/// carry it.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, sqlx::Type,
)]
#[serde(transparent)]
#[sqlx(transparent)]
pub struct FileId(String);

impl FileId {
    #[must_use]
    pub const fn new(id: String) -> Self {
        Self(id)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub const FOLDER_MIME: &str = "application/vnd.google-apps.folder";
pub const SHORTCUT_MIME: &str = "application/vnd.google-apps.shortcut";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriveFile {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub parents: Vec<String>,
    pub drive_id: Option<String>,
    pub modified_time: Option<DateTime<Utc>>,
    pub version: Option<String>,
    pub md5_checksum: Option<String>,
    pub size: Option<String>,
    pub web_view_link: Option<String>,
    #[serde(default)]
    pub trashed: bool,
    pub shortcut_details: Option<ShortcutDetails>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutDetails {
    pub target_id: String,
    pub target_mime_type: Option<String>,
}

impl DriveFile {
    pub fn version_key(&self) -> String {
        if let Some(checksum) = &self.md5_checksum {
            return checksum.clone();
        }
        format!(
            "{}:{}:{}",
            self.version.as_deref().unwrap_or(""),
            self.md5_checksum.as_deref().unwrap_or(""),
            self.modified_time
                .map(|t| t.to_rfc3339())
                .unwrap_or_default()
        )
    }
    pub fn uri(&self) -> String {
        self.web_view_link
            .clone()
            .unwrap_or_else(|| format!("https://drive.google.com/file/d/{}/view", self.id))
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePage {
    #[serde(default)]
    pub files: Vec<DriveFile>,
    pub next_page_token: Option<String>,
    #[serde(default)]
    pub incomplete_search: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePage {
    #[serde(default)]
    pub changes: Vec<Change>,
    pub next_page_token: Option<String>,
    pub new_start_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub file_id: Option<FileId>,
    #[serde(default)]
    pub removed: bool,
    pub file: Option<DriveFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProjectAssignment {
    pub id: String,
    pub name: String,
    pub path: String,
}

pub fn assign_project<S: std::hash::BuildHasher>(
    file: &DriveFile,
    files: &HashMap<String, DriveFile, S>,
    source: &DriveSource,
) -> ProjectAssignment {
    let mut ancestry = Vec::new();
    let mut seen = HashSet::new();
    let mut parent = file.parents.first();
    while let Some(id) = parent {
        if id == &source.drive_id || !seen.insert(id.clone()) {
            break;
        }
        let Some(folder) = files.get(id) else {
            break;
        };
        ancestry.push(folder);
        parent = folder.parents.first();
    }
    ancestry.reverse();
    let path = ancestry
        .iter()
        .map(|f| f.name.as_str())
        .chain(std::iter::once(file.name.as_str()))
        .collect::<Vec<_>>()
        .join("/");
    let root = ancestry
        .first()
        .filter(|f| f.parents.contains(&source.drive_id));
    if root.is_some_and(|f| f.name == source.shared_folder) {
        return ProjectAssignment {
            id: "shared".to_owned(),
            name: "Shared".to_owned(),
            path,
        };
    }
    if root.is_some_and(|f| f.name == source.projects_folder)
        && let Some(project) = ancestry.get(1)
    {
        return ProjectAssignment {
            id: project.id.clone(),
            name: project.name.clone(),
            path,
        };
    }
    ProjectAssignment {
        id: "unassigned".to_owned(),
        name: "Unassigned".to_owned(),
        path,
    }
}

pub fn export_format(mime: &str) -> Option<(&'static str, &'static str)> {
    match mime {
        "application/vnd.google-apps.document" => Some((
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "docx",
        )),
        "application/vnd.google-apps.spreadsheet" => Some((
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "xlsx",
        )),
        "application/vnd.google-apps.presentation" => Some((
            "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "pptx",
        )),
        "application/vnd.google-apps.drawing" => Some(("image/png", "png")),
        _ => None,
    }
}


pub fn has_removed_ancestor<S: std::hash::BuildHasher, T: std::hash::BuildHasher>(
    file: &DriveFile,
    files: &HashMap<String, DriveFile, S>,
    removed: &HashSet<String, T>,
) -> bool {
    let mut parent = file.parents.first();
    let mut seen = HashSet::new();
    while let Some(id) = parent {
        if removed.contains(id) {
            return true;
        }
        if !seen.insert(id) {
            break;
        }
        parent = files.get(id).and_then(|folder| folder.parents.first());
    }
    false
}
