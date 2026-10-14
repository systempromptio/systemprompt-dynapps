---
name: indaws-odoo-version-api-reference
description: >
  Referencia completa de la API Odoo (v18 canonical + diffs v15-v19): firmas, parametros de campos,
  operadores de dominio, utilidades y migracion entre versiones.
  Triggers: "que API uso en v18", "diferencias v17 v18", "esta deprecated name_get",
  "_read_group en v19", "parametros fields.Many2one", "como uso float_compare", "@http.route".
version: 4.2.16
effort: low
model: sonnet
---
<!-- verified: v18.0 @ 2026-06 -->

# Odoo Version API Reference v4.1

Complete Odoo API documentation. **v18 is the canonical base** — learn v18 first, then check the diff file for your target version.

> **Esta skill es dato documentado: NO requiere el código fuente local.** Para
> verificar algo NUEVO contra fuente, resuelve la ruta vía
> `references/shared/odoo-source.md` (no asumas `$ODOO_SRC`); si no hay
> fuente, usa el dato documentado avisando de que no se re-verificó.

## How to use

1. **Writing code for v18?** Read the relevant `v18-base/` file directly
2. **Writing code for another version?** Read the `v18-base/` file first, then check `diffs/v{version}-diff.md` for what's different
3. **Migrating between versions?** Read both the source and target diffs + the migration checklist at the bottom of each diff

## Reference Structure

### v18 Base — Complete API (read first)

| File | Contents | When to read |
|------|----------|-------------|
| `v18-base/orm-models.md` | BaseModel: CRUD, search, navigation, display name, aggregation, context, defaults | Writing any model code |
| `v18-base/orm-models-advanced.md` | Access control, cache/flush, import/export, metadata | Security checks, raw SQL, data import |
| `v18-base/orm-fields-simple.md` | Simple field types (Char-Json) + base/computed/related parameters, automatic fields | Declaring simple fields |
| `v18-base/orm-fields-relational.md` | Relational fields (M2O, O2M, M2M) + Command class, translate, write_sequence | Relational fields, writing to relations |
| `v18-base/orm-environment.md` | Environment attrs/methods, API decorators, Transaction, Cache, Prefetch, SQL execution | Using env, decorators, raw SQL |
| `v18-base/orm-domains.md` | 18 operators (incl. any/child_of/=?), AND/OR helpers, constants, SQL compilation, index recs | Writing domains, search optimization |
| `v18-base/http.md` | @route params, Request/Response/Stream, auth types, dispatchers, session, common patterns | Controllers, portal, API endpoints |
| `v18-base/javascript-components.md` | Module system, OWL components, lifecycle hooks, reactivity | Component creation, setup() |
| `v18-base/javascript-hooks.md` | 70+ Odoo hooks (essential, UI, view/form, field) | Using hooks in setup() |
| `v18-base/javascript-services.md` | 31 services, ORM service usage, registering services | Service access, ORM calls from JS |
| `v18-base/javascript-views-widgets.md` | Registry, view architecture, 88+ field widgets, templates, assets, common patterns | Views, widgets, tours, patching |
| `v18-base/tools.md` | float_utils, date_utils, mail/HTML, image, safe_eval, formatting, ormcache, SQL utils, misc | Any utility function |
| `v18-base/exceptions.md` | Exception hierarchy (7 types), when to use each, catching patterns | Error handling |
| `v18-base/base-models.md` | ir.rule, ir.cron, ir.sequence, ir.attachment, ir.ui.view, ir.actions, res.partner/company/users, mixins (mail.thread, portal.mixin) | Inheriting base models, security, cron |
| `v18-base/modules.md` | Registry, manifest fields, module loading, migration scripts (pre/post/end) | Module structure, version upgrades |
| `v18-base/services.md` | RPC execution flow, DB management, authentication, server architecture | Integration, deployment, debugging |

### Version Diffs (read after v18 base)

| File | Contents | When to read |
|------|----------|-------------|
| `diffs/v19-diff.md` | 20 breaking changes + new features + migration checklist | Targeting v19 or migrating v18→v19 |
| `diffs/v20-diff.md` | ⚠ Solo motor de PDF (Paper Muncher, **reportado/sin verificar**); negativo v18/v19 verificado | Preguntas sobre Paper Muncher / motor PDF v20 |
| `diffs/v17-diff.md` | What v17 lacks vs v18 + migration checklist | Targeting v17 or migrating v17→v18 |
| `diffs/v16-diff.md` | What v16 lacks vs v18 (_read_group change, no precompute, no aggregator) + migration checklist | Targeting v16 or migrating v16→v18 |
| `diffs/v15-diff.md` | What v15 lacks vs v18 (no Command, no display_name, legacy JS) + migration checklist | Targeting v15 or migrating v15→v18 |

### Patterns & Security (version-independent)

