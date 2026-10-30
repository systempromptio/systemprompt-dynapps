use std::collections::BTreeMap;

use systemprompt::files::documents::codecs::CodecRegistry;
use systemprompt::files::documents::templates::{TemplateRegistry, load_catalog};
use systemprompt::identifiers::DocumentTemplateId;
use systemprompt_provider_contracts::documents::{
    DocumentFormat, DocumentTemplateProvider, TemplateParams, TemplateSource,
    registered_template_providers,
};
use systemprompt_web_shared::documents_templates::{
    DYNAPPS_TEMPLATE_PRIORITY, DynappsTemplateProvider,
};

use crate::support::repo_root;

const CATALOG: [(&str, DocumentFormat); 4] = [
    ("dynapps-deck", DocumentFormat::Pptx),
    ("dynapps-html", DocumentFormat::Html),
    ("dynapps-sheet", DocumentFormat::Xlsx),
    ("dynapps-word", DocumentFormat::Docx),
];

fn registry() -> TemplateRegistry {
    let catalog = load_catalog(&repo_root().join("services")).expect("catalog loads");
    TemplateRegistry::discover(catalog)
}

fn titled(title: &str) -> TemplateParams {
    TemplateParams {
        values: BTreeMap::from([("title".to_owned(), title.to_owned())]),
    }
}

#[test]
fn the_services_catalog_lists_the_four_dynapps_templates() {
    let catalog = load_catalog(&repo_root().join("services")).expect("catalog loads");
    let mut listed: Vec<(String, DocumentFormat, u32)> = catalog
        .iter()
        .map(|entry| {
            (
                entry.template.id.as_str().to_owned(),
                entry.template.format,
                entry.priority,
            )
        })
        .collect();
    listed.sort_by(|a, b| a.0.cmp(&b.0));
    let expected: Vec<(String, DocumentFormat, u32)> = CATALOG
        .iter()
        .map(|(id, format)| ((*id).to_owned(), *format, DYNAPPS_TEMPLATE_PRIORITY))
        .collect();
    assert_eq!(listed, expected);
    for entry in &catalog {
        assert!(matches!(
            entry.template.source,
            TemplateSource::ServicesPath(_)
        ));
        assert_eq!(entry.template.placeholders.len(), 1);
        assert_eq!(entry.template.placeholders[0].key, "title");
        assert_eq!(
            entry.template.placeholders[0].default.as_deref(),
            Some("Untitled")
        );
    }
}

#[test]
fn the_dynapps_provider_is_discovered_through_the_inventory() {
    let linked = DynappsTemplateProvider;
    assert_eq!(linked.provider_id(), "dynapps");
    let provider = registered_template_providers()
        .find(|provider| provider.provider_id() == "dynapps")
        .expect("dynapps template provider registered");
    assert_eq!(provider.priority(), DYNAPPS_TEMPLATE_PRIORITY);
    let templates = provider.templates();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].id.as_str(), "dynapps-note");
    assert_eq!(templates[0].format, DocumentFormat::Markdown);
    assert!(matches!(
        templates[0].source,
        TemplateSource::Embedded(bytes) if bytes.starts_with(b"# {{title}}")
    ));
}

#[test]
fn every_dynapps_template_loads_and_projects_with_its_core_codec() {
    let registry = registry();
    let codecs = CodecRegistry::discover();
    for (id, format) in CATALOG {
        let id = DocumentTemplateId::new(id);
        let bytes = registry.load_bytes(&id).expect("template bytes load");
        let codec = codecs.for_format(format).expect("core codec for format");
        let projection = codec.project(&bytes).expect("template projects");
        assert_eq!(projection.format, format);
        let text = codec.plain_text(&bytes).expect("template text");
        assert!(
            text.contains("{{title}}"),
            "{id} carries the title placeholder"
        );
        assert!(text.contains("Dynapps"), "{id} carries the brand line");
    }
}

#[test]
fn instantiating_an_dynapps_template_fills_the_title() {
    let registry = registry();
    let codecs = CodecRegistry::discover();
    for id in CATALOG
        .iter()
        .map(|(id, _)| *id)
        .chain(std::iter::once("dynapps-note"))
    {
        let id = DocumentTemplateId::new(id);
        let made = registry
            .instantiate(&id, &titled("Quarterly plan"), &codecs)
            .expect("template instantiates");
        let codec = codecs.for_format(made.format).expect("codec");
        let text = codec.plain_text(&made.bytes).expect("instance text");
        assert!(text.contains("Quarterly plan"), "{id} fills the title");
        assert!(!text.contains("{{title}}"), "{id} leaves no placeholder");
    }
}
