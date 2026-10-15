# Troubleshooting Reference

## Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `pivot.colGroupBys is undefined` | Wrong `version` or `odooVersion`, or missing `columns`/`rows` in pivot | Use `version: 21` (or `22`), `odooVersion: 12`. Ensure every pivot has `columns` and `rows` arrays (even if empty `[]`) |
| `Cannot find undefined in this registry!` | Pivot definition missing `"type": "ODOO"` field | Add `"type": "ODOO"` to every Odoo pivot definition. Without it, the JS registry lookup fails |
| `No hay ninguna tabla dinámica con el ID "uuid..."` | `=PIVOT()` formula references UUID instead of formulaId | Use `=PIVOT(1)` not `=PIVOT("9ca92e1f-...")`. formulaId is the numeric string |
| Figures overlap with PIVOT data | Figures between PIVOTs — **architectural limitation of v18**. Figures use absolute pixel positions that NEVER move. PIVOTs expand dynamically. No coordination exists. Adding margin is a temporary fix that breaks when data grows | **Reorganize to Pattern 1**: ALL figures (scorecards, charts) at the TOP, ALL PIVOTs BELOW. This is the ONLY permanent fix. Follow the Modification Decision Tree in `layout.md`. Never place figures between dynamic content |
| `formulaId` type mismatch in filters | v18 stores `formulaId` as string (`"1"`), new pivot used int (`6`) | Always use strings for `formulaId` and `pivotNextId` in v18: `"6"` not `6` |
| Empty dashboard (blank page) | `spreadsheet_data` is minimal; Enterprise serves from snapshot | For new dashboards, put full data in `spreadsheet_data`. Use `get_readonly_dashboard` to read real data |
| `forbidden opcode(s) in ... IMPORT_NAME` | Server action code uses `import` statements | Remove all `import` lines. Use pre-injected `datetime`, `dateutil` directly: `datetime.date.today()`, `dateutil.relativedelta.relativedelta()` |
| `No puede modificar... Ningún grupo actualmente permite` | Missing `ir.model.access` for custom model | Create access rights: admin group (full CRUD) + user group (read-only) |
| Duplicate snapshot records | `.mapped('x_many2one').ids` was `.mapped('x_many2one')` | For Many2one fields, `.mapped()` returns recordsets, not IDs. Add `.ids` |
| Filter shows but doesn't affect pivot | Missing `fieldMatching` entry | Add filter UUID to pivot's `fieldMatching` with correct `chain` and `type` |
| Format not applied to cells | Using cell-level `{"style": 1}` instead of sheet-level ranges | Use sheet-level `"styles": {"A1:H1": 1}`, not cell-level |
| Percentage shows as 300% | Format `"0.00%"` applied to field storing `3.0` | `0.00%` multiplies ×100. Store as decimal (0.03) or use `"0.00"` format |
| Menu item not visible | `ir.actions.server` linked directly to `ir.ui.menu` | Menus need `ir.actions.act_window` or `ir.actions.client`. Bind server action to model's Action dropdown instead |
| Year filter doesn't filter data | Connected to integer `x_year` field instead of date field | Date filters must connect to `date` or `datetime` fields via fieldMatching. Use `x_snapshot_date`, not `x_year` |
| Pivot domain uses JSON array | `"domain": [["field","=",value]]` | Must be Python string: `"domain": "[(\"field\", \"=\", value)]"` |
| `#BAD_EXPR` in PIVOT.VALUE cells | Formula uses `;` as argument separator in JSON | **Always use `,` (comma) in JSON formulas**, never `;`. The `formulaArgSeparator: ";"` setting only affects UI display, NOT stored JSON. Example: `=PIVOT.VALUE(1, "amount:sum")` not `=PIVOT.VALUE(1; "amount:sum")` |
| `InvalidDomainError: Invalid domain representation` | Pivot domain uses `;` as tuple separator | Domains are Python strings — always use commas: `"[(\"state\", \"!=\", \"cancelled\")]"` not `"[(\"state\"; \"!=\"; \"cancelled\")]"`. Semicolons are NEVER valid in Python tuples |
| `#BAD_EXPR` in all formula cells | Double-encoding of JSON string quotes | When building JSON in Python, cell content `"=PIVOT.VALUE(1, \"amount:sum\")"` — the `\"` is standard JSON escaping. Verify with `json.loads()` that the parsed content is `=PIVOT.VALUE(1, "amount:sum")` |
| Date filter doesn't affect `datetime` field | fieldMatching uses `"type": "date"` for a `datetime` field | Use `"type": "datetime"` for `fields.Datetime` model fields (e.g., `date_done`, `create_date`, `open_datetime`) |
| Chart domain uses Python string format | Charts expect JSON array domains, not Python strings | In chart `searchParams.domain` use JSON array `[["field","=","value"]]`, NOT Python string `"[(\"field\",\"=\",\"value\")]"`. This is different from pivot domains |
| `#N/A` in PIVOT.VALUE with positional index | Using `"user_id"` instead of `"#user_id"` for positional lookup | Add `#` prefix for positional: `=PIVOT.VALUE(1, "__count", "#user_id", 1)` — the `#` means "1st in the group", not a record ID |
| Version 21 vs 22 confusion | Odoo standard uses 21, custom dashboards may use 22 | Both work in Odoo 18. Version 21 is the native format; 22 triggers a harmless migration. Prefer 21 for compatibility |
| Dynamic table not expanding | Using `type: "static"` with single-cell range | Dynamic tables need `"type": "dynamic"` with single-cell range (e.g., `"A1"` not `"A1:G10"`) |
| `includeChildren` filter not working | Missing `includeChildren: true` on relation filter | Only works with hierarchical models (`product.category`, etc.). Set `"includeChildren": true` in the filter definition |
| Missing `revisionId` warning | JSON root missing `revisionId` field | Add `"revisionId": "START_REVISION"` to root JSON for collaborative editing compatibility |
| Scorecard shows title but no number | `keyValue` contains an inline formula like `=PIVOT.VALUE(...)` instead of a cell reference | Scorecards do NOT evaluate formulas. Put PIVOT.VALUE in a Data sheet cell and reference it: `"keyValue": "Data!B2"`. See data-sheet-pattern.md |
| `odoo_bar`/`odoo_line`/`odoo_pie` chart renders empty (axes visible, no data) | `groupBy` is empty `[]` in both `metaData` and `searchParams` | Odoo charts MUST have at least one `groupBy` field to create X-axis categories. For aggregate KPIs without grouping, use scorecards or standard `bar` chart with cell references instead |
| Boolean pivot measure returns total record count instead of True count | Using `count` aggregator on boolean field — counts all non-null values (True AND False) | `sum` is not available for booleans. Use domain-filtered pivot pattern: add `("bool_field", "=", True)` to pivot domain and use `__count` measure. See pivots.md Boolean Counting Pattern |
| Bar chart legend shows generic "Serie 1" | Single data series with `legendPosition` set to `"top"` and `dataSetsHaveTitle: false` | Set `"legendPosition": "none"` to hide legend on single-series charts, or use `"dataSetsHaveTitle": true` with a labeled first cell in the dataRange |
| Chart has no drill-down (clicking bars/slices does nothing) | Using standard `bar`/`line`/`pie` chart (cell-range based) instead of `odoo_bar`/`odoo_line`/`odoo_pie` | Standard charts are static — no Odoo integration. Always prefer `odoo_*` types for drill-down and filter support. If no `groupBy` field exists, use multiple scorecards or propose creating a computed field in Odoo to enable proper `odoo_*` chart grouping |
| Text/links hidden behind figures or figures overlap with cell content | Figures (pixel-positioned) placed over rows that contain cell content, or cell content placed in rows within a figure zone | Figures float over cells — they cover any cell content at the same pixel position. Plan layout in zones: each zone is either figures-only or cells-only. Calculate cumulative row heights to align figure Y positions with cell rows. See charts-core.md Positioning Strategy |
| Links appear duplicated or misaligned under scorecards | Link cells placed in consecutive columns (A,B,C) but scorecards span ~2 columns each at 160px/col | Use columns A, C, E for links under 3 scorecards at x=0, x=320, x=640. Each scorecard spans ~2 grid columns |
| Filter + domain double-filtering returns empty results | Pivot domain has `[("date", ">=", "2026-01-01")]` AND a date filter is connected via fieldMatching — both conditions apply (AND), over-filtering the data | **Never duplicate filter conditions in pivot/list/chart domains.** Use empty domains `"[]"` and let global filters handle all dynamic filtering. Filters add extra conditions to domains, they don't replace them |
| Scorecard baseline shows 0% despite data existing | `baseline` points to a FORMAT.LARGE.NUMBER cell (returns string "171k") instead of raw numeric cell. Scorecard can't calculate % difference from a string → shows 0% | Point `baseline` to raw numeric cell (e.g., `"Data!C2"` with `=PIVOT.VALUE(2, "measure:sum")`), NOT to FORMAT.LARGE.NUMBER. Also: `baselineMode: "percentage"` with previous=0 returns 0% — use `"difference"` when previous period may lack data |
| Percentage KPI shows absurd value (e.g., 5242.7%) | Formula multiplies by 100 a field that already stores percentages directly (52.43 = 52.43%, not 0.5243) | **Always verify field storage format** with sample `read()` before building formulas. If field stores 52.43, use `=ROUND(B7, 1) & "%"`. If stores 0.5243, use `=ROUND(B7*100, 1) & "%"` |
| `odoo_*` chart renders empty (title visible, no data) despite correct groupBy | Missing `"mode"` property in `metaData` and/or using `"comparison": {}` instead of `null` in `searchParams` | Add `"mode": "bar"/"line"/"pie"` to `metaData`. Set `"comparison": null` (not `{}`). Set `"order": null` (not string). Copy structure from a working `odoo_*` chart in the same instance |
| `odoo_*` chart ignores some global filters | Chart `fieldMatching` only connects a subset of filters — unconnected filters are silently ignored for that chart | Include ALL `globalFilter` IDs in every chart's `fieldMatching`, not just the "relevant" ones. Every filter must have an entry mapping to the corresponding field chain |
| Scorecard `humanize` + FORMAT.LARGE.NUMBER conflict | `keyValue` points to FORMAT.LARGE.NUMBER cell AND `humanize: true` — double formatting produces inconsistent results | Choose ONE: either `humanize: true` with raw numeric keyValue (preferred), or `humanize: false` with FORMAT.LARGE.NUMBER keyValue. Never combine both |
| Scorecard not clickable despite `chartOdooMenusReferences` entry | Menu referenced has `active=False` or is under an inactive parent → frontend menu service doesn't load it → `hasOdooMenu` returns false | Menus MUST be `active=True` and under an active parent. Place custom menus under a deep existing tree (e.g., `Module > Configuration > Dashboard Actions` with `sequence: 999`) instead of using `active=False`. **Consider using fake scorecards (Solution 2) instead** — zero Odoo setup, no menu visibility issues |
| Fake scorecard card shows raw number without formatting | Number cell references `=Data!B2` which returns raw PIVOT.VALUE (e.g., 170811.45) | Format in Data sheet: use `FORMAT.LARGE.NUMBER(PIVOT.VALUE(...))` for compact display (171k), or apply cell number format `#,##0` for thousands separator. Alternatively use `=ROUND(Data!B2, 0)` |
| Fake scorecard link not clickable | `odoo://view` link JSON has escaping issues — common when building links manually | **Always build links programmatically** with `json.dumps()`. Never hand-escape JSON. Use the `odoo_link()` helper pattern from `navigation.md` Solution 2 |
| Fake scorecard colors not showing | Using cell-level styles `{"style": 1}` instead of range-based `"styles": {"A3:B3": 1}` | Styles in Odoo spreadsheets are **range-based on the sheet level**, not cell-level. Use `sheet["styles"]["A3:B3"] = style_id` |
| `cum_y` calculation returns wrong Y for charts | `row_sizes.get(str(r), 26)` returns dict instead of int — row sizes are stored as `{"size": 28}` not plain int | Use `row_sizes.get(str(r), {"size": 26})["size"]` to extract the integer height from the row dict |
| `#SPILL!` error on `=PIVOT()` formula | A `=PIVOT(N)` expands downward and collides with content in a lower cell. Common with multi-level row grouping (`rows: [{fieldName: "A"}, {fieldName: "B"}]`) which generates far more rows than single-level: header + (group subtotals × sub-rows per group) + total | **Estimation formula for multi-level:** `rows = 2 (header) + Σ(1 + sub_count_per_group) + 1 (total)`. Example: 3 rappels × 9 partners = 2+3+9+1 = 15 rows minimum, but with monthly column headers add +1-2 rows. **Always add 30%+ margin** between consecutive PIVOTs. Use `read_group` to count distinct values per grouping level BEFORE placing cells. See layout.md Expansion Problem |

