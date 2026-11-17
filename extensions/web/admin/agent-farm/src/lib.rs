//! Farm domain of the admin dashboard: read-only pages over the Dynapps
//! farm-api, for console admins.
//!
//! The console holds one service credential with farm-api's platform read
//! scope, so an admin sees every tenant: `services/web/config/farm.yaml`
//! names farm-api's URL and the secrets-store key of its token ([`config`]).
//! The list pages show all tenants mixed, by asking farm-api for `?tenant=*`;
//! `?tenant=<slug>` narrows a page to one tenant. A detail page names the row's
//! own tenant. When the credential is missing or farm-api fails, the page says
//! so in the shell's "could not be loaded" notice; it never answers 500.
//!
//! The pages: a dashboard that summarises and links the rest, Tenants (one row
//! each), Tasks (and one task), Agents (and one agent), Schedules, Credentials
//! (metadata only, never a value) and Cost. The pages used to live under
//! `/admin/agent-farm`; that path redirects to `/admin/farm`.
//!
//! What a task cost and what it said come from systemprompt, not from
//! farm-api: [`sp`] reads `ai_requests` and the conversation tables through
//! the admin read pool, keyed by the task's gateway session and Claude Code
//! session. Costs are shown in USD, as systemprompt stores them.
//!
//! - [`config`] — `services/web/config/farm.yaml`.
//! - [`client`] — the farm-api HTTP client (GET only).
//! - [`models`] — the response shapes the pages read.
//! - [`views`] — the task page view models, pure.
//! - [`sp`] — systemprompt's record of each task's calls and conversation.
//! - [`usage`] — the view models of that record, pure.
//! - [`routes`] — the SSR routes the admin router merges.

pub mod agents;
pub mod agents_detail;
pub mod client;
pub mod config;
pub mod cost;
pub mod cost_range;
pub mod cost_report;
pub mod credentials;
pub mod dashboard;
pub mod handlers;
pub mod handlers_task;
pub mod models;
pub mod routes;
pub mod schedules;
pub mod sp;
pub mod tenants;
pub mod types;
pub mod usage;
pub mod views;
pub mod views_task;
pub mod views_tenant;
