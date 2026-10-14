---
name: indaws-odoo-spreadsheets
version: 1.1.1
effort: medium
model: opus
description: "Create and manage Odoo 18/19 spreadsheet dashboards via API. Build pivot tables, scorecards, charts (bar/line/pie/funnel/geo), filters, and deploy to Odoo. Use when: create spreadsheet, crear dashboard, spreadsheet Odoo, cuadro de mando, pivot dashboard, odoo spreadsheet, spreadsheet template, KPI dashboard, informe dinamico."
---

# Odoo Spreadsheets Skill

Create and deploy Odoo 18/19 spreadsheet dashboards via API. Covers model analysis, pivot/list/chart design, JSON construction, filters, styling, and deployment.

## Design Principles

| Pillar | Rule |
|--------|------|
| **Dinamismo** | All data refreshes in real time. Use `PIVOT.VALUE` formulas, global filters with `fieldMatching`, and `odoo_*` chart types. Never embed fixed numbers. |
| **Maxima informacion** | Every element must be navigable. `odoo_*` charts (drill-down on click). Every scorecard needs an `odoo://view` link below it. Maximize global filters from `fields_get` analysis. |

## Prerequisites

- Odoo v18/v19 Enterprise with `spreadsheet_dashboard` module
- Active connection via `odoo-pilot` skill

## Workflow

### Step 0: Gather Requirements [STOP]

Collect missing info. Ask all missing questions at once:

| # | Need | Ask if missing |
|---|------|---------------|
| 1 | Odoo version | "Que version de Odoo? (18 o 19)" |
| 2 | Data model | "Que datos? (ventas, CRM, contabilidad, stock, RRHH...)" |
| 3 | Visualizations | "Que tipo? (pivot, scorecards, graficos, ranking, lista, gauge)" -- version-compatible only |
| 4 | Filters | "Que filtros? (fecha, comercial, cliente, producto...)" |
| 5 | Deploy target | "Donde? (URL, BD)" -- skip if user just wants JSON |

Rules: Ask only missing. If user says "tu decides", propose standard layout and confirm. Never load references until Q1-Q3 answered. Validate version compatibility first: load `references/version-compatibility.md`.

### Step 0.5: Verify Real Data [STOP if anomalies]

Query actual data via `odoo-pilot`: `fields_get`, `read` sample records, `read_group`, `search_read` on `spreadsheet.dashboard`. See `references/model-analysis.md`.

### Step 1: Load References (on demand)

Always load (pick by version): `core-v18.md` OR `core-v19.md` + `styles.md`. Then load per feature needed (see Reference Index below).

### Step 2: Design and Build

Plan vertical layout BEFORE writing figures. JSON root templates in `core-v18.md` / `core-v19.md`. Key: locale must be `es_ES` with `,` as formula separator in JSON (`;` only in UI display).

| Complexity | Sheets | When |
|-----------|--------|------|
| Simple | 1 sheet | Data table + filter |
| Standard | 2 sheets (Dashboard + Data) | Scorecards, KPI comparison |
| Advanced | 3-7 sheets | Multiple analysis sections |

### Step 3: Validate

Read `references/gotchas.md`. Top 5 killers: `formulaId` must be string, `type: "ODOO"` required on pivots, scorecard keyValue = cell ref only, figures first / PIVOTs below (v18), formulas always use `,` in JSON.

### Step 4: Deploy

Via `odoo-pilot`. Details in `dashboard-api-v18.md` / `dashboard-api-v19.md`.

See references/shared/odoo-connection.md for connection pattern.

## GOTCHAS

1. **`formulaId` as integer**: MUST be string (`"1"` not `1`). Causes silent filter failures where filters appear to work but data does not update.
2. **Missing `type: "ODOO"` on pivots**: Without it, Odoo's spreadsheet registry throws a JS error and the pivot does not render.
3. **Scorecard keyValue with inline formula**: keyValue MUST be a cell reference (`"Data!B2"`), not an inline formula (`"=PIVOT.VALUE(...)"`). Inline formulas render blank.
4. **Figures placed after PIVOTs in v18**: In v18, figures and pivots are two independent layers. Figures MUST be placed first (top rows), PIVOTs below. Mixing them causes visual overlap.
5. **Formula separator `;` in JSON**: Formulas in JSON MUST use `,` as separator. The `;` is only for UI display. Using `;` in JSON causes parse errors.
6. **Modifying existing dashboards blindly**: Always read the existing JSON first. The Modification Decision Tree in `layout.md` prevents the most expensive mistakes.
7. **Non-stored computed fields in pivots**: Pivot measures MUST use stored fields. Non-stored computed fields silently return zero.

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | fields_get, read_group, CRUD on spreadsheet.dashboard | Bash inline |
| odoo-module-creator | developer | Convenciones de naming y diseño de modelos para campos del dashboard | No |
| odoo-code-auditor | developer | Patrones de seguridad y rendimiento de modelos custom consultados en dashboards | No |
| indaws-core-developer | global | Convenciones de capas Indaws y fixes Odoo 18/19 al diseñar dashboards | No |

## Consumed by

Standalone. Used when creating or modifying Odoo spreadsheet dashboards.

## References

| File | Content | Load when |
|------|---------|-----------|
| `references/core-v18.md` | JSON schema, figure positioning (x/y), locale -- v18 | v18 projects |
| `references/core-v19.md` | JSON schema, positioning (col/row/offset), v19 features | v19 projects |
| `references/charts-core.md` | Scorecards, odoo_line/bar/pie, gauge, combo, layout | Charts needed |
| `references/charts-v19.md` | 9 new chart types, carousel, zoom, fullscreen | v19 charts |
| `references/filters-core.md` | Date/relation/text filters, fieldMatching -- v18 | Filters needed |
| `references/filters-v19.md` | numeric/boolean/selection filters, relative periods | v19 filters |
| `references/formulas-core.md` | 16 Odoo formulas, locale rule, combos | Formulas needed |
| `references/formulas-extended.md` | VLOOKUP/EDATE/INDEX patterns | Advanced formulas |
| `references/dashboard-api-v18.md` | CRUD, groups, snapshots, refresh -- v18 | Deploy v18 |
| `references/dashboard-api-v19.md` | v19 delta: company_ids, favorites, HTTP | Deploy v19 |
| `references/odoo-internals.md` | Domains, stored/computed, relations, security | Model analysis |
| `references/pivots.md` | Definitions, measures, #positional, granularities | Pivots needed |
| `references/lists.md` | ODOO.LIST definitions and formulas | Lists needed |
| `references/styles.md` | Colors, 13 formats, borders, wrapping, sizing | Always |
| `references/conditional-formatting.md` | DataBar, ColorScale, IconSet, CellIs | Rankings/visual |
| `references/tables.md` | Static + dynamic tables, customTableStyles | Tables needed |
| `references/data-sheet-pattern.md` | Dashboard+Data architecture, multi-sheet, panes | Standard+ builds |
| `references/navigation.md` | odoo://view, fake scorecards, 3 nav solutions | Clickable KPIs |
| `references/model-analysis.md` | Direct vs snapshot, field analysis, report models | Step 0.5 |
| `references/layout.md` | Two-layer positioning, modification decision tree | Design + modify |
| `references/version-compatibility.md` | v18/v19 chart/filter/feature matrix, alternatives | Step 0 |
| `references/gotchas.md` | 28 real failure patterns with fixes | Step 3 |
| `references/troubleshooting.md` | 35+ errors with cause and fix | Debugging |

**Total: ~38K tokens. Typical v18 dashboard loads 8-12K. Typical v19: 9-14K.**
