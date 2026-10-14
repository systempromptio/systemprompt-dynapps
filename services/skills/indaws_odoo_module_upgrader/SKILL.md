---
name: indaws-odoo-module-upgrader
description: >-
  Porta código de módulos Odoo entre versiones (14→19): vistas XML, API Python,
  OWL/JS, SCSS de temas y manifest. También triage post-upgrade invocado desde
  odoo-database-upgrader FASE 3.5. NOT repo completo (→ dynapps-code-migration);
  NOT scripts migrations/{version} (→ odoo-module-migration-scripts).
  Triggers: "migra este módulo a v18/v19", "upgrade Odoo module", "fix version
  compatibility", "migrate theme between versions", "resolve Odoo 17/18/19
  migration errors", "port module to Odoo 18".
version: 1.1.7
effort: high
model: opus
---

# Odoo Module Upgrader

Migrates Odoo module source code between versions (14→19).

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Scan module | sonnet | Delegate to odoo-module-scanner |
| Analyze migration path | **opus** | Version diff decisions |
| Apply migration patterns | **opus** | Code transformation |
| Validate | sonnet | Test compilation, imports |

## When to Read Which Reference

| Situation | Read |
|---|---|
| 16→17 migration (attrs, states, name_get) | `patterns/odoo16_to_17.md` |
| 17→18 migration (slug, reports, layout) | `patterns/odoo17_to_18.md` |
| 18→19 migration (RPC, kanban, groups) | `patterns/odoo18_to_19.md` |
| JavaScript/OWL v19 migration | `patterns/javascript_v19.md` |
| Common patterns (all versions) | `patterns/common_patterns.md` |
| QWeb PDF report migration | `fixes/qweb_report_fixes.md` |
| JavaScript batch fixes | `fixes/javascript_fixes.md` |
| XML automated fixes v19 | `fixes/xml_fixes.md` |
| XML automated fixes v16→v17 (attrs, states) | `fixes/xml_fixes_v16_v17.md` |
| XML automated fixes v17→v18 (t-raw, bootstrap) | `fixes/xml_fixes_v17_v18.md` |
| Maintaining one feature across coexisting live branches (no drift) | `reference/parallel-version-forward-port.md` |
| Post-upgrade triage (OCA/CUSTOM/INDAWS) | `reference/post_upgrade_triage.md` |
| Error diagnosis | `reference/error_catalog.md` |
| Full example | `examples/upgrade-example.md` |
| Version API diffs | `odoo-version-api-reference` skill |

## Workflow

### STEP 0: Context & Triage [STOP — ask user]

**0a. Gather info:**
- Module path(s) to upgrade
- Source version → target version
- If coming from `odoo-database-upgrader` FASE 3.5: read `logs/upgrade_credentials.json`
- **Pre-fill from registry:** Check if `desarrollos/PROJECT_REGISTRY.md` exists. If found, extract module classification (CE/EE/OCA/Indaws Core/Custom) — this pre-fills step 0b. Also read `desarrollos/PROJECT_REGISTRY_DEPS.md` for the dependency tree to determine migration order.

**0b. Classify modules** (CRITICAL — do this before writing any code):

Classify each module in `to upgrade` / `to install` state as **CORE / OCA / INDAWS / CUSTOM / OBSOLETE** — detection (author/name SQL) + per-type action in `reference/post_upgrade_triage.md`. If the registry already classified them, use that as the starting point.

**0c. Present triage to user** and wait for confirmation per module.

**0d. For CUSTOM modules only — inventario + checklist DETERMINISTAS [scripts, ~0 tokens LLM]:**
- **Inventario:** `../odoo-module-scanner/scripts/scan_module.py <path>` (models/fields/views/security/JS).
- **Checklist de patrones:** `scripts/detect_upgrade_patterns.py --module <path> --from <N> --to <M> --md`
  → lista EXACTA de qué patrones de versión aplican y DÓNDE (attrs/tree/name_get/t-raw/kanban-box/RPC…). NO adivines el checklist.
- **Fallback:** si un script falla, invoca `odoo-module-scanner` / detecta a mano con `patterns/` y `fixes/`.

Presenta el plan (módulo, nº ficheros, checklist) al usuario. Las TRANSFORMACIONES (steps 2-5) son juicio → las hace el LLM (opus).

### STEP 1: Manifest Update
- Version: `"18.0.1.0.0"` format
- Add `'license': 'LGPL-3'` if missing
- Remove dead depends (`http_routing` absorbed in v18, etc.)
- Declare `external_dependencies`

