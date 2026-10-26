//! The shipped catalogue holds together end to end: every marketplace,
//! plugin, skill and MCP server id resolves, every one of them is covered by
//! `services/access-control/rules.yaml` (directly, or by inheritance from the
//! parent that carries it), every skill directory ships its `SKILL.md` with
//! frontmatter, and nothing removed with the Salesforce, Atlassian, GitHub
//! MCP and Astound clean-up has crept back into the config.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use systemprompt::loader::ConfigLoader;
use systemprompt::manifest::services::ServicesConfig;

use crate::support::repo_root;

const REMOVED_TOKENS: &[&str] = &["atlassian", "jira", "confluence", "salesforce", "astound"];

const REMOVED_IDS: &[&str] = &[
    "dynapps-salesforce",
    "github",
    "account_context",
    "account_tiering",
    "admin_ai_usage",
    "admin_critical_projects",
    "admin_daily_brief",
    "admin_person_activity",
    "calendar_events",
    "call_follow_up",
    "call_prep",
    "close_plan",
    "customer_health",
    "customer_voice",
    "daily_briefing",
    "deal_advance_gap",
    "deal_review",
    "deal_signals",
    "deal_slip_scenario",
    "draft_outreach",
    "expansion_whitespace",
    "find_internal_answer",
    "forecast_narrative",
    "handle_objection",
    "inbox_sweep",
    "lead_triage",
    "pipeline_review",
    "query_activity_history",
    "renewal_radar",
    "rep_context",
    "research_prospect",
    "salesforce_hygiene_check",
    "stakeholder_map",
    "team_pipeline",
    "using_display_widget",
    "voice_profile",
    "weekly_wrap",
    "win_loss_review",
];

fn services_dir() -> PathBuf {
    repo_root().join("services")
}

fn services() -> ServicesConfig {
    ConfigLoader::load_from_path(&services_dir().join("config/config.yaml"), None)
        .unwrap_or_else(|e| panic!("services tree could not be composed: {e}"))
}

fn yaml(path: &Path) -> serde_yaml::Value {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()))
}

fn subdirs(relative: &str) -> Vec<(String, PathBuf)> {
    let dir = services_dir().join(relative);
    let mut out: Vec<(String, PathBuf)> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .map(|p| (p.file_name().unwrap().to_string_lossy().into_owned(), p))
        .collect();
    out.sort();
    out
}

fn includes(doc: &serde_yaml::Value, root: &str, section: &str) -> Vec<String> {
    doc[root][section]["include"]
        .as_sequence()
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn declared_entities() -> BTreeSet<String> {
    let doc = yaml(&services_dir().join("access-control/rules.yaml"));
    doc["entities"]
        .as_sequence()
        .expect("rules.yaml has an entities list")
        .iter()
        .filter_map(|e| e["entity"].as_str().map(str::to_owned))
        .collect()
}

#[test]
fn every_catalogue_reference_resolves() {
    services()
        .validate()
        .unwrap_or_else(|e| panic!("services tree does not validate: {e:?}"));
}

#[test]
fn every_entity_is_covered_by_rules_yaml() {
    let declared = declared_entities();
    let services = services();
    for id in services.marketplaces.keys() {
        let key = format!("marketplace/{}", id.as_str());
        assert!(declared.contains(&key), "{key} has no block in rules.yaml");
    }
    for id in services.mcp_servers.keys() {
        let key = format!("mcp_server/{id}");
        assert!(declared.contains(&key), "{key} has no block in rules.yaml");
    }

    let mut covered_plugins: BTreeSet<String> = BTreeSet::new();
    for (id, dir) in subdirs("marketplaces") {
        if declared.contains(&format!("marketplace/{id}")) {
            let doc = yaml(&dir.join("config.yaml"));
            covered_plugins.extend(includes(&doc, "marketplace", "plugins"));
        }
    }
    let mut covered_skills: BTreeSet<String> = BTreeSet::new();
    for (id, dir) in subdirs("plugins") {
        let own = declared.contains(&format!("plugin/{id}"));
        assert!(
            own || covered_plugins.contains(&id),
            "plugin/{id} has no rules.yaml block and no covered marketplace carries it"
        );
        let doc = yaml(&dir.join("config.yaml"));
        covered_skills.extend(includes(&doc, "plugin", "skills"));
    }
    for (id, _) in subdirs("skills") {
        assert!(
            declared.contains(&format!("skill/{id}")) || covered_skills.contains(&id),
            "skill/{id} has no rules.yaml block and no plugin carries it"
        );
    }
}

#[test]
fn every_skill_dir_ships_skill_md_with_frontmatter() {
    for (id, dir) in subdirs("skills") {
        assert!(
            dir.join("config.yaml").is_file(),
            "skills/{id} has no config.yaml"
        );
        let path = dir.join("SKILL.md");
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("skills/{id}/SKILL.md: {e}"));
        let body = text
            .strip_prefix("---\n")
            .unwrap_or_else(|| panic!("skills/{id}/SKILL.md does not open with frontmatter"));
        let end = body
            .find("\n---")
            .unwrap_or_else(|| panic!("skills/{id}/SKILL.md frontmatter is not closed"));
        let front: serde_yaml::Value = serde_yaml::from_str(&body[..end])
            .unwrap_or_else(|e| panic!("skills/{id}/SKILL.md frontmatter: {e}"));
        for field in ["name", "description"] {
            assert!(
                front[field].as_str().is_some_and(|v| !v.trim().is_empty()),
                "skills/{id}/SKILL.md frontmatter lacks `{field}`"
            );
        }
    }
}

fn yaml_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .flatten()
    {
        let path = entry.path();
        if path.is_dir() {
            yaml_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "yaml" || e == "yml") {
            out.push(path);
        }
    }
}

#[test]
fn removed_ids_appear_nowhere_in_services_config() {
    let mut files = Vec::new();
    yaml_files(&services_dir(), &mut files);
    assert!(!files.is_empty(), "no services YAML found");
    let mut hits = Vec::new();
    for path in &files {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let lower = text.to_lowercase();
        for token in REMOVED_TOKENS {
            if lower.contains(token) {
                hits.push(format!("{}: {token}", path.display()));
            }
        }
        // Exact identifier match, so `indaws_call_prep` does not trip `call_prep`
        // and prose mentioning the GitHub farm sync is not an MCP id.
        let idents: BTreeSet<&str> = text
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '/'))
            .collect();
        for id in REMOVED_IDS.iter().filter(|id| **id != "github") {
            for form in [
                (*id).to_owned(),
                format!("skill/{id}"),
                format!("plugin/{id}"),
            ] {
                if idents.contains(form.as_str()) {
                    hits.push(format!("{}: {form}", path.display()));
                }
            }
        }
        if idents.contains("mcp_server/github") {
            hits.push(format!("{}: mcp_server/github", path.display()));
        }
    }
    assert!(
        hits.is_empty(),
        "removed ids still referenced:\n{}",
        hits.join("\n")
    );
    let services = services();
    assert!(
        !services.mcp_servers.contains_key("github"),
        "the github MCP server is back"
    );
}
