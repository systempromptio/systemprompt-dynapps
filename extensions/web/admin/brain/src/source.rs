//! Where the pages get the Brain, and who may enter.

use axum::Extension;
use axum::extract::FromRequestParts;
use axum::extract::rejection::ExtensionRejection;
use axum::http::header::{AUTHORIZATION, COOKIE};
use axum::http::request::Parts;
use dynbrain::console::{self, Console};
use systemprompt::identifiers::JwtToken;
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

/// The Brain the pages read, and the signed-in admin's session token. A proxy
/// instance's source answers as a person, so the reads that go to it carry
/// that token; a native instance does not need it.
#[derive(Debug, Clone)]
pub struct Reach {
    pub source: Source,
    pub token: Option<JwtToken>,
}

impl<S: Send + Sync> FromRequestParts<S> for Reach {
    type Rejection = ExtensionRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(source) = Extension::<Source>::from_request_parts(parts, state).await?;
        let bearer = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::to_owned);
        let cookie = parts
            .headers
            .get_all(COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .flat_map(|value| value.split(';'))
            .find_map(|cookie| cookie.trim().strip_prefix("access_token="))
            .map(str::to_owned);
        let token = bearer
            .or(cookie)
            .filter(|token| !token.is_empty())
            .map(JwtToken::new);
        Ok(Self { source, token })
    }
}
