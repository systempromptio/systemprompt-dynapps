# Organic Module Design

How to make a custom Odoo module feel **native to the ERP** instead of a parallel
island bolted on top. Shared by `odoo-module-creator` and `odoo-orm-modeler`
(and usable by `odoo-module-upgrader` to make migrated custom more organic).
**Do NOT reimplement this in a skill — reference this document.**

## What "organic" means

A module is organic when it is designed **inside-out**: it reuses what the
instance already provides, builds new surface **on top of standard engines**, and
**plugs into existing ERP flows** rather than duplicating them. Correct code that
duplicates `stage_id` with its own `state`, or builds its own PDF/numbering/
messaging, is still *foreign* — it ages badly and migrates worse.

**Grounding rule (read first).** The source of truth for "what already exists and
how to extend it" is **the Odoo SOURCE for the target version + the Indaws Core
catalog** — never memory. Verify every model/field/method name against the
version's source (`$ODOO_SRC`, `$ODOO_ENTERPRISE_SRC`, `$OCA_SRC`,
`indawsodoo`). A **live instance is an OPTIONAL overlay** (see §4): valuable when
reachable, never required — the three axes below hold fully offline.

The three axes, in order: **reuse before create → ride standard engines → plug
into flows.**

---

## Axis 1 — Reuse before create

For every model / field / feature, before writing it, walk this ladder and stop
at the first hit:

1. **Standard Odoo** — does a stock model/field/feature already cover it? Check
   the source for the target version.
2. **Indaws Core module** — does an installable Core module provide it? Check the
   catalog and dependency tree via `indaws-core-developer`
   (`references/module-catalog.md`, `references/dependency-tree.md`).
3. **Studio** — is it already configured as an `x_studio_*` field / automation on
   the instance? (Only verifiable with instance access — see §4.)
4. **Custom** — only what survives 1–3 is genuinely missing; build that, and only
   that.

For the full Standard / Studio / Core / Custom decision, defer to
`common-skills:odoo-classification-guide` and, for whether the dev is warranted at all,
`common-skills:dev-assessment`. Note the **AI shift on Studio↔Custom** (governed module +
AI/skills review now outweighs ungoverned Studio for non-trivial logic) documented in
`odoo-classification-guide/references/ai-impact-studio-vs-custom.md`. **Output:** a one-line reuse verdict per piece, so the
plan records *why* each custom element exists.

---

## Axis 2 — Ride the standard engines (new surface, standard underneath)

Creating a new module does **not** mean reimplementing mechanics. Pick the lowest
rung that delivers the capability; new surface should sit *on top of* a standard
engine.

| Mechanism | Use when | Effect |
|-----------|----------|--------|
| **Delegation `_inherits`** | your new model *is-a* standard record (e.g. a specialized partner/product) | your record owns + delegates to the standard one; inherits its fields & behaviour |
| **Classic `_inherit`** | you extend the standard model in place | adds fields/methods to the existing model |
| **Mixins** | the new model needs a whole subsystem | inherit it for free: `mail.thread` (chatter), `mail.activity.mixin` (activities), `portal.mixin` (portal + access token), `rating.mixin`, `analytic.mixin` |
| **Compose via standard records/API** | a transactional capability | *create and drive* `account.move`, `stock.picking`, `sale.order`, `ir.sequence`, `mail.template`, `mail.activity`, `ir.cron`, `base.automation` — don't reimplement the engine |
| **Related / computed** | you only need to surface standard data | expose it without copying |
| **Standard infra** | reports / wizards / scheduling / security | QWeb + `report_action`, `TransientModel`, `ir.cron`, existing groups & record rules |

`_inherit` vs `_inherits` vs new model: see `odoo-orm-modeler`
`references/inheritance-and-relations.md`.

