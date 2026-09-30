//! The one handler the composition root serves itself: the login page, which
//! answers before any session exists. Every other handler lives in the admin
//! domain crate that owns its data.

mod login;

pub(crate) use login::login_page;
