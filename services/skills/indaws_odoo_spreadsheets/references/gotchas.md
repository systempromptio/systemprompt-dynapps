# Gotchas — Real Failure Patterns

Built from production failures. Ordered by frequency of occurrence.

## JSON Structure

1. **`formulaId` must be string** — In v18, `formulaId` values are `"1"`, `"2"` (strings), not integers. Same for `pivotNextId`. Int causes type mismatches in filter `pivotFields` lookups.
2. **`type: "ODOO"` required on pivots** — Every Odoo pivot definition MUST include `"type": "ODOO"`. Missing → `Cannot find undefined in this registry!` JS error.
3. **v18 version = integer (`21`)** — Never a string. v19 version = semver string (`"18.5.1"`) — never an integer, no `odooVersion` key.
4. **Pivot keys = UUIDs, formulas use numeric `formulaId`** — `str(uuid.uuid4())` for the key, `"1"` for the formulaId. Don't mix them up.
5. **`pivotNextId` > max formulaId** — Same for `listNextId`. If max formulaId is `"3"`, pivotNextId must be `4` or higher.

## Formulas

6. **Formulas use `,` ALWAYS** — Never `;` in JSON. Locale separator (`;`) only affects UI display, not stored formulas.
7. **Pivot domain = Python string** — `"[(\"field\", \"=\", \"value\")]"` with commas inside string.
8. **Chart domain = JSON array** — `[["field","=","value"]]` — DIFFERENT format from pivot domains!
9. **PIVOT() is an array function** — `=PIVOT(1)` auto-expands. Parameters: `pivot_id, [row_count], [include_total], [include_column_titles], [column_count]`. Use `0` to disable.

## Scorecards

10. **Scorecard keyValue = cell ref** — `"Data!B2"` not `"=PIVOT.VALUE(...)"`. Inline formulas render BLANK.
11. **Scorecard baseline must be NUMERIC** — Point to raw numeric cell, never FORMAT.LARGE.NUMBER or string-formatted. Strings return 0%.
12. **Every scorecard needs a navigation link** — Place `odoo://view` link cell below. A scorecard without navigation is incomplete.
13. **Prefer `baselineMode: "difference"`** — `"percentage"` divides by previous period. If previous=0, result is undefined (0%). `"difference"` shows absolute delta safely.
14. **Scorecard navigation: 3 solutions** — Sol.2 (RECOMMENDED): Fake scorecards with `odoo://view` link row. Sol.1: link below native scorecard. Sol.3: Custom menus (fragile). **None apply global dashboard filters on click** — only static domains. See `navigation.md`.

## Layout

15. **Figures ≠ Cells: two independent layers** — Figures use absolute pixel `x,y` (v18) or col/row (v19). Cells use grid refs. They do NOT coordinate. PIVOT expanding into figure zone causes overlap.
16. **Figures first, PIVOTs below (v18 MANDATORY)** — All fixed-size figures at TOP. All dynamic content below. Never place figures between PIVOTs. See `layout.md`.
17. **Styles are range-based** — `"A1:H1": 1` at sheet level, NOT cell-level styling.

## Charts

18. **`odoo_*` charts need groupBy** — Empty `groupBy: []` renders empty chart. Use scorecards for ungrouped aggregates.
19. **Always prefer `odoo_*` charts** — Drill-down, filters, dynamic data. Standard `bar`/`line`/`pie` are static — use only as last resort.
20. **Single-series bar legend** — Set `"legendPosition": "none"` to avoid generic "Serie 1" label.

## Filters

21. **fieldMatching type must match field** — `"date"` for Date, `"datetime"` for Datetime, `"many2one"` for M2O.
22. **`#field` prefix for positional** — `"#user_id"` = position N in group (Top N rankings).
23. **Never duplicate filter conditions in domains** — Global filters ADD conditions (AND). Same condition in both domain and filter → double-filtered → empty results. Use `"[]"` domains.
24. **Maximize global filters** — Analyze model `fields_get` and add filters for ALL useful date, M2O, M2M, selection fields. Connect every filter to every pivot via `fieldMatching`.

## Data Verification

25. **Verify percentage field storage** — Query sample data first. Fields may store 52.43 (already %) or 0.5243 (needs *100). Getting this wrong → values like 5242.7%.
26. **Verify relation model names** — `fields_get` → `relation` attribute. Wrong `modelName` in filter → filter silently fails. `x_studio_field` may point to unexpected model.
27. **Boolean ≠ sum** — `sum` not available for boolean fields. Use domain-filtered pivots with `__count`.

## Modification

28. **Modifying existing dashboards** — Follow Modification Decision Tree in `layout.md`. Classify element type → assess SAFE/RISKY/IMPOSSIBLE → warn user → reorganize if needed. Never silently place figure between dynamic content.
