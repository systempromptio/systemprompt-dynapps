# Layout & Positioning Reference

> **Version scope**: This document applies to **Odoo v18 AND v19**. v19 figures use cell-anchored positioning (`col`, `row`, `offset`) instead of absolute pixels, but analysis of the `o_spreadsheet.js` source code confirms that **the figure plugin has NO handler for `ADD_COLUMNS_ROWS`** and the OT registry has **NO transform for row insertion → figure repositioning**. Pattern 1 (figures top, PIVOTs below) remains mandatory in both versions.

## Two Independent Layers (CRITICAL)

An Odoo spreadsheet renders **two independent layers** that do NOT coordinate automatically:

| Layer | Positioning | Dynamic? | Examples |
|-------|------------|----------|----------|
| **Cells** | Grid references (`A1`, `B5`) | Yes — `=PIVOT()` expands | Text, formulas, PIVOT(), ODOO.LIST() |
| **Figures** | Absolute pixels (`x`, `y`) | No — fixed position | Scorecards, charts, images |

**Figures float OVER cells.** They do not push cells down. Cells do not push figures down. If a PIVOT() expands into the area where a figure sits, they **overlap**.

## The Expansion Problem

### Dynamic formulas expand unpredictably

`=PIVOT(id)` at cell A4 expands:
- **Down**: 1 header + N data rows + 1 total = N+2 rows
- **Right**: (N_groups × N_measures) + total columns

The expansion size depends on **live data**, which changes with filters. You cannot know the exact size at design time.

### Estimation formula

```
rows = 1 (header) + count(distinct row groupby values) + 1 (total) + 1 (margin)
cols = 1 (row label) + count(distinct col groupby values) × count(measures) + count(measures) (total group)
```

**Example**: Pivot with 3 commercials × 12 months × 3 measures:
- Rows: 1 + 3 + 1 + 1 = 6
- Cols: 1 + 12×3 + 3 = 40

### Multi-level row grouping estimation (CRITICAL)

When `rows` has **multiple fields** (e.g., `[{fieldName: "rappel_id"}, {fieldName: "partner_id"}]`), the expansion is NOT simply `count(A) + count(B)`. Odoo generates a **nested hierarchy** with subtotal rows:

```
rows = 2 (header rows for column groups) + Σ per group_A(1 subtotal + count(sub_B in group_A)) + 1 (grand total)
```

**Example**: 3 rappels, 9 partners (distributed 4+3+2), 12 monthly columns:
- Headers: 2 rows (month labels + measure labels)
- Group 1: 1 subtotal + 4 partners = 5
- Group 2: 1 subtotal + 3 partners = 5
- Group 3: 1 subtotal + 2 partners = 3
- Grand total: 1
- **Total: 2 + 5 + 5 + 3 + 1 = 16 rows minimum**

**⚠️ ALWAYS add 30%+ margin** when placing content below a multi-level pivot. Data grows over time (new partners, new products), and `#SPILL!` errors break the dashboard silently.

**Pre-calculation script:**
```python
# Count distinct values per grouping level BEFORE placing cells
for field in pivot_row_fields:
    groups = call("model", "read_group", [domain], {"fields": [field], "groupby": [field]})
    print(f"{field}: {len(groups)} distinct values")
# Then apply the multi-level formula above
```

### Pixel estimation

Default sizes:
- Row height: **23px** (Odoo default)
- Column width: **96px** (Odoo default), often overridden to 120-200px

Custom sizes override defaults via `sheet.rows` and `sheet.cols`:
```json
"rows": {"0": {"size": 45}, "2": {"size": 32}}
"cols": {"0": {"size": 200}, "1": {"size": 120}}
```

**Pixel position of row N** = sum of heights of rows 0..N-1 (using default 23px for unspecified rows).

## Figure Positioning Rules

```json
{"id": "uuid", "x": 8, "y": 48, "width": 220, "height": 110, "tag": "chart", "data": {...}}
```

