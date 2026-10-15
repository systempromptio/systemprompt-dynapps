//! Farm definition parsers against the real agent-farm kit files (copied
//! into `fixtures/agent-farm/`), plus the refusals `FARM.md` requires.

use std::collections::BTreeMap;
use std::path::PathBuf;

use systemprompt_web_admin_farm::services::farm_sync::{
    Assertion, CapturedFile, FixtureFile, check_agents_listed, fixture_manifest, group_fixtures,
    materialise_fixture, parse_agent_markdown, parse_eval_suite, parse_farm_manifest,
    parse_farm_tree, render_entry_prompt,
};

fn kit_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/agent-farm")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(kit_dir().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn files_in(dir: &str) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(kit_dir().join(dir))
        .unwrap_or_else(|e| panic!("read_dir {dir}: {e}"))
        .flatten()
        .map(|e| format!("{dir}/{}", e.file_name().to_string_lossy()))
        .collect();
    names.sort();
    names
}

fn captured(text: &str) -> CapturedFile {
    CapturedFile {
        bytes: text.as_bytes().to_vec(),
        executable: false,
    }
}

fn kit_tree() -> BTreeMap<String, CapturedFile> {
    let mut tree = BTreeMap::new();
    tree.insert("farm.yaml".to_owned(), captured(&read("farm.yaml")));
    for path in files_in("agents").into_iter().chain(files_in("evals")) {
        tree.insert(path.clone(), captured(&read(&path)));
    }
    tree.insert("fixtures/README.md".to_owned(), captured("ground truth"));
    tree.insert(
        "fixtures/night_audit/README.txt".to_owned(),
        captured("seed"),
    );
    tree
}

const AGENT: &str = "---\nname: abc_agent\ndescription: d\ntools: [Read]\nplugin: p\n\
entry_prompt: \"go {{fixture_path}} {{case.prompt}}\"\nfixture: fix_one\n---\nBody text\n";

#[test]
fn every_kit_agent_parses() {
    let agents = files_in("agents");
    assert_eq!(agents.len(), 1);
    for path in agents {
        let agent = parse_agent_markdown(&path, &read(&path)).unwrap_or_else(|e| panic!("{e}"));
        assert!(path.ends_with(&format!("{}.md", agent.name)));
        assert!(!agent.tools.is_empty());
        assert!(!agent.body.is_empty());
        assert!(agent.output.is_some());
        assert!(agent.entry_prompt.contains("{{fixture_path}}"));
    }
}