## v19-Specific Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `version` type error / migration fails | Using integer `21` in v19 JSON | v19 requires semver string: `"version": "18.5.10"`. Remove `odooVersion` key |
| Figure not rendering / position wrong | Using `x`/`y` pixel positioning in v19 | v19 uses `col`/`row`/`offset`: `{"col": 0, "row": 0, "offset": {"x": 208, "y": 9}}` |
| `get_readonly_dashboard` not found | Method renamed in v19 | Use `_get_serialized_readonly_dashboard()` in v19 (returns JSON string, not dict) |
| `company_id` field error | Field changed to Many2many | v19 uses `company_ids` (M2M): `[(6, 0, [company_id])]` instead of `company_id: id` |
| `rangeType` ignored / filter broken | Property removed in v19 | v19 date filters no longer use `rangeType`. Date presets also renamed (e.g., `last_year` → `last_12_months`) |
| `ODOO.FILTER.VALUE` returns object | v19 changed return type to structured data | For v18 behavior in v19, use `ODOO.FILTER.VALUE.V18()`. Migration auto-renames existing formulas |
| New chart type not rendering | Missing `chartId` or `dataSets` on chart | v19 requires `chartId` (UUID) and `dataSets` (array, can be `[]`) on ALL `odoo_*` chart types |

## Pre-Build Verification (BEFORE writing JSON)

