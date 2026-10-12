//! The farm dashboard's form actions. Each is console-gated and answers with
//! a flash redirect; the launch action re-renders its dialog with a 422 when
//! the submission is refused, so nothing typed is lost.

mod cancel;
mod launch;
mod run_eval;

pub(crate) use cancel::cancel_run_action;
pub(crate) use launch::launch_run_action;
pub(crate) use run_eval::run_now_action;

fn field<'a>(fields: &'a [(String, String)], name: &str) -> Option<&'a str> {
    fields
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}
