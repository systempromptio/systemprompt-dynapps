//! The kit exporter must preserve the configured marketplace while producing
//! a portable tree that carries neither this instance's access grants nor its
//! MCP connection configuration.

use std::path::Path;

use systemprompt::loader::ConfigLoader;
use systemprompt::manifest::services::split_frontmatter;
use systemprompt_kit_export::{export_kit, render_skill_md, round_trip};

fn services_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../services")
        .canonicalize()
        .expect("services tree")
}

#[test]
fn commons_export_round_trips_without_instance_access_or_mcp_files() {
    let services_root = services_root();
    let services = ConfigLoader::load_from_path(&services_root.join("config/config.yaml"), None)
        .expect("load configured services");
    let out = tempfile::tempdir().expect("kit output directory");

    let report = export_kit(&services, &services_root, "dynapps-commons", out.path())
        .expect("export configured commons marketplace");
    assert_eq!(report.marketplace, "dynapps-commons");
    assert_eq!(report.plugins, vec!["dynapps-commons".to_owned()]);
    assert!(report.skills.contains("who_am_i"));
    assert!(report.files > 0);
    let diffs = round_trip(out.path(), &services_root, &report).expect("strict re-import");
    assert!(
        diffs.is_empty(),
        "the kit preserves every portable marketplace, plugin, and skill declaration: {diffs:#?}"
    );

    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(out.path().join(".claude-plugin/marketplace.json"))
            .expect("marketplace manifest"),
    )
    .expect("portable marketplace JSON");
    assert_eq!(manifest["name"], "dynapps-commons");
    assert_eq!(manifest["plugins"][0]["version"], "1.0.0");
    let sidecar = std::fs::read_to_string(out.path().join(".claude-plugin/systemprompt.yaml"))
        .expect("marketplace sidecar");
    assert!(
        !sidecar.contains("access:"),
        "audience grants belong to the source instance, never the kit"
    );
    assert!(
        !out.path()
            .join("plugins/dynapps-commons/.mcp.json")
            .exists(),
        "instance MCP connection configuration is not portable"
    );
    assert!(
        !out.path()
            .join("plugins/dynapps-commons/config.yaml")
            .exists(),
        "the source config file, which can carry instance-only fields, is not copied"
    );
    let exported_skill = std::fs::read_to_string(
        out.path()
            .join("plugins/dynapps-commons/skills/who-am-i/SKILL.md"),
    )
    .expect("exported skill instructions");
    let source_skill = std::fs::read_to_string(services_root.join("skills/who_am_i/SKILL.md"))
        .expect("source skill instructions");
    let source_body = split_frontmatter(&source_skill)
        .expect("skill frontmatter and body")
        .body
        .trim();
    assert!(
        exported_skill.contains(source_body),
        "instruction body survives the frontmatter projection"
    );
}

#[test]
fn export_refuses_an_unknown_marketplace_without_creating_a_tree() {
    let services_root = services_root();
    let services = ConfigLoader::load_from_path(&services_root.join("config/config.yaml"), None)
        .expect("load configured services");
    let out = tempfile::tempdir().expect("kit output directory");

    let error = export_kit(
        &services,
        &services_root,
        "missing-contract-marketplace",
        out.path(),
    )
    .expect_err("unknown marketplace must not export");
    assert!(
        error
            .to_string()
            .contains("no marketplace 'missing-contract-marketplace'")
    );
    assert!(
        std::fs::read_dir(out.path())
            .expect("empty output directory")
            .next()
            .is_none(),
        "failed lookup writes no partial kit"
    );
}

fn skill_dir(config: &str, skill_md: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("skill directory");
    std::fs::write(dir.path().join("config.yaml"), config).expect("config.yaml");
    std::fs::write(dir.path().join("SKILL.md"), skill_md).expect("SKILL.md");
    dir
}

#[test]
fn exported_skill_md_passes_authored_keys_through_after_the_platform_keys() {
    let dir = skill_dir(
        "id: field_notes\nname: Field Notes\ndescription: Notes.\ntags: [notes]\n\
         frontmatter:\n  model: opus\n  allowed-tools: [Write]\n",
        "---\nname: field-notes\ndescription: stale\ntitle: stale\nallowed-tools:\n- Read\n\
         disable-model-invocation: true\nhooks:\n  PreToolUse:\n  - matcher: Bash\n---\nBody.\n",
    );
    let md = render_skill_md(dir.path(), "field-notes").expect("render");
    let front = split_frontmatter(&md).expect("frontmatter");
    let parsed: serde_yaml::Mapping = serde_yaml::from_str(front.yaml).expect("valid yaml");
    let keys: Vec<&str> = parsed
        .keys()
        .filter_map(serde_yaml::Value::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "name",
            "title",
            "description",
            "tags",
            "allowed-tools",
            "disable-model-invocation",
            "hooks",
            "model",
        ],
        "platform keys from config.yaml, then the SKILL.md keys, then config-only keys: {md}"
    );
    assert_eq!(parsed["title"], "Field Notes");
    assert_eq!(parsed["description"], "Notes.");
    assert_eq!(parsed["allowed-tools"][0], "Read", "the authored file wins");
    assert_eq!(parsed["hooks"]["PreToolUse"][0]["matcher"], "Bash");
    assert_eq!(front.body, "Body.\n");
}

#[test]
fn exported_skill_md_without_authored_keys_is_unchanged() {
    let dir = skill_dir(
        "id: who_am_i\nname: Who Am I\ndescription: Who.\n",
        "---\nname: who-am-i\ndescription: Who.\n---\nBody.\n",
    );
    assert_eq!(
        render_skill_md(dir.path(), "who-am-i").expect("render"),
        "---\nname: who-am-i\ntitle: Who Am I\ndescription: Who.\n---\nBody.\n"
    );
}
