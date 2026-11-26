//! Where the pages get the Brain, and who may enter.

use dynbrain::console::{self, Console};
use systemprompt_web_admin_core::error::AdminError;
use systemprompt_web_admin_core::ssr::page::Page;

/// The Brain the pages read: the one the extension published, or a fixed one
/// a suite hands in.
#[derive(Debug, Clone)]
pub enum Source {
    Live,
    Fixed(Box<Console>),
}

impl Source {
    pub fn console(&self) -> Option<Console> {
        match self {
            Self::Live => console::current(),
            Self::Fixed(console) => Some(console.as_ref().clone()),
        }
    }
}

pub fn require_admin(shell: &Page) -> Result<(), AdminError> {
    if shell.user.is_console {
        Ok(())
    } else {
        Err(AdminError::Forbidden("Admin access required.".to_owned()))
    }
}
