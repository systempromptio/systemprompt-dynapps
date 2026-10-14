//! Odoo MCP server.
//!
//! Odoo (self-hosted Community) is the system of record for CRM, and this
//! server is the only route an agent takes to it. It speaks Odoo's JSON-RPC
//! external API directly — there is no vendor SDK and no proxy service in
//! between.
//!
//! **Every call executes as the caller's own Odoo credential.** [`credential`]
//! resolves the platform caller — on this instance a service account such as
//! the farm's — to the sealed `odoo_api_key` machine credential it holds (URL,
//! database, login, API key), and [`client`] signs in and issues `execute_kw`
//! with it. So Odoo's record rules decide what an agent can read, Odoo's audit
//! log names the login behind every change, and a caller holding no credential
//! is refused — never quietly served someone else's data.
//!
//! Destructive and erasure tools carry a second, tool-layer gate in [`guard`]:
//! a required `reason`, a read before the write, and — while `ODOO_DEMO_SCOPE`
//! is on — a refusal for any record not tagged `demo-*`.
//!
//! The tool surface ([`tools`]) covers leads, partners, activities, and the
//! record-anchored knowledge bank — chatter notes and attachments — plus one
//! composite briefing that aggregates in Odoo rather than pulling records back
//! to count them.
//!
//! There is no separate knowledge store because Odoo Community has no Knowledge
//! app; that is an Enterprise module. What it does have is `mail.message` and
//! `ir.attachment`, both anchored to a `(res_model, res_id)` pair, so every
//! note and every file is already filed against the lead or partner it
//! concerns. [`server::notes`] and [`server::attachments`] read that as the
//! knowledge plane it effectively is.

pub mod apps;
pub mod attachment;
pub mod audit;
pub mod caller;
pub mod client;
pub mod credential;
pub mod error;
pub mod format;
pub mod guard;
pub mod resolve;
pub mod server;
pub mod shape;
pub mod text;
pub mod tools;

pub use server::OdooServer;
