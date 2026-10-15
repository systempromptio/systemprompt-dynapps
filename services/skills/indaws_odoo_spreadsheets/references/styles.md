# Styles, Formats & Borders Reference

## Two-Level Architecture (CRITICAL)

Styles work at **two levels**:

1. **Root level** (`snapshot.styles`): Defines style **definitions** as `{id: properties}`
2. **Sheet level** (`sheet.styles`): Maps cell **ranges** to style IDs as `{"A1:H1": 1}`

Same pattern for `formats` and `borders`.

**Cell-level `{"style": 1}` is a legacy mechanism.** The **range-based sheet-level mapping** is what works for dashboard styling.

```json
{
  "styles": {
    "1": {"bold": true, "textColor": "#01666b", "fontSize": 16},
    "2": {"bold": true, "fillColor": "#007AAF", "textColor": "#FFFFFF", "align": "center"}
  },
  "sheets": [{
    "styles": {"A1:H1": 1, "A5:H5": 2},
    "formats": {"B5:B20": 1, "C5:C20": 2},
    "borders": {"A2:H2": 1}
  }]
}
```

## Style Properties

| Property | Type | Values |
|----------|------|--------|
| `bold` | boolean | `true`/`false` |
| `italic` | boolean | `true`/`false` |
| `underline` | boolean | `true`/`false` |
| `strikethrough` | boolean | `true`/`false` |
| `fontSize` | integer | Default 10, titles use 14-16 |
| `textColor` | hex string | e.g. `"#01666b"` |
| `fillColor` | hex string | e.g. `"#C9DAF8"` |
| `align` | string | `"left"`, `"center"`, `"right"` |
| `verticalAlign` | string | `"top"`, `"middle"`, `"bottom"` |
| `wrapping` | string | `"wrap"`, `"overflow"`, `"clip"` |

## Standard Odoo Dashboard Color Palette

Extracted from all 33 official Odoo 18 dashboards (community + enterprise):

### Primary Colors

| Role | Hex | Usage |
|------|-----|-------|
| Odoo Teal | `#01666b` | Titles, accent text, separator lines |
| Dark Gray | `#434343` | Body text, section headers, chart titles |
| Blue Header | `#007AAF` | Column/section headers (white text on blue) |
| Light Blue | `#C9DAF8` | Alternating rows, data cell backgrounds, custom table primary |
| Light Green | `#D9EAD3` | Total rows, positive indicators |
| Light Gray | `#f2f2f2` | Subtotal rows, separators |
| White | `#FFFFFF` | Text on colored backgrounds |

### Scorecard Background Colors

| Hex | Name | Usage |
|-----|------|-------|
| `#FFFFFF` | White | Neutral/default |
| `#FFF7ED` | Light orange (warm) | Primary KPIs (Sales, CRM) |
| `#FEF2F2` | Light pink | Alert/attention KPIs |
| `#EFF6FF` | Light blue (cool) | Secondary KPIs |
| `#F0F4F7` | Light gray-blue | Tertiary KPIs |
| `#ECFDF5` | Light green | Positive/growth KPIs |

### Baseline Indicator Colors

| Context | Up (▲) | Down (▼) |
|---------|--------|----------|
| Default (higher=better) | `#00A04A` (green) | `#DC6965` (red) |
| Inverted (lower=better) | `#DC6965` (red) | `#00A04A` (green) |

### Gauge Section Colors

| Zone | Normal | Inverted (lower=better) |
|------|--------|------------------------|
| Lower | `#EA9999` (red) | `#B6D7A8` (green) |
| Middle | `#F9CB9C` (orange) | `#F9CB9C` (orange) |
| Upper | `#B6D7A8` (green) | `#EA9999` (red) |

### DataBar Colors (conditional formatting, as decimal integers)

| Decimal | Hex | Visual | Usage |
|---------|-----|--------|-------|
| 15531509 | `#ECF1F5` | Light blue-grey | Default ranking bars |
| 16708338 | `#FEF1F2` | Light pink | Secondary ranking |
| 15726335 | `#EFEDFF` | Light purple | Tertiary ranking |
| 16775149 | `#FFF7ED` | Light orange | Highlight ranking |

## Standard Style Definitions

Reusable palette for any dashboard:

```json
{
  "styles": {
    "1": {"bold": true, "textColor": "#01666b", "fontSize": 16},
    "2": {"fontSize": 11, "textColor": "#434343", "verticalAlign": "middle", "bold": true},
    "3": {"textColor": "#01666B", "verticalAlign": "middle"},
    "4": {"bold": true, "fillColor": "#007AAF", "textColor": "#FFFFFF", "align": "center"},
    "5": {"fillColor": "#C9DAF8"},
    "6": {"align": "center"},
    "7": {"align": "center", "bold": true},
    "8": {"bold": true, "fillColor": "#D9EAD3", "align": "center"},
    "9": {"fillColor": "#f2f2f2"},
    "10": {"align": "center", "verticalAlign": "middle", "bold": true, "fontSize": 14}
  }
}
```

| ID | Role | Usage |
|----|------|-------|
| 1 | Title | Dashboard title, teal bold 16px |
| 2 | Section header | Section labels, dark gray bold |
| 3 | Accent text | Teal body text |
| 4 | Column header | Blue background, white text, centered |
| 5 | Light blue bg | Alternating row highlight |
| 6 | Centered | Center-aligned text |
| 7 | Centered bold | Center-aligned headers |
| 8 | Total row | Green background, bold, centered |
| 9 | Gray bg | Subtotal/separator rows |
| 10 | KPI number | Large centered bold (for scorecard-like cells) |

