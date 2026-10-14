---
name: indaws-odoo-carrier-developer
description: >-
  Crea módulos de transportista Odoo (v17-v19) que integran APIs de empresas de
  envío: patrón dynamic dispatch, send_shipping/rate_shipment/get_tracking_link/
  cancel_shipment, request service class, extensión delivery.carrier y
  stock.picking, etiquetas y tracking URL.
  Triggers: "crear módulo de carrier Odoo", "integrar transportista",
  "delivery carrier module", "send_shipping method", "rate_shipment Odoo",
  "nuevo módulo de envío", "shipping provider integration Odoo".
version: 2.2.0
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, EnterPlanMode]
effort: high
model: opus
---

# Odoo Carrier Developer Skill

**Document ID:** DEV-SKILL-009 | **Classification:** Critical Technical Know-How

Encapsulates the 3-layer module system (`delivery` -> `stock_delivery` -> `delivery_<provider>`), the dynamic dispatch pattern (`getattr` + `delivery_type`), the request service class pattern, and the interface contracts.

## Base: `odoo-module-creator` (do NOT reimplement scaffolding)

DOMAIN layer on top of `odoo-module-creator` — full contract in
`references/shared/base-domain-delegation.md`. **Delegate the skeleton** (manifest, security
ACL+ir.rules, multi-company, README, icon, provenance, anti-laxity gate, tests, + module-scanner and
version-api-reference); never re-create it here. This skill owns only the **domain**:
`<provider>_request.py`, the `delivery.carrier` extension (dispatch + 4 methods + config fields with
`groups="base.group_system"`), `stock.package.type`/`stock.picking`, data/cron, form view,
labels/base64, provider pitfalls.

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Gather carrier API specs | sonnet | Read documentation (domain only) |
| Scaffold base module | — | **DELEGATE → `odoo-module-creator`** (skeleton + security + tests) |
| Design + generate domain code | **opus** | Carrier-specific implementation on top of the skeleton |
| Validate domain contracts | sonnet | 4-method signatures, `delivery_type` consistency, labels |

## Reference Files

| File | Content |
|------|---------|
| [carrier-architecture.md](references/carrier-architecture.md) | 3-layer system, model fields, dispatch pattern, DeliveryPackage/DeliveryCommodity |
| [api-contracts.md](references/api-contracts.md) | Complete method signatures for all 4 mandatory + optional methods |
| [provider-patterns.md](references/provider-patterns.md) | REST/SOAP patterns, request class templates, auth retry, extended features (POD, pickup, manifest) |
| [workflow-steps.md](references/workflow-steps.md) | Steps 0-5: info gathering, plan mode, analysis, module creation templates, validation, **install + run the suite** |
| [module-structure.md](references/module-structure.md) | Directory layout, master data file rules, cron template |
| [common-pitfalls.md](references/common-pitfalls.md) | 47 pitfalls: architecture, API, labels, state tracking, error handling, version-specific + the "verified the hard way" sets (mocked-tests-hide-wire-bugs, docIn RAW, 401 retry, base helpers, webhook secret; no-status-API, daily token, no-sandbox, warnings≠errors, mm dimensions, cancel-that-only-flags, ZPL consequences; **mock-at-the-transport-boundary, success-criterion-from-the-success-sample, assertions-that-cannot-fail, undocumented-prod-host, master-data-lost-in-the-port**) |
| [carrier-platform-map.md](references/carrier-platform-map.md) | **ES carrier → shared backend platform map** (WSintegracion/asmred, GTS/Alertran, SAGEC, SEUR ATLAS, Correos Express): transport, auth, docIn/response shape, quirks. Identify platform → clone the sibling module |
| [migration-v18-to-v19.md](references/migration-v18-to-v19.md) | **Canonical Odoo skeleton, reference providers to copy (CE/EE/OCA), OCA reuse map, and the v18→v19 migration checklist** |

Read in the table's order, starting at workflow-steps.md (Steps 0-5).

## When to Use

- Create a new delivery carrier module / integrate a transport company API (REST, SOAP, or other)
- Implement shipping label generation, tracking, or rate calculation
- Extend an existing carrier module with new features
- Understand Odoo's delivery carrier architecture — **NOT for 16.0**: `stock_delivery` appears in 17.0 and the picking-side contract lives there; per-version method map in [carrier-architecture.md](references/carrier-architecture.md)

## Execution Sequence

