//! Request handlers behind the farm routes: the dashboard pages and their
//! fragments, the public run API, the console JSON API, and the worker API
//! a run on a Fly Machine reports to.

pub(crate) mod admin;
pub(crate) mod public;
pub(crate) mod ssr;
pub(crate) mod worker;
