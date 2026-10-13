//! Unit tests for `systemprompt-secrets-onepassword` pure logic:
//! - `op://` reference parsing and display
//! - the `secrets.external` block parse: defaults, bounds, backend rules and
//!   the refusal of a literal token
//! - field mapping: label-then-id resolution, CONCEALED/STRING only, and
//!   ambiguity as an error
//! - the Connect retry classification and backoff schedule
//! - the provider's description, refresh hint and fetch over a stub backend

#[cfg(test)]
mod config;
#[cfg(test)]
mod mapping;
#[cfg(test)]
mod provider_describe;
#[cfg(test)]
mod reference;
#[cfg(test)]
mod retry;
