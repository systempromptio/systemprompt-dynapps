# Security Defaults (by construction)

An MCP exposure is a security surface. The goal is to make a leak *structurally
impossible*, not to patch one after a review. These are the defaults to bake in
before handing anything off. For the auditor's-eye version of the same rules,
use the **odoo-code-auditor** skill → its `mcp-exposure-security.md` reference.

## The invariant

> An MCP call can never touch more than `exposure ∩ caller-ir.model.access ∩ ir.rule`.

You design the **exposure** (positive-list). The caller's ACL and record rules are
the floor underneath it and must always apply — which means **reads run as the
real user, never `sudo`**. If you ever find yourself reaching for `sudo` to make a
read "work", you are bypassing the floor; fix the user's ACL instead.

## Two server archetypes (decide this first)

Before designing fields and ops, decide which *shape* of server the engagement
needs. They sit at opposite ends and the choice drives everything after.

| | **Generic ORM proxy** | **Curated exposure** |
|---|---|---|
| Tools | `orm_search_read/create/write/unlink/execute_kw` on **every** model | positive-list of models/fields/ops only |
| Floor | the user's `ir.model.access` + `ir.rule` only (`with_user`, never sudo) | exposure ∩ ACL ∩ ir.rule |
| Build cost | ~zero (no exposure design) | the real work of this skill |
| Safe for | **internal/trusted only** — one real Odoo user whose ACL *is* the boundary | **client / external / multi-tenant** |
| Risk | leaks anything the caller's ACL allows, incl. system models (`ir.config_parameter`, HR…) | bounded by the plan |

The corporate Indaws MCP is a **generic proxy** — verified live: `orm_search_read('ir.config_parameter')` returns rows. That is acceptable *because* the only principal is a trusted developer whose Odoo ACL is the intended boundary. **Never ship a generic proxy to a client, an external app, or a multi-user/multi-tenant context** — there the ACL alone is not a defensible boundary and you must curate. When in doubt, curate; the rest of this document assumes the curated path.

## Least-privilege ladder

Grant the *minimum* rung the use case needs, and justify each step up:

