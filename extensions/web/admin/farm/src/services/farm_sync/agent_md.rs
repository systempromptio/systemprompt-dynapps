//! `agents/<agent_id>.md`: YAML frontmatter between `---` lines, then the
//! system prompt body. Unknown frontmatter keys are an error.

use std::path::Path;

use serde::{Deserialize, Serialize};

use super::validate::{
    ALLOWED_TOOLS, PERMISSION_MODES, definition_error, is_identifier, is_safe_relative_path,
    require,
};
use crate::error::FarmResult;

pub const FIXTURE_PATH_PLACEHOLDER: &str = "{{fixture_path}}";
pub const CASE_PROMPT_PLACEHOLDER: &str = "{{case.prompt}}";
pub const DEFAULT_HOST: &str = "claude-code";
pub const HOSTS: [&str; 4] = [DEFAULT_HOST, "codex-cli", "opencode", "hermes"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentOutput {
    pub path: String,
    pub format: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct Frontmatter {
    name: String,
    description: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    host: Option<String>,
    tools: Vec<String>,
    plugin: String,
    #[serde(default)]
    skills: Vec<String>,
    entry_prompt: String,
    fixture: String,
    #[serde(default)]
    max_turns: Option<u32>,
    #[serde(default)]
    max_budget_usd: Option<f64>,
    #[serde(default)]
    permission_mode: Option<String>,
    #[serde(default)]
    output: Option<AgentOutput>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AgentDefinition {
    pub name: String,
    pub description: String,
    pub model: Option<String>,
    pub host: String,
    pub tools: Vec<String>,
    pub plugin: String,
    pub skills: Vec<String>,
    pub entry_prompt: String,
    pub fixture: String,
    pub max_turns: Option<u32>,
    pub max_budget_usd: Option<f64>,
    pub permission_mode: Option<String>,
    pub output: Option<AgentOutput>,
    pub source_path: String,
    pub body: String,
}

fn split_frontmatter<'a>(path: &str, text: &'a str) -> FarmResult<(&'a str, &'a str)> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))
        .ok_or_else(|| definition_error(path, "must open with a `---` frontmatter line"))?;
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            let yaml = rest.get(..offset).unwrap_or_default();
            let body = rest.get(offset + line.len()..).unwrap_or_default();
            return Ok((yaml, body));
        }
        offset += line.len();
    }
    Err(definition_error(
        path,
        "frontmatter has no closing `---` line",
    ))
}

fn check_placeholders(path: &str, template: &str) -> FarmResult<()> {
    let mut rest = template;
    while let Some(start) = rest.find("{{") {
        let tail = rest.get(start..).unwrap_or_default();
        let end = tail
            .find("}}")
            .ok_or_else(|| definition_error(path, "entry_prompt has an unclosed `{{`"))?;
        let token = tail.get(..end + 2).unwrap_or_default();
        require(
            token == FIXTURE_PATH_PLACEHOLDER || token == CASE_PROMPT_PLACEHOLDER,
            path,
            format!(
                "entry_prompt placeholder {token:?} is not {FIXTURE_PATH_PLACEHOLDER} or \
                 {CASE_PROMPT_PLACEHOLDER}"
            ),
        )?;
        rest = tail.get(end + 2..).unwrap_or_default();
    }
    Ok(())
}

fn validate(path: &str, stem: &str, fm: &Frontmatter) -> FarmResult<()> {
    require(
        is_identifier(&fm.name),
        path,
        format!("name {:?} does not match ^[a-z][a-z0-9_]{{2,63}}$", fm.name),
    )?;
    require(
        fm.name == stem,
        path,
        format!("name {:?} must equal the file stem {stem:?}", fm.name),
    )?;
    require(
        !fm.description.trim().is_empty(),
        path,
        "description is empty",
    )?;
    if let Some(host) = &fm.host {
        require(
            HOSTS.contains(&host.as_str()),
            path,
            format!("agent {:?}: host {host:?} is not one of {HOSTS:?}", fm.name),
        )?;
    }
    require(!fm.tools.is_empty(), path, "tools must be a non-empty list")?;
    if let Some(tool) = fm
        .tools
        .iter()
        .find(|t| !ALLOWED_TOOLS.contains(&t.as_str()))
    {
        return Err(definition_error(
            path,
            format!("tool {tool:?} is not one of {ALLOWED_TOOLS:?}"),
        ));
    }
    require(!fm.plugin.trim().is_empty(), path, "plugin is empty")?;
    require(
        is_identifier(&fm.fixture),
        path,
        format!("fixture {:?} is not a valid fixture name", fm.fixture),
    )?;
    require(
        !fm.entry_prompt.trim().is_empty(),
        path,
        "entry_prompt is empty",
    )?;
    check_placeholders(path, &fm.entry_prompt)?;
    require(fm.max_turns != Some(0), path, "max_turns must be > 0")?;
    require(
        fm.max_budget_usd.is_none_or(|b| b.is_finite() && b > 0.0),
        path,
        "max_budget_usd must be > 0",
    )?;
    if let Some(mode) = &fm.permission_mode {
        require(
            PERMISSION_MODES.contains(&mode.as_str()),
            path,
            format!("permission_mode {mode:?} is not one of {PERMISSION_MODES:?}"),
        )?;
    }
    if let Some(output) = &fm.output {
        require(
            is_safe_relative_path(&output.path),
            path,
            format!(
                "output.path {:?} must be relative without `..`",
                output.path
            ),
        )?;
        require(
            output.format == "markdown" || output.format == "json",
            path,
            format!("output.format {:?} is not markdown or json", output.format),
        )?;
    }
    Ok(())
}

pub fn parse_agent_markdown(path: &str, text: &str) -> FarmResult<AgentDefinition> {
    let stem = Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let (yaml, body) = split_frontmatter(path, text)?;
    let fm: Frontmatter = serde_yaml::from_str(yaml).map_err(|e| definition_error(path, e))?;
    validate(path, stem, &fm)?;
    Ok(AgentDefinition {
        name: fm.name,
        description: fm.description.trim().to_owned(),
        model: fm.model,
        host: fm.host.unwrap_or_else(|| DEFAULT_HOST.to_owned()),
        tools: fm.tools,
        plugin: fm.plugin,
        skills: fm.skills,
        entry_prompt: fm.entry_prompt,
        fixture: fm.fixture,
        max_turns: fm.max_turns,
        max_budget_usd: fm.max_budget_usd,
        permission_mode: fm.permission_mode,
        output: fm.output,
        source_path: path.to_owned(),
        body: body.trim().to_owned(),
    })
}

#[must_use]
pub fn render_entry_prompt(
    template: &str,
    fixture_path: &str,
    case_prompt: Option<&str>,
) -> String {
    template
        .replace(FIXTURE_PATH_PLACEHOLDER, fixture_path)
        .replace(CASE_PROMPT_PLACEHOLDER, case_prompt.unwrap_or_default())
}
