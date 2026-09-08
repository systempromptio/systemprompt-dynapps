//! Governed `OpenSpec` artifact lifecycle, immutable evidence and approval
//! receipts.
pub mod authz;
pub mod jobs;
pub mod publication;
pub mod server;
pub mod store;
pub mod validation;

pub use systemprompt_requirements_model::{model, signing};

mod evidence;
