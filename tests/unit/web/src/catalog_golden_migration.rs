//! The committed `services/ai` catalog resolves every model id the way core
//! 0.72.1 resolved the pre-migration tree (`tests/fixtures/catalog/0.72.1/
//! dynapps/golden/resolution.json`), and the legacy tree is refused.
//!
//! Compared per requested id: whether it dispatches, `advertised_to`, and per
//! hop of the plain chain the URL, wire, credential, upstream model, price and
//! weight. The allowlist is exactly the intended changes: an alias route
//! forwards its target model's `upstream_model`; DeepSeek V4 Pro is one model
//! whose chain gains the OpenCode Go deployment after DeepSeek's own API; and
//! `go-deepseek-v4-pro`, now an alias of that model, is no longer listed to
//! clients. `{install}` user agents and endpoint-scoped betas are intended
//! too but are not part of the snapshot schema.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use systemprompt::loader::ConfigLoader;
use systemprompt::manifest::services::catalog::{
    CatalogContext, MIGRATE_HINT, ModelRequest, ResolvedCatalog, ResolvedHop,
};
use systemprompt::models::providers::ClientFamily;
use tempfile::TempDir;

const PRICE_FIELDS: [&str; 5] = [
    "input_per_million",
    "output_per_million",
    "cache_read_per_million",
    "cache_write_per_million",
    "per_image_cents",
];

const CATALOG_FILES: [&str; 6] = [
    "config.yaml",
    "credentials.yaml",
    "providers.yaml",
    "models.yaml",
    "deployments.yaml",
    "gateway.yaml",
];

/// Requested ids whose chain may differ from 0.72.1, and how.
const ALLOWED_CHAIN_GROWTH: [(&str, usize); 1] = [("deepseek-v4-pro", 2)];
const ALLOWED_UNADVERTISED: [&str; 1] = ["go-deepseek-v4-pro"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn golden_dir() -> PathBuf {
    repo_root().join("tests/fixtures/catalog/0.72.1/dynapps")
}

/// A scratch services tree holding only the AI files, so the comparison sees
/// the catalog and nothing else of the install.
fn ai_tree(source: &Path, files: &[&str]) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("tempdir");
    let services = dir.path().join("services");
    std::fs::create_dir_all(services.join("ai")).expect("ai dir");
    std::fs::create_dir_all(services.join("config")).expect("config dir");
    for name in files {
        let from = source.join(name);
        if from.exists() {
            std::fs::copy(&from, services.join("ai").join(name)).expect("copy catalog file");
        }
    }
    std::fs::write(
        services.join("config/config.yaml"),
        "includes:\n  - ../ai/config.yaml\n",
    )
    .expect("root config");
    let root = services.join("config/config.yaml");
    (dir, root)
}

fn committed_catalog() -> ResolvedCatalog {
    let (_dir, root) = ai_tree(&repo_root().join("services/ai"), &CATALOG_FILES);
    ConfigLoader::load_from_path(&root)
        .expect("committed services/ai loads")
        .resolve_catalog(&CatalogContext::default())
        .expect("committed catalog resolves")
}

fn hop_value(hop: &ResolvedHop) -> Value {
    let credential = hop.credential.platform_secret().map_or_else(
        || json!({ "source": "personal", "owner": hop.credential.id.as_str() }),
        |secret| json!({ "source": "platform", "secret": secret.as_str() }),
    );
    json!({
        "url": hop.url,
        "wire": hop.wire,
        "credential": credential,
        "upstream_model": hop.upstream_model,
        "pricing": hop.pricing,
        "weight": hop.weight,
    })
}

fn price_eq(a: &Value, b: &Value) -> bool {
    PRICE_FIELDS.iter().all(|field| {
        match (
            a.get(field).and_then(Value::as_f64),
            b.get(field).and_then(Value::as_f64),
        ) {
            (Some(x), Some(y)) => (x - y).abs() <= 1e-9 * x.abs().max(y.abs()).max(1.0),
            (None, None) => true,
            _ => false,
        }
    })
}

fn hop_diffs(golden: &Value, new: &Value) -> Vec<&'static str> {
    let mut diffs: Vec<&'static str> = ["url", "wire", "credential", "upstream_model", "weight"]
        .into_iter()
        .filter(|field| golden.get(field) != new.get(field))
        .collect();
    let priced = golden
        .get("pricing")
        .zip(new.get("pricing"))
        .is_some_and(|(g, n)| price_eq(g, n));
    if !priced {
        diffs.push("pricing");
    }
    diffs
}