| File | Contents | When to read |
|------|----------|-------------|
| `engine-patterns.md` | Performance patterns: precompute, indexes, N+1, batch, store decisions, security patterns | Writing optimized code |
| `engine-internals.md` | Cache, prefetch, flush mechanics, computed engine, domain→SQL, security layers, OWL, QWeb | Debugging, understanding "why" |
| `gotchas.md` | 40 real error patterns Claude makes frequently | Before generating any code |
| `secure-coding-quick.md` | 12 security gates — rules only, no examples (~50 lines) | Always read when generating code |
| `secure-coding-blocking.md` | Gates 1-7 BLOQUEANTES with full code examples | When you need blocking gate examples |
| `secure-coding-warning.md` | Gates 8-12 WARNING with full code examples + summary matrix | When you need warning gate examples |
| `migration-scripts-decision.md` | **Decision matrix**: cuándo necesitas pre/post/end-migrate, por tipo de cambio (campos, modelos, relaciones, XML, manifest) + templates + checklist | Bumping module version, renaming fields/models, changing field types |
| `migration-scripts-writing.md` | **Cómo escribirlos**: `odoo.tools.sql` helpers, patrones (batch, backup/restore, recompute, M2O→M2M), anti-patrones, testing, checklist | Writing any migration script |

## Quick Version Matrix

| Feature | 15.0 | 16.0 | 17.0 | 18.0 | 19.0 |
|---------|------|------|------|------|------|
| `name_get()` | Standard | Deprecated (16.4) | Deprecated | Avoid | Removed |
| `_compute_display_name` | — | From 16.4 | Standard | Standard | Standard |
| `group_operator` | Standard | Standard | Deprecated (17.2) | Avoid | Removed |
| `aggregator` | — | — | From 17.2 | Standard | Standard |
| `precompute=True` | — | — | Available | Standard | Standard |
| `Command` class | — | Available | Standard | Standard | Standard |
| `is_storable=True` | — | — | — | Standard | Standard |
| `<tree>` / `<list>` | `<tree>` | `<tree>` | `<tree>` | Both | `<list>` only |
| Search `<group>` | Yes | Yes | Yes | Yes | Removed |
| Kanban template | `kanban-box` | `kanban-box` | `kanban-box` | `kanban-box` | `card` |
| JS modules | `odoo.define` | Both | Both | ES modules | ES modules |
| JS tests | QUnit | QUnit | QUnit | Hoot | Hoot |
| Tours | `tour.register` | `tour.register` | `registry.add` | `registry.add` | `registry.add` |
| `freeze_time` | freezegun | freezegun | freezegun | Built-in | Built-in |
| `@api.returns` | Standard | Standard | Standard | Standard | Removed |
| `expression.AND/OR` | Standard | Standard | Standard | Standard | `Domain()` |
| RPC service (frontend) | Yes | Yes | Yes | Yes | Removed |
| Form import | `.common` | `.common` | `odoo.tests` | `odoo.tests` | `odoo.tests` |

For shared ecosystem patterns, see `common:references/shared/`.

> **Ciclo de vida / retirada de Odoo.sh / licenciamiento:** esta matriz cubre la **API**, no las fechas de negocio.
> Para el calendario de retirada de Odoo.sh por versión y la política del +25% en renovación Enterprise, consulta el
> SSOT `references/shared/odoo-lifecycle-licensing.md`.

## GOTCHAS
1. **v18-base is the canonical reference, not the only version** — always check diffs/ for the target version.
2. **Engine patterns reference is version-agnostic** — precompute, indexes work the same across v15-v19.
3. **gotchas.md has 51 entries** — don't read all of them. Search by keyword for the relevant ones.
4. **JavaScript references are split across 4 files** — components, hooks, services, views-widgets. Check all when doing JS.
5. **Secure coding has 3 tiers** — quick (always read), warning (read if security-sensitive), blocking (read before delivery).

## Workflow
1. Determine target Odoo version from user request or PROJECT_REGISTRY
2. Read `references/v18-base/` for the canonical API (always start here)
3. If target != v18: read `references/diffs/v{target}-diff.md` for differences
4. If writing code: read `references/engine-patterns.md` for performance patterns
5. If security-sensitive: read `references/secure-coding-quick.md` (always), then warning/blocking tiers if needed

## Delegates to
None — this is a pure reference skill.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| odoo-database-analyzer | common | Version diffs for migration analysis |
| migration-assistant | common | Version guidance per phase |
| odoo-module-creator | developer | Version-specific API patterns |
| odoo-module-upgrader | developer | Breaking changes between versions |
| odoo-code-auditor | developer | Deprecated API detection |
| odoo-test-generator | developer | Version-specific test patterns |
| odoo-carrier-creator | developer | Version-specific carrier API |
| odoo-rebrander | developer | API changes for rebranding |
| odoo-flow-tester | common | Version-specific flow validation |
| odoo-mockup-generator | common | View tags, widgets y deltas de versión para mockups fieles |

## How other skills should reference this

```markdown
For version-specific API patterns, consult `odoo-version-api-reference`:
- `references/v18-base/orm-models.md` for ORM methods
- `references/v18-base/orm-fields-simple.md` + `orm-fields-relational.md` for field types
- `references/diffs/v{version}-diff.md` for version differences
```
