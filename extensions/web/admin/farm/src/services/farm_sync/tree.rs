//! Parses a captured farm directory as a whole.
//!
//! Every definition file is parsed, then the references between them
//! (manifest ↔ agent files, agent and case fixtures, suite agents) are
//! checked, so a farm is accepted or refused in one piece.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::agent_md::{AgentDefinition, parse_agent_markdown};
use super::capture::CapturedFile;
use super::eval_yaml::{EvalSuiteDefinition, parse_eval_suite};
use super::farm_yaml::{FARM_MANIFEST, FarmManifest, check_agents_listed, parse_farm_manifest};
use super::fixtures::{FixtureFile, group_fixtures};
use super::validate::{definition_error, require};
use crate::error::{FarmError, FarmResult};

pub const MAX_FARM_FILES: usize = 256;
pub const MAX_FARM_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedFarm {
    pub manifest: FarmManifest,
    pub agents: Vec<AgentDefinition>,
    pub suites: Vec<EvalSuiteDefinition>,
    pub fixtures: BTreeMap<String, Vec<FixtureFile>>,
    pub warnings: Vec<String>,
}

fn text<'a>(path: &str, file: &'a CapturedFile) -> FarmResult<&'a str> {
    std::str::from_utf8(&file.bytes).map_err(|source| FarmError::NotUtf8 {
        path: path.to_owned(),
        source,
    })
}

fn has_extension(name: &str, ext: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

fn stem(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_owned()
}

fn check_bounds(files: &BTreeMap<String, CapturedFile>) -> FarmResult<()> {
    let total: usize = files.values().map(|f| f.bytes.len()).sum();
    require(
        files.len() <= MAX_FARM_FILES && total <= MAX_FARM_BYTES,
        "farm/",
        format!(
            "{} files / {total} bytes exceeds {MAX_FARM_FILES} files / 8 MiB",
            files.len()
        ),
    )
}

fn check_references(parsed: &ParsedFarm) -> FarmResult<()> {
    let agent_names: BTreeSet<&str> = parsed.agents.iter().map(|a| a.name.as_str()).collect();
    for agent in &parsed.agents {
        require(
            parsed.fixtures.contains_key(&agent.fixture),
            &agent.source_path,
            format!(
                "fixture {:?} has no fixtures/{}/ directory",
                agent.fixture, agent.fixture
            ),
        )?;
    }
    for suite in &parsed.suites {
        require(
            agent_names.contains(suite.agent.as_str()),
            &suite.source_path,
            format!("agent {:?} has no agents/{}.md", suite.agent, suite.agent),
        )?;
        require(
            stem(&suite.source_path) == suite.agent,
            &suite.source_path,
            format!("agent {:?} must equal the file stem", suite.agent),
        )?;
        for case in &suite.cases {
            if let Some(fixture) = &case.fixture {
                require(
                    parsed.fixtures.contains_key(fixture),
                    &suite.source_path,
                    format!("case {:?}: fixture {fixture:?} does not exist", case.id),
                )?;
            }
        }
    }
    Ok(())
}

pub fn parse_farm_tree(files: &BTreeMap<String, CapturedFile>) -> FarmResult<ParsedFarm> {
    check_bounds(files)?;
    let manifest_file = files
        .get(FARM_MANIFEST)
        .ok_or_else(|| definition_error(FARM_MANIFEST, "is missing"))?;
    let manifest = parse_farm_manifest(text(FARM_MANIFEST, manifest_file)?)?;
    let mut agents = Vec::new();
    let mut suites = Vec::new();
    let mut warnings = Vec::new();
    for (path, file) in files {
        if let Some(name) = path.strip_prefix("agents/") {
            if has_extension(name, "md") && !name.contains('/') {
                agents.push(parse_agent_markdown(path, text(path, file)?)?);
            } else {
                warnings.push(format!("{path}: ignored, agents/ holds only <id>.md files"));
            }
        } else if let Some(name) = path.strip_prefix("evals/") {
            if has_extension(name, "yaml") && !name.contains('/') {
                suites.push(parse_eval_suite(path, text(path, file)?)?);
            } else {
                warnings.push(format!(
                    "{path}: ignored, evals/ holds only <agent>.yaml files"
                ));
            }
        }
    }
    let agent_files: BTreeSet<String> = agents.iter().map(|a| a.name.clone()).collect();
    check_agents_listed(&manifest, &agent_files)?;
    let with_suites: BTreeSet<&str> = suites.iter().map(|s| s.agent.as_str()).collect();
    warnings.extend(
        agents
            .iter()
            .filter(|a| !with_suites.contains(a.name.as_str()))
            .map(|a| format!("agent {:?} has no eval suite", a.name)),
    );
    let parsed = ParsedFarm {
        manifest,
        agents,
        suites,
        fixtures: group_fixtures(files)?,
        warnings,
    };
    check_references(&parsed)?;
    Ok(parsed)
}
