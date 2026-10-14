# Filters — v19 Changes (BREAKING)

## Removed Properties
- `rangeType` — no longer present on date filters
- `disabledPeriods` — removed
- `rangeOfAllowedValues` — replaced by `domainOfAllowedValues`

## Renamed Date Presets

| v18 | v19 |
|-----|-----|
| `last_week` | `last_7_days` |
| `last_month` | `last_30_days` |
| `last_three_months` | `last_90_days` |
| `last_year` | `last_12_months` |
| `last_six_months` | **Removed** |

## Relative Date Periods (NEW in v19)

9 predefined relative periods available for date filters:

| Period key | Label | Granularity |
|-----------|-------|-------------|
| `today` | Today | Automatic (hour if datetime, day if date) |
| `yesterday` | Yesterday | Day |
| `last_7_days` | Last 7 Days | Day |
| `last_30_days` | Last 30 Days | Day |
| `last_90_days` | Last 90 Days | Day |
| `month_to_date` | Month to Date | Day |
| `last_month` | Last Month | Day |
| `last_12_months` | Last 12 Months | Month |
| `year_to_date` | Year to Date | Month |

**Automatic granularity detection**: The system chooses the best granularity based on the date range:
- ≤ 1 day + datetime field → `hour`
- ≤ 1 day + date field → `day`
- ≤ 90 days → `day`
- ≤ 3 years → `day`
- > 3 years → implicit yearly

## domainOfAllowedValues (replaces rangeOfAllowedValues)

Restricts filter values using an Odoo domain instead of a cell range:
```json
{"type": "relation", "label": "Customer", "modelName": "res.partner", "domainOfAllowedValues": "[['partner_share', '=', true]]"}
```

## New Filter Types (3 new, 6 total)

v18 has 3 types: `date`, `relation`, `text`. v19 adds **3 more**:

### Selection Filter
Filters by selection field values (dropdown from `field.selection`):
```json
{
  "type": "selection",
  "label": "Estado",
  "resModel": "sale.order",
  "selectionField": "state"
}
```
- Operators: `in`, `not in`
- Value format: `{"operator": "in", "selectionValues": ["sale", "done"]}`
- UI: Multi-select dropdown with all selection options from the field definition

### Numeric Filter
Filters by numeric comparison:
```json
{
  "type": "numeric",
  "label": "Importe mínimo"
}
```
- Operators: `=`, `!=`, `>`, `<`, `between`
- Value format (single): `{"operator": ">", "targetValue": 1000}`
- Value format (between): `{"operator": "between", "minimumValue": 100, "maximumValue": 5000}`

### Boolean Filter
Filters by set/not-set (field has value or is null):
```json
{
  "type": "boolean",
  "label": "Tiene email"
}
```
- Operators: `set`, `not set`
- Generates domain: `[field, "!=", false]` or `[field, "=", false]`

## Enhanced Existing Filter Operators (v19)

| Filter type | v18 operators | v19 NEW operators |
|------------|--------------|-------------------|
| `text` | `ilike`, `not ilike` | + `in`, `not in`, `starts with`, `set`, `not set` |
| `relation` | `in`, `not in`, `child_of` | + `ilike`, `not ilike`, `set`, `not set` |

## Search Bar Integration (NEW in v19)

Global filters can be managed directly from the search bar in the dashboard header, providing a unified filtering experience without the separate filter panel.

## ODOO.FILTER.VALUE Changes

`ODOO.FILTER.VALUE()` returns **structured data** in v19 (not plain text). During v18→v19 migration, all existing calls are auto-renamed to `ODOO.FILTER.VALUE.V18()` which preserves text behavior.

For new v19 dashboards, use `ODOO.FILTER.VALUE()` with the new structured return format.
