# MRP v18→v19 Breaking Changes

## Breaking Changes

- **`lot_producing_id`** (Many2one) → `lot_producing_ids` (Many2many) — multi-lot production now supported, data + code migration required
- **Move domain** — `scrapped=False` → `location_dest_usage != 'inventory'` on `move_raw_ids`/`move_finished_ids`
- **UoM domain** — category-based → `allowed_uom_ids` (computed M2M)
- **User domain** — `groups_id` → `all_group_ids`
- **Domain API** — `expression.AND/OR` → `Domain.AND/OR` (`from odoo.fields import Domain`)
- **Category** — `'Manufacturing/Manufacturing'` → `'Supply Chain/Manufacturing'`

## New Models

- **`mrp.production.group`** — group multiple MOs together (parent/child)
- **`stock.reference` inheritance** — MO-to-stock reference linking

## New Fields on mrp.production

- **`lot_producing_ids`** (M2M, replaces M2O) — multi-lot production
- **`production_group_id`** (M2O) — MO grouping
- **`reference_ids`** (M2M → stock.reference) — document linking
- **`allowed_uom_ids`** (M2M computed) — explicit UoM filtering
- **`show_generate_bom`** (Boolean computed) — UI flag

## Migration Checklist

- [ ] `lot_producing_id` → `lot_producing_ids` (data + code)
- [ ] Move domain filters: `scrapped` → `location_dest_usage`
- [ ] UoM domains: category → `allowed_uom_ids`
- [ ] User domains: `groups_id` → `all_group_ids`
- [ ] `expression` → `Domain` API
- [ ] Handle new `production_group_id` field
