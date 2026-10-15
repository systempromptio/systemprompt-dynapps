//! The farm's runner over core's `agent_runs` executor.
//!
//! A launch queues an `agent_runs` row whose request [`prepare`] built;
//! [`FarmRunnerService`] claims it, the handler materialises the run
//! directory and runs core's executor, and [`observer`] stamps the farm row
//! from the finished `agent_runs` row. [`plugin`] composes an agent's plugin
//! from the kit's services tree. [`fly`] runs a claimed run on its own Fly
//! Machine instead of a local child when `runner.mode` is `fly_machine`.
//! [`report`] leaves the task-completion report, with [`tool_tally`]'s count
//! of the run's MCP reads, writes and `DynBrain` delivery.

pub mod cost_window;
pub mod fly;
mod handler;
pub mod identity;
pub mod observer;
pub mod plugin;
pub mod prepare;
pub mod report;
mod service;
pub mod tool_policy;
pub mod tool_tally;

pub use service::{FarmRunnerService, ORIGIN_KIND};
