# Data Sheet Pattern Reference

The most common architecture in Odoo standard dashboards is a **two-sheet pattern**: a visible "Dashboard" sheet with visual layout, and a "Data" sheet that holds all PIVOT.VALUE computations and KPI calculations.

## Architecture

```
┌──────────────────────────────────────┐
│ Sheet "Dashboard" (isVisible: true)  │
│                                      │
│  ┌─ Scorecard ─┐ ┌─ Scorecard ─┐    │
│  │ Data!D4     │ │ Data!D5     │    │
│  │ ▲ Data!E4   │ │ ▲ Data!E5   │    │
│  └─────────────┘ └─────────────┘    │
│                                      │
│  ┌─ Chart ──────────────────────┐    │
│  │ odoo_bar / odoo_line         │    │
│  └──────────────────────────────┘    │
│                                      │
│  [Top Salespeople](odoo://view/...)  │
│  Salesperson | # Leads | Revenue     │
│  PIVOT.HEADER/VALUE tables           │
│                                      │
└──────────────────────────────────────┘
         │ references
         ▼
┌──────────────────────────────────────┐
│ Sheet "Data" (isVisible: true*)      │
│                                      │
│    KPI        Current   Previous     │
│    Count      =PIVOT.VALUE(1,...)    │
│    Revenue    =PIVOT.VALUE(1,...)    │
│    Closed     =PIVOT.VALUE(1,...)    │
│    % Closed   =IFERROR(B3/B2)       │
│                                      │
│    Formatted  Current   Previous     │
│    Count      =FORMAT.LARGE.NUMBER() │
│    Revenue    =FORMAT.LARGE.NUMBER() │
│                                      │
└──────────────────────────────────────┘
```

*Note: In Odoo standard dashboards, the Data sheet is visible but positioned as the second tab. Users rarely navigate to it.

## Data Sheet Structure (CRM Pipeline example)

```
   A (labels)            B (Current)                          C (Previous)                          D (Formatted Current)       E (Formatted Previous)
1  =_t("KPI")            =_t("Current")                       =_t("Previous")                       =_t("Current")              =_t("Previous")
2  =_t("Expected count") =PIVOT.VALUE(1,"__count")             =PIVOT.VALUE(2,"__count")             =FORMAT.LARGE.NUMBER(B2)    =FORMAT.LARGE.NUMBER(C2)
3  =_t("Closed count")   =PIVOT.VALUE(1,"__count","won","won") =PIVOT.VALUE(2,"__count","won","won") =FORMAT.LARGE.NUMBER(B3)    =FORMAT.LARGE.NUMBER(C3)
4  =_t("Expected rev")   =PIVOT.VALUE(1,"prorated_revenue")    =PIVOT.VALUE(2,"prorated_revenue")    =FORMAT.LARGE.NUMBER(B4)    =FORMAT.LARGE.NUMBER(C4)
5  =_t("Closed rev")     =PIVOT.VALUE(1,"revenue","won","won") =PIVOT.VALUE(2,"revenue","won","won") =FORMAT.LARGE.NUMBER(B5)    =FORMAT.LARGE.NUMBER(C5)
6  =_t("% Closed")       =IFERROR(B3/B2)                      =IFERROR(C3/C2)                      =B6                          =C6
7  =_t("To close")       =PIVOT.VALUE(1,"__count","pending")   =PIVOT.VALUE(2,"__count","pending")   =FORMAT.LARGE.NUMBER(B7)    =FORMAT.LARGE.NUMBER(C7)
```

### Key Concepts

1. **Pivot 1 = Current period**, **Pivot 2 = Previous period** (same model, different `offset` in fieldMatching)
2. **Columns B/C**: Raw PIVOT.VALUE results (numbers)
3. **Columns D/E**: Formatted versions using `FORMAT.LARGE.NUMBER()` for scorecard display
4. **Derived KPIs**: Calculated from raw values (e.g., `=IFERROR(B3/B2)` for close rate)

## Dashboard Sheet Patterns

