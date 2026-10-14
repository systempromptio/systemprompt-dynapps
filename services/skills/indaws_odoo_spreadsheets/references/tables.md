# Tables Reference

Tables are structured data ranges with automatic styling, headers, and optional banding. Defined per-sheet in the `tables` array.

## Structure

```json
{
  "sheets": [{
    "tables": [
      {
        "range": "A9:G90",
        "type": "static",
        "config": {
          "hasFilters": false,
          "totalRow": false,
          "firstColumn": false,
          "lastColumn": false,
          "numberOfHeaders": 1,
          "bandedRows": false,
          "bandedColumns": false,
          "automaticAutofill": true,
          "styleId": "TableStyleLight11"
        }
      }
    ]
  }]
}
```

## Properties

| Property | Type | Description |
|----------|------|-------------|
| `range` | String | Cell range (e.g. `"A9:G90"`) or single cell for dynamic tables (`"A1"`) |
| `type` | String | `"static"` (fixed range) or `"dynamic"` (auto-expanding) |
| `config` | Object | Table configuration |

## Table Types

### Static Tables

Fixed range, does not auto-expand. Most common in dashboards:

```json
{"range": "A9:G90", "type": "static", "config": {...}}
```

### Dynamic Tables

Auto-expanding from a single cell anchor. Used for data that grows:

```json
{
  "range": "A1",
  "type": "dynamic",
  "config": {
    "hasFilters": true,
    "totalRow": false,
    "firstColumn": true,
    "lastColumn": false,
    "numberOfHeaders": 1,
    "bandedRows": true,
    "bandedColumns": false,
    "styleId": "TableStyleMedium5",
    "automaticAutofill": true
  }
}
```

Key differences from static:
- `range` is a single cell (anchor point), not a range
- `hasFilters: true` adds dropdown filters on headers
- Automatically extends as data is added below
- `firstColumn: true` highlights first column (for row labels)

## Config Options

| Property | Default | Description |
|----------|---------|-------------|
| `hasFilters` | `false` | Show dropdown filters on headers |
| `totalRow` | `false` | Show totals row at bottom |
| `firstColumn` | `false` | Bold/highlight first column |
| `lastColumn` | `false` | Bold/highlight last column |
| `numberOfHeaders` | `1` | Number of header rows (1 or 2) |
| `bandedRows` | `false` | Alternate row shading |
| `bandedColumns` | `false` | Alternate column shading |
| `automaticAutofill` | `true` | Auto-extend table when data is added |
| `styleId` | - | Predefined table style name |

## Available Style IDs

### Light Styles (subtle, dashboard-friendly)

| Style ID | Description |
|----------|-------------|
| `TableStyleLight1` - `TableStyleLight21` | Light styles with thin borders |
| `TableStyleLight5` | **Most used in Odoo dashboards** - Clean, minimal |
| `TableStyleLight11` | Used in Accounting dashboard |

### Medium Styles (more prominent)

| Style ID | Description |
|----------|-------------|
| `TableStyleMedium1` - `TableStyleMedium28` | Medium weight with colored headers |
| `TableStyleMedium5` | Used in Purchase dashboard with banded rows |

### Dark Styles (bold, high contrast)

| Style ID | Description |
|----------|-------------|
| `TableStyleDark1` - `TableStyleDark11` | Dark headers, high contrast |

## Usage in Odoo Standard Dashboards

### Accounting Dashboard
Large table for P&L / Balance Sheet data:
```json
{
  "range": "A9:G90",
  "type": "static",
  "config": {
    "hasFilters": false,
    "totalRow": false,
    "numberOfHeaders": 1,
    "bandedRows": false,
    "automaticAutofill": true,
    "styleId": "TableStyleLight11"
  }
}
```

### Purchase Dashboard
KPI table with 2 header rows:
```json
{
  "range": "A8:D11",
  "type": "static",
  "config": {
    "hasFilters": false,
    "totalRow": false,
    "numberOfHeaders": 2,
    "bandedRows": false,
    "automaticAutofill": true,
    "styleId": "TableStyleLight5"
  }
}
```

Vendor ranking with banded rows and first-column highlight:
```json
{
  "range": "G14:H17",
  "type": "static",
  "config": {
    "hasFilters": false,
    "totalRow": false,
    "firstColumn": true,
    "bandedRows": true,
    "styleId": "TableStyleMedium5",
    "automaticAutofill": true
  }
}
```

## When to Use Tables

| Scenario | Use Table? | Why |
|----------|-----------|-----|
| PIVOT auto-expand (`=PIVOT(1)`) | No | Pivot renderer handles its own styling |
| PIVOT.VALUE/HEADER manual layout | **Yes** | Table provides consistent formatting |
| ODOO.LIST cell-by-cell layout | **Yes** | Clean headers and row styling |
| KPI summary grids | **Yes** | Structured header+data look |
| Accounting reports (P&L, BS) | **Yes** | Large data ranges need structure |

## Custom Table Styles

Defined at root level in `customTableStyles` (see [json-format.md](json-format.md)):

```json
{
  "customTableStyles": {
    "c0afab57-uuid": {
      "displayName": "Custom Table Style",
      "templateName": "darkNoBorders",
      "primaryColor": "#C9DAF8"
    }
  }
}
```

Referenced in table config:
```json
{"range": "A25:C34", "type": "static", "config": {"styleId": "c0afab57-uuid"}}
```

| Template Name | Description |
|---------------|-------------|
| `darkNoBorders` | Dark header, no borders — modern look |
| `lightNoBorders` | Light header, no borders |
| `mediumNoBorders` | Medium weight header, no borders |

Used in CRM Pipeline dashboard for ranking tables with custom brand colors.

## Tips

- Tables and sheet-level `styles` can coexist — table style takes precedence within its range
- Use `numberOfHeaders: 2` when you have a title row + column headers
- `automaticAutofill: true` ensures the table grows if new PIVOT.VALUE rows are added
- `hasFilters: false` is standard for dashboards (filtering is done via globalFilters)
- Prefer `TableStyleLight5` as the default — it's clean and unobtrusive
- For dynamic tables, use `type: "dynamic"` with single-cell range
- Custom styles via `customTableStyles` allow brand-specific colors