| Property | Unit | Description |
|----------|------|-------------|
| `x` | pixels | Horizontal offset from sheet left edge |
| `y` | pixels | Vertical offset from sheet top edge |
| `width` | pixels | Figure width |
| `height` | pixels | Figure height |

### Converting cell position to pixel position

To place a figure "at row 10":
```
y = sum(row_heights[0..9])
  = (rows with custom height) + (rows with default 23px)
```

**Helper calculation:**
```python
def cell_to_pixel_y(target_row, row_sizes, default_height=23):
    y = 0
    for r in range(target_row):
        y += row_sizes.get(str(r), {}).get("size", default_height)
    return y
```

## Layout Patterns for Dashboards

### Pattern 1: Figures First, Pivots Below (MANDATORY for v18)

```
FIGURE ZONE (fixed height, never changes):
  Row 0:     Title (cell, 45px)
  y=50:      [Scorecard] [Scorecard] [Scorecard] [Scorecard]  ← figures, fixed
  y=175:     [Chart 1]   [Chart 2]                             ← figures, fixed
  y=510:     [Chart 3 full-width]                              ← figures, fixed
  ~80px gap

PIVOT ZONE (dynamic, expands freely downward):
  Row N:     Section title (cell, 32px)
  Row N+1:   =PIVOT(1)     ← expands freely ↓, no ceiling
  Row N+20:  Section title
  Row N+21:  =PIVOT(2)     ← expands freely ↓
```

**Why**: Figures have fixed size. Pivots expand. Fixed things go first, expanding things last. **This is the ONLY layout that is immune to data growth.** See "Why Figures Between PIVOTs NEVER Works" section for the architectural proof.

**Rule**: When ANY modification request would require placing a figure below dynamic content or between two PIVOTs, the dashboard MUST be reorganized to follow this pattern. Show the warning and preview to the user (see "Modification Decision Tree").

### Pattern 2: Separate Sheets

- Sheet 1 "Dashboard": Only figures (scorecards + charts) referencing pivots via PIVOT.VALUE()
- Sheet 2 "Data": Only cells with =PIVOT() expansions (hidden or visible)

**Why**: Complete separation eliminates all overlap risk.

### Pattern 3: Pivots First, Figures Below (FRAGILE — NOT RECOMMENDED)

Only works if you can **reliably estimate** the pivot's row count and add sufficient margin. **This pattern WILL break when data grows beyond the margin.**

```
Row 0:     Title
Row 1:     =PIVOT(1)  ← expands ~8 rows
Row 12:    Margin (empty rows for safety)
Row 15+:   [Figures]  ← y = pixel position of row 15
```

**Risk**: Any data growth (new partners, new products, new months) expands the PIVOT into the figure zone. No margin is safe forever. **Use Pattern 1 instead.**

## Modifying Existing Dashboards (CRITICAL CHECKLIST)

When updating an existing dashboard's `spreadsheet_data`:

### Step 1: Read EVERYTHING first
```python
data = json.loads(dash.spreadsheet_data)
sheet = data["sheets"][0]
existing_cells = sheet.get("cells", {})
existing_figures = sheet.get("figures", [])  # DO NOT FORGET
existing_merges = sheet.get("merges", [])
existing_styles_sheet = sheet.get("styles", {})  # range-based
```

### Step 2: Plan the vertical layout
1. List all visual elements in order (figures + cell sections)
2. Estimate pixel height of each section
3. Calculate cumulative Y positions
4. Leave **margin rows** (2-3 empty rows) between dynamic sections

### Step 3: Reposition figures
For each figure, recalculate `x` and `y` based on the new layout:
```python
for fig in sheet["figures"]:
    fig["y"] = new_calculated_y_position
```

### Step 4: Update cells
Place cell content (titles, PIVOT() formulas) at rows that don't conflict with figures.

### Step 5: Verify no overlaps
```python
# Check that no figure occupies space where a PIVOT() will expand
for fig in sheet["figures"]:
    fig_top = fig["y"]
    fig_bottom = fig["y"] + fig["height"]
    # Ensure no =PIVOT() cell expands into [fig_top, fig_bottom] pixel range
```

