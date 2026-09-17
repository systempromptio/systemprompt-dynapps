//! Session evaluation: the queue of closed sessions, the inputs a criterion
//! reads, the deterministic scorers, and the judge runs that result.

pub mod activity;
pub mod inputs;
pub mod metrics;
pub mod queue;
pub mod report_inputs;
pub mod reports;
pub mod runs;
pub mod runs_summary;
pub mod scoring;
pub mod skill_scores;
pub mod version_scores;

pub use crate::repositories::config::evaluation::{
    CriterionName, InputSlice, LoadedCriterion, LoadedEvaluation, MetricThresholds, Scoring,
};
