//! `onepassword` external secrets source.
//!
//! A profile with `secrets.source: external` and `external.kind: onepassword`
//! loads its secrets from a 1Password vault, either through the `op` CLI
//! with a service-account token (`backend: cli`, the default) or a
//! self-hosted Connect server (`backend: connect`). The crate registers the
//! kind with core through `register_secrets_source!`; a binary picks it up by
//! linking the crate (`use systemprompt_secrets_onepassword as _;`).
//!
//! Nothing here logs or describes a secret value or the token.

mod backend;
pub mod cli;
pub mod config;
pub mod connect;
pub mod error;
pub mod item;
pub mod mapping;
pub mod provider;
pub mod reference;
pub mod retry;

pub use backend::Backend;
pub use config::{BackendKind, Mapping, OnePasswordConfig};
pub use error::{KIND, OnePasswordError};
pub use provider::{OnePasswordProvider, build};
pub use reference::OpReference;

systemprompt::config::register_secrets_source!(KIND, build);
