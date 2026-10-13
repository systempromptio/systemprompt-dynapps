//! Eval suites: expanding cases into runs, checking assertions, judging.

pub mod assertions;
mod case;
pub mod judge;
mod orchestrator;
pub mod transcript;

pub use assertions::{AssertionContext, AssertionResult, evaluate_assertions};
pub use judge::{EvalJudge, GatewayJudge, JudgeFuture, JudgeInput, JudgeVerdict, StubJudge};
pub use orchestrator::{EvalLaunch, cancel_eval_run, start_eval_run};