fn chain(catalog: &ResolvedCatalog, id: &str) -> Option<Vec<Value>> {
    catalog
        .resolve(&ModelRequest::new(id))
        .ok()
        .map(|target| target.chain.iter().map(hop_value).collect())
}

fn alias_target_upstream(catalog: &ResolvedCatalog, id: &str) -> Option<String> {
    let model = catalog.routes().iter().find_map(|r| r.aliases(id))?;
    catalog
        .enabled_hops(model.as_str())
        .next()
        .map(|hop| hop.upstream_model.clone())
}

fn advertised(catalog: &ResolvedCatalog, id: &str) -> Value {
    let model = catalog.model(id);
    let tags: Vec<&str> = ClientFamily::ALL
        .into_iter()
        .filter(|family| model.is_some_and(|m| m.is_advertised_to(*family)))
        .map(|family| {
            if family == ClientFamily::Claude {
                "anthropic"
            } else {
                family.as_tag()
            }
        })
        .collect();
    json!(tags)
}

fn compare_chain(
    catalog: &ResolvedCatalog,
    id: &str,
    golden: &[Value],
    new: &[Value],
    out: &mut Vec<String>,
) {
    let grown = ALLOWED_CHAIN_GROWTH
        .iter()
        .any(|(allowed, len)| *allowed == id && new.len() == *len);
    if golden.len() != new.len() && !grown {
        out.push(format!(
            "{id}: chain has {} hops, 0.72.1 had {}",
            new.len(),
            golden.len()
        ));
    }
    let aliased = alias_target_upstream(catalog, id);
    for (index, (g, n)) in golden.iter().zip(new).enumerate() {
        let mut diffs = hop_diffs(g, n);
        if diffs == ["upstream_model"] && aliased.as_deref() == n["upstream_model"].as_str() {
            diffs.clear();
        }
        if !diffs.is_empty() {
            out.push(format!(
                "{id} hop {index}: {} differ (0.72.1 {g}, now {n})",
                diffs.join(",")
            ));
        }
    }
}

fn findings() -> Vec<String> {
    let catalog = committed_catalog();
    let text = std::fs::read_to_string(golden_dir().join("golden/resolution.json"))
        .expect("golden snapshot");
    let golden: Value = serde_json::from_str(&text).expect("golden json");
    let mut out = Vec::new();
    for (id, entry) in golden["models"].as_object().expect("models") {
        let new_chain = chain(&catalog, id);
        let Some(golden_chain) = entry.get("chain").and_then(Value::as_array) else {
            if new_chain.is_some() {
                out.push(format!("{id}: dispatches now; 0.72.1 refused it"));
            }
            continue;
        };
        let Some(new_chain) = new_chain else {
            out.push(format!("{id}: no longer dispatches; 0.72.1 served it"));
            continue;
        };
        compare_chain(&catalog, id, golden_chain, &new_chain, &mut out);
        let listed = advertised(&catalog, id);
        let unlisted_alias = ALLOWED_UNADVERTISED.contains(&id.as_str()) && listed == json!([]);
        if catalog.model(id).is_some() && entry["advertised_to"] != listed {
            out.push(format!(
                "{id}: advertised to {listed} (0.72.1 {})",
                entry["advertised_to"]
            ));
        } else if catalog.model(id).is_none() && !unlisted_alias && !id.ends_with("[1m]") {
            out.push(format!("{id}: not a catalog model and not an allowed alias"));
        }
    }
    out
}

#[test]
fn committed_catalog_resolves_as_0_72_1_did() {
    let diffs = findings();
    assert!(
        diffs.is_empty(),
        "differences outside the allowlist:\n{}",
        diffs.join("\n")
    );
}

#[test]
fn default_model_is_the_bare_sonnet_id() {
    let catalog = committed_catalog();
    assert_eq!(
        catalog.default_model().map(|m| m.as_str()),
        Some("claude-sonnet-5-5")
    );
}

#[test]
fn legacy_tree_is_refused_with_the_migrate_hint() {
    let legacy = golden_dir().join("legacy/ai");
    let (_dir, root) = ai_tree(&legacy, &["config.yaml", "providers.yaml", "gateway.yaml"]);
    let error = ConfigLoader::load_from_path(&root).expect_err("a legacy tree must not load");
    assert!(error.to_string().contains(MIGRATE_HINT), "{error}");
}
