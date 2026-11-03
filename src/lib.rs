//! Dynapps platform (`SystemPrompt` template fork)
//!
//! This crate re-exports extensions for use with the `SystemPrompt` runtime.
//! Extensions are automatically discovered via the `inventory` crate.

pub use dynwork_relay as relay;
pub use systemprompt::{cli, *};
pub use systemprompt_web_extension as web;
