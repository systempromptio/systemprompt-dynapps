//! The provider catalogue the matrix runs against: every entry of
//! `services/ai/providers.yaml`, plus one synthetic provider for each wire the
//! catalogue does not use yet, so every outbound encoder is exercised.

use std::path::PathBuf;

use systemprompt::manifest::services::ProviderEntry;

// Why: a loopback stub for the e2e suite, not an upstream any harness reaches.
const SKIPPED: &[&str] = &["e2e-mock"];

const SYNTHETIC: &[&str] = &[
    "name: harness-gemini\nwire: gemini\nsurface: backend\n\
     endpoint: https://generativelanguage.googleapis.com/v1beta\n\
     models:\n- id: gemini-2.5-flash\n",
    "name: harness-openai-responses\nwire: openai-responses\nsurface: backend\n\
     endpoint: https://api.openai.com/v1\nmodels:\n- id: gpt-5\n",
];

pub(crate) fn catalogue() -> Vec<ProviderEntry> {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../services/ai/providers.yaml");
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc: serde_yaml::Value = serde_yaml::from_str(&raw).expect("providers.yaml parses");
    let listed = doc["providers"]
        .as_sequence()
        .expect("providers list")
        .iter()
        .map(|v| serde_yaml::from_value::<ProviderEntry>(v.clone()).expect("provider entry"));
    let synthetic = SYNTHETIC
        .iter()
        .map(|yaml| serde_yaml::from_str::<ProviderEntry>(yaml).expect("synthetic provider"));
    listed
        .chain(synthetic)
        .filter(|p| !SKIPPED.contains(&p.name.as_str()) && !p.models.is_empty())
        .collect()
}

pub(crate) fn named(name: &str) -> ProviderEntry {
    catalogue()
        .into_iter()
        .find(|p| p.name.as_str() == name)
        .unwrap_or_else(|| panic!("provider {name} is not in the catalogue"))
}

// The same entry with its endpoint's host swapped for the mock upstream,
// keeping the path the real provider is called on.
pub(crate) fn pointed_at(entry: &ProviderEntry, base: &str) -> ProviderEntry {
    let after_scheme = entry
        .endpoint
        .split_once("://")
        .map_or(entry.endpoint.as_str(), |(_, rest)| rest);
    let path = after_scheme.find('/').map_or("", |at| &after_scheme[at..]);
    let mut pointed = entry.clone();
    pointed.endpoint = format!("{base}{path}");
    pointed
}