### Scorecards Reference Data Sheet

Scorecards point to the formatted cells in the Data sheet:

```json
{
  "tag": "chart",
  "data": {
    "type": "scorecard",
    "title": {"text": "Expected", "bold": true, "color": "#434343"},
    "background": "#FFF7ED",
    "keyValue": "Data!D4",
    "baseline": "Data!E4",
    "baselineDescr": "since last period",
    "baselineMode": "percentage",
    "baselineColorUp": "#00A04A",
    "baselineColorDown": "#DC6965",
    "humanize": false
  }
}
```

### PIVOT.HEADER/VALUE Tables on Dashboard Sheet

Manual ranking tables with full layout control:

```
A25: =_t("Opportunity")          B25: =_t("Stage")           C25: =_t("Salesperson")
A26: =PIVOT.HEADER(3,"#user_id",1)  B26: =PIVOT.VALUE(3,"__count","#user_id",1)  ...
A27: =PIVOT.HEADER(3,"#user_id",2)  B27: =PIVOT.VALUE(3,"__count","#user_id",2)  ...
...
A35: =PIVOT.HEADER(3,"#user_id",10) B35: =PIVOT.VALUE(3,"__count","#user_id",10) ...
```

### Section Headers with View Links

```json
{
  "A24": {
    "content": "[Top Opportunities](odoo://view/{\"viewType\":\"list\",\"action\":{\"domain\":\"...\",\"modelName\":\"crm.lead\",\"views\":[[false,\"list\"]]},\"name\":\"Pipeline\"})"
  }
}
```

## Current vs Previous Period Pattern

