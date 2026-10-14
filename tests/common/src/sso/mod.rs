//! Fake identity providers for the SSO suites.
//!
//! A SAML `IdP` that signs real responses ([`saml_idp`]) and an Entra-shaped
//! OIDC provider on loopback HTTP ([`oidc_idp`]), both keyed by
//! [`keys::TestIdpKeys`].

pub mod keys;
pub mod oidc_idp;
pub mod redirect;
pub mod saml_idp;

pub use keys::TestIdpKeys;
pub use oidc_idp::{FakeOidcIdp, IdTokenSpec, pkce_challenge};
pub use redirect::{ParsedRedirect, parse_authn_request_redirect};
pub use saml_idp::{SamlIdpFixture, SamlResponseSpec, Tamper};
