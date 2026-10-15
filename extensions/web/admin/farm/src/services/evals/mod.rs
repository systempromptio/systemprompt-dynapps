//! Eval suites: expanding cases into runs, checking assertions, judging,
//! and the triggers (suite, single case, demo) that start them.

pub mod assertions;
mod case;
pub mod history;
pub mod judge;
mod orchestrator;
pub mod resume;
pub mod transcript;
mod triggers;

pub use assertions::{AssertionContext, AssertionResult, evaluate_assertions};
pub use history::{EvalHistoryPoint, list_visible_history, parse_since};
pub use judge::{
    EvalJudge, GatewayJudge, JudgeFuture, JudgeInput, JudgeVerdict, StubJudge, gateway_judge,
};
pub use orchestrator::{EvalLaunch, cancel_eval_run, start_eval_run};
pub use resume::resume_eval_runs;
pub use transcript::final_result_text;
pub use triggers::{CaseLaunch, CaseLaunched, start_case_eval};
