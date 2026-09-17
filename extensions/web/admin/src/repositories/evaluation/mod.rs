//! Session evaluation: the queue of closed sessions, the inputs a criterion
//! reads, the deterministic scorers, and the judge runs that result.

pub mod activity;
pub mod inputs;
pub mod metrics;
pub mod queue;
pub mod runs;
pub mod runs_summary;
pub mod scoring;

pub use crate::repositories::config::evaluation::{
    CriterionName, InputSlice, LoadedCriterion, LoadedEvaluation, MetricThresholds, Scoring,
};
