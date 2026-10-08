//! Marketplace-version analysis.
//!
//! Versions read the per-skill half of the
//! conversation rollup (`conversation_skill_facts`), so a version's figures
//! survive raw-event retention; publications are the decisions taken on a
//! version; plugin evaluation scores each conversation from its stored
//! messages by fixed rules.

pub mod marketplace_versions;
pub mod plugin_eval;
pub mod publications;
