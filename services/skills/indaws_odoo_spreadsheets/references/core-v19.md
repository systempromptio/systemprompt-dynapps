# Core Reference — Odoo 19

## JSON Root

```json
{
  "version": "18.5.1", "revisionId": "START_REVISION",
  "sheets": [], "styles": {}, "formats": {}, "borders": {},
  "pivots": {}, "lists": {}, "globalFilters": [],
  "customTableStyles": {}, "chartOdooMenusReferences": {},
  "pivotNextId": 1, "listNextId": 1, "uniqueFigureIds": true,
  "settings": {"locale": {"code": "es_ES", "thousandsSeparator": ".", "decimalSeparator": ",", "dateFormat": "dd/mm/yyyy", "timeFormat": "hh:mm:ss", "formulaArgSeparator": ";"}}
}
```

### Version
- `version`: `"18.5.1"` (semver **string**, last migration key in `o_spreadsheet.js`). **Never use integer.**
- `odooVersion`: **Removed** — do NOT include this key
- Version is dynamic — use `getCurrentVersion()` from the migration registry. Check the latest `.add("X.Y.Z")` entry in the source if unsure.

## Sheet Structure

Same as v18 (see core-v18.md for full structure). Differences: figure positioning, carousel tag.

## Figure Positioning (v19) — BREAKING

Cell-anchored with pixel offset (NOT absolute pixels):
```json
{"id": "uuid", "col": 2, "row": 0, "offset": {"x": 16, "y": 9}, "width": 200, "height": 102, "tag": "chart", "data": {...}}
```

| Property | Description |
|----------|-------------|
| `col` | Column index (0-based) — anchor cell column |
| `row` | Row index (0-based) — anchor cell row |
| `offset` | `{x, y}` pixel offset within the anchor cell |
| `width`, `height` | Size in pixels (same as v18) |
| `tag` | `"chart"`, `"image"`, or **`"carousel"`** (new in v19) |

**Rendering formula**: `pixel_x = colStart(col) + offset.x`, `pixel_y = rowStart(row) + offset.y`

**Migration from v18**: Automatic — `col: 0, row: 0, offset: {x: old_x, y: old_y}`, old `x`/`y` keys deleted.

### ⚠️ Figure-PIVOT Overlap: STILL NOT SOLVED in v19

From `o_spreadsheet.js` source code analysis:
- The figure plugin has **NO handler for `ADD_COLUMNS_ROWS`** — figures don't reposition when rows are inserted
- The OT registry has **NO transform for `ADD_COLUMNS_ROWS` → `CREATE_FIGURE`/`UPDATE_FIGURE`**
- PIVOT() expands at render time by filling cells, NOT by inserting rows — figure anchor doesn't shift

**Pattern 1 (figures top, PIVOTs below) remains MANDATORY in v19.** The cell-anchored format improves XLSX interoperability but does not create a flow layout. See `layout.md`.

## Cell Content & Locale

Same as v18. **CRITICAL**: Formulas ALWAYS use `,` in JSON, never `;`.

## Root Properties

Same as v18. See core-v18.md.

## Data Storage — BREAKING

| | v18 | v19 |
|---|---|---|
| Read API | `get_readonly_dashboard()` → dict | `_get_serialized_readonly_dashboard()` → JSON **string** |
| HTTP endpoint | — | `GET /spreadsheet/dashboard/data/<int:dashboard>` |
| Company | `company_id` (M2O) | `company_ids` (M2M): `[(6, 0, [id1, id2])]` |
| Favorites | — | `favorite_user_ids` (M2M), `is_favorite`, `action_toggle_favorite()` |
| Translations | — | `_get_dashboard_translation_namespace()` |
| Empty check | Basic `search_count` | `_dashboard_is_empty()` with access permission check |

## v19-Only Features Summary

| Feature | Reference file | Impact |
|---------|---------------|--------|
| **9 new Odoo chart types** (funnel, geo, treemap, sunburst, waterfall, pyramid, radar, scatter, combo) | charts-v19.md | Much richer visualizations |
| **Carousel figures** — multi-view containers | charts-v19.md | Multiple charts in one space |
| **Zoomable time-based charts** | charts-v19.md | Interactive temporal drill-down |
| **Fullscreen figures** | charts-v19.md | Click to expand any chart |
| New chart properties (chartId, dataSets, actionXmlId, humanize) | charts-v19.md | Better UX and click-through |
| Enriched titles (bold, fontSize, color) on charts and carousels | charts-v19.md | Styling consistency |
| **3 new filter types** (numeric, boolean, selection) | filters-v19.md | Typed filtering |
| **9 relative date periods** (today, yesterday, last_7/30/90_days, MTD, YTD...) | filters-v19.md | No manual date configuration |
| Enhanced text/relation filter operators (in, not in, starts with, ilike) | filters-v19.md | More precise filtering |
| Removed rangeType/disabledPeriods, renamed date presets | filters-v19.md | Breaking API changes |
| domainOfAllowedValues (replaces rangeOfAllowedValues) | filters-v19.md | Domain-based value restriction |
| ODOO.FILTER.VALUE structured return / .V18 compat | filters-v19.md | Breaking formula change |
| Multi-company dashboards | (this file) | One dashboard for multiple companies |
| Dashboard favorites | (this file) | Per-user quick access |
| Scorecard `baselineDescr` → `{text: "..."}` | (migration 18.4.2) | Breaking format change |
| Pivot `sortedColumn.measure` uses measure ID | (migration 18.4.3) | Breaking pivot sorting |
| Calculated measures in pivots | (documented in v19 UI) | Custom computed measures |
