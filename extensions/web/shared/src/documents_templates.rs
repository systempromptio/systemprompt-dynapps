//! Dynapps compiled-in document templates.
//!
//! The branded Office and HTML templates ship as the services catalog under
//! `services/documents/templates/`; this provider carries the one template
//! that needs no file, through core's template provider inventory.

use std::sync::Arc;

use systemprompt::identifiers::DocumentTemplateId;
use systemprompt_provider_contracts::documents::{
    DocumentFormat, DocumentTemplate, DocumentTemplateProvider, TemplatePlaceholder, TemplateSource,
};
use systemprompt_provider_contracts::register_document_template_provider;

pub const DYNAPPS_TEMPLATE_PRIORITY: u32 = 200;

const DYNAPPS_NOTE: &[u8] = b"# {{title}}\n\n_Dynapps_\n";

/// Publishes the `dynapps-note` Markdown template at priority 200.
#[derive(Debug, Clone, Copy, Default)]
pub struct DynappsTemplateProvider;

impl DocumentTemplateProvider for DynappsTemplateProvider {
    fn provider_id(&self) -> &'static str {
        "dynapps"
    }

    fn templates(&self) -> Vec<DocumentTemplate> {
        vec![DocumentTemplate {
            id: DocumentTemplateId::new("dynapps-note"),
            format: DocumentFormat::Markdown,
            name: "Dynapps note".to_owned(),
            description: "Markdown note with a title heading and the Dynapps byline.".to_owned(),
            source: TemplateSource::Embedded(DYNAPPS_NOTE),
            placeholders: vec![TemplatePlaceholder {
                key: "title".to_owned(),
                label: "Title".to_owned(),
                default: Some("Untitled".to_owned()),
            }],
        }]
    }

    fn priority(&self) -> u32 {
        DYNAPPS_TEMPLATE_PRIORITY
    }
}

fn dynapps_template_provider() -> Arc<dyn DocumentTemplateProvider> {
    Arc::new(DynappsTemplateProvider)
}

register_document_template_provider!(dynapps_template_provider);
