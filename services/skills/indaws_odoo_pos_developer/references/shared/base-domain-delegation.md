# Base ⇄ domain delegation (shared contract)

Single source of truth for how domain-specific developer skills relate to the base
scaffolder `odoo-module-creator`. Skills that follow this pattern link here in ONE line
instead of re-explaining it — keeps the wording from drifting across skills.

**Applies to:** `odoo-carrier-developer`, `odoo-pos-developer`, `odoo-barcode-developer`
(and any future `odoo-*-developer` domain layer). ⇄ with `odoo-module-creator`.

## The rule: the invoked skill orchestrates; the other is a subroutine

The direction of support is **not fixed** — it follows the entry point:

- **User invokes the domain skill** (e.g. "extiende el POS", "intégrame Correos Express"):
  the **domain skill is the orchestrator**. It owns the flow and calls `odoo-module-creator`
  **only for the skeleton, and only when the module does not exist yet**. Most domain tasks
  extend a module that already exists → no scaffolding needed → the base is not invoked at all.
- **User invokes `odoo-module-creator`** ("créame un módulo nuevo que extienda X"): the
  **base is the orchestrator**. It scaffolds and **routes the domain fill-in** to the domain
  skill (see its "Delegates to" table).

Control stays with whoever the user called. Both delegation links exist in the metadata
(base→domain for routing, domain→base for scaffold), which forms a cycle in the graph —
that is expected and safe **because this rule breaks it at runtime**.

## Anti-loop guard (read before delegating)

- When the **base** routes domain work to a domain skill, that domain skill **does NOT call
  back** to the base for scaffolding — the base already produced the skeleton. Fill domain only.
- When a **domain skill** delegates the skeleton to the base, the base's own STEP 1 already
  STOPs for user confirmation — **do not add a second confirmation gate** on the domain side.

## Responsibility split (who owns what)

| Concern | Owner |
|---|---|
| manifest + brand profile, `__init__` | **base** (`odoo-module-creator`) |
| security (ACL + ir.rules), multi-company classification | **base** |
| README/CHANGELOG, `static/description/icon.png` | **base** |
| provenance marker, anti-laxity self-audit gate | **base** |
| unit + E2E flow tests | **base** |
| `odoo-module-scanner` + `odoo-version-api-reference` subagents (for scaffold) | **base** |
| domain models/methods, widgets, client actions, request classes, screens, RPC, labels | **domain skill** |
| domain data (records, cron), domain-specific views | **domain skill** |
| domain gotchas and version diffs | **domain skill** |

## Handoff payload (domain skill → base)

When a domain skill delegates the skeleton, pass the base:

- module technical name + target version + brand profile
- `depends` (the domain skill knows the right ones — e.g. `stock_delivery`+`mail` for carrier,
  `point_of_sale` for POS, `stock_barcode`/`mrp_workorder` for barcode)
- the **domain model list** so the base emits ACL + ir.rules for each `company_id` model
- any **asset-bundle rule** specific to the app (e.g. POS: declare the module's glob into
  `point_of_sale._assets_pos` via the manifest `assets` key — not file-by-file)

## Governance rule

There is **ONE** module creator: `odoo-module-creator`. Everything app/domain-specific is a
`-developer` (or domain) layer that **delegates the skeleton to the base**. Do **not** create
new `-creator` skills that re-implement scaffolding — that reintroduces the duplication this
contract exists to prevent.
