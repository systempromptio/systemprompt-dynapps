//! Unit tests for `dynwork-relay` pure logic:
//! - the envelope header: size bounds, version, recipient, sender stamping
//! - peer, device and pairing ids and the device kind
//! - device credentials: shape, hashing, redaction
//! - the per-connection rate limiter
//! - the desktop ring buffer: count, byte and age eviction
//! - the hub: same-user routing only, the other kind of end only, holding for
//!   an offline peer, revocation, replacement and slow consumers
//! - the limits' defaults and validation
//! - pairing tokens against a real token authority: audience separation

#[cfg(test)]
mod support;

#[cfg(test)]
mod audit;
#[cfg(test)]
mod credential;
#[cfg(test)]
mod envelope;
#[cfg(test)]
mod extension;
#[cfg(test)]
mod hub;
#[cfg(test)]
mod ids;
#[cfg(test)]
mod limits;
#[cfg(test)]
mod pairing_token;
#[cfg(test)]
mod rate;
#[cfg(test)]
mod ring;
