//! The document classes `services/documents/classes.yaml` declares, in
//! strictness order, and the agent cap each grants an attended or an
//! unattended actor.
//!
//! The file is compiled in: the agent access policy is registered through
//! `inventory` with no runtime context to read a services root from, and a
//! class matrix that changed under a running server would change what agents
//! may do without a deploy.

use std::sync::Arc;

use serde::Deserialize;
use systemprompt::files::documents::agents::AgentCap;

const CLASSES_YAML: &str = include_str!("../../../../../../services/documents/classes.yaml");

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentClass {
    pub id: String,
    pub label: String,
    pub reason: String,
    pub attended: AgentCap,
    pub unattended: AgentCap,
}

impl DocumentClass {
    #[must_use]
    pub const fn cap_for(&self, attended: bool) -> AgentCap {
        if attended {
            self.attended
        } else {
            self.unattended
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassCatalog {
    pub default_for_people: String,
    pub default_for_agents: String,
    pub classes: Vec<DocumentClass>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassesFile {
    document_classes: ClassCatalog,
}

#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("services/documents/classes.yaml does not parse: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("services/documents/classes.yaml names unknown class `{0}`")]
    UnknownDefault(String),
}

impl ClassCatalog {
    pub fn declared() -> Result<Self, CatalogError> {
        Self::parse(CLASSES_YAML)
    }

    pub fn parse(text: &str) -> Result<Self, CatalogError> {
        let catalog = serde_yaml::from_str::<ClassesFile>(text)?.document_classes;
        for id in [&catalog.default_for_people, &catalog.default_for_agents] {
            if catalog.find(id).is_none() {
                return Err(CatalogError::UnknownDefault(id.clone()));
            }
        }
        Ok(catalog)
    }

    #[must_use]
    pub fn find(&self, id: &str) -> Option<&DocumentClass> {
        self.classes.iter().find(|c| c.id == id)
    }

    #[must_use]
    pub fn strictest(&self) -> Option<&DocumentClass> {
        self.classes.last()
    }

    #[must_use]
    pub fn rank(&self, id: &str) -> Option<usize> {
        self.classes.iter().position(|c| c.id == id)
    }

    #[must_use]
    pub fn default_for(&self, created_by_agent: bool) -> Option<&DocumentClass> {
        let id = if created_by_agent {
            &self.default_for_agents
        } else {
            &self.default_for_people
        };
        self.find(id)
    }
}

/// The catalog parsed once at a composition root and injected.
///
/// A malformed file is kept as its error: every policy call then caps agents
/// at `none` and every class handler fails, never widening a cap.
#[derive(Debug, Clone)]
pub struct DeclaredClasses(Result<Arc<ClassCatalog>, Arc<CatalogError>>);

impl DeclaredClasses {
    #[must_use]
    pub fn load() -> Self {
        let parsed = ClassCatalog::declared().map(Arc::new).map_err(|error| {
            tracing::error!(%error, "document class catalog unusable; agents capped at none");
            Arc::new(error)
        });
        Self(parsed)
    }

    pub fn get(&self) -> Result<&ClassCatalog, Arc<CatalogError>> {
        self.0.as_deref().map_err(Arc::clone)
    }
}
