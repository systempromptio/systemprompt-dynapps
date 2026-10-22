//! Config loading is deliberately non-fatal: a broken `services/web/` tree
//! degrades the affected section to "absent" instead of taking the server down,
//! and every section is loaded once into one `SiteConfig` the extension owns.
//! The skills loader adds its own filters — a directory without a readable,
//! parseable `config.yaml` is skipped, and a disabled skill never reaches the
//! page even though its file parsed fine.

use std::error::Error;
use std::fs;
use std::path::Path;

use systemprompt_web_site::config_loader::skills::{parse_skill_entries, read_skills_dir};
use systemprompt_web_site::config_loader::{ConfigError, SiteConfig};

fn write_skill(root: &Path, dir: &str, yaml: &str) {
    let path = root.join(dir);
    fs::create_dir_all(&path).expect("skill dir");
    fs::write(path.join("config.yaml"), yaml).expect("skill config");
}

fn write_web_config(services: &Path, file: &str, yaml: &str) {
    let dir = services.join("web/config");
    fs::create_dir_all(&dir).expect("web config dir");
    fs::write(dir.join(file), yaml).expect("web config file");
}

#[test]
fn an_empty_services_tree_loads_every_section_as_absent() {
    let temp = tempfile::tempdir().expect("tempdir");
    let config = SiteConfig::load_from(temp.path());
    assert!(config.branding.is_none());
    assert!(config.navigation.is_none());
    assert!(config.homepage.is_none());
    assert!(config.skills_page.is_none());
}

#[test]
fn a_broken_section_is_absent_while_its_siblings_load() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_web_config(
        temp.path(),
        "navigation.yaml",
        "main: [oops
",
    );
    write_web_config(
        temp.path(),
        "theme.yaml",
        "branding:
  brand_name: Dynapps
",
    );
    write_skill(
        &temp.path().join("skills"),
        "good",
        "id: good
name: Good
description: Loads.
category: CRM
",
    );

    let config = SiteConfig::load_from(temp.path());
    assert!(config.navigation.is_none(), "unparseable YAML is absent");
    assert!(config.branding.is_some(), "a sibling section still loads");
    assert_eq!(config.skills_page.as_ref().map(|s| s.skills.len()), Some(1));
}

#[test]
fn a_parse_failure_keeps_the_yaml_error_as_its_cause() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_web_config(
        temp.path(),
        "navigation.yaml",
        "main: [oops
",
    );
    let error =
        systemprompt_web_site::config_loader::load_config_section(temp.path(), "navigation.yaml")
            .expect_err("unparseable YAML is an error");
    assert!(matches!(error, ConfigError::Parse { .. }));
    assert!(
        error
            .source()
            .is_some_and(<dyn Error>::is::<serde_yaml::Error>)
    );
}

#[test]
fn a_missing_skills_directory_is_absent_not_an_error() {
    let temp = tempfile::tempdir().expect("tempdir");
    let result = read_skills_dir(&temp.path().join("nope")).expect("missing dir is not an error");

    assert!(result.is_none());
    assert!(
        read_skills_dir(temp.path())
            .expect("existing dir reads")
            .is_some()
    );
}

#[test]
fn only_enabled_skills_with_a_parseable_config_are_loaded() {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = temp.path();

    write_skill(
        root,
        "good",
        "id: good\nname: Good\ndescription: Loads.\ncategory: CRM\n",
    );
    write_skill(
        root,
        "disabled",
        "id: disabled\nname: Disabled\ndescription: Hidden.\nenabled: false\n",
    );
    write_skill(root, "unparseable", "id: [oops\n");
    write_skill(root, "incomplete", "name: No Id\n");
    fs::create_dir_all(root.join("no_config")).expect("bare dir");
    fs::write(root.join("loose.yaml"), "id: loose\n").expect("stray file");

    let entries = read_skills_dir(root)
        .expect("dir reads")
        .expect("dir exists");
    let skills = parse_skill_entries(entries);

    let ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["good"]);
    assert_eq!(skills[0].category.as_deref(), Some("CRM"));
}
