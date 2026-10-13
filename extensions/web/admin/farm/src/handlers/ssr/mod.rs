//! Dashboard pages under `/admin/farm` and the HTML fragments they poll.

pub(crate) mod actions;
pub(crate) mod curl;
pub(crate) mod farm_agent;
pub(crate) mod farm_detail;
pub(crate) mod farm_eval_run;
pub(crate) mod farm_evals;
pub(crate) mod farm_overview;
pub(crate) mod farm_people;
pub(crate) mod farm_run_detail;
pub(crate) mod farm_runs;
pub(crate) mod fragments;
pub(crate) mod run_events_view;
pub(crate) mod shared;
pub(crate) mod sync_card;

pub(crate) use actions::{cancel_run_action, launch_run_action, run_now_action};
pub(crate) use farm_agent::farm_agent_page;
pub(crate) use farm_detail::farm_detail_page;
pub(crate) use farm_eval_run::eval_run_page;
pub(crate) use farm_evals::evals_page;
pub(crate) use farm_overview::farm_overview_page;
pub(crate) use farm_people::people_page;
pub(crate) use farm_run_detail::run_detail_page;
pub(crate) use farm_runs::runs_page;
pub(crate) use fragments::activity::activity_fragment;
pub(crate) use fragments::eval_run::eval_run_fragment;
pub(crate) use fragments::run::run_fragment;
pub(crate) use fragments::runs::runs_fragment;
