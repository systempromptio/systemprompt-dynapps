//! Server-rendered platform pages.
//!
//! Each module owns one page: it builds a typed template context and renders a
//! `.hbs` template from `storage/files/admin/templates/` at request time.

mod configuration;
mod devices;
mod drive;
mod ssr_bridge_device_link;
mod ssr_connect;
mod ssr_connectors;
pub(crate) mod ssr_connectors_cards;
mod ssr_setup;
mod ssr_sync;
mod ssr_sync_import;

pub(crate) use configuration::{configuration_page, secrets_reload};
pub(crate) use devices::devices_page;
pub(crate) use drive::{drive_document_page, drive_page, drive_people};
pub(crate) use ssr_bridge_device_link::{device_link_approve, device_link_deny, device_link_page};
pub(crate) use ssr_connect::connect_page;
pub(crate) use ssr_connectors::connectors_page;
pub(crate) use ssr_setup::{bridge_setup_page, setup_page};
pub(crate) use ssr_sync::sync_page;
pub(crate) use ssr_sync_import::import_preview_page;

pub(crate) mod data_console;
