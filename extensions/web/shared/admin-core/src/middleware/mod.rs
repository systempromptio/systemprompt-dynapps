//! Route gates shared by every admin router: each reads the `UserContext`
//! the session middleware attached and refuses the request when the caller
//! lacks the role or scope the route needs.

pub mod gates;
