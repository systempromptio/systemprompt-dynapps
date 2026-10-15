---
name: indaws-odoo-mcp-builder
description: >
  Construye un servidor MCP seguro dentro de Odoo desde una peticion vaga, convirtiendo el ask en
  un plan de exposicion (modelos, campos, operaciones) con least-privilege y validacion multi-principal.
  Triggers: "MCP para ventas", "exponer Odoo a Claude", "MCP para compras", "quiero un MCP",
  "AI agent que gestione pedidos", "configure MCP server in Odoo", "expose Odoo to Cursor".
version: 1.1.7
effort: high
model: opus
---

# Odoo MCP Builder

Turn a loose request — "the client wants an MCP for sales" — into a concrete,
secure, working MCP server in Odoo. The client rarely gives a spec; your job is
to convert "AI that handles our sales" into an exact exposure (which models,
fields, operations, methods) plus the security envelope, then materialise and
validate it. This skill encodes the domain knowledge and safety rules so you
don't rediscover them per engagement.

## How MCP works here (read first)

An MCP "server" is **config records**, not a new module: `mcp.server.config` (the
server) + `mcp.model`/`mcp.model.field`/`mcp.method` (the catalog) +
`mcp.server.model` (per-server exposure with `allow_read/create/write/unlink`).
Authorisation is layered on every call: OAuth 2.1 → server allowlist →
positive-list exposure → Odoo `ir.model.access` + `ir.rule` **as the real user
(never sudo)**. So the server can never grant more than
`exposure ∩ caller-ACL ∩ ir.rule` — you design the *exposure*; the rest is the
floor. The product also ships a **MCP GOD server** (meta-tools to design other
servers); it gives the mechanism, this skill gives the domain judgement it lacks.

## Workflow

1. **Clarify the ask (vague → capabilities).** Pin three things: domain(s);
   *what must the agent DO* — report-only (read) vs transactional (create/confirm)
   vs nothing destructive; and who/where (users, company, client app). Default to
   **read-only**; add each write/method only on explicit need. Also decide the
   **archetype**: generic ORM proxy (internal/trusted only) vs curated exposure
   (client/external) — see [references/security-defaults.md](references/security-defaults.md);
   when in doubt, curate. **[STOP]** confirm these with the client before designing.
2. **Map to an exposure plan.** Use [references/domain-playbooks.md](references/domain-playbooks.md)
   for the curated per-domain exposure (models, fields, ops, methods, relations to
   resolve). Always add the universal entities (`res.partner`, products) and keep
   `core_tools` on. Produce a `model → ops → fields → methods` table.
3. **Apply security defaults.** Run the plan through
   [references/security-defaults.md](references/security-defaults.md):
   least-privilege ladder, per-company + OAuth allowlist, "metadata is data",
   reads non-su, domain `instructions`, user-language. **[STOP]** get explicit
   sign-off before exposing any write/method.
