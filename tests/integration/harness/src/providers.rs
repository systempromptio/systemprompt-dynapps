//! The lanes the matrix runs against: every `provider/endpoint` of the
//! shipped catalog (`services/ai/*.yaml`) that has a deployment, plus one
//! synthetic endpoint for each wire the catalog does not use yet, so every
//! outbound encoder is exercised.
//!
//! A lane is served by the first model, in `models.yaml` order, deployed on
//! its endpoint. Pointing a lane at the mock upstream rewrites only that
//! endpoint's host and re-resolves the whole catalog, so the hop the gateway
//! would build (upstream name, betas, user agent, forwarded headers) is the
//! one the matrix captures.

use std::collections::BTreeSet;
use std::path::PathBuf;

use systemprompt::identifiers::ModelId;
use systemprompt::manifest::services::catalog::{
    CATALOG_FILES, CatalogContext, CatalogDocument, EndpointRef, ResolvedCatalog, ResolvedHop,
};
use systemprompt::wire::WireProtocol;
use systemprompt::wire::anthropic::AnthropicBeta;

// Why: a loopback stub for the e2e suite, not an upstream any harness reaches.
const SKIPPED: &[&str] = &["e2e-mock"];

const INSTALL: &str = "dynapps";

const SYNTHETIC: &str = "\
credentials:
  harness:
    kind: platform
    secret: harness
providers:
  harness-gemini:
    credential: harness
    endpoints:
      api:
        wire: gemini
        url: https://generativelanguage.googleapis.com/v1beta
  harness-openai-responses:
    credential: harness
    endpoints:
      api:
        wire: openai-responses
        url: https://api.openai.com/v1
models:
  gemini-2.5-flash:
    clients: [gemini]
    pricing: {input_per_million: 1.0, output_per_million: 1.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0}
  gpt-5:
    clients: [openai]
    pricing: {input_per_million: 1.0, output_per_million: 1.0, cache_read_per_million: 0.1, cache_write_per_million: 1.0}
deployments:
  gemini-2.5-flash:
  - harness-gemini/api
  gpt-5:
  - harness-openai-responses/api
";

#[derive(Debug, Clone)]
pub(crate) struct Lane {
    pub name: String,
    pub endpoint: EndpointRef,
    pub wire: WireProtocol,
    pub model: ModelId,
    pub user_agent: Option<String>,
    pub forward_headers: Vec<String>,
    pub accepted_betas: Option<BTreeSet<AnthropicBeta>>,
    document: CatalogDocument,
}

fn ai_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../services/ai")
}

fn shipped_document() -> (CatalogDocument, Vec<ModelId>) {
    let mut merged = CatalogDocument::default();
    for name in CATALOG_FILES {
        let path = ai_dir().join(name);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let document = CatalogDocument::from_yaml_str(name, &text)
            .unwrap_or_else(|e| panic!("parse {}: {e}", path.display()));
        merged.merge(document).expect("catalog files merge");
    }
    let synthetic =
        CatalogDocument::from_yaml_str("synthetic", SYNTHETIC).expect("synthetic catalog");
    merged.merge(synthetic).expect("synthetic catalog merges");
    (merged, model_order())
}

// Why: the catalog maps are sorted by id; the lane's model is chosen in the
// order an operator wrote `models.yaml`, so read that file's key order.
fn model_order() -> Vec<ModelId> {
    let path = ai_dir().join("models.yaml");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let doc: serde_yaml::Value = serde_yaml::from_str(&text).expect("models.yaml parses");
    let shipped = doc["models"]
        .as_mapping()
        .expect("models mapping")
        .keys()
        .filter_map(serde_yaml::Value::as_str)
        .map(ModelId::new);
    shipped
        .chain(["gemini-2.5-flash", "gpt-5"].map(ModelId::new))
        .collect()
}

fn resolve(document: &CatalogDocument) -> ResolvedCatalog {
    document
        .resolve(&CatalogContext::for_install(INSTALL))
        .expect("the harness catalog resolves")
}

pub(crate) fn catalogue() -> Vec<Lane> {
    let (document, order) = shipped_document();
    let catalog = resolve(&document);
    catalog
        .endpoints()
        .filter(|endpoint| !SKIPPED.contains(&endpoint.provider.as_str()))
        .filter_map(|endpoint| {
            let endpoint_ref = endpoint.endpoint_ref();
            let model = order.iter().find(|model| {
                catalog
                    .deployments(model.as_str())
                    .iter()
                    .any(|hop| hop.endpoint_ref() == endpoint_ref)
            })?;
            let provider = catalog
                .provider(endpoint.provider.as_str())
                .expect("endpoint's provider");
            Some(Lane {
                name: endpoint_ref.to_string(),
                endpoint: endpoint_ref,
                wire: endpoint.wire,
                model: model.clone(),
                user_agent: provider.user_agent.clone(),
                forward_headers: provider.forward_headers.clone(),
                accepted_betas: endpoint.betas.clone(),
                document: document.clone(),
            })
        })
        .collect()
}

pub(crate) fn named(name: &str) -> Lane {
    catalogue()
        .into_iter()
        .find(|lane| lane.name == name)
        .unwrap_or_else(|| panic!("lane {name} is not in the catalogue"))
}

// The lane's catalog with its endpoint's host swapped for the mock upstream,
// keeping the path the real provider is called on, and the hop of `model` on
// that endpoint.
pub(crate) fn pointed_at(
    lane: &Lane,
    base: &str,
    model: &str,
) -> Result<(ResolvedCatalog, ResolvedHop), String> {
    let mut document = lane.document.clone();
    let endpoint = document
        .providers
        .providers
        .get_mut(&lane.endpoint.provider)
        .and_then(|provider| provider.endpoints.get_mut(&lane.endpoint.endpoint))
        .ok_or_else(|| format!("{} is not declared", lane.name))?;
    let after_scheme = endpoint
        .url
        .split_once("://")
        .map_or(endpoint.url.as_str(), |(_, rest)| rest);
    let path = after_scheme.find('/').map_or("", |at| &after_scheme[at..]);
    endpoint.url = format!("{base}{path}");
    let catalog = resolve(&document);
    let hop = catalog
        .deployments(model)
        .iter()
        .find(|hop| hop.endpoint_ref() == lane.endpoint)
        .cloned()
        .ok_or_else(|| format!("{model} is not deployed on {}", lane.name))?;
    Ok((catalog, hop))
}
