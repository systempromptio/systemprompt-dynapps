//! Astound Digital platform (`SystemPrompt` template fork)
//!
//! This crate re-exports extensions for use with the `SystemPrompt` runtime.
//! Extensions are automatically discovered via the `inventory` crate.

pub use systemprompt::{cli, *};
pub use systemprompt_web_extension as web;

// Why: Keeps the evaluator job's inventory-bearing core module reachable
// through aggressive release and incremental linking.
#[doc(hidden)]
pub fn evaluation_supervisor_registration_anchor() {
    let job: &'static dyn Job = &scheduler::EvaluationSupervisorJob;
    std::hint::black_box(job.name());
}