## Common Mistakes

| Mistake | Result | Fix |
|---------|--------|-----|
| Replace `cells` without updating `figures` | Figures overlap with new pivot data | Always reposition figures |
| Put `=PIVOT()` above figures | Pivot expands under scorecards/charts | Put figures first, pivots below |
| No margin between sections | New data rows push into next section | Add 2-3 empty margin rows |
| Forget `figures` key exists | Original figures remain at old positions | Always read and update figures |
| Assume PIVOT size is static | Works today, breaks when data grows | Always add margin + use Pattern 1 |
| Default `colNumber: 26` too small for wide pivots | Columns beyond Z are not allocated → pivot truncated or renders incorrectly | Calculate needed cols: `1 (row label) + col_groups × measures + measures (total)`. Set `sheet["colNumber"]` to at least that + 5 margin |
| Default `rowNumber: 100` too small for multiple large pivots | Pivots below row 100 don't render | Calculate total rows needed by all PIVOTs + margins + figure zone. Set `sheet["rowNumber"]` accordingly (use 200+ for safety) |
| Large pivot without `=PIVOT(id, row_limit)` | Pivot with 50+ row groups expands massively, slow to load, causes `#SPILL!` with content below | Use `=PIVOT(id, row_limit)` to cap rows. Example: `=PIVOT(5, 20)` limits to 20 data rows. Also use `=PIVOT(id, row_limit, TRUE, TRUE, col_limit)` to cap columns |

## Why Figures Between PIVOTs NEVER Works (v18)

This is the most common layout mistake. It looks correct at design time but **breaks as soon as data changes**.

### The Architectural Root Cause

From the Odoo source code (`o_spreadsheet.js`):

```javascript
// When rows/columns change, figures DO NOT reposition:
const newY = Math.min(figure.y, gridHeight - figure.height);
const newX = Math.min(figure.x, gridWidth - figure.width);
// That's it. No flow, no anchoring, no cell-relative adjustment.
```

- **Figures are rendered at absolute pixel coordinates** (`x`, `y`). Once placed, they stay at that exact pixel position forever.
- **Cells (PIVOT) expand dynamically** based on live data. More partners → more rows. New month → more columns.
- **There is NO layout engine** that coordinates the two layers. No auto-push, no reflow, no anchoring.

### Visual proof of the failure

```
AT DESIGN TIME (4 users):        AFTER DATA CHANGE (7 users):

┌─────────────────┐              ┌─────────────────┐
│   PIVOT(6)      │ 6 rows       │   PIVOT(6)      │ 9 rows
│   4 users       │              │   7 users       │
├─────────────────┤              │                 │
│  ▓▓ CHART ▓▓   │ y=300px      │  ▓▓ CHART ▓▓   │ y=300px (UNCHANGED!)
│  ▓▓▓▓▓▓▓▓▓▓▓   │              │  ▓▓▓▓▓▓▓▓▓▓▓   │ ← PIVOT rows UNDER chart
├─────────────────┤              │  ▓▓▓▓▓▓▓▓▓▓▓   │
│   PIVOT(5)      │              ├─────────────────┤
└─────────────────┘              │   PIVOT(5)      │ ← data hidden behind chart
                                 └─────────────────┘
```

**No amount of margin fixes this.** Any finite margin can be exceeded by sufficient data growth. The only solution is to **never place figures between dynamic content**.

### The ONLY safe layout rule (v18)

> **All figures (scorecards, charts) go at the TOP. All dynamic cell content (PIVOTs, LISTs) goes BELOW. Never mix them vertically.**

---

## Modification Decision Tree (WARNING SYSTEM)

When a user asks to **add a new element to an existing dashboard**, follow this decision tree BEFORE making any changes. Always show the warning and preview to the user.

### Step 1: Classify the new element

