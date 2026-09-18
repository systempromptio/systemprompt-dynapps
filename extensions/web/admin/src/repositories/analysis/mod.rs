//! Analysis repositories.
//!
//! Conversations and skills read the deterministic record — the
//! `conversation_facts` rollup, the hook plane, the tool ledger and the
//! governance spine — with the judge's one label joined on top; tools and
//! artifacts read the ledger through `tool_activity`; reports hold the
//! on-demand AI digests and findings; marketplace
//! versions read the retained reporting data (normalized facts and
//! snapshots) so a version's figures survive raw-event retention.
pub mod conversations;
pub mod inventory_index;
pub mod marketplace_versions;
pub mod publications;
pub mod reports;
pub mod retained;
pub mod skills;
pub mod tools;