The **core pattern** for KPI comparison. Uses two identical pivots with different `offset` in fieldMatching (see [pivots.md](pivots.md#current-vs-previous-period-offset-pattern) for pivot definitions).

**Flow**: Pivot(offset=0) + Pivot(offset=-1) → Data sheet PIVOT.VALUE → FORMAT.LARGE.NUMBER → Scorecard references

```
Data sheet:
B2: =PIVOT.VALUE(1,"__count")     C2: =PIVOT.VALUE(2,"__count")     ← Raw values
D2: =FORMAT.LARGE.NUMBER(B2)     E2: =FORMAT.LARGE.NUMBER(C2)      ← Formatted

Scorecard: keyValue="Data!D2", baseline="Data!E2", baselineMode="percentage"
→ Shows "1.2k ▲ 22%"
```

## Benefits of This Pattern

| Benefit | Explanation |
|---------|-------------|
| **Separation of concerns** | Data computation separate from visual layout |
| **Scorecard compatibility** | **CRITICAL**: Scorecards CANNOT evaluate inline formulas — they MUST reference cells. `"keyValue": "=PIVOT.VALUE(...)"` renders blank; `"keyValue": "Data!B2"` works |
| **Derived KPIs** | Calculate ratios, percentages from raw PIVOT values |
| **FORMAT.LARGE.NUMBER** | Humanize numbers (1.2k, 3.5M) for compact display |
| **Period comparison** | Same pivot definition, different `offset` in fieldMatching |
| **Maintainability** | Change a formula in Data sheet, all referencing elements update |

## When to Use Each Pattern

| Pattern | When |
|---------|------|
| `=PIVOT(1)` auto-expand | Simple data table, no custom layout needed |
| Data sheet + PIVOT.VALUE | KPI scorecards, custom rankings, period comparisons |
| PIVOT.HEADER/VALUE on Dashboard | Manual table layout with data bars, custom styling |
| Both sheets with cross-references | Complex dashboards with 3+ scorecards and ranking tables |
| Data sheet + Fake scorecards | KPI cards with clickable navigation (see below) |

## Fake Scorecard Data Flow

When using fake scorecards (styled cells instead of scorecard figures — see `navigation.md` Solution 2), the Data sheet serves as the formula engine:

```
Data sheet:
B2: =PIVOT.VALUE(1,"savings:sum")        ← Raw numeric value
B5: =IFERROR(B2/B8, 0)                  ← Derived KPI
D7: =ROUND(B7, 1) & "%"                 ← Formatted percentage (field stores 52.43 = 52.43%)

Dashboard sheet (fake scorecard card):
A4: =Data!B2                             ← Number cell references Data sheet
A5: [▶ Ver detalle](odoo://view/...)     ← odoo://view link with filtered domain
```

**Key difference from native scorecards**: No FORMAT.LARGE.NUMBER indirection needed — the number cell displays the raw value directly. Number formatting can be done in the Data sheet formula or via cell number format. The `humanize` property is not available, so use `FORMAT.LARGE.NUMBER()` in the Data sheet cell if compact display is needed (e.g., `=FORMAT.LARGE.NUMBER(PIVOT.VALUE(1,"savings:sum"))`).

## Multi-Sheet Architectures (from Odoo standard dashboards)

### Standard: 2 sheets (most common)

Used by: Sales, Invoicing, CRM, HR Expense, Timesheet, Events, Livechat, POS

```
Sheet 1: "Dashboard" (visible) — KPIs, charts, ranking tables
Sheet 2: "Data" (visible) — PIVOT.VALUE calculations, formatted values
```

### Advanced: 3 sheets

Used by: MRR Evolution

```
Sheet 1: "Dashboard" — KPIs + line chart
Sheet 2: "Data" — All PIVOT.VALUE calculations
Sheet 3: "Sheet1" — Helper/temporary calculations
```

### Complex: 4+ sheets

Used by: Marketing Automation (4 sheets), Stock Operation Analysis (7 sheets)

**Stock Operation Analysis (7 sheets, 18 pivots):**
```
Sheet 1: "Dashboard" — KPIs + main charts
Sheet 2: "Data" — Central calculations hub
Sheet 3: "By product overview fill rate - On time" — Product detail analysis
Sheet 4: "WH Operations" — Warehouse operations data
Sheet 5: "Cycle time" — Performance metrics
Sheet 6: "Inventory adjustments by category" — Inventory analysis
Sheet 7: "Scrap" — Scrap tracking
```

### Sheet Naming Conventions

| Name | Purpose |
|------|---------|
| `Dashboard` | Main visible sheet with visual elements |
| `Data` | Calculation hub with PIVOT.VALUE formulas |
| Descriptive names | Analysis-specific sheets (e.g., "Cycle time", "WH Operations") |

### Cross-Sheet Reference Syntax

```
// Simple reference (no spaces in name)
=Data!D4
=Data!B2:B10

// Sheet name with spaces (single quotes required)
='By product overview fill rate - On time'!B1
='WH Operations'!K2:K

// Entire column reference (for VLOOKUP ranges)
='Cycle time'!N:N
=Data!A:D

// In scorecard/chart references
"keyValue": "Data!D4"
"baseline": "Data!E4"
"dataRange": "'Sheet Name'!E20:E30"
```

### Sheet Visibility

All sheets in standard Odoo dashboards are `isVisible: true`. The Data sheet is the second tab — users can see it but rarely navigate to it. There is no mechanism to hide sheets in the dashboard viewer.

## Header Groups (Collapsible Sections)

Collapsible row/column groups for hiding detail sections. Found in Marketing Automation and Retention dashboards:

```json
{
  "headerGroups": {
    "ROW": [{"start": 10, "end": 36, "isFolded": false}],
    "COL": [{"start": 3, "end": 8, "isFolded": false}]
  }
}
```

| Property | Description |
|----------|-------------|
| `start` | First row/column index (0-based) |
| `end` | Last row/column index |
| `isFolded` | `true` = collapsed on load, `false` = expanded |

Used to group detail rows that can be collapsed to show only summary.

## Frozen Panes

Freeze header rows/columns so they stay visible while scrolling. Found in 3 standard dashboards:

```json
{
  "panes": {"xSplit": 0, "ySplit": 1}
}
```

- `xSplit`: Number of frozen columns from left (0 = none)
- `ySplit`: Number of frozen rows from top (1 = freeze first row)
