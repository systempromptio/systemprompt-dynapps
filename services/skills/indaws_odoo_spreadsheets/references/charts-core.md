# Charts & Figures Reference (v18 core + v18/v19 shared)

## Chart Types

### Scorecard (KPI Card)

```json
{
  "type": "scorecard",
  "title": {"text": "Revenue", "bold": true, "color": "#434343"},
  "background": "#FFFFFF",
  "keyValue": "Data!D4",
  "baseline": "Data!E4",
  "baselineDescr": "desde el último periodo",
  "baselineMode": "difference",
  "baselineColorUp": "#00A04A",
  "baselineColorDown": "#DC6965",
  "humanize": true
}
```

- `keyValue`: **MUST be a cell reference** to a NUMERIC cell (e.g., `"Data!B2"` containing `=PIVOT.VALUE(...)`). Put PIVOT.VALUE formulas in a Data sheet and reference the cell. **Do NOT point to FORMAT.LARGE.NUMBER cells** — `humanize: true` already handles number formatting, and FORMAT.LARGE.NUMBER returns a string which can break baseline calculations.
- `baseline`: Comparison value — also a **cell reference to a NUMERIC cell** (e.g., `"Data!C2"` containing `=PIVOT.VALUE(2, "measure:sum")`). Use empty string `""` if no comparison needed. **CRITICAL**: baseline MUST point to a raw number, NEVER to a FORMAT.LARGE.NUMBER or concatenated string cell. The scorecard needs a number to calculate difference/percentage. Strings produce 0%.
- `baselineMode`: `"difference"` (absolute delta), `"percentage"` (% change), or `"text"` (display raw value). **Prefer `"difference"` when previous period may have no data** — `"percentage"` divides by previous, and if previous=0 the result is undefined (shows 0%).
- `humanize`: `true` → "563k" instead of "563,000". **Do NOT combine with FORMAT.LARGE.NUMBER** — humanize already formats the number. Using both creates redundancy and potential conflicts.

**CRITICAL**: Scorecards do NOT evaluate inline formulas. `"keyValue": "=PIVOT.VALUE(1, \"amount:sum\")"` will render **blank** — the scorecard shows no number. Always use the Data Sheet pattern: put formulas in cells, reference cells from scorecards. See data-sheet-pattern.md.

**MANDATORY**: Every scorecard MUST have a companion `odoo://view` link in a cell row immediately below it. A scorecard without a navigation link is incomplete — the user sees a number but can't explore the underlying records. Place the link in the cell that aligns with the scorecard's X position (see Positioning Strategy). The `odoo://view` link is the PRIMARY navigation mechanism for scorecards — it provides filtered access to the exact records behind the KPI. `chartOdooMenusReferences` is OPTIONAL (only opens a generic menu with no filtering).

**Scorecard + Link pattern:**
```
Figure:  Scorecard at x=0, y=55, w=300, h=102     → shows "154"
Cell:    A13 = [Ver tareas →](odoo://view/{"viewType":"list","action":{"domain":"...","modelName":"project.task","views":[[false,"list"],[false,"form"]]},"name":"Tasks"})
```

The `odoo://view` link should include the same domain filter as the pivot that feeds the scorecard, so the user sees exactly the records behind the number. See navigation.md for full syntax.

**Background colors**: `#FFFFFF` (white), `#FFF7ED` (warm orange), `#FEF2F2` (pink), `#EFF6FF` (blue), `#F0F4F7` (gray-blue), `#ECFDF5` (green)
**Baseline colors**: Up `#00A04A` (green), Down `#DC6965` (red). Swap for inverted (lower=better).

### Alternative: Fake Scorecards (Cell-Based KPI Cards)

When clickable navigation matters more than native scorecard aesthetics (humanize, baseline), replace scorecard figures with **styled merged cells**. Each card = 3 rows (title + number + `odoo://view` link), 2 columns wide, with color-coded backgrounds. No figures, no `chartOdooMenusReferences`, no Odoo-side menu setup.

**See `navigation.md` → Solution 2** for the complete implementation pattern with style definitions, link builder, card builder function, and layout grid.

