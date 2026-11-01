//! `dynwork-relay` end to end: the relay served over loopback TCP, driven by
//! real WebSocket clients and real RS256 tokens.
//!
//! - socket authentication: missing, forged and wrong-audience tokens, and
//!   unknown or revoked device credentials
//! - pairing: claim, reuse, expiry, the device and open-pairing limits
//! - the device list and revocation, cut within 5 s by the route and by the
//!   revocation watch
//! - routing: phone to desktop and back, the sender stamped, order kept, other
//!   users unreachable, envelopes held for an offline phone
//! - limits: oversize envelopes, the rate limit, text frames, the heartbeat,
//!   the session lifetime, the tunnel cap and replacement
//! - drain through core's `BackgroundTasks` and `StreamDrain`
//! - the router inside core's middleware stack, including `load_shed`
//! - the Postgres store against a throwaway database

#[cfg(test)]
mod memory_store;
#[cfg(test)]
mod support;

#[cfg(test)]
mod core_stack;
#[cfg(test)]
mod devices;
#[cfg(test)]
mod drain;
#[cfg(test)]
mod pairing_flow;
#[cfg(test)]
mod postgres_store;
#[cfg(test)]
mod tunnel_limits;
#[cfg(test)]
mod tunnels;
#[cfg(test)]
mod ws_auth;
