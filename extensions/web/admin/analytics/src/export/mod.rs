//! One export surface for every table the console renders.
//!
//! A page declares a
//! [`DataSet`](systemprompt_web_admin_core::export::model::DataSet) — its
//! columns and the repository read behind them — and registers it in
//! [`registry`](systemprompt_web_admin_core::export::registry). The
//! [`handler`](systemprompt_web_admin_core::export::handler) serves any dataset
//! in any [`Format`](systemprompt_web_admin_core::export::format::Format) with
//! any column selection over any window the dataset's contract allows, and the
//! preview endpoint tells the dialog how many rows and cells a download will
//! hold before it is asked for. `datasets/` holds one file per table.
//! `document/` is the other shape: one conversation with every body, tool
//! call, decision, finding and hook event, as JSON, Markdown or JSON Lines.

pub(crate) mod datasets;
pub(crate) mod document;
