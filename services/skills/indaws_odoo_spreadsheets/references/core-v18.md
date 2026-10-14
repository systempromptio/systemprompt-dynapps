# Core Reference — Odoo 18

## JSON Root

```json
{
  "version": 21, "odooVersion": 12, "revisionId": "START_REVISION",
  "sheets": [], "styles": {}, "formats": {}, "borders": {},
  "pivots": {}, "lists": {}, "globalFilters": [],
  "customTableStyles": {}, "chartOdooMenusReferences": {},
  "pivotNextId": 1, "listNextId": 1, "uniqueFigureIds": true,
  "settings": {"locale": {"code": "es_ES", "thousandsSeparator": ".", "decimalSeparator": ",", "dateFormat": "dd/mm/yyyy", "timeFormat": "hh:mm:ss", "formulaArgSeparator": ";"}}
}
```

### Version
- `version`: `21` (integer). `22` also works (harmless migration). **Never use lower values.**
- `odooVersion`: `12` (required in v18)

## Sheet Structure

```json
{
  "id": "uuid-string", "name": "Dashboard", "colNumber": 26, "rowNumber": 100,
  "cols": {"0": {"size": 200}}, "rows": {"0": {"size": 40}},
  "cells": {"A1": {"content": "Title"}, "A3": {"content": "=PIVOT(1)"}},
  "merges": ["D1:N1"], "figures": [],
  "conditionalFormats": [], "tables": [],
  "styles": {"A1:H1": 1}, "formats": {"B5:B20": 1}, "borders": {"A2:H2": 1},
  "areGridLinesVisible": false, "isVisible": true,
  "headerGroups": {"ROW": [], "COL": []}, "panes": {},
  "comments": {}, "dataValidationRules": []
}
```

### Key Properties
- `cells`: Only non-empty cells. Content is string, formulas start with `=`
- `styles`/`formats`/`borders`: **Range-based** mapping to root definition IDs (e.g. `"A1:H1": 1`)
- `areGridLinesVisible: false`: Clean dashboard look
- `panes`: `{"xSplit": 0, "ySplit": 1}` freezes first row
- `headerGroups`: Collapsible sections. `[{"start": 10, "end": 36, "isFolded": false}]`

## Figure Positioning (v18)

Absolute pixel positioning:
```json
{"id": "uuid", "x": 208, "y": 9, "width": 200, "height": 102, "tag": "chart", "data": {...}}
```

| Property | Description |
|----------|-------------|
| `x`, `y` | Position in pixels from top-left |
| `width`, `height` | Size in pixels |
| `tag` | `"chart"` or `"image"` |

## Cell Content

```json
{"A1": {"content": "Static text"}, "B5": {"content": "=PIVOT.VALUE(1,\"amount:sum\")"}}
```

**CRITICAL**: Formulas in JSON ALWAYS use `,` as separator, regardless of locale. `formulaArgSeparator: ";"` only affects UI display.

## Locale / Settings

```json
"settings": {"locale": {"code": "es_ES", "thousandsSeparator": ".", "decimalSeparator": ",", "dateFormat": "dd/mm/yyyy", "timeFormat": "hh:mm:ss", "formulaArgSeparator": ";"}}
```

## Root Properties

| Property | Description |
|----------|-------------|
| `revisionId` | `"START_REVISION"` — required for collaborative editing |
| `customTableStyles` | Custom table themes (see tables.md) |
| `uniqueFigureIds` | `true` — ensures figure IDs are globally unique |
| `chartOdooMenusReferences` | Map chart UUIDs → menu XML IDs (e.g. `{"chart-uuid": "crm.crm_menu_root"}`) |

## Data Storage
- `spreadsheet_data`: JSON string on `spreadsheet.dashboard`
- Enterprise stores real data in `ir.attachment` snapshot
- Use `get_readonly_dashboard([id])` to read actual data (not `.read()`)
- New dashboards: put full data in `spreadsheet_data` — snapshot created on first load
