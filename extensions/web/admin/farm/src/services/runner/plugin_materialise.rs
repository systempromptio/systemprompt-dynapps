//! Turns one plugin of the composed services tree into the plugin directory
//! Claude Code loads with `--plugin-dir`: a manifest plus one `SKILL.md`
//! directory per included skill.
//!
//! Kit hooks are left out; the run-local governance plugin is the one
//! reporter.

use std::path::{Path, PathBuf};
use systemprompt::identifiers::PluginId;

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::error::{FarmError, FarmResult};

#[derive(Debug, Default, Deserialize)]
struct PluginDoc {
    #[serde(default)]
    plugin: PluginSection,
}

#[derive(Debug, Default, Deserialize)]
struct PluginSection {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    skills: SkillsSection,
}

#[derive(Debug, Default, Deserialize)]
struct SkillsSection {
    #[serde(default)]
    include: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PluginManifest<'a> {
    name: &'a str,
    description: &'a str,
    version: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialisedPlugin {
    pub dir: PathBuf,
    pub skills: Vec<String>,
    pub missing_skills: Vec<String>,
}

pub fn materialise_plugin(
    services_root: &Path,
    plugin_id: &PluginId,
    out_root: &Path,
) -> FarmResult<MaterialisedPlugin> {
    let plugin_id = plugin_id.as_str();
    let config = services_root
        .join("plugins")
        .join(plugin_id)
        .join("config.yaml");
    let raw = std::fs::read_to_string(&config).map_err(|source| FarmError::PluginConfigRead {
        plugin_id: plugin_id.to_owned(),
        path: config.display().to_string(),
        source,
    })?;
    let doc: PluginDoc =
        serde_yaml::from_str(&raw).map_err(|source| FarmError::PluginConfigParse {
            plugin_id: plugin_id.to_owned(),
            source,
        })?;
    let section = doc.plugin;

    let dir = out_root.join(plugin_id);
    let manifest_dir = dir.join(".claude-plugin");
    std::fs::create_dir_all(&manifest_dir)?;
    let manifest = PluginManifest {
        name: plugin_id,
        description: section
            .description
            .as_deref()
            .or(section.name.as_deref())
            .unwrap_or(plugin_id),
        version: section.version.as_deref().unwrap_or("0.0.0"),
    };
    let json = serde_json::to_vec_pretty(&manifest).map_err(|source| FarmError::Json {
        at: "plugin manifest".to_owned(),
        source,
    })?;
    std::fs::write(manifest_dir.join("plugin.json"), json)?;

    let mut skills = Vec::new();
    let mut missing_skills = Vec::new();
    for skill_id in &section.skills.include {
        let source = services_root.join("skills").join(skill_id);
        if !source.join("SKILL.md").is_file() {
            missing_skills.push(skill_id.clone());
            continue;
        }
        let dashed = skill_id.replace('_', "-");
        copy_skill(&source, &dir.join("skills").join(&dashed))?;
        skills.push(dashed);
    }
    Ok(MaterialisedPlugin {
        dir,
        skills,
        missing_skills,
    })
}

fn copy_skill(source: &Path, target: &Path) -> FarmResult<()> {
    for entry in WalkDir::new(source).follow_links(false) {
        let entry = entry.map_err(|err| FarmError::Io(std::io::Error::other(err)))?;
        let Ok(relative) = entry.path().strip_prefix(source) else {
            continue;
        };
        if relative.as_os_str().is_empty() || relative == Path::new("config.yaml") {
            continue;
        }
        let dest = target.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&dest)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}
