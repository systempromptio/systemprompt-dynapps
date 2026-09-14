//! Authored holdout content remains independent and meets unchanged campaign
//! floors.
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
#[test]
fn every_skill_has_ten_distinct_development_and_holdout_cases() {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join("services/evaluations/super-admin").is_dir() {
        assert!(root.pop());
    }
    let suite = root.join("services/evaluations/super-admin");
    let mut counts = BTreeMap::<String, (usize, usize)>::new();
    let mut ids = BTreeSet::new();
    let mut prompts = BTreeSet::new();
    let mut fixtures = BTreeMap::<String, BTreeSet<String>>::new();
    for file in std::fs::read_dir(suite.join("cases")).unwrap() {
        // JSON: inspect authored benchmark documents as the importer reads them.
        let case: serde_json::Value =
            serde_json::from_slice(&std::fs::read(file.unwrap().path()).unwrap()).unwrap();
        assert!(ids.insert(case["id"].as_str().unwrap().to_owned()));
        let skill = case["skill_id"].as_str().unwrap();
        let partition = case["partition"].as_str().unwrap();
        assert!(
            prompts.insert((
                skill.to_owned(),
                case["prompt"].as_str().unwrap().to_owned()
            )),
            "duplicate prompt within a skill"
        );
        let count = counts.entry(skill.to_owned()).or_default();
        match partition {
            "development" => count.0 += 1,
            "holdout" => count.1 += 1,
            _ => panic!("partition"),
        };
        let fixture = case["fixture"].as_str().unwrap();
        let contents =
            std::fs::read(suite.join("fixtures").join(format!("{fixture}.json"))).unwrap();
        // JSON: normalize the opaque fixture document before independence comparison.
        let normalized: serde_json::Value = serde_json::from_slice(&contents).unwrap();
        fixtures
            .entry(format!("{skill}:{partition}"))
            .or_default()
            .insert(normalized.to_string());
        assert!(!case["assertions"].as_array().unwrap().is_empty());
    }
    assert_eq!(counts.len(), 4);
    for (skill, (development, holdout)) in counts {
        assert!(
            development >= 10 && holdout >= 10,
            "{skill}: {development}/{holdout}"
        );
        let development = &fixtures[&format!("{skill}:development")];
        let holdout = &fixtures[&format!("{skill}:holdout")];
        assert!(
            development.is_disjoint(holdout),
            "{skill} reused development fixture as holdout"
        );
    }
}