| Element type | Layer | Behavior |
|-------------|-------|----------|
| Scorecard | Figure | Fixed size, static position |
| Chart (odoo_bar, odoo_line, odoo_pie, etc.) | Figure | Fixed size, static position |
| PIVOT table (`=PIVOT(N)`) | Cell | Dynamic expansion, unpredictable size |
| Text / title | Cell | Fixed size (1 row) |
| ODOO.LIST() | Cell | Dynamic expansion |

### Step 2: Classify the insertion request

| Request | Assessment | Action |
|---------|-----------|--------|
| "Add chart/scorecard to the top" | **SAFE** | Add figure to figure zone, shift other figures down if needed |
| "Add PIVOT below all existing content" | **SAFE** | Add cells below last content, increase `rowNumber` |
| "Add chart/scorecard BETWEEN two PIVOTs" | **IMPOSSIBLE** | Show warning, propose reorganization |
| "Add PIVOT ABOVE existing figures" | **IMPOSSIBLE** | Show warning, propose reorganization |
| "Add PIVOT BETWEEN existing PIVOTs" | **RISKY** | Possible if generous margin, but fragile — warn user |
| "Add chart next to existing chart" | **SAFE** | Place at same `y`, different `x` (if width allows) |

### Step 3: Show warning (when reorganization is needed)

When the request is IMPOSSIBLE or RISKY, show this message to the user:

```
⚠️ LAYOUT WARNING (Odoo v18)

Your request: Add [element] between [A] and [B]

Problem: In Odoo v18, charts/scorecards use FIXED pixel positions while
PIVOT tables expand DYNAMICALLY based on live data. These two layers do
NOT coordinate — placing a fixed element between dynamic content causes
overlap when data grows.

This is an architectural limitation of Odoo's spreadsheet engine (v18).
Charts cannot "flow" with cells. There is no workaround.

Proposed solution: REORGANIZE the dashboard layout:
  - All fixed elements (charts, scorecards) → TOP of the sheet
  - All dynamic elements (PIVOTs, LISTs) → BELOW all fixed elements

This guarantees zero overlap regardless of data growth.
```

### Step 4: Show layout preview

Before applying changes, show the user a text preview of the proposed layout:

```
📋 PROPOSED LAYOUT:

FIGURE ZONE (fixed, top of sheet):
  y=0-45:     Title row
  y=50-160:   [Scorecard 1] [Scorecard 2] [Scorecard 3] [Scorecard 4]
  y=175-495:  [Bar Chart: XXX]  [Pie Chart: YYY]
  y=510-830:  [Line Chart: ZZZ]
  y=850-1185: [NEW Bar Chart: WWW]  ← your new element

PIVOT ZONE (dynamic, free expansion):
  Row 54:  Title + =PIVOT(6)   ← expands freely ↓
  Row 73:  Title + =PIVOT(5)   ← expands freely ↓
  Row 110: Title + =PIVOT(4)   ← expands freely ↓
  Row 125: Title + =PIVOT(3)   ← expands freely ↓

Gap between zones: ~80px (safe margin)
```

### Step 5: Execute reorganization

Follow the reorganization workflow:

1. **Read everything**: cells, figures, styles, rows, merges
2. **Calculate figure zone height**: sum all figure heights + gaps (15-20px between figure rows)
3. **Determine PIVOT zone start row**: first row after figure zone + 2-3 gap rows
4. **Rebuild cells dict**: delete all old cells, create new cells at new row positions
5. **Reposition all figures**: update `y` coordinates to pack into figure zone
6. **Update styles**: map style assignments to new cell positions
7. **Update row sizes**: title rows get custom height (32px)
8. **Ensure rowNumber is sufficient**: last PIVOT start + estimated max rows + 50
9. **Save and verify**

