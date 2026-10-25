//! The `agent-farm` MCP server: a run-centric tool set over core's
//! `agent_runs` for developers and project managers working from Claude Code
//! or Cowork.
//!
//! Every run tool resolves the caller into a [`server::Viewer`] and passes
//! the run through [`server::authorize_run`]: a person sees their own runs,
//! the runs of projects they belong to, and nothing else; an administrator
//! sees everything, and only an administrator may ask for cross-user
//! aggregates. Tools are defined in [`tools`], their inputs and outputs in
//! [`typed`], and the rmcp surface in [`server`].
//!
//! Evals are deterministic assertions only. No judge is ever attached and no
//! judge field is ever returned.

pub mod env;
pub mod error;
pub mod server;
pub mod tools;
pub mod typed;

pub use env::Env;
pub use error::{ToolError, ToolResult};
pub use server::{AgentFarmServer, RunOrigin, ServerAccess, Viewer, authorize_run};