**Most dashboard errors are caused by incorrect assumptions about field formats. Always verify BEFORE building:**

```python
# 1. Check field types and relations
fields = call("model", "fields_get", [field_list], {"attributes": ["type", "relation", "string"]})
# → Verify: many2one relations match filter modelName, field types match fieldMatching types

# 2. Read sample records to see actual values
records = call("model", "read", [sample_ids], {"fields": field_list})
# → Verify: percentage fields (0.52 vs 52.43?), monetary precision, boolean distribution

# 3. Check aggregated totals
agg = call("model", "read_group", [[domain]], {"fields": measures, "groupby": []})
# → Verify: totals match expected dashboard values

# 4. Copy chart structure from working dashboards
dashboards = call("spreadsheet.dashboard", "search_read", [[]], {"fields": ["name", "spreadsheet_data"]})
# → Extract odoo_* chart JSON from a dashboard that works → use as template
```

## Debugging Workflow

### 1. Read the actual data being served

```python
# v18
result = env['spreadsheet.dashboard'].get_readonly_dashboard([id])
snapshot = json.loads(result['snapshot'])
# v19
result = env['spreadsheet.dashboard']._get_serialized_readonly_dashboard([id])
snapshot = json.loads(result)
print(snapshot.get('version'))
print(snapshot.get('pivots', {}).keys())
```

