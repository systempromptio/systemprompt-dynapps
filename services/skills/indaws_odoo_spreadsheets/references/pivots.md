# Pivot Definitions Reference

## Pivot Structure

Pivots are defined in the root `pivots` dictionary. Keys are **UUIDs**, but formulas reference them by **`formulaId`** (numeric string).

```json
{
  "pivots": {
    "9ca92e1f-b607-4a21-98d8-775348302a55": {
      "type": "ODOO",
      "columns": [{"fieldName": "month_field"}],
      "rows": [{"fieldName": "group_field"}],
      "measures": [
        {"id": "amount:sum", "fieldName": "amount", "aggregator": "sum"}
      ],
      "model": "account.invoice.report",
      "domain": "[]",
      "context": {"lang": "es_ES"},
      "fieldMatching": {},
      "sortedColumn": null,
      "name": "Human readable name",
      "formulaId": "1",
      "actionXmlId": "module.xml_id"
    }
  },
  "pivotNextId": 2
}
```

## Critical Rules

1. **Keys must be UUIDs**: `str(uuid.uuid4())` in Python
2. **Formula references use `formulaId`**: `=PIVOT(1)` NOT `=PIVOT("uuid")`
3. **`formulaId`**: String of sequential integer starting from `"1"`
4. **`pivotNextId`**: Integer, must be > highest formulaId used
5. **`actionXmlId`**: Optional. Links to the Odoo action for drill-down

## Measures

```json
{
  "id": "field_name:aggregator",
  "fieldName": "field_name",
  "aggregator": "sum"
}
```

The `id` field MUST follow the format `"fieldName:aggregator"`.

### Aggregator Selection Guide

| Field type | Recommended aggregators | Notes |
|------------|------------------------|-------|
| Monetary/Float (amounts) | `sum` | Totals make sense |
| Monetary/Float (rates/%) | `max`, `avg` | Summing percentages is meaningless |
| Integer (quantities) | `sum`, `avg` | Sum for totals, avg for averages |
| Integer (tiers/levels) | `max` | Shows highest tier reached |
| Date/Datetime | `min`, `max`, `count` | First/last date, or count records |
| Many2one/Char | `count`, `count_distinct` | Count occurrences |
| Boolean | `bool_and`, `bool_or`, `count` | Logical aggregation |

### Available Aggregators by Field Type

| Type | sum | avg | min | max | count | count_distinct | bool_and | bool_or |
|------|-----|-----|-----|-----|-------|----------------|----------|---------|
| integer/float/monetary | Y | Y | Y | Y | Y | Y | | |
| date/datetime | | | Y | Y | Y | Y | | |
| boolean | | | | | Y | Y | Y | Y |
| char/many2one/reference | | | | | Y | Y | | |

## Rows and Columns

```json
"rows": [
  {"fieldName": "user_id"},
  {"fieldName": "partner_id"}
],
"columns": [
  {"fieldName": "date", "order": "asc", "granularity": "month"}
]
```

- Multiple row fields create nested grouping (hierarchy)
- Column fields typically represent time periods
- Date/datetime fields support `granularity`: `"month"`, `"quarter"`, `"year"`, `"day"`
- Optional `order`: `"asc"` or `"desc"` for sorting within the dimension

### Granularity Syntax

There are **two formats** for date granularity (both work):

```json
// Format 1: Colon syntax in fieldName (shorthand)
{"fieldName": "date:month"}

// Format 2: Explicit granularity property (used by Odoo standard dashboards)
{"fieldName": "date", "order": "asc", "granularity": "month"}
```

Available granularities: `"day"`, `"week"`, `"month"`, `"quarter"`, `"year"`, `"day_of_week"`, `"hour_number"`

**Odoo standard dashboards** use Format 2 (explicit property). Both are equivalent.

## Domain Format

**Must be a Python string representation**, NOT a JSON array:

```json
// CORRECT
"domain": "[(\"state\", \"=\", \"posted\")]"

// CORRECT - empty
"domain": "[]"

// WRONG - JSON array
"domain": [["state", "=", "posted"]]
```

Complex domains with AND/OR operators:
```json
// AND (implicit — all conditions must match)
"domain": "[(\"state\", \"not in\", (\"draft\", \"cancel\")), (\"move_type\", \"=\", \"out_invoice\")]"

// AND (explicit — using & operator)
"domain": "[\"&\", (\"type\", \"=\", \"opportunity\"), (\"user_id\", \"!=\", False)]"

// OR — using | operator (prefix notation)
"domain": "[\"|\", (\"move_type\", \"=\", \"out_invoice\"), (\"move_type\", \"=\", \"out_refund\")]"

// Complex multi-way OR (nested — | applies to next 2 items)
"domain": "[\"|\", (\"event_type\", \"=\", \"0_creation\"), \"|\", (\"event_type\", \"=\", \"1_expansion\"), \"|\", (\"event_type\", \"=\", \"15_contraction\"), (\"event_type\", \"=\", \"2_churn\")]"

// Combined AND + OR
"domain": "[\"&\", (\"state\", \"not in\", (\"draft\", \"cancel\")), \"|\", (\"move_type\", \"=\", \"out_invoice\"), (\"move_type\", \"=\", \"out_refund\")]"
```

**Domain operator rules** (Polish/prefix notation):
- `"&"` — AND: applies to the next 2 domain elements
- `"|"` — OR: applies to the next 2 domain elements
- `"!"` — NOT: applies to the next 1 domain element
- Multiple conditions without operators are implicitly AND-ed
- For N-way OR: chain `N-1` `"|"` operators before the conditions

**Best practice**: Keep pivot domain empty `"[]"` and use global filters to control data scope. This makes the dashboard dynamic.

