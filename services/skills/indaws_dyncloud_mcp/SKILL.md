---
name: indaws-dyncloud-mcp
version: 1.3.1
effort: medium
model: opus
description: >-
  Drives the DynCloud MCP to operate customer Odoos from DynCloud Manager: discover companies,
  projects and environments; read and write data over the ORM; install, upgrade or uninstall
  modules; upload and download files; run Odoo's standard Excel/CSV import; and request JIT
  permissions. Use it whenever someone wants to inspect or change something in a customer's Odoo
  through DynCloud, or works with list_projects / list_environments / orm_search_read / orm_write /
  admin_install_module / file_upload_request / request_mcp_access — even if they never say
  "dyncloud-mcp". Also explains why it fails when it fails: 404s, timeouts, denied permissions.
  Triggers in Spanish too: "entorno del cliente", "subir un excel", "no me aparece", "instalar
  modulo".
---

# DynCloud MCP — driving customer Odoos through the manager

This MCP does **not** talk to a database. Every ORM and admin tool is **proxied** to a real
customer Odoo, one of 300+, and picking the wrong one is a change on somebody's live
business. Understand the machine before you call anything.

**Where things are:** `references/mcp/README.md` — tool index and the conventions common to
all of them · `references/mcp/workflows/README.md` — pick a recipe without opening anything
else · `references/gotchas.md` — **read before reporting any result back.**

Write this skill and its references in English, examples included; **reply to the user in
the language they wrote in**, every turn.

## What DynCloud Manager is, and what the MCP actually does

**DynCloud Manager** is the Odoo 19 backend that operates the platform: it owns the
inventory of companies, projects and environments, deploys them, and holds the access
rules. The MCP server lives inside it.

A tool call is **four hops**, not one:

```
you → MCP server (in the manager) → signed JWT → dyncloud_api (in the CUSTOMER's Odoo) → its ORM
```

The manager mints a **short-lived Ed25519 JWT** and calls the `dyncloud_api` module
installed **on that customer's environment**, impersonating `target_user_login`. On the
target, `dyncloud_auth` verifies the signature against the manager's public keys.

Three consequences you will meet on day one:

- **The MCP depends on a module being installed on the target.** If `dyncloud_api` is not
  there, every ORM/admin call answers **404** — the endpoint does not exist. This is the
  single most frequent failure. It is not your call being wrong (`workflows/troubleshooting.md` §1).
- **Permissions are the TARGET user's, not yours.** `target_user_login` decides what the
  call may read or write, with that user's record rules. Pick it deliberately.
- **Five location args are mandatory on every ORM/admin call** — `company_id`,
  `project_id`, `environment_id`, `db`, `target_user_login`. There is no "current
  environment" fallback for them, on purpose: with 300+ live environments a silent default
  is exactly the mistake that must not happen on a write.

## Who is asking? (decide this early — it changes what you say when things break)

The same failure has two right answers depending on who is in front of you. Infer the
profile from the conversation, don't ask for a badge:

| Signals | Profile | What you do when something fails |
| --- | --- | --- |
| Repos, MRs, `kubectl`, logs, module code, "deploy", "pipeline" | **Technical** | Give the concrete diagnosis and the checks to run. They can act on it. |
| Customers, functional configuration, data, "no me aparece", accounting/sales/stock wording | **Consulting** | **Do not send them debugging.** Say what happened in their terms and hand them a ready-to-send report for the technical team. |
| Neither is clear | **Assume consulting** | Safer: an explanation nobody needed beats a person poking at infrastructure. |

For a consulting profile, a good handoff carries: **environment and project names**, the
**time**, the **tool** you called, and the **literal error** — that is what lets the
technical team find it without reproducing anything. Never leave them with "ha fallado".

## Before you touch anything: find the right environment