#[test]
fn every_kit_eval_suite_parses() {
    let suites = files_in("evals");
    assert_eq!(suites.len(), 1);
    for path in suites {
        let suite = parse_eval_suite(&path, &read(&path)).unwrap_or_else(|e| panic!("{e}"));
        assert!(!suite.cases.is_empty());
    }
    let audit = parse_eval_suite(
        "evals/accounting_night_audit.yaml",
        &read("evals/accounting_night_audit.yaml"),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let full = &audit.cases[0];
    assert_eq!(full.id, "nightly_previous_month");
    assert_eq!(full.effective_judge_threshold(), Some(70));
    assert!(
        full.assertions
            .contains(&Assertion::NoToolOutsideAllowlist {})
    );
    assert!(full.assertions.contains(&Assertion::MaxTurns { value: 40 }));
}

#[test]
fn kit_manifest_and_tree_parse_together() {
    let manifest = parse_farm_manifest(&read("farm.yaml")).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(manifest.id, "dynapps-night-audit");
    assert_eq!(manifest.agents.len(), 1);
    let parsed = parse_farm_tree(&kit_tree()).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(parsed.agents.len(), 1);
    assert_eq!(parsed.suites.len(), 1);
    assert_eq!(parsed.fixtures.len(), 1);
    assert!(parsed.warnings.is_empty(), "{:?}", parsed.warnings);
}

#[test]
fn tree_refuses_unlisted_agent_and_missing_fixture() {
    let mut tree = kit_tree();
    tree.insert("agents/abc_agent.md".to_owned(), captured(AGENT));
    assert!(parse_farm_tree(&tree).is_err());
    let mut tree = kit_tree();
    tree.remove("fixtures/night_audit/README.txt");
    assert!(parse_farm_tree(&tree).is_err());
}

#[test]
fn agent_minimal_parses_and_renders() {
    let agent =
        parse_agent_markdown("agents/abc_agent.md", AGENT).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(agent.body, "Body text");
    assert_eq!(
        render_entry_prompt(&agent.entry_prompt, "/w", Some("now")),
        "go /w now"
    );
    assert_eq!(
        render_entry_prompt(&agent.entry_prompt, "/w", None),
        "go /w "
    );
}

#[test]
fn agent_refusals() {
    let cases = [
        AGENT.replace("fixture: fix_one", "fixture: fix_one\ncolour: red"),
        AGENT.replace("name: abc_agent", "name: other_agent"),
        AGENT.replace("name: abc_agent", "name: Ab"),
        AGENT.replace("tools: [Read]", "tools: []"),
        AGENT.replace("tools: [Read]", "tools: [Hammer]"),
        AGENT.replace("{{case.prompt}}", "{{case.other}}"),
        AGENT.replace("---\nBody", "Body"),
    ];
    for text in cases {
        assert!(
            parse_agent_markdown("agents/abc_agent.md", &text).is_err(),
            "{text}"
        );
    }
    let bad_stem = parse_agent_markdown("agents/x.md", AGENT);
    assert!(bad_stem.is_err());
}

const SUITE: &str = "schema: 1\nagent: abc_agent\ncases:\n  - id: one\n    assertions:\n      - type: file_exists\n        path: OUT.md\n";

#[test]
fn eval_refusals() {
    assert!(parse_eval_suite("evals/abc_agent.yaml", SUITE).is_ok());
    let cases = [
        SUITE.replace("path: OUT.md", "path: OUT.md\n        extra: 1"),
        SUITE.replace("type: file_exists", "type: file_absent"),
        SUITE.replace(
            "type: file_exists\n        path: OUT.md",
            "type: regex\n        pattern: \"(unclosed\"",
        ),
        SUITE.replace("path: OUT.md", "path: ../etc/passwd"),
        SUITE.replace("id: one", "id: One"),
        SUITE.replace("schema: 1", "schema: 2"),
        format!("{SUITE}  - id: one\n    assertions: []\n"),
        SUITE.replace(
            "    assertions:",
            "    judge_threshold: 101\n    assertions:",
        ),
        SUITE.replace("agent: abc_agent", "agent: abc_agent\nowner: me"),
    ];
    for text in cases {
        assert!(
            parse_eval_suite("evals/abc_agent.yaml", &text).is_err(),
            "{text}"
        );
    }
}

#[test]
fn manifest_refusals_and_listing() {
    let ok = "schema: 1\nid: my-farm\nmarketplace: m\nagents: [abc_agent]\n";
    let manifest = parse_farm_manifest(ok).unwrap_or_else(|e| panic!("{e}"));
    let listed = std::iter::once("abc_agent".to_owned()).collect();
    assert!(check_agents_listed(&manifest, &listed).is_ok());
    let extra = ["abc_agent".to_owned(), "def_agent".to_owned()]
        .into_iter()
        .collect();
    assert!(check_agents_listed(&manifest, &extra).is_err());
    for text in [
        ok.replace("id: my-farm", "id: My_Farm"),
        ok.replace("agents: [abc_agent]", "agents: [abc_agent, abc_agent]"),
        format!("{ok}color: blue\n"),
    ] {
        assert!(parse_farm_manifest(&text).is_err(), "{text}");
    }
}

#[test]
fn fixtures_group_skip_readme_and_materialise() {
    let mut tree = BTreeMap::new();
    tree.insert("fixtures/README.md".to_owned(), captured("truth"));
    tree.insert("fixtures/fix_one/a/b.txt".to_owned(), captured("hello"));
    tree.insert(
        "fixtures/fix_one/run.sh".to_owned(),
        CapturedFile {
            bytes: b"#!/bin/sh\n".to_vec(),
            executable: true,
        },
    );
    let groups = group_fixtures(&tree).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(groups.len(), 1);
    let files = &groups["fix_one"];
    assert_eq!(fixture_manifest(files).file_count, 2);
    let dir = tempfile::tempdir().unwrap_or_else(|e| panic!("{e}"));
    let target = dir.path().join("fix_one");
    let manifest = materialise_fixture(&target, files).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(manifest.bytes, 15);
    assert!(target.join("a/b.txt").is_file());
    assert!(!dir.path().join("README.md").exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(target.join("run.sh")).map(|m| m.permissions().mode());
        assert_eq!(mode.unwrap_or_default() & 0o777, 0o755);
    }
    let escape = [FixtureFile {
        relpath: "../x".to_owned(),
        bytes: vec![],
        executable: false,
    }];
    assert!(materialise_fixture(&target, &escape).is_err());
    let mut stray = BTreeMap::new();
    stray.insert("fixtures/loose.txt".to_owned(), captured("x"));
    assert!(group_fixtures(&stray).is_err());
}
