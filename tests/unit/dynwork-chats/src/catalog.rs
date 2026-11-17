//! The model list: what the committed catalog of this instance serves, by
//! which secrets the instance holds.

use std::collections::BTreeSet;
use std::path::PathBuf;

use dynwork_chats::catalog::{ModelInfo, listed};
use systemprompt::loader::ConfigLoader;
use systemprompt::manifest::services::{CatalogContext, ResolvedCatalog};

fn repo_root() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        if dir.join("services/ai/models.yaml").is_file() {
            return dir;
        }
        assert!(dir.pop(), "repository root not found");
    }
}

fn catalog() -> ResolvedCatalog {
    ConfigLoader::load_from_path(&repo_root().join("services/config/config.yaml"), None)
        .expect("the services tree loads")
        .resolve_catalog(&CatalogContext::default())
        .expect("the catalog resolves")
}

fn with_secrets(names: &[&str]) -> Vec<ModelInfo> {
    let held: BTreeSet<String> = names.iter().map(|name| (*name).to_owned()).collect();
    listed(&catalog(), |name| held.contains(name))
}

fn ids(models: &[ModelInfo]) -> Vec<&str> {
    models.iter().map(|model| model.id.as_str()).collect()
}

#[test]
fn the_list_follows_the_secrets_the_instance_holds() {
    let both = with_secrets(&["anthropic", "deepseek"]);
    let claude = with_secrets(&["anthropic"]);
    let deepseek = with_secrets(&["deepseek"]);
    assert_ne!(ids(&both), ids(&claude));
    assert_ne!(ids(&claude), ids(&deepseek));
    assert!(claude.iter().all(|model| model.provider_id == "anthropic"));
    assert!(deepseek.iter().all(|model| model.provider_id == "deepseek"));
    assert!(!claude.is_empty() && !deepseek.is_empty());
    assert_eq!(both.len(), claude.len() + deepseek.len());
}

#[test]
fn an_instance_with_no_secrets_serves_nothing() {
    assert!(with_secrets(&[]).is_empty());
    assert!(with_secrets(&["opencode-go", "unrelated"]).is_empty());
}

#[test]
fn models_billed_to_a_members_own_key_are_not_listed() {
    let catalog = catalog();
    let personal: Vec<&str> = catalog
        .advertised_models(None)
        .filter(|model| {
            catalog
                .enabled_hops(model.id.as_str())
                .all(|hop| hop.credential.is_member_paid())
        })
        .map(|model| model.id.as_str())
        .collect();
    assert!(
        !personal.is_empty(),
        "the committed catalog advertises member-paid models"
    );
    let held: BTreeSet<String> = ["anthropic", "deepseek", "opencode-go"]
        .iter()
        .map(|name| (*name).to_owned())
        .collect();
    let models = listed(&catalog, |name| held.contains(name));
    for id in personal {
        assert!(!ids(&models).contains(&id), "{id} needs the gateway");
    }
}

#[test]
fn a_listed_model_names_the_provider_that_serves_it() {
    let catalog = catalog();
    for model in with_secrets(&["anthropic", "deepseek"]) {
        let served = catalog
            .enabled_hops(&model.id)
            .any(|hop| hop.provider.as_str() == model.provider_id);
        assert!(served, "{}", model.id);
        assert!(!model.name.is_empty());
        assert!(!model.provider_name.is_empty());
        assert!(
            model.tools,
            "the catalog marks these models as tool-capable"
        );
    }
}

#[test]
fn the_list_is_sorted_by_provider_then_id() {
    let models = with_secrets(&["anthropic", "deepseek"]);
    let keys: Vec<(&str, &str)> = models
        .iter()
        .map(|model| (model.provider_id.as_str(), model.id.as_str()))
        .collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted);
}
