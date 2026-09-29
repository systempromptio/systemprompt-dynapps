//! The one outcome notice a form post leaves on the page it returns to.
//!
//! A server-rendered mutation ends in a redirect, and the page it lands on
//! has no memory of what just happened. Every such handler used to invent
//! its own query parameter and its own notice; this is the single spelling.
//! The outcome rides the redirect as `flash` and `flash_tone`, the shell's
//! `services/flash.js` draws it as a notice above the content and strips the
//! parameters from the address bar, so a reload or a shared link never
//! repeats it. Without JavaScript the page still renders; only the notice
//! is lost, which is the same as before.

use axum::response::Redirect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashTone {
    Ok,
    Warn,
    Err,
    Info,
}

impl FlashTone {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Err => "err",
            Self::Info => "info",
        }
    }
}

const MESSAGE_MAX: usize = 400;

#[must_use]
pub fn flash_url(base: &str, tone: FlashTone, message: &str) -> String {
    let message: String = message.chars().take(MESSAGE_MAX).collect();
    let joiner = if base.contains('?') { '&' } else { '?' };
    format!(
        "{base}{joiner}flash={}&flash_tone={}",
        urlencoding::encode(&message),
        tone.as_str()
    )
}

pub fn redirect_with_flash(base: &str, tone: FlashTone, message: &str) -> Redirect {
    Redirect::to(&flash_url(base, tone, message))
}