| Feature | Native scorecard | Fake scorecard (cells) |
|---------|-----------------|----------------------|
| Humanize (171k) | YES | No (use FORMAT.LARGE.NUMBER in Data sheet) |
| Baseline comparison | YES | No |
| Animated values | YES | No |
| Clickable navigation | Only via `chartOdooMenusReferences` (fragile) | YES — `odoo://view` link in card |
| Color backgrounds | YES | YES (fillColor in styles) |
| Odoo-side setup | Menus + actions (Sol.3) | None |
| Grid lines | Must be OFF in figure zone | `areGridLinesVisible: false` |

### Odoo Line Chart

```json
{
  "type": "odoo_line",
  "title": {"text": "Monthly Sales"}, "background": "#FFFFFF",
  "legendPosition": "top", "verticalAxisPosition": "left",
  "stacked": false, "fillArea": true,
  "metaData": {"groupBy": ["date:month"], "measure": "amount_untaxed", "order": null, "resModel": "sale.report", "mode": "line"},
  "searchParams": {"domain": [["state", "not in", ["draft", "cancel"]]], "groupBy": ["date:month"], "orderBy": [], "comparison": null, "context": {}},
  "fieldMatching": {"filter-uuid": {"chain": "date", "type": "date", "offset": 0}}
}
```

- `fillArea`: `true` = area chart
- `cumulative`: `true` = running total (native line)
- `cumulated`/`cumulatedStart`: For `odoo_line` only — accumulates over time

### Odoo Bar Chart

```json
{
  "type": "odoo_bar",
  "title": {"text": "Sales by Category"}, "background": "#FFFFFF",
  "legendPosition": "top", "stacked": false, "verticalAxisPosition": "left",
  "metaData": {"groupBy": ["categ_id"], "measure": "price_subtotal", "order": null, "resModel": "account.invoice.report", "mode": "bar"},
  "searchParams": {"domain": [], "groupBy": ["categ_id"], "orderBy": [], "comparison": null, "context": {}},
  "fieldMatching": {}
}
```

- `stacked: true` for stacked bars. Multiple `groupBy` creates series.
- `fieldMatching` uses `"datetime"` for timestamp fields, dot notation for relations

**CRITICAL**: `odoo_bar`/`odoo_line`/`odoo_pie` charts **MUST have a non-empty `groupBy`** in both `metaData` and `searchParams`. An empty `groupBy: []` produces an empty chart with no data — there are no categories for the X-axis. If you need to show aggregate KPIs without grouping (e.g., total savings), use **scorecards** instead.

**CRITICAL**: `odoo_*` chart `metaData` MUST include `"mode"` matching the chart type (`"bar"`, `"line"`, `"pie"`). Use `"order": null` (not a string). `searchParams` MUST include `"comparison": null` (not `{}`). Missing or wrong values can cause charts to render empty or with incorrect scale. **Always copy the structure from a working chart** in the same Odoo instance before building new charts.

**CRITICAL**: Chart `fieldMatching` must include entries for ALL global filters, not just the relevant ones. If a filter exists in `globalFilters` but has no entry in a chart's `fieldMatching`, the filter is silently ignored for that chart — which can cause inconsistent behavior. Connect every filter to every chart.

**DESIGN PRINCIPLE**: Always prefer `odoo_*` chart types over standard cell-range charts (`bar`, `line`, `pie`). Odoo-native charts provide **drill-down** (click a bar → opens filtered list view), **global filter integration**, and **dynamic data refresh**. Standard cell-range charts are static visualizations with none of these features.

**When standard charts are NOT an option, look for alternatives:**

| Scenario | Preferred solution | Why |
|----------|-------------------|-----|
| Data grouped by a field (category, month, user) | `odoo_bar` / `odoo_line` / `odoo_pie` | Drill-down, filters, dynamic |
| Single KPI number | `scorecard` referencing Data sheet cell | Native KPI display |
| Multiple independent KPIs (e.g., 6 boolean counts) | **Multiple scorecards** (one per KPI) | Each shows one value, native |
| No groupBy field exists but chart is needed | **Ask user to create a computed field** (selection/many2one) in Odoo that consolidates the data into categories, then use `odoo_bar` with groupBy on that field | Enables drill-down and filters |
| Last resort only — no other option works | Standard `bar` + PIVOT.VALUE cells | Static, no drill-down, no filters — document this limitation to the user |