## Number Formats

Excel-compatible format strings:

```json
{
  "formats": {
    "1": "#,##0.00[$€]",
    "2": "0.00%",
    "3": "0%",
    "4": "#,##0.00",
    "5": "#,##0",
    "6": "0.0%",
    "7": "0.0",
    "8": "mm/yyyy",
    "9": "m/d/yyyy",
    "10": "dd/mm/yyyy",
    "11": "mmmm yyyy",
    "12": "@",
    "13": "0"
  }
}
```

### Format Usage Guide (from Odoo standard dashboards)

| Format | When to use |
|--------|-------------|
| `#,##0.00[$€]` | Monetary amounts (revenue, cost) |
| `0.00%` | Percentages with 2 decimals (close rate, margin) |
| `0%` | Percentages rounded (progress, completion) |
| `0.0%` | Percentages with 1 decimal (Accounting Benchmark KPIs) |
| `#,##0.00` | Non-monetary decimals |
| `#,##0` | Integer amounts (counts, quantities) |
| `0.0` | Single decimal numbers (ratings, scores) |
| `mm/yyyy` | Month/year display (Accounting timeline) |
| `dd/mm/yyyy` | Full date (Spanish locale) |
| `mmmm yyyy` | Full month name + year (Accounting timeline headers) |
| `@` | Text format (force text, no number interpretation) |
| `0` | Integer, no thousands separator |

**Percentage format note:** `"0.00%"` multiplies raw value ×100. A field storing `0.03` displays as `3,00%`. If the field stores `3.0` (already a percentage), use `"0.00"` + manual "%" suffix or the value shows as `300%`.

## Borders

Same two-level pattern as styles: root defines border definitions, sheet maps ranges to IDs.

### Root-Level Definitions

```json
{
  "borders": {
    "1": {"bottom": {"style": "thin", "color": "#CCCCCC"}},
    "2": {"top": {"style": "thin", "color": "#CCCCCC"}},
    "3": {"bottom": {"style": "thin", "color": "#01666b"}}
  }
}
```

### Sheet-Level Range Mappings

```json
{
  "sheets": [{
    "borders": {
      "A7:I7": 1,
      "A8:I8": 2,
      "A24:D24": 3,
      "A31:D31": 3,
      "F24:I24": 3
    }
  }]
}
```

### Available Border Properties

- **Sides**: `top`, `bottom`, `left`, `right`
- **Styles**: `"thin"`, `"thick"`, `"medium"`, `"dashed"`
- **Color**: Any hex color string

### Odoo Standard Border Patterns (from Accounting/CRM)

| ID | Definition | Usage |
|----|-----------|-------|
| 1 | `{"bottom": {"style": "thin", "color": "#CCCCCC"}}` | Section separator (light) |
| 2 | `{"top": {"style": "thin", "color": "#CCCCCC"}}` | Section separator top (pairs with 1) |
| 3 | `{"bottom": {"style": "thin", "color": "#01666b"}}` | Title/header separator (teal accent) |
| 4 | `{"top": {"style": "thin", "color": "#CCCCCC"}, "bottom": {"style": "thin", "color": "#CCCCCC"}}` | Enclosed row |

## Formats on Pivot Output

**Styles on pivot cells**: Mostly **overridden** by pivot's auto-styling. Don't rely on range styles for pivot headers/data appearance.

**Formats on pivot cells**: **DO WORK**. You CAN apply `#,##0.00[$€]` or `0.00%` to pivot data ranges and they render correctly.

### Mixed-Format Pivot Strategy

When a pivot has measures with different formats (e.g. € amounts and % rates), apply column-specific ranges:

```json
// 2 measures (Facturado €, Rappel %) × 3 groups (Jan, Feb, Total) = 6 columns
"formats": {
  "B5:B20": 1,  "D5:D20": 1,  "F5:F20": 1,   // € columns (odd positions)
  "C5:C20": 2,  "E5:E20": 2,  "G5:G20": 2    // % columns (even positions)
}
```

**Note**: Column positions depend on number of groups (months). This is fragile — adjust as data grows. Acceptable for dashboards with known data shape.

## Row & Column Sizing

Row and column sizes are defined per-sheet using 0-indexed string keys:

```json
{
  "sheets": [{
    "rows": {
      "0": {"size": 40},
      "6": {"size": 40},
      "23": {"size": 49},
      "24": {"size": 27}
    },
    "cols": {
      "0": {"size": 175},
      "1": {"size": 100},
      "4": {"size": 50}
    }
  }]
}
```

### Standard Row Heights (from Odoo dashboards)

| Height | Usage |
|--------|-------|
| 27 | Data rows (compact ranking tables) |
| 30 | Standard data rows |
| 36 | Sub-header rows |
| 40 | Section title rows, header rows |
| 49 | Large section headers |

### Standard Column Widths

| Width | Usage |
|-------|-------|
| 50 | Spacer/separator column |
| 100 | Numeric data columns |
| 175 | Label/name columns |
| 200+ | Wide label columns (first column) |

**Note**: Only non-default sizes need to be specified. Default row height is ~21px, default column width is ~96px.

## Other Visual Properties

```json
{
  "areGridLinesVisible": false,
  "merges": ["D1:N1"],
  "headerGroups": {"ROW": [], "COL": []}
}
```

| Property | Effect |
|----------|--------|
| `areGridLinesVisible: false` | Clean dashboard look without grid |
| `merges: ["D1:N1"]` | Merge cells for title spanning |
| `headerGroups` | Collapsible row/column groups (present in all Odoo dashboards, usually empty) |