## Pivot Context

Optional context dict passed with pivot queries. Used for default values and language:

```json
"context": {
  "lang": "es_ES",
  "default_type": "opportunity",
  "search_default_assigned_to_me": 1,
  "default_team_id": 1
}
```

| Key | Purpose |
|-----|---------|
| `lang` | Language for translated field labels |
| `default_*` | Default values when drilling down to form view |
| `search_default_*` | Pre-activated search filters in drill-down views |

## Formula Usage

### PIVOT() - Auto-expand full pivot table
```
=PIVOT(1)
=PIVOT(1, 10)           // limit to 10 rows
=PIVOT(1, 10, true)     // include totals
=PIVOT(1, 10, true, true) // include headers
```

### PIVOT.VALUE() - Single cell value
```
// Grand total
=PIVOT.VALUE(1, "amount:sum")

// Filtered by specific record ID
=PIVOT.VALUE(1, "amount:sum", "user_id", 9)

// Filtered by two dimensions
=PIVOT.VALUE(1, "amount:sum", "user_id", 9, "date:month", "01/2026")

// Using # prefix for positional index (Top N rankings)
=PIVOT.VALUE(1, "__count", "#user_id", 1)           // 1st user's count
=PIVOT.VALUE(1, "prorated_revenue:sum", "#user_id", 3)  // 3rd user's revenue

// Dynamic cell reference for pivot ID
=PIVOT.VALUE($J$14, "__count", "#product_id", $A21, "is_late", false)

// Boolean filter
=PIVOT.VALUE(1, "__count", "event_type", "0_creation", "event_date:month", C12)
```

#### The `#` Prefix (Positional Index)

The `#` prefix means "position N in the group", not a specific record ID. This is the foundation of **all ranking tables** in Odoo standard dashboards:

```
// Headers
A25: =PIVOT.HEADER(3, "#user_id", 1)    // Name of 1st ranked user
A26: =PIVOT.HEADER(3, "#user_id", 2)    // Name of 2nd ranked user
...
A34: =PIVOT.HEADER(3, "#user_id", 10)   // Name of 10th ranked user

// Values
B25: =PIVOT.VALUE(3, "__count", "#user_id", 1)   // 1st user's count
C25: =PIVOT.VALUE(3, "revenue:sum", "#user_id", 1)  // 1st user's revenue
```

Used in: CRM Pipeline, CRM Leads, Helpdesk, HR, Purchase, Sales — every dashboard with Top N tables.

### PIVOT.HEADER() - Header label
```
=PIVOT.HEADER(1, "user_id", 9)              // → "John Doe" (by record ID)
=PIVOT.HEADER(1, "date:month", "01/2026")   // → "January 2026"
=PIVOT.HEADER(1, "#user_id", 1)             // → 1st user name (positional)
=PIVOT.HEADER(1, "#user_id", 10)            // → 10th user name (positional)
```

## Current vs Previous Period (Offset Pattern)

The most important pattern in Odoo standard dashboards. Two identical pivots on the same model, connected to the same date filter but with different `offset` values:

```json
{
  "pivots": {
    "uuid-current": {
      "formulaId": "1",
      "model": "crm.lead",
      "measures": [
        {"id": "__count", "fieldName": "__count", "aggregator": "count"},
        {"id": "prorated_revenue:sum", "fieldName": "prorated_revenue", "aggregator": "sum"}
      ],
      "columns": [],
      "rows": [],
      "domain": "[(\"type\", \"=\", \"opportunity\")]",
      "fieldMatching": {
        "period-filter-uuid": {"chain": "date_open", "type": "date", "offset": 0}
      },
      "name": "Current Period"
    },
    "uuid-previous": {
      "formulaId": "2",
      "model": "crm.lead",
      "measures": [
        {"id": "__count", "fieldName": "__count", "aggregator": "count"},
        {"id": "prorated_revenue:sum", "fieldName": "prorated_revenue", "aggregator": "sum"}
      ],
      "columns": [],
      "rows": [],
      "domain": "[(\"type\", \"=\", \"opportunity\")]",
      "fieldMatching": {
        "period-filter-uuid": {"chain": "date_open", "type": "date", "offset": -1}
      },
      "name": "Previous Period"
    }
  }
}
```

### How offset works

| Filter selection | Pivot offset=0 | Pivot offset=-1 |
|-----------------|----------------|-----------------|
| "Last Year" (2025) | 2025 data | 2024 data |
| "This Year" (2026) | 2026 data | 2025 data |
| "Last Month" (Feb) | Feb 2026 | Jan 2026 |

### Usage in Data sheet

```
B2: =PIVOT.VALUE(1, "__count")           // Current period: 150
C2: =PIVOT.VALUE(2, "__count")           // Previous period: 120
D2: =FORMAT.LARGE.NUMBER(B2)            // "150"
E2: =FORMAT.LARGE.NUMBER(C2)            // "120"
```

Scorecard references D2 (keyValue) and E2 (baseline), showing "150 ▲ 25%"

Used in: CRM Pipeline (4 pairs/8 pivots), CRM Leads, Helpdesk, HR Payroll, Purchase. See [data-sheet-pattern.md](data-sheet-pattern.md) for the full Data sheet architecture.

## Multiple Pivots Per Sheet

Each pivot needs unique `formulaId`. Multiple pivots can coexist:

```json
"pivots": {
  "uuid-1": {"formulaId": "1", "name": "Summary", ...},
  "uuid-2": {"formulaId": "2", "name": "Detail", ...}
},
"pivotNextId": 3
```

Sheets reference them:
```json
"cells": {
  "A1": {"content": "=PIVOT(1)"},
  "A20": {"content": "=PIVOT(2)"}
}
```
