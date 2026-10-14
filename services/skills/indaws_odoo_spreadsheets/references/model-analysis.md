# Model Analysis for Datasources

## Step 1: Find Candidate Models

```bash
# Search models by keyword
./scripts/search_records.sh ir.model '[["model","ilike","rappel"]]' '["model","name"]' 20

# Or search by name
./scripts/search_records.sh ir.model '[["name","ilike","invoice"]]' '["model","name"]' 20
```

## Step 2: Analyze Model Fields

```bash
# Get all fields with key properties
./scripts/search_records.sh ir.model.fields \
  '[["model","=","x_model_name"]]' \
  '["name","field_description","ttype","store","compute","relation","readonly"]' 100
```

### Critical Field Properties

| Property | Impact on Dashboards |
|----------|---------------------|
| `store: true` | Field is in database, fast queries, works with GROUP BY |
| `store: false` | Computed on-the-fly, can be slow, may not work in pivots |
| `compute` (non-empty) | Has compute method — check if stored |
| `ttype` | Determines available aggregators and filter types |
| `relation` | For Many2one/Many2many — the related model |
| `readonly` | Informational, doesn't affect pivot usage |

### Decision: Direct Model vs Snapshot

| Scenario | Approach | Why |
|----------|----------|-----|
| All measure fields are `store=true` | **Direct model** | Fast, always up-to-date |
| Key fields are computed `store=false` | **Snapshot model** | Freeze computed values monthly |
| Need historical data (monthly snapshots) | **Snapshot model** | Can't query past computed values |
| Simple reporting on standard models | **Direct model** | `sale.report`, `account.invoice.report` already optimized |
| Complex cross-model calculations | **Snapshot model** | Pre-calculate and store |

## Step 3: Check Field Types for Measures

```bash
# Focus on numeric fields (potential measures)
./scripts/search_records.sh ir.model.fields \
  '[["model","=","x_model_name"],["ttype","in",["float","integer","monetary"]]]' \
  '["name","field_description","ttype","store","compute"]' 50
```

### Aggregator by Field Semantic

| Field semantic | Aggregator | Example |
|----------------|-----------|---------|
| Amount (revenue, cost) | `sum` | `price_subtotal:sum` |
| Rate/percentage | `max` or `avg` | `x_percentage:max` |
| Quantity | `sum` | `product_qty:sum` |
| Level/tier | `max` | `x_tramo:max` |
| Count records | `count` | `id:count` |
| Unique count | `count_distinct` | `partner_id:count_distinct` |

## Step 4: Check Field Types for Grouping (Rows/Columns)

Good row groupBy fields:
- `Many2one`: user_id, partner_id, category_id, team_id
- `Selection`: state, type
- `Char`: Only if low cardinality

Good column groupBy fields:
- `Date/Datetime`: With granularity (`:month`, `:quarter`, `:year`)
- `Selection`: state, type (if few values)
- `Char`: month name, period label (custom snapshot models)

## Step 5: Check Available Filters

For each global filter, identify the model field to connect:

| Filter type | Model field type | fieldMatching type |
|-------------|-----------------|-------------------|
| Date filter | `date` or `datetime` | `"date"` or `"datetime"` |
| Relation filter (res.users) | `Many2one` to `res.users` | `"many2one"` |
| Relation filter (res.partner) | `Many2one` to `res.partner` | `"many2one"` |
| Text filter | `Char` or `Text` | (via `ilike` match) |

Dot notation for indirect relations:
- Model has `partner_id` (Many2one → res.partner)
- res.partner has `country_id` (Many2one → res.country)
- Filter on country: `"chain": "partner_id.country_id"`

## Creating a Snapshot Model

When direct model isn't suitable, create a snapshot model via Studio (x_ prefix):

### Required Fields

| Field | Type | Purpose |
|-------|------|---------|
| `x_name` | Char | Display name (e.g. "Commercial - Client (MM/YYYY)") |
| `x_snapshot_date` | Date | When snapshot was taken (connect date filters here) |
| `x_year` | Integer | Year for quick filtering |
| `x_month` | Char | Month number as string ("1"-"12") |

### Typical Measure Fields

| Field | Type | Purpose |
|-------|------|---------|
| `x_invoiced` | Float/Monetary | Amount invoiced |
| `x_percentage` | Float | Rate/percentage (store as decimal: 0.03 = 3%) |
| `x_amount` | Float/Monetary | Calculated amount |

### Typical Relation Fields

| Field | Type | Purpose |
|-------|------|---------|
| `x_commercial_id` | Many2one (res.users) | Salesperson |
| `x_partner_id` | Many2one (res.partner) | Customer |
| `x_source_id` | Many2one (source model) | Link to original record |

### Server Action for Monthly Snapshot

Key pattern with per-record deduplication:

```python
# Get existing snapshot IDs for this month
existing_ids = set(
    env['x_snapshot'].search([
        ('x_year', '=', target_year),
        ('x_month', '=', str(target_month)),
    ]).mapped('x_source_id').ids  # .ids for Many2one!
)

# Only create for records not yet snapshotted
for record in source_records:
    if record.id in existing_ids:
        skipped += 1
        continue
    # ... create snapshot
```

**CRITICAL**: `.mapped('x_many2one_field')` returns recordsets. Use `.mapped('x_many2one_field').ids` to get integer IDs for `in` comparison.

### Access Rights

Create `ir.model.access` records:
- Admin group: full CRUD
- User group: read-only (for dashboard viewing)

## Standard Odoo Report Models

Pre-built models optimized for reporting:

| Model | Source | Fields |
|-------|--------|--------|
| `sale.report` | Sale orders | date, partner_id, user_id, amount, product_id, state |
| `account.invoice.report` | Invoices | date, partner_id, user_id, price_subtotal, product_id |
| `purchase.report` | Purchases | date, partner_id, product_id, price_total |
| `stock.report` | Stock moves | date, product_id, quantity, location_id |
| `crm.lead` | CRM pipeline | date_open, user_id, partner_id, expected_revenue |

These have stored fields optimized for GROUP BY and are the preferred datasource when available.
