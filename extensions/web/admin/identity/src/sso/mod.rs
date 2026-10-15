//! Browser single sign-on, as providers of core's SSO driver.
//!
//! - [`entra_oidc`] — Microsoft Entra ID over `OpenID Connect`.
//! - [`events`] — the activity, membership and revocation side effects.
//! - [`roles`] — the role resolver: directory roles joined with manual grants,
//!   and revocation when a sign-in drops admin.
//! - [`error`] — the providers' typed failures, mapped onto core's reasons.
//! - [`refresh`] — the background-refreshed `IdP` discovery documents.
//!
//! Each provider's factory reads a config the aggregator installs before the
//! driver discovers providers; with none installed the provider is off.

pub mod entra_oidc;
pub mod error;
pub mod events;
pub mod refresh;
pub mod roles;

systemprompt::sso::register_sso_provider!(entra_oidc::factory);
systemprompt::sso::register_auth_event_sink!(events::factory);
systemprompt::sso::register_role_resolver!(roles::factory);
