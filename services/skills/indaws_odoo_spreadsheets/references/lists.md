# ODOO.LIST Definitions Reference

## List Structure

Lists are defined in the root `lists` dictionary, similar to pivots.

```json
{
  "lists": {
    "uuid-string": {
      "type": "ODOO",
      "columns": [
        {"fieldName": "partner_id"},
        {"fieldName": "user_id"},
        {"fieldName": "amount_untaxed"}
      ],
      "model": "sale.order",
      "domain": "[(\"state\", \"=\", \"sale\")]",
      "context": {"lang": "es_ES"},
      "orderBy": [{"fieldName": "amount_untaxed", "order": "desc"}],
      "limit": 10,
      "fieldMatching": {},
      "sortedColumn": null,
      "name": "Top Sales Orders",
      "formulaId": "1",
      "actionXmlId": "sale.action_quotations_with_onboarding"
    }
  },
  "listNextId": 2
}
```

## Key Properties

| Property | Description |
|----------|-------------|
| `columns` | Array of fields to display. Order matters for ODOO.LIST.HEADER |
| `model` | Odoo model name |
| `domain` | Python string domain (same format as pivots) |
| `orderBy` | Sort order: `[{"fieldName": "field", "order": "asc"/"desc"}]` |
| `limit` | Maximum number of records to fetch |
| `formulaId` | String integer, used in ODOO.LIST() formulas |
| `fieldMatching` | Connects to global filters (same as pivots) |
| `actionXmlId` | Optional, for drill-down on click |

## Formula Usage

### Building a list manually in cells

```
// Headers (row 1)
A1: =ODOO.LIST.HEADER(1, "partner_id")     → "Cliente"
B1: =ODOO.LIST.HEADER(1, "user_id")        → "Comercial"
C1: =ODOO.LIST.HEADER(1, "amount_untaxed") → "Importe"

// Data rows (row 2 onwards, 1-indexed)
A2: =ODOO.LIST(1, 1, "partner_id")
B2: =ODOO.LIST(1, 1, "user_id")
C2: =ODOO.LIST(1, 1, "amount_untaxed")

A3: =ODOO.LIST(1, 2, "partner_id")
B3: =ODOO.LIST(1, 2, "user_id")
C3: =ODOO.LIST(1, 2, "amount_untaxed")

// ... up to limit
A11: =ODOO.LIST(1, 10, "partner_id")
B11: =ODOO.LIST(1, 10, "user_id")
C11: =ODOO.LIST(1, 10, "amount_untaxed")
```

### Key Differences from PIVOT

| Aspect | PIVOT | LIST |
|--------|-------|------|
| Auto-expand | `=PIVOT(1)` expands all cells | Must build cell by cell |
| Data shape | Aggregated groups | Individual records |
| Styling | Auto-styled by pivot renderer | Fully customizable per cell |
| Sorting | Via pivot columns/rows | Via `orderBy` in definition |
| Use case | Summary/analytics | Top N, detail tables |

## Lists with Filters

Same fieldMatching mechanism as pivots:

```json
{
  "lists": {
    "uuid": {
      "fieldMatching": {
        "date-filter-uuid": {"chain": "date_order", "type": "date", "offset": 0},
        "user-filter-uuid": {"chain": "user_id", "type": "many2one"}
      }
    }
  }
}
```

## Practical Example: Top 10 Customers

```json
{
  "lists": {
    "abc-123-uuid": {
      "type": "ODOO",
      "columns": [
        {"fieldName": "partner_id"},
        {"fieldName": "user_id"},
        {"fieldName": "amount_untaxed"}
      ],
      "model": "sale.report",
      "domain": "[(\"state\", \"not in\", (\"draft\", \"cancel\"))]",
      "context": {"lang": "es_ES"},
      "orderBy": [{"fieldName": "amount_untaxed", "order": "desc"}],
      "limit": 10,
      "fieldMatching": {},
      "name": "Top 10 Customers",
      "formulaId": "1"
    }
  }
}
```