1. **read** — the default. Answering questions, reporting, lookups.
2. **aggregate / resolve** — come free with `core_tools`; no extra grant.
3. **create / write** — only for a stated transactional capability ("take an
   order"), and only on the specific fields that capability touches.
4. **call a workflow method** (`action_confirm`, `button_validate`…) — only the
   exact methods the capability needs; never a blanket method exposure.
5. **unlink** — almost never. Deletion by an autonomous agent is rarely
   defensible; require explicit, written demand.

Default the server to read-only and add rungs deliberately. Record *why* each
write/method exists in the handoff doc — future-you and the auditor will ask.

## Metadata is data

The discovery tools — `list_models`, `describe_model`, `get_selection`,
`capabilities` — leak the *shape* of the database (model names, fields, selection
values). That is information. They must be filtered by the **caller's** ACL, not
just the admin positive-list, so a user who can't read a model never even learns
it exists. In `dynapps_mcp_server` this is enforced by `_gate` (per-call ACL
check) and `_readable_exposures` (the shared filter behind `list_models` and
`capabilities`). When you design a server, don't defeat this by exposing a model
"just for discovery" — if a user shouldn't read it, it shouldn't be in their plan.

## Domain validation

If the exposure allows `aggregate`/search with caller-supplied domains, every
field referenced — in `group_by`, in measures, **and in the domain** — must be in
the positive-list, and dotted relational traversal (`partner_id.credit_limit`)
must be rejected (it reaches a field one hop past the list). This is built into
the core `aggregate` tool; just don't add custom tools that re-open the hole.

## Scoping & access

- **Company**: set `company_id`; a server is single-company. Multi-company
  clients get one server per company (or rely on the user's `company_ids`).
- **Users / groups**: an empty `user_ids` *and* `group_ids` = PRIVATE (deny all).
  Grant the narrowest group that fits. In v19, authorisation checks use
  `all_group_ids` (direct + implied), not `group_ids` — see the indaws-core-developer
  odoo19 fixes; allowing `base.group_user` (held implicitly by most internal
  users) behaves differently than you'd expect if you reason about direct groups.
- **OAuth client allowlist** (`allowed_client_ids`): pin to the client apps the
  engagement uses (Claude, ChatGPT…). Empty = any authenticated client; non-empty
  = only tokens from those clients. Set it for production servers.
- **`allow_sudo_tools` / `sudo_execution`**: leave **off**. These bypass the
  real-user floor; an admin enables them manually after review, never the builder.

## Make it self-sufficient

- **`core_tools = True`** so the agent can `whoami` (orient), `describe_model`
  (learn schema), `resolve` (name→id), `aggregate` (counts/sums), `list_models`
  / `get_selection` — without per-client wiring.
- **`instructions`** — write a short, domain-specific operating note (the
  playbooks give snippets). It is injected at `initialize` and read via
  `odoo://instructions`; it tells the agent the conventions (currency, which
  models hold what, "use aggregate not paging", read-only vs transactional).
- **Respond in the user's language**: the server resolves the caller's language
  (`effective_lang`) and speaks it. Nothing to configure per server, but write
  `instructions` in a neutral/localisable way and rely on translated field labels.
- **`instructions` is advisory free text — make it match reality.** It is *not*
  enforced; it only shapes how the agent behaves. A real Indaws server shipped
  `instructions` claiming it "runs as superuser with sudo, bypassing record
  rules" while the code actually used `env[model].with_user(user)` (rules
  enforced). A wrong note makes agents reason about a security model that doesn't
  exist. When you write it, describe the *actual* exposure and access model; when
  you **audit an existing/third-party server, trust the HTTP probe (step 5), never
  the announced `instructions` or capability blurb** — the text can lie.

## Exclude by default

If it isn't in the agreed plan, it isn't exposed. Be especially deliberate about:
HR (`hr.employee`, payroll), `ir.config_parameter` and other config/system models,
users/groups, and anything with personal or financial data outside the domain.
The positive-list means omission = denial — so the safe move is to add narrowly,
never to "expose the model and hide fields later".

## Full-ORM WRITE server: the blacklist FLOOR (code-exec gap)

> Lesson from the **web_shell/dynapps_ai_assistant** build (2026-07) — a real
> CRITICAL caught in pentest.

A full-ORM write server (`orm_full_access='full'`, e.g. an agent's write path) is
the opposite of a positive-list: everything writable EXCEPT a **blacklist floor**
that must hold for EVERY user regardless of their ACL. A `prefix.*` blacklist
(`ir.*`, `mcp.*`, `oauth.*`, `res.users`, `res.groups`, `res.company`) is necessary
but **misses code-execution models that are NOT under an `ir.*` prefix**:

- **`mail.template`** — jinja/qweb body → code execution on render.
- **`base.automation`** — writing a rule can trigger server actions.

List these **explicitly** in the floor. Reconcile the floor on upgrade (`-u` may
not re-run the init hook that seeds it → add a migration or a reconcile in the
provisioning method). Keep `allow_sudo_tools=False` (runs as the user, `su=False`)
and PII masking on. Bind the server to its OAuth client / source IPs when the
deployment allows, and **revoke the bearer when its session ends** (don't rely on
the TTL alone). Prove it: writing to any floor model must be refused **even when a
human approves it** at the gate — the server is the last line, not the UI.

## Pre-handoff gate

Never ship without proving the invariant with **≥2 principals** (admin + a
restricted user). Generate the tests with the **odoo-test-generator** skill
(pattern `22-mcp-multi-principal`) and audit with the **odoo-code-auditor** skill
(`mcp-exposure-security.md`).
The single most common real bug is a server that looks fine under one admin login
and leaks under a restricted user — so a single-principal "it works" is not
evidence.