```
STEP 0: Domain gathering (AskUserQuestion) -- provider info ONLY: name + delivery_type key,
        API docs/auth/type, features (rate/ship/label/track/cancel/POD/pickup), credentials,
        service & package types, weight unit, AND explicitly: any status retrieval
        (endpoint / webhook / public portal)? any sandbox host? "no" to either changes
        the design. (version, tech name, paths, multi-company and brand are asked by
        odoo-module-creator in STEP 1 — do NOT duplicate them here.)
STEP 0.5: Plan Mode (optional) -- provider API analysis + mapping to the 4 methods ONLY.
STEP 1: Scaffold base [DELEGATE → odoo-module-creator] with: module `delivery_<provider>`,
        depends `['stock_delivery','mail']` (+ `indaws_delivery_state_tracking` if used),
        target version, brand profile, domain model list. It emits manifest, __init__,
        security, multi-company, README, icon, provenance, test scaffold, and handles the
        CONFIRMATION. Delegation is conditional — inline path in workflow-steps.md STEP 1.3.
STEP 2: Fill domain code [opus, this skill] on top of the skeleton:
        <provider>_request.py, delivery_<provider>.py (dispatch + 4 methods + config fields),
        stock_package_type / stock_picking (if needed), data/delivery_<provider>.xml,
        ir_cron.xml, and the carrier form view. Detail in workflow-steps.md STEP 2-3.
STEP 3: Implementation Details (state tracking, fallbacks, labels, packages, auth retry)
STEP 4: Domain validation (4-method contracts, delivery_type consistency, labels/base64,
        version-specific delivery). Generic gates (multi-company ir.rules, anti-laxity,
        E2E tests) are run by odoo-module-creator, not here.
STEP 5: INSTALL AND RUN THE SUITE in a throwaway container (~5 min, NOT optional).
        Steps 0-4 validate by READING, which never catches a fixture the code under test
        discards. Report real pass/fail numbers. Recipe: workflow-steps.md STEP 5.
```

## Gotchas (Top 6)

0. **Code in English** -- identifiers, comments, docstrings always in English regardless of carrier/client locale; only user-facing strings are localized. See `references/shared/coding-standards.md`.
1. **`ondelete` on `selection_add`** -- always define uninstall behavior for delivery_type
2. **`sudo()` for credentials** -- API keys use `groups="base.group_system"`
3. **Multi-package handling** -- `send_shipping` pickings may have multiple packages
4. **Auth token expiry** -- implement retry logic in `_send_request()` for session-based carriers
5. **Double base64 labels** -- some carriers (Correos Express) require `b64decode(b64decode(data))`
6. **Mock `session.post`, never `_send_request`** -- patching the helper drops the envelope from coverage: 38 green tests, dead on every real call (#43)

See [common-pitfalls.md](references/common-pitfalls.md) for all 47 pitfalls.

## Prohibitions

- NEVER proceed to implementation without collecting ALL required info (Step 0)
- NEVER skip user confirmation before generating files (Step 1)
- NEVER hardcode API URLs -- use `prod_environment` toggle
- NEVER leave `prod_environment` looking functional when the carrier has **no sandbox**: one host
  means the toggle switches nothing and every call ships for real while the user thinks they are
  testing. Refuse to register with it off, or say so in the field help — safety, not style (#38)
- NEVER store credentials without `groups="base.group_system"`
- NEVER use `numbercall` in cron XML (removed in Odoo 18)
- NEVER report a module as done on an **unexecuted** test suite (STEP 5)

**Provenance marker:** stamp the **domain** files with `Made with odoo-carrier-developer · v<the version in this file's frontmatter>` — never a literal copied into prose, it drifts; the base stamps the skeleton. Gate and syntax: `references/shared/skill-provenance-marker.md`. Field types and command tuples: `references/shared/field-mapping-syntax.md`.

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| **odoo-module-creator** | developer | **BASE** — scaffold the skeleton (manifest, security ACL+ir.rules, multi-company, README, icon, provenance, anti-laxity gate, unit + E2E tests) BEFORE filling domain. It also runs module-scanner + version-api-reference. | Skill (preferred) or subagent **if fan-out is available**; otherwise inline — see below |
| odoo-version-api-reference | common | Domain API doubts only (module-creator already loads it during scaffold) | Subagent (optional) |
| odoo-module-scanner | common | Scan an existing carrier module before extending it | Subagent (optional) |

**Delegation is conditional, never mandatory** — a "mandatory subagent" step simply does not run where
fan-out is forbidden and nobody finds out. Inline fallback: `workflow-steps.md` STEP 1.3.

## Consumed by

**odoo-module-creator** (developer) routes the carrier domain here when a new module's `depends`
include `stock_delivery`/`delivery`; **odoo-integration-builder** (developer), for carrier/connector
modules inside an integration.

## Success Criteria

- Module installs without errors
- `delivery_type` selection shows the new provider
- Configuration fields visible when provider selected
- All 4 methods return correct signatures (see [api-contracts.md](references/api-contracts.md))
- Labels attached to pickings; tracking URL works **or `get_tracking_link` returns `False`
  deliberately** (never a hand-built URL — see `c-ontime.md`): no tracking at all is a valid outcome
- API credentials protected, debug logging functional
- **The test suite has actually been RUN** and its pass/fail numbers reported (STEP 5)
- **Wire format verified against the real endpoint** when reachable (read-only smoke test: server
  date/time, track a fake ref, or bad-credentials call). Green MOCKED tests alone do NOT prove this.
- **Success criterion taken from the provider's sample of a CORRECT response** (not inferred from
  its error table) and prod host confirmed by DNS; both pinned by a literal assertion (#44, #45, #46)
