//! `services/web/config/agent-farms.yaml`: which farms this instance syncs,
//! where their source lives, and the runner's concurrency.

mod farms_yaml;

pub use farms_yaml::{
    FARMS_FILE, FarmDef, FarmDefaults, FarmsDoc, GitSource, RunnerConfig, parse_farms_doc,
    read_farms_doc,
};
