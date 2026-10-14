//! The seam between the provider and the two ways of reaching 1Password.
//! Both backends take an explicit binary path or base URL, so tests point
//! them at a fake `op` script or a fake Connect server.

use async_trait::async_trait;

use crate::error::OnePasswordError;
use crate::item::Item;
use crate::reference::OpReference;

#[async_trait]
pub trait Backend: Send + Sync {
    fn label(&self) -> &'static str;

    async fn fetch_items(
        &self,
        vault: &str,
        titles: &[String],
    ) -> Result<Vec<Item>, OnePasswordError>;

    async fn read_reference(&self, _reference: &OpReference) -> Result<String, OnePasswordError> {
        Err(OnePasswordError::Unsupported)
    }
}
