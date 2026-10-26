//! `services/web/config/agent-farms.yaml`: which farms this instance syncs,
//! where their source lives, the runner's concurrency and the eval judge.

mod clients_yaml;
mod evals_yaml;
mod farms_yaml;
mod run_as;
pub use run_as::RunAsError;
mod runner_yaml;
mod tool_policy_yaml;

pub use clients_yaml::{AgentOverride, ClientDef};
pub use evals_yaml::{DEFAULT_JUDGE_MODEL, DEFAULT_JUDGE_PROVIDER, EvalsConfig};
pub use run_as::{RunAs, RunAsMode};
pub use tool_policy_yaml::FarmToolPolicy;

pub use farms_yaml::{
    FARMS_FILE, FarmDef, FarmDefaults, FarmsDoc, GitSource, RunnerConfig, parse_farms_doc,
    read_farms_doc,
};
pub use runner_yaml::{
    FlyRunnerConfig, RUNNER_MODE_ENV, RunnerMode, read_runner_config, with_mode_override,
};
