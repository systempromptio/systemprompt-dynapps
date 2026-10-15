//! `services/web/config/agent-farms.yaml`: which farms this instance syncs,
//! where their source lives, the runner's concurrency and the eval judge.

mod evals_yaml;
mod farms_yaml;
mod runner_yaml;

pub use evals_yaml::{DEFAULT_JUDGE_MODEL, DEFAULT_JUDGE_PROVIDER, EvalsConfig};

pub use farms_yaml::{
    FARMS_FILE, FarmDef, FarmDefaults, FarmsDoc, GitSource, RunnerConfig, parse_farms_doc,
    read_farms_doc,
};
pub use runner_yaml::{
    FlyRunnerConfig, RUNNER_MODE_ENV, RunnerMode, read_runner_config, with_mode_override,
};
