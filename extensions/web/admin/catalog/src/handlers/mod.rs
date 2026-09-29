//! HTTP handlers for the catalog: the SSR catalog and versions pages, the
//! managed-resource authoring API, plugin and agent listings, and manifest
//! sharing. Handlers own status mapping and call repositories for data.

pub(crate) mod catalog;
pub(crate) mod managed_resources;
pub(crate) mod plugins;
pub(crate) mod resources;
pub(crate) mod share;
pub(crate) mod ssr;
