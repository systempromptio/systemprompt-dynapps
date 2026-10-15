//! The farm's runner over core's `agent_runs` executor.
//!
//! A launch queues an `agent_runs` row whose request [`prepare`] built;
//! [`FarmRunnerService`] claims it, the handler materialises the run
//! directory and runs core's executor, and [`observer`] stamps the farm row
//! from the finished `agent_runs` row. [`plugin`] composes an agent's plugin
//! from the kit's services tree.

mod handler;
pub mod observer;
pub mod plugin;
pub mod prepare;
pub mod report;
mod service;

pub use service::{FarmRunnerService, ORIGIN_KIND};
