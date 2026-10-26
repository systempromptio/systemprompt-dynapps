//! The booted services view and core's subject-attribute source, carried
//! together.
//!
//! Every handler and service that reads the services config also resolves
//! subject attributes through the one set core builds at boot, so the two are
//! taken from the extension context (or the authz-hook and marketplace factory
//! contexts) once and passed down as one value. It dereferences to the
//! [`ServicesView`], so services reads stay as they are.

use std::ops::Deref;
use std::sync::Arc;

use systemprompt::extension::prelude::ExtensionContext;
use systemprompt::governance::authz::AuthzHookContext;
use systemprompt::identifiers::UserId;
use systemprompt::manifest::services::ServicesView;
use systemprompt::marketplace::MarketplaceFactoryContext;
use systemprompt::models::authz::{
    SubjectAttributeSource, SubjectAttributes, SubjectDimension, SubjectSourceError,
};

/// The services view and subject-attribute source an extension call reads.
#[derive(Clone)]
pub struct ExtensionServices {
    view: ServicesView,
    subjects: Arc<dyn SubjectAttributeSource>,
}

impl std::fmt::Debug for ExtensionServices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionServices").finish_non_exhaustive()
    }
}

impl ExtensionServices {
    #[must_use]
    pub fn new(view: ServicesView, subjects: Arc<dyn SubjectAttributeSource>) -> Self {
        Self { view, subjects }
    }

    #[must_use]
    pub fn from_context(ctx: &dyn ExtensionContext) -> Self {
        Self::new(ctx.services(), ctx.subject_attributes())
    }

    #[must_use]
    pub fn from_hook(ctx: &AuthzHookContext) -> Self {
        Self::new(ctx.services.clone(), Arc::new(ctx.subjects.clone()))
    }

    #[must_use]
    pub fn from_marketplace(ctx: &MarketplaceFactoryContext) -> Self {
        Self::new(ctx.services.clone(), Arc::new(ctx.subjects.clone()))
    }

    #[must_use]
    pub const fn view(&self) -> &ServicesView {
        &self.view
    }

    #[must_use]
    pub fn dimensions(&self) -> &[SubjectDimension] {
        self.subjects.dimensions()
    }

    pub async fn subject_attributes(
        &self,
        user: &UserId,
    ) -> Result<SubjectAttributes, SubjectSourceError> {
        self.subjects.resolve_all(user).await
    }
}

impl Deref for ExtensionServices {
    type Target = ServicesView;

    fn deref(&self) -> &ServicesView {
        &self.view
    }
}