**Anti-patterns (the "foreign" tells):**
- own `state` field instead of `stage_id` / standard statusbar
- hand-built PDF instead of the QWeb report engine
- custom messaging/notes instead of `mail.thread` chatter
- own counter instead of `ir.sequence`
- polling/custom scheduler instead of `ir.cron` / `base.automation`
- copied data instead of related/computed

### When YOU are the engine (building a reusable capability, not consuming a standard one)

The table above is about *riding* standard engines. When the reusable capability is genuinely new
and *you* build it (a geospatial engine, a routing engine, a scoring engine…), keep the SAME
organic discipline internally: the engine must be **business-agnostic** and injected, never
coupled to a business model.

- Implement the engine as **`AbstractModel` / service methods** that know nothing about any
  concrete business model. It operates on data passed in / on `self`, not on `self.env['sale.order']`.
- **Inject the capability via a mixin the business model inherits** — the business model composes
  engine + itself: `_inherit = ["business.model", "engine.mixin"]`. The engine module never
  imports or references the business module (dependency points one way: business → engine).
- **Bindings are declared per consumer**, not in the engine module (same rule as agnostic wizards:
  `binding_model_id` on each consumer's action — see `odoo-wizard-builder`
  `references/context-and-defaults.md`).
- Litmus test: you could add a second, unrelated consumer of the engine **without touching the
  engine module**. If you can't, the engine leaked business knowledge.

---

## Axis 3 — Plug into the ERP flows

Hook the new surface into the machinery the ERP already runs, so it participates
in standard flows instead of running beside them.

| Need | Hook here (don't parallel) |
|------|----------------------------|
| Conversation / log / tracking | `mail.thread` + `tracking=True` on fields; `message_post` |
| Tasks / reminders | `mail.activity.mixin` + activity types |
| Customer-facing | `portal.mixin` + `access_url` / `access_token`; portal controllers |
| Accounting impact | create `account.move` / `account.move.line` via standard API; respect journals & posting |
| Inventory impact | `stock.move` / `stock.picking`; never write `qty_available` directly |
| Menus / views / actions | **inherit** existing views (`xpath`), extend menus & actions; don't clone |
| Access | reuse/extend existing **groups** + `ir.rule`; new groups only for genuinely new privileges |
| Multi-company | `company_id` + company `ir.rule` + `check_company` on relations |
| i18n | translatable UI strings; never hardcode user-facing text |

For cross-model behaviour and integration maps, lean on `odoo-deep-knowledge`.

---

## Axis 4 — Instance awareness (OPTIONAL overlay)

When — and only when — the target instance is reachable, enrich the design with
live facts; otherwise skip this axis entirely (axes 1–3 stand on source + Core):

- **Access paths** (see `common:references/shared/odoo-connection.md`): Indaws MCP
  (corporate v19), `odoo-pilot` (pre-v19 / headless), or a dump driven through
  `odoo-context-briefing`.
- **What it adds:** confirm the exact field/method exists *in that instance &
  version*; discover reusable existing fields; **detect Studio `x_studio_*`** on
  the model you're extending; flag **collisions** with other custom modules.
- **Degradation:** no instance → no §4. The module is still organic from §1–3;
  you just verify names against source instead of the live model.

> Most client instances will NOT have the MCP installed. Treat live access as a
> bonus, never a dependency. The grounding that makes a module organic is source +
> Core, which you always have.

---

## Organic pre-flight checklist (before writing code)

- [ ] Every model/field/feature passed the **reuse ladder** (Axis 1); each custom
      piece has a one-line "why not standard/Core/Studio" justification.
- [ ] Each new piece **rides a standard engine** where one exists (Axis 2); no
      anti-pattern from the list.
- [ ] New surface **hooks into** chatter/activities/portal/accounting/stock/
      views/groups/multi-company/i18n as applicable (Axis 3).
- [ ] All model/field/method names **verified against the target-version source**
      (not memory).
- [ ] If the instance is reachable: verified live + Studio/collision check (Axis 4).
