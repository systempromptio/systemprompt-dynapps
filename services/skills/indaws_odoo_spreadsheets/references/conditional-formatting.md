# Conditional Formatting Reference

Conditional formatting rules are defined per-sheet in the `conditionalFormats` array. Odoo standard dashboards use them extensively for visual data bars in ranking tables.

## Structure

```json
{
  "sheets": [{
    "conditionalFormats": [
      {
        "id": "uuid-string",
        "ranges": ["A25:A34"],
        "rule": { ... }
      }
    ]
  }]
}
```

## Rule Types

### DataBarRule (most common in Odoo dashboards)

Displays horizontal bars proportional to cell values. Used in CRM, Helpdesk, HR, etc. for Top N rankings.

```json
{
  "id": "56e061bd-04c6-4e67-ac3f-08c29b2d2801",
  "ranges": ["A25:A34"],
  "rule": {
    "type": "DataBarRule",
    "color": 15531509,
    "rangeValues": "C25:C34"
  }
}
```

| Property | Description |
|----------|-------------|
| `ranges` | Cells where bars are drawn (the label/name column) |
| `rangeValues` | Cells containing the numeric values that determine bar length |
| `color` | Bar color as decimal integer (convert to hex for readability) |

**Color conversion**: `15531509 → #ED1F75` (decimal to hex). Common Odoo colors:

| Decimal | Hex | Usage |
|---------|-----|-------|
| 15531509 | #ECF1F5 | Light blue-grey |
| 16708338 | #FEF1F2 | Light pink |
| 15726335 | #EFEDFF | Light purple |
| 16775149 | #FFF7ED | Light orange |

**Pattern**: Bars in column A, values in column C. This creates a visual ranking where the bar length is proportional to the value:

```
| Salesperson  [████████████] | 45 | €125,000 |
| Salesperson  [████████]     | 32 | €89,000  |
| Salesperson  [████]         | 18 | €45,000  |
```

### CellIsRule

Color cells based on value conditions:

```json
{
  "id": "uuid",
  "ranges": ["B5:B5"],
  "rule": {
    "type": "CellIsRule",
    "operator": "GreaterThan",
    "values": ["0"],
    "style": {
      "textColor": "#00A04A"
    }
  }
}
```

Operators: `"GreaterThan"`, `"LessThan"`, `"Equal"`, `"NotEqual"`, `"GreaterThanOrEqual"`, `"LessThanOrEqual"`, `"Between"`, `"NotBetween"`, `"IsEmpty"`, `"IsNotEmpty"`, `"ContainsText"`, `"NotContainsText"`

### ColorScaleRule

Gradient coloring from min to max values:

```json
{
  "id": "uuid",
  "ranges": ["D2:D50"],
  "rule": {
    "type": "ColorScaleRule",
    "minimum": {"type": "value", "color": 16711680},
    "midpoint": {"type": "value", "color": 16776960},
    "maximum": {"type": "value", "color": 65280}
  }
}
```

Point types: `"value"` (auto min/max), `"number"` (with `value` property), `"percentage"`, `"percentile"`

### IconSetRule

Display icons (arrows, traffic lights) based on thresholds:

```json
{
  "id": "uuid",
  "ranges": ["E2:E50"],
  "rule": {
    "type": "IconSetRule",
    "icons": {
      "upper": "arrowGood",
      "middle": "arrowNeutral",
      "lower": "arrowBad"
    },
    "lowerInflectionPoint": {"type": "number", "value": "33", "operator": "gt"},
    "upperInflectionPoint": {"type": "number", "value": "66", "operator": "gt"}
  }
}
```

## Odoo Standard Dashboard Patterns

### Top N Ranking with Data Bars (CRM Leads pattern)

Two-column layout with symmetric data bars:

```json
{
  "conditionalFormats": [
    {
      "id": "uuid1",
      "ranges": ["A25:A34"],
      "rule": {"type": "DataBarRule", "color": 15531509, "rangeValues": "C25:C34"}
    },
    {
      "id": "uuid2",
      "ranges": ["E25:E34"],
      "rule": {"type": "DataBarRule", "color": 16708338, "rangeValues": "G25:G34"}
    }
  ]
}
```

This creates left and right ranking tables side by side:
- Left: Top Salespeople (bars in A, values in C)
- Right: Top Sales Teams (bars in E, values in G)

### Multiple Ranking Sections (vertical)

CRM Leads dashboard stacks 4 ranking sections vertically, each with its own DataBarRule pair:

```
Rows 25-34: Top Opportunities / Top Countries
Rows 38-47: Top Salespeople / Top Sales Teams
Rows 51-60: Top Cities / Top Mediums
Rows 64-73: Top Sources / Top Tags
```

Each section has 2 DataBarRules (left + right columns), for 8 total conditional formats.

## Tips

- DataBarRule is the most impactful visual enhancement for ranking tables
- Always pair with PIVOT.HEADER/PIVOT.VALUE formulas (not =PIVOT() auto-expand)
- Use different colors for different metric types (revenue vs count vs %)
- The `rangeValues` range must have the same number of rows as `ranges`