### Odoo Pie Chart

```json
{
  "type": "odoo_pie",
  "title": {"text": "Distribution"}, "background": "#FFFFFF", "legendPosition": "right",
  "metaData": {"groupBy": ["state"], "measure": "__count", "order": null, "resModel": "sale.order", "mode": "pie"},
  "searchParams": {"domain": [], "groupBy": ["state"], "orderBy": [], "comparison": null, "context": {}},
  "fieldMatching": {}
}
```

### Standard Charts (cell-range based)

```json
{"type": "bar", "dataSets": [{"dataRange": "Sheet1!B2:B10", "label": "Revenue"}], "labelRange": "Sheet1!A2:A10", "title": {"text": "Title"}, "stacked": false, "aggregated": false}
```

Types: `"bar"`, `"line"`, `"pie"`, `"combo"`, `"scatter"`, `"waterfall"`, `"gauge"`, `"doughnut"`, `"pyramid"`

**From Odoo views** (Insert in Spreadsheet): `line`, stacked line, `bar` (column), stacked column, `area`, stacked area, `pie`
**Manual only** (Insert > Chart in spreadsheet): `combo`, horizontal bar, stacked bar, `doughnut`, `scatter`, `gauge`, `scorecard`, `waterfall`, `pyramid` (population pyramid)
- `labelsAsText`: `true` = text categories (use when labels are text strings, not numbers)
- `dataSetsHaveTitle`: `true` = first row/cell of each dataRange is used as series title
- `legendPosition`: `"none"` to hide legend entirely (recommended for single-series charts to avoid generic "Serie 1" label)

### Gauge Chart

```json
{
  "type": "gauge",
  "title": {"text": "Gross profit margin", "bold": true, "color": "#434343"},
  "dataRange": "Data!B2",
  "sectionRule": {
    "colors": {"lowerColor": "#EA9999", "middleColor": "#F9CB9C", "upperColor": "#B6D7A8"},
    "rangeMin": "0", "rangeMax": "1",
    "lowerInflectionPoint": {"type": "number", "value": "0.2"},
    "upperInflectionPoint": {"type": "number", "value": "0.5"}
  }
}
```

Swap `lowerColor`/`upperColor` for inverted metrics (lower=better).

### Combo Chart (Dual Axis)

```json
{
  "type": "combo", "dataSetsHaveTitle": true,
  "dataSets": [
    {"dataRange": "'Sheet'!E20:E30", "yAxisId": "y", "label": "Rate", "trend": {"type": "polynomial", "order": 1, "display": false}},
    {"dataRange": "'Sheet'!D20:D30", "yAxisId": "y1", "label": "Count"}
  ],
  "labelRange": "'Sheet'!B21:B30", "legendPosition": "top",
  "axesDesign": {"y1": {"title": {"text": "Orders count"}}}, "showValues": false
}
```

- `yAxisId`: `"y"` = left (line), `"y1"` = right (bars)
- `trend`: Optional trend line (polynomial order 1 = linear)

### Chart Domain Rule

**CRITICAL**: `searchParams.domain` uses **JSON array** `[["field","=","value"]]`, NOT Python string. Different from pivots!

### Chart fieldMatching

Odoo charts support `fieldMatching` in their `data` object (same syntax as pivots, see filters-core.md).

## Layout

### Standard Sizes (v18)
| Type | Width | Height | Notes |
|------|-------|--------|-------|
| Scorecard | 280-300 | 102 | 3 per row at 320px spacing |
| Gauge | 225 | 184 | Pair with scorecard above |
| Full-width chart | 940-1000 | 344-436 | Single chart across full width |
| Half-width chart | 460-480 | 344 | Two charts side by side |
| Pie | 475-525 | 344 | Best at half-width |
| Combo | 525 | 364 | Needs space for dual axes |

### Positioning Strategy (v18 — pixel-based)

**CRITICAL**: Figures (scorecards, charts) use absolute pixel `x`/`y` positioning. Cells (text, links) use row/column grid. When mixing both, **you must calculate cumulative row heights** to ensure cells and figures don't overlap.