4. **Materialise.** Pick a path in [references/materialize.md](references/materialize.md):
   installable data module *(recommended deliverable)*, in-product GOD workflow,
   or live via `odoo-pilot` (prototype). Live Odoo API work goes through the
   **odoo-pilot** scripts — never ad-hoc rpc. Custom functions (logic/shape that
   auto-CRUD + core tools can't produce) → [references/authoring-tools.md](references/authoring-tools.md).
5. **Validate (the real gate).** Prove `restricted ⊊ admin` over HTTP with
   [references/validation-harness.md](references/validation-harness.md) +
   [scripts/mcp_probe.py](scripts/mcp_probe.py); add model-layer tests via
   **odoo-test-generator** pattern 22; audit via **odoo-code-auditor**
   `mcp-exposure-security.md`. **[STOP]** do not hand off until green.
6. **Hand off.** Document slug + URL (`/mcp/<db>/<slug>`), allowed users/clients,
   the as-built plan, and which ops are write/method (the agent's blast radius).

## Design checklist

- [ ] Capabilities pinned (domain + read-only vs transactional + who/where)
- [ ] Exposure plan written and confirmed; every write/method justified; no `unlink`
- [ ] Universal entities exposed; `core_tools` on; `instructions` + user-language set
- [ ] Per-company + OAuth allowlist; no sensitive/unrelated models (metadata too)
- [ ] Materialised reproducibly (prefer data module)
- [ ] Validated ≥2 principals over HTTP + unit tests + audit; handoff doc written

## Supporting references

Lean on existing knowledge; don't duplicate. For live-connection/batch/field
mechanics see `common:references/shared/` and the **odoo-pilot** scripts (this
skill never opens raw rpc).

- [references/authoring-tools.md](references/authoring-tools.md) — writing a good
  custom `mcp.tool` / method exposure: when NOT to, the blacklist, the safe_eval
  sandbox, token ergonomics, the 5 PII-mask strategies + read-path bypass, the
  propose→validate→apply lifecycle, and the v15→v19 matrix (structuredContent /
  masking are v19-only; the authoring contract is stable across branches).

- `common-skills:odoo-version-api-reference` — confirm a field/method exists & is spelled
  the same in the target version (names drift v16→v19).
- `indaws-core-developer` (odoo18/19 fixes) — exposure-specific cross-version
  gotchas (`has_access` vs `check_access_rights`, `env._` vs `_()`, v19
  `all_group_ids` access split).

## GOTCHAS

1. **Single-admin testing hides every leak.** Admin reads everything, so a leak
   to a restricted user is invisible. Always validate with ≥2 principals (step 5).
2. **Metadata is data.** `list_models`/`describe_model`/`capabilities` must be
   ACL-filtered per caller — never expose a model "just for discovery".
3. **Reaching for `sudo` to make a read work** bypasses the ir.rule/ACL floor.
   Fix the user's ACL instead; core reads stay non-su.
4. **Domain field validation.** Every field a caller references (group_by, measures,
   domain) must be exposed and dotted traversal rejected — don't re-open the hole.
5. **Pre-seeded demo exposures** may already include a field you assume is hidden;
   when testing "denied", pick a model/field you actually control.
6. **`_ensure_god_exposure` is read-only.** For write/method exposures create the
   `mcp.server.model` record explicitly so you control the `allow_*` flags.
7. **A server's announced `instructions`/capability text can lie** (a real one
   claimed "sudo, bypasses record rules" while the code ran `with_user(user)`).
   Write `instructions` to match the real access model; when auditing a third-party
   server, trust the HTTP probe (step 5), never the announced text.
8. **Binaries don't come back inline.** Binary/image fields return a value-ref,
   never base64; files travel via `attachment_link`/`request_upload` pre-signed URLs.
   If the use case touches documents, expose the attachment tools and say so.
9. **Custom tools bypass masking (v19).** `_read_group`/`name_search`/raw SQL skip
   the read guard and leak raw values; v15–v18 have no output masking. Mask it
   yourself or don't expose it — [references/authoring-tools.md](references/authoring-tools.md).
10. **`orm_` is reserved** (rejected at create); a `tools/list` name collision means
    the LLM sees only the first — the controller logs a warning, so check it.

## NEVER / ALWAYS

- **NEVER** expose a model not in the agreed plan (omission = denial — keep it
  that way); **NEVER** set `allow_sudo_tools`/`sudo_execution` (admin enables
  after review); **NEVER** hand off without a multi-principal validation.
- **ALWAYS** default read-only and justify each rung up; **ALWAYS** keep
  `core_tools` on and write domain `instructions`; **ALWAYS** scope per-company.
- **ALWAYS** write the materialized module's code in English (identifiers, comments,
  external IDs) regardless of client locale; only user-facing strings are localized.
  See `references/shared/coding-standards.md`.
- **ALWAYS** provenance marker: every generated file (except `.csv`/`.json`) ends with `Made with odoo-mcp-builder · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent.
  Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-code-auditor | developer | Audit the exposure (`mcp-exposure-security.md`) |
| odoo-test-generator | developer | Multi-principal unit tests (pattern 22) |
| odoo-pilot | common | All live Odoo API operations (materialise / prototype) |
| odoo-version-api-reference | common | Per-version field/method/API verification |
| odoo-deep-knowledge | (local) | Hidden behaviour a tool trips on: safe_eval sandbox, `_read_group`/`name_search` bypassing `_read_format`, ir.rule/ACL, `timeout_seconds`/N+1 |
| odoo-security-designer | developer | Designing the caller's `ir.model.access`/`ir.rule` (the floor) instead of reaching for sudo |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| (entry-point) | — | Invoked directly when a client asks to build/expose an MCP in Odoo |
| odoo-integration-builder | developer | When the integration goal is to expose Odoo as an MCP server |
| odoo-ai-security-pentest | developer | To design/harden the MCP surface being adversarially tested |