```python
# Reorganization template
def reorganize_dashboard(sheet):
    figures = sheet['figures']

    # Sort figures by visual hierarchy: scorecards first, then charts
    scorecards = [f for f in figures if f['data'].get('type') == 'scorecard']
    charts = [f for f in figures if f['data'].get('type') != 'scorecard']

    # Pack scorecards in row (horizontal)
    y_cursor = 50  # After title row
    sc_x = 8
    for sc in scorecards:
        sc['x'] = sc_x
        sc['y'] = y_cursor
        sc_x += sc['width'] + 12  # 12px gap between scorecards

    if scorecards:
        y_cursor += scorecards[0]['height'] + 15  # 15px gap after scorecard row

    # Pack charts (2 per row if width allows, full-width otherwise)
    sheet_width = 1024  # typical dashboard width
    for chart in sorted(charts, key=lambda c: c.get('y', 0)):
        chart['y'] = y_cursor
        y_cursor += chart['height'] + 15

    # Figure zone ends at y_cursor
    # Calculate PIVOT zone start row
    pivot_start_row = 2  # rows after pixel position
    while cell_to_pixel_y(pivot_start_row, sheet.get('rows', {})) < y_cursor + 50:
        pivot_start_row += 1

    return pivot_start_row
```

---

## Sheet Size Calculation (rowNumber / colNumber)

**CRITICAL**: The sheet's `rowNumber` and `colNumber` must accommodate ALL content. Odoo does not auto-expand these.

### colNumber

Default is `26` (columns A-Z). Wide pivots with monthly columns easily exceed this.

```
needed_cols = 1 (row labels) + count(col_groups) × count(measures) + count(measures) (total group) + 5 (margin)
```

**Example**: 12 months × 3 measures + 3 (total) + 1 (label) + 5 = 45 columns → set `colNumber: 50`

### rowNumber

Default is `100`. Multiple consecutive PIVOTs quickly exceed this.

```
needed_rows = figure_zone_rows + Σ(title_row + pivot_estimated_rows + margin) for each PIVOT
```

**Rule of thumb**: Set `rowNumber` to at least `last_PIVOT_start_row + largest_pivot_rows + 20`.

### When to use PIVOT() limits

| Scenario | Solution |
|----------|----------|
| Pivot has 50+ potential row groups | `=PIVOT(id, 30)` — limit to top 30 |
| Pivot has 20+ column groups (e.g., daily granularity) | `=PIVOT(id, 0, TRUE, TRUE, 12)` — limit to 12 column groups |
| Dashboard must fit a fixed visual area | Use both row and column limits |
| Pivot data is bounded and predictable (e.g., 4 commercials) | No limit needed, but add 30% margin for growth |

## Scorecard Figure Format (v18)

Scorecards are `tag: "chart"` with `type: "scorecard"` in their data:

```json
{
  "id": "uuid",
  "x": 8, "y": 48, "width": 220, "height": 110,
  "tag": "chart",
  "data": {
    "type": "scorecard",
    "title": "Total Facturado",
    "keyValue": "=PIVOT.VALUE(1,\"amount_invoiced:sum\")",
    "baseline": "=PIVOT.VALUE(2,\"amount_invoiced:sum\")",
    "baselineMode": "percentage",
    "baselineDescr": "vs período anterior",
    "background": "#EFF5F8",
    "baselineColorUp": "#2E7D32",
    "baselineColorDown": "#C62828",
    "fontColor": "#434343",
    "humanize": true
  }
}
```

## Chart Figure Format (v18)

Charts use `tag: "chart"` with Odoo-specific types:

```json
{
  "id": "uuid",
  "x": 8, "y": 300, "width": 500, "height": 340,
  "tag": "chart",
  "data": {
    "type": "odoo_bar",
    "title": "Rappel por Cliente",
    "metaData": {
      "groupBy": ["partner_id"],
      "measure": "amount_rappel",
      "order": "DESC",
      "resModel": "rappel.settlement",
      "mode": "byRecord"
    },
    "searchParams": {
      "domain": [["scope", "=", "sale"]],
      "groupBy": ["partner_id"],
      "orderBy": []
    },
    "background": "#FFFFFF",
    "legendPosition": "top",
    "stacked": false
  }
}
```

**Chart types**: `odoo_bar`, `odoo_line`, `odoo_pie`, `bar`, `line`, `pie`, `scorecard`, `gauge`

**Prefer `odoo_*` types** — they support drill-down into Odoo views.