#### Step 1: Define the visual zones top-to-bottom

Plan the dashboard vertically in **zones**. Each zone contains either figures OR cell content, never both overlapping.

```
Zone type          | Contains                        | Typical height
───────────────────|─────────────────────────────────|───────────────
Title row          | Cell with title text            | 45px (1 row)
Spacing            | Empty row                       | 10px (1 row)
Scorecard row      | 3 scorecard figures             | 102px (~4 rows of 26px)
Section header     | Cell with section text          | 30px (1 row)
Links row          | Cells with odoo://view links    | 25px (1 row)
Chart zone         | 1-2 odoo_* chart figures        | 344-436px (~13-17 rows)
Table zone         | Cells with PIVOT.HEADER/VALUE   | Variable
```

#### Step 2: Calculate cumulative Y positions

Track the running Y position as you stack zones. Example:

```
Y=0    ┌─ Row 0: Title (45px) ─────────────────────────────┐
Y=45   ├─ Row 1: Spacing (10px) ───────────────────────────┤
Y=55   ├─ Scorecards: Ahorros (3x) ── h=102 ──────────────┤
Y=157  ├─ Row 6: Spacing (20px) ──────────────────────────-┤
Y=177  ├─ Row 7: Section header "Medidas" (30px) ──────────┤
Y=207  ├─ Row 8: Spacing (8px) ────────────────────────────┤
Y=215  ├─ Scorecards: Medidas row 1 (3x) ── h=102 ────────┤
Y=317  ├─ Row 13: Links "Ver tareas →" (25px) ─────────────┤
Y=342  ├─ Row 14: Spacing (13px) ──────────────────────────┤
Y=355  ├─ Scorecards: Medidas row 2 (3x) ── h=102 ────────┤
Y=457  ├─ Row 19: Links "Ver tareas →" (25px) ─────────────┤
Y=482  └───────────────────────────────────────────────────-┘
```

#### Step 3: Assign row heights to match figure positions

Each row's height must be set so that cumulative heights align with figure Y positions. Use this formula:

```
Figure Y position = sum of all row heights from row 0 to (figure_start_row - 1)
```

**Rules:**
- **Figures float over cells** — a figure at `y=55` will cover whatever cells are at that pixel position
- **Never place cell content in rows occupied by figures** — it will be hidden behind the figure
- **Links/text rows go BETWEEN figure zones** — after one set of figures ends, before the next begins
- **Default row height is 26px** — but the dashboard viewer may adjust. Use explicit `"rows"` sizing for every row in the figure zone
- **3 scorecards per row**: `x=0, 320, 640` with `width=300` leaves 20px gaps between cards

#### Step 4: X positioning for columns of figures

| Layout | X positions | Widths |
|--------|------------|--------|
| 3 scorecards | `0, 320, 640` | `300` each (20px gaps) |
| 4 scorecards | `0, 245, 490, 735` | `230` each (15px gaps) |
| 5 scorecards | `0, 200, 400, 600, 800` | `185` each (15px gaps) |
| 2 charts side by side | `0, 500` | `480` each (20px gap) |
| 1 full-width chart | `0` | `940-1000` |
| 1 chart + 1 pie | `0, 525` | `510, 475` |

#### Link rows below scorecards

When scorecards need navigation links (`odoo://view`), place link cells in the row immediately after the scorecard zone:

```
Scorecard at x=0     → Link in column A (same row)
Scorecard at x=320   → Link in column C
Scorecard at x=640   → Link in column E
```

Use columns A, C, E (not A, B, C) because each scorecard spans ~2 columns at 160px/col.

#### Complete zone reference

| Zone | Figure Y | Cell rows | Height budget |
|------|----------|-----------|---------------|
| Title | — | Row 0 | 45px |
| Post-title spacing | — | Row 1 | 10px |
| Scorecard row N | `y = cumulative` | — (figures only) | 102px |
| Post-scorecard links | — | 1 row after scorecard zone | 25px |
| Section header | — | 1 row | 30px |
| Pre-figure spacing | — | 1 row | 5-10px |
| odoo_* chart | `y = cumulative` | — (figures only) | 344-436px |
| Table (PIVOT cells) | — | Variable rows | 26px per row |