`workflows/discovery.md`. The chain — `list_my_companies` → `list_projects` →
`list_environments` → `list_databases` → `list_target_users` — feeds each id into the next,
and `list_environments` also hands you the environment's `uuid` and `aud`.

Two things worth knowing up front: anything with `mcp_enabled = false` **is invisible
here**, so "it does not appear" can mean "not exposed to MCP" rather than "does not exist";
and `list_databases` / `list_target_users` are **real round-trips to the customer's Odoo**,
which makes them your cheapest health check. `set_active_context` saves repetition **for
the `list_*` tools only** — ORM and admin tools ignore it by design.

## Permissions are decided per stage, at call time (JIT)

Nothing is granted permanently. Each tool has a **category**; each environment sits on a
**stage**, and the stage decides whether that category is open, needs a grant, or is
forbidden. `development` usually has most open; `production` gates writes, module admin and
system ops. Four answers, four different meanings (`references/mcp/access-jit.md`):

| Answer | Means | Do |
| --- | --- | --- |
| `E_CATEGORY_NEEDS_GRANT` | Gated here, obtainable | `request_mcp_access` with **the same category** and a real reason (it is audited), then retry |
| `E_SCOPE_NOT_REQUESTABLE` | Already open on this stage | Just call the tool |
| `E_CATEGORY_FORBIDDEN` / `E_SCOPE_FORBIDDEN` | Never grantable here | **Stop and say so.** Asking again fails identically |
| Grant created but `pending` | You are not an approver | It waits for a human — tell the user who to chase |

Never retry a denied call blindly: the denial names what to do.

## Timeouts: six in series, and the first to fire is usually not ours

Between you and the customer's Odoo sit **six timeouts** (MCP client → Envoy → gunicorn →
manager→environment → Envoy → gunicorn). All surface as "it timed out", so a report is
useless unless it says which. Two shapes tell them apart, and the difference decides who
can fix it:

- A **bare** `The operation timed out.` with **no `error_code`** → the **client** gave up.
  Configured on the user's machine, not in DynCloud.
- **`E_TARGET_TIMEOUT`** → the **manager** gave up on the environment. The message names the
  layer, the limit and the field that changes it.

**A timeout cancels nothing.** The environment keeps working — installing a module can time
out and still finish. **Check the state before retrying**, or you start the same work twice.
Full map in `references/mcp/README.md` → "The six timeouts".

## Writes are somebody's real data

Anything under `data_write` or `module_admin` lands on a live customer. Before a write:
confirm the environment is the one they meant, prefer archiving over deleting, and for
`admin_uninstall_module` **say out loud what will be destroyed** — uninstalling drags every
dependant module out with it, and the answer does not list them.

## Typical flows

Pick one from `references/mcp/workflows/README.md` without opening anything else. The ones
that come up daily: discovery of an unknown customer; reading data as a chosen user;
**loading an Excel** through Odoo's own importer (`workflows/imports.md` — and **skip
`parse_preview`**, the proxy truncates its answer); attaching or fetching a file, where the
bytes never cross this conversation; installing or removing modules in **one call with all
the names**; and turning on an Odoo option via `res.config.settings` create + `execute`.

## Gotchas

`references/gotchas.md`, before you report anything. The costliest: **404 means the module
is missing on the target**, not that the record is; a **timeout cancels nothing**; the
**tool catalogue is cached when your client starts**, so after a manager deploy you must
restart to see new tools; `state: active` on a grant can still be **expired** (read
`effective_state`); and an **uninstall removes modules it never names**.

## Delegates to

Nothing mandatory. Target-side Odoo semantics (fields, models, version differences) →
`common-skills:odoo-version-api-reference`; platform questions (deploys, GitOps, clusters) → the
**devops** suite. **Not `references/shared/odoo-connection.md`**: that contract is for an
Odoo you hold credentials for, and here you hold none — see `references/gotchas.md`.

## Consumed by

**Nobody today** — a person drives it, usually mid-incident. Said, not left blank: a measured no.
