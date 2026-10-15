//! Farm runs on Fly Machines (`runner.mode: fly_machine`).
//!
//! - [`api`] — the Machines REST calls.
//! - [`pool`] — taking, launching and returning pooled runner machines.
//! - [`registry`] — the runs this API process is waiting on a worker for.
//! - [`remote`] — the handler's side: launch, record, wait, finish.
//! - [`protocol`] — the worker API's wire types, shared with the
//!   `systemprompt-farm-worker` binary.

pub mod api;
pub mod pool;
pub mod protocol;
pub mod registry;
pub mod remote;
