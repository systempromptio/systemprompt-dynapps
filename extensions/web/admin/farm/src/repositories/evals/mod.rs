//! Queries over eval suites, cases, eval runs and per-case results.

mod history;
mod matrix;
mod results;
mod runs;
mod suites;

pub use history::list_eval_history;
pub use matrix::list_latest_results_matrix;
pub use results::{
    EvalOutcomeUpdate, find_eval_result, insert_eval_result, list_eval_results,
    update_eval_result_outcome, update_eval_result_run,
};
pub use runs::{
    NewEvalRun, claim_eval_run_driver, find_eval_run, insert_eval_run, is_eval_run_cancelled,
    list_eval_runs_paged, list_recent_eval_runs, update_eval_run_counts, update_eval_run_finished,
    update_eval_run_status,
};
pub use suites::{
    delete_eval_suites_not_in, find_eval_suite, list_all_eval_suites, list_eval_cases,
    list_eval_suites, replace_eval_cases, upsert_eval_suite,
};