### STEP 2: Apply Version-Specific Patterns
**Load API ref [SONNET SUBAGENT]:** Launch Agent(model: sonnet, description: "load API ref v{version}") — read odoo-version-api-reference v18-base/{topics} + diffs/v{version}-diff.md if != v18. Output: filtered API summary with version diffs. Read the appropriate `patterns/odooXX_to_YY.md` file(s) for the version jump.
For multi-version jumps (e.g. 16→18), apply patterns sequentially: 16→17 then 17→18.

### STEP 3: XML/View Transformations
Apply using patterns from the reference files. Key transforms by version:
- **16→17**: `attrs` → expressions, `states` → `invisible`
- **17→18**: **`<tree>` → `<list>`** (aquí, NO en 16→17 — corregido 2026-08-07: `('tree','Tree')` en `odoo17/odoo/addons/base/models/ir_ui_view.py:163`, `View.type` vs `('list','List')` en `odoo18/…:153`), `t-raw` → `t-out`, Bootstrap class updates, report layout
- **18→19**: search `<group>` removal, `kanban-box` → `card`, `active_id` → `id`

### STEP 4: Python API Migrations
- **16→17**: `name_get` → `_compute_display_name`, `_name_search` signature
- **17→18**: `slug`/`unslug` relocation, report render API
- **18→19**: `@api.returns` removal, `groups_id` → `group_ids`, `url_for` move

### STEP 5: JavaScript/Frontend
- **16→17**: `@odoo-module` annotation mandatory
- **17→18**: Tour registration changes
- **18→19**: RPC service → fetch, `.include()` → `patch()`, `publicWidget` → `Interaction`

### STEP 6: Validate
```bash
docker compose run --rm odoo odoo-bin -u <module> --stop-after-init
```

### STEP 7: CHANGELOG
Update or create `CHANGELOG.md` with migration entry. See `examples/upgrade-example.md`.

## GOTCHAS
0. **Code stays in English** — when migrating/rewriting, never translate identifiers, comments or docstrings to the client's language; source code is always English. Only user-facing strings are localized. See `references/shared/coding-standards.md`.
1. **OCA modules: install v18 from OCA repo** — do NOT migrate code. Only migrate if no v18 port exists.
2. **l10n_es moved to core in v18** — author "Spanish Localization Team" ships with CE/EE, not OCA.
3. **Studio models (x_*) lack access rules after upgrade** — add `ir.model.access` or AccessError.
4. **attrs conversion is NOT mechanical** — nested `'|'` domains need careful Python `or` conversion.
5. **Multi-version jumps are sequential** — 16→18 = apply 16→17 then 17→18. Never skip versions.
6. **http_routing absorbed in v18** — remove from depends, now in core `ir.http`.
7. **name_get works in v17** but deprecated — only refactor if actively maintaining.
8. **indaws_* modules** — delegate to `common-skills:indaws-core-auditor`, don't use generic patterns.
9. **product_pack, auditlog, server_action_mass_edit** — have v18 OCA ports, check before migrating.
10. **Deployment target matters** — odoo.sh vs indaws.cloud (`indaws/odoo-slim`). Test with correct image.
11. **Remove dead code BEFORE migrating** — less code = fewer errors. Use `odoo-module-scanner` first.
12. **Dropped fields stay in DB** — column persists in PostgreSQL. Use `pre-migrate.py` or leave it.
13. **Migration scripts** — consult `odoo-version-api-reference` → `migration-scripts-decision.md` + `migration-scripts-writing.md`.
14. **`t-out="0"` / `t-raw="0"` is the `t-call` SLOT, not the number** — when porting QWeb templates (esp. `t-esc`/`t-raw`→`t-out` conversions), `<t t-out="0"/>` inside a template called via `t-call` injects the **caller's body**. If you rewrite arch programmatically or convert directives, do NOT treat `"0"` as a numeric expression there — leaving it or converting it wrong drops the called content.
15. **Provenance marker (ALWAYS)** — every rewritten/generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-module-upgrader · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Scripts

| Script | Purpose |
|---|---|
| `scripts/detect_upgrade_patterns.py` | **Motor detección** — checklist determinista de patrones aplicables por salto (JSON/`--md`) |
| `scripts/upgrade_manifest.py` | Batch update manifests (version, license, deps) |
| `scripts/fix_rpc_service.py` | Automated RPC→fetch migration for JS files |

For shared ecosystem patterns, see common:references/shared/.
## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-version-api-reference | common | Migration patterns per version |
| odoo-module-scanner | common | Inventory before migrating |
| odoo-deep-knowledge | odoo-knowledge | Breaking changes y comportamientos v19 antes de migrar código |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| service-desk | consultant | Escalated bugs requiring migration |
| migration-assistant | common | Module migration phase |
