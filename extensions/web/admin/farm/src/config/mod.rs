//! `services/web/config/agent-farms.yaml`: which farms this instance syncs,
//! where their source lives, the runner's concurrency and the eval judge.

mod evals_yaml;
mod farms_yaml;

pub use evals_yaml::{DEFAULT_JUDGE_MODEL, DEFAULT_JUDGE_PROVIDER, EvalsConfig};

pub use farms_yaml::{
    FARMS_FILE, FarmDef, FarmDefaults, FarmsDoc, GitSource, RunnerConfig, parse_farms_doc,
    read_farms_doc,
};
