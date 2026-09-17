//! `services/evaluation/` — the declared judge: which criteria exist, which
//! model judges each, and the instructions it receives.

pub mod loader;
pub mod types;

pub use loader::load_evaluation;
pub use types::{
    CriterionName, InputSlice, LoadedCriterion, LoadedEvaluation, MetricThresholds, Scoring,
};