### 2. Check pivot structure

```python
for uid, pivot in snapshot['pivots'].items():
    print(f"UUID: {uid[:12]}...")
    print(f"  formulaId: {pivot.get('formulaId')}")
    print(f"  model: {pivot.get('model')}")
    print(f"  columns: {pivot.get('columns')}")
    print(f"  rows: {pivot.get('rows')}")
    print(f"  measures: {[m['id'] for m in pivot.get('measures', [])]}")
    print(f"  domain: {pivot.get('domain')}")
    print(f"  fieldMatching keys: {list(pivot.get('fieldMatching', {}).keys())}")
```

### 3. Check filter ↔ pivot connections

```python
filters = snapshot.get('globalFilters', [])
for f in filters:
    print(f"Filter: {f['label']} ({f['type']}) id={f['id'][:12]}...")
    for uid, pivot in snapshot['pivots'].items():
        fm = pivot.get('fieldMatching', {})
        if f['id'] in fm:
            print(f"  → Pivot {pivot['formulaId']}: chain={fm[f['id']].get('chain')}")
        else:
            print(f"  ✗ Pivot {pivot['formulaId']}: NOT connected")
```

### 4. Validate cell formulas

```python
for sheet in snapshot['sheets']:
    for cell_ref, cell in sheet.get('cells', {}).items():
        content = cell.get('content', '')
        if 'PIVOT' in content or 'LIST' in content:
            print(f"  {sheet['name']}!{cell_ref}: {content}")
```

### 5. Test pivot data availability

```bash
# Verify the model has data
./scripts/search_records.sh model.name '[]' '["id"]' 5

# Verify specific domain returns results
./scripts/search_records.sh model.name '[["year","=",2026]]' '["id"]' 5
```

## Performance Considerations

- **Computed non-stored fields** in pivot measures: Very slow on large datasets. Odoo must compute each record individually before aggregating. Use snapshot models for these.
- **Too many pivot groups**: Pivots with 100+ row groups or 50+ column groups become slow and unreadable. Use `limit` or narrower domains.
- **Multiple pivots per sheet**: Each pivot makes independent RPC calls. 5+ pivots on one sheet can be slow to load.
- **Global filters**: Each filter change triggers re-computation of ALL connected pivots/lists/charts. Keep connections minimal.
