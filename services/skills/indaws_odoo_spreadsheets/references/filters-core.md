# Global Filters Reference (v18 core + shared)

Filters appear in the dashboard header bar. Defined in `globalFilters` array at root level, connected to pivots/lists via `fieldMatching`.

## Filter Types

### 1. Date Filter (`type: "date"`) — v18

| rangeType | UI | defaultValue | Use case |
|-----------|---|-------------|----------|
| `"relative"` | Dropdown presets | `"this_year"`, `"last_month"`, etc. | **Most used** |
| `"fixedPeriod"` | Fiscal periods | `"this_year"`, `"this_quarter"` | Accounting |
| `"from_to"` | Two date pickers | `{}` | Custom ranges |
| `"year"` | `◂ 2026 ▸` | `{"yearOffset": 0}` | Annual dashboards |
| `"month"` | Month + Year | `{"yearOffset": 0, "period": "march"}` | Monthly |
| `"quarter"` | Quarter + Year | `{"yearOffset": 0, "period": "Q1"}` | Quarterly |

```json
{"id": "uuid", "type": "date", "label": "Año", "rangeType": "year", "defaultValue": {"yearOffset": 0}}
```

Relative presets: `this_year`, `this_month`, `this_quarter`, `last_month`, `last_quarter`, `last_three_months`, `last_six_months`, `last_year`

### 2. Relation Filter (`type: "relation"`)

```json
{"id": "uuid", "type": "relation", "label": "Comercial", "modelName": "res.users", "defaultValue": [], "includeChildren": false}
```

- `defaultValue`: `[]` (none), `[id1, id2]` (pre-selected), or `"current_user"` (logged-in user)
- `includeChildren: true`: Include descendants (hierarchical models like `product.category`)

Common models: `res.users`, `res.partner`, `res.country`, `crm.team`, `crm.stage`, `product.product`, `product.category`, `stock.warehouse`, `stock.picking.type`, `utm.medium`, `utm.source`, `mailing.mailing`, `sale.subscription.plan`

### 3. Text Filter (`type: "text"`)

```json
{"id": "uuid", "type": "text", "label": "Search", "defaultValue": ""}
```

With dropdown (v18): `"rangeOfAllowedValues": "'Sheet'!H6:H7"` — cell range with allowed values.

## fieldMatching

Connects filters to pivots/lists/charts:

```json
"fieldMatching": {
  "filter-uuid-date": {"chain": "date_field", "type": "date", "offset": 0},
  "filter-uuid-relation": {"chain": "partner_id", "type": "many2one"}
}
```

| Property | Description |
|----------|-------------|
| `chain` | Field path. Dot notation: `"partner_id.country_id"`, `"picking_type_id.warehouse_id"` |
| `type` | `"date"`, `"datetime"`, `"many2one"`, `"many2many"` |
| `offset` | `0` = same period, `-1` = previous period (comparison pivots) |

**Rules:**
- `date` for `fields.Date`, `datetime` for `fields.Datetime` — wrong type = filter won't work
- No fieldMatching entry = filter doesn't affect that pivot
- Same filter → different fields in different pivots
- Charts have own `fieldMatching` inside `data`

## Design Patterns

### Year filter (dynamic vs hardcoded domain)
Empty pivot domain `"[]"` + year filter with fieldMatching → dynamic filtering.

### Current vs Previous period
Two pivots, same model, same filter: `offset: 0` (current) and `offset: -1` (previous).

### Cascading relation filters
Multiple independent relation filters (commercial + client) on same pivot via separate fieldMatching entries.

### Real dashboard filter sets
| Dashboard | Filters |
|-----------|---------|
| **CRM** (9) | date(relative) + 5×relation + text(city) |
| **Stock** (7) | 2×text(rangeOfAllowedValues) + date + 4×relation |
| **MRR** (6) | date(relative) + 5×relation |

### Text filter for dynamic chart switching
`rangeOfAllowedValues` + `ODOO.FILTER.VALUE()` + `IF()` or `INDEX/MATCH` to switch data source.
