//! Analysis repositories.
//!
//! Conversations and skills read the deterministic record — the
//! `conversation_facts` rollup, the hook plane, the tool ledger and the
//! governance spine — with the judge's one label joined on top; tools and
//! artifacts read the ledger through `tool_activity`; reports hold the
//! on-demand AI digests and findings.
pub mod conversations;
pub mod reports;
pub mod skills;
