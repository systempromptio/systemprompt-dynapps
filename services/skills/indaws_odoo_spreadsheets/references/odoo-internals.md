# Odoo Internals for Spreadsheet Dashboards

Knowledge about Odoo's internal mechanics that directly impacts dashboard creation.
For general Odoo development patterns, delegate to specialized skills (see Cross-references at bottom).

---

## 1. Domains — Complete Guide

Domains filter records in pivots, lists, charts, and global filters. Getting them wrong = silent empty data.

### Format in Spreadsheets

**Pivots/Lists**: Python string representation (escaped quotes):
```json
"domain": "[(\"state\", \"=\", \"posted\"), (\"move_type\", \"=\", \"out_invoice\")]"
```

**Charts (odoo_*)**: JSON array (no escaping):
```json
"searchParams": {"domain": [["state", "=", "posted"], ["move_type", "=", "out_invoice"]]}
```

**CRITICAL**: Never mix formats. Pivot with JSON array = parse error. Chart with Python string = silent failure.

### All Operators

| Operator | Meaning | Value type | Example |
|----------|---------|------------|---------|
| `=` | Equals | scalar | `("state", "=", "posted")` |
| `!=` | Not equals | scalar | `("state", "!=", "cancel")` |
| `>`, `<`, `>=`, `<=` | Comparison | number/date | `("amount", ">", 1000)` |
| `in` | In list | list | `("state", "in", ["draft", "posted"])` |
| `not in` | Not in list | list | `("user_id", "not in", [3, 7])` |
| `like` | SQL LIKE (case-sensitive) | string | `("name", "like", "INV%")` |
| `ilike` | LIKE case-insensitive | string | `("name", "ilike", "%john%")` |
| `=like` | Exact pattern match | string | `("ref", "=like", "SO%")` |
| `=ilike` | Exact pattern (case-insensitive) | string | `("email", "=ilike", "%@gmail.com")` |
| `child_of` | Hierarchical descendants | int/list | `("partner_id", "child_of", 5)` |
| `parent_of` | Hierarchical ancestors | int/list | `("categ_id", "parent_of", 12)` |

### Polish (Prefix) Notation — AND / OR / NOT

Multiple conditions without operators are implicitly AND:
```python
# Implicit AND: state=posted AND amount>1000
[("state", "=", "posted"), ("amount", ">", 1000)]
```

Explicit logical operators use **prefix** notation (before the operands):
```python
# OR: state=draft OR state=posted
["|", ("state", "=", "draft"), ("state", "=", "posted")]

# NOT: state != cancel (using NOT)
["!", ("state", "=", "cancel")]

# Complex: (state=posted) AND (type=out_invoice OR type=out_refund)
["&", ("state", "=", "posted"), "|", ("move_type", "=", "out_invoice"), ("move_type", "=", "out_refund")]
```

**N-way OR**: Chain N-1 `|` operators:
```python
# 3-way OR: state in (draft, posted, sent)
["|", "|", ("state", "=", "draft"), ("state", "=", "posted"), ("state", "=", "sent")]
# Simpler equivalent:
[("state", "in", ["draft", "posted", "sent"])]
```

### Special Values

| Value | Python repr in domain string | Meaning |
|-------|------------------------------|---------|
| NULL check | `("field", "=", False)` | Field is empty/null |
| NOT NULL | `("field", "!=", False)` | Field has a value |
| True boolean | `("field", "=", True)` | Boolean field is True |
| Date | `("date", ">=", "2026-01-01")` | ISO format string |
| Current user | Not available in static domains | Use global filter instead |

**Common mistake**: `("field", "=", "False")` — string "False" instead of Python `False`. In JSON this becomes `false` (lowercase), in Python string domains use `False` (capitalized).

### Escaping in Pivot Domains (JSON string)

Since pivot domains are strings inside JSON, quotes must be double-escaped:
```json
"domain": "[(\"partner_id.country_id\", \"=\", 68), (\"state\", \"in\", [\"posted\", \"draft\"])]"
```

**Rule**: Every `"` inside the domain string becomes `\"` in the JSON.

### v18 vs v19

No changes in domain syntax itself. Both versions use identical operators, Polish notation, and escaping rules.

---

## 2. Stored vs Computed Fields — Impact on Pivots

### Why It Matters

Odoo pivots translate to SQL `GROUP BY` queries. Only **stored** fields exist in the database and can be grouped/aggregated.

| Field type | `store` | Can be pivot measure? | Can be pivot groupBy? | Can be filtered? |
|-----------|---------|----------------------|----------------------|-----------------|
| Regular stored | `True` | YES | YES | YES |
| Computed + stored | `True` | YES | YES | YES |
| Computed NOT stored | `False` | **NO** — silent empty | **NO** — JS error | **NO** |
| Related (stored) | `True` | YES | YES | YES |
| Related (not stored) | `False` | **NO** | **NO** | **NO** |

### How to Detect

Use `odoo-pilot` to introspect:
```bash
./scripts/search_records.sh ir.model.fields \
  '[["model","=","sale.report"],["store","=",true]]' \
  '["name","ttype","store","relation","aggregator"]' 200
```

**Red flags**: If `fields_get` returns a field with `store: false` and `compute` is non-empty — that field CANNOT be used in pivots.

### Solution: Snapshot Model Pattern

When needed fields are computed and not stored, create a snapshot model that freezes values periodically. See `model-analysis.md` for the full pattern.

### v18 vs v19

| Aspect | v18 | v19 |
|--------|-----|-----|
| Field aggregation parameter | `group_operator='sum'` (deprecated but works) | `aggregator='sum'` (only accepted form) |
| `_read_group()` internal | `domain, ['field'], ['group_by']` style | `domain, groupby=[], aggregates=['field:sum']` style |
| Pivot impact | None — spreadsheet handles internally | None — spreadsheet handles internally |

**Important**: The `aggregator` vs `group_operator` change affects **custom model definitions**, not the spreadsheet JSON directly. But if a custom model uses the old `group_operator`, it may not aggregate correctly in v19 pivots.

---

## 3. Selection Fields — Key vs Label

### How Selections Work

Selection fields store a **key** (string) but display a **label** (translatable):
```python
state = fields.Selection([
    ('draft', 'Draft'),      # key='draft', label='Draft'
    ('posted', 'Posted'),    # key='posted', label='Posted'
    ('cancel', 'Cancelled'), # key='cancel', label='Cancelled'
])
```

### Impact on Dashboards

| Context | Uses key or label? |
|---------|-------------------|
| Domain filter | **Key**: `("state", "=", "posted")` |
| Pivot groupBy | Groups by **key**, displays **label** |
| Global filter (text type) | User sees **label**, filter sends **key** |
| PIVOT.VALUE formula | **Key**: `=PIVOT.VALUE(1, "amount:sum", "state", "posted")` |
| PIVOT.HEADER formula | Returns **label** (translated) |

### Discovering Selection Values

```bash
# Get field definition including selection options
./scripts/execute_method.sh model.name fields_get '["state"]' '{"attributes": ["type", "selection", "string"]}'
```

Returns: `{"state": {"type": "selection", "selection": [["draft", "Borrador"], ["posted", "Publicado"], ...]}}`

### Common Mistakes

- Using label in domain: `("state", "=", "Publicado")` — **WRONG**, use `"posted"`
- Using label in PIVOT.VALUE: `=PIVOT.VALUE(1, "amount:sum", "state", "Publicado")` — **WRONG**
- Assuming keys are integers: keys are always strings, even if they look numeric

### v18 vs v19

No changes in selection field behavior.

---

## 4. Relational Fields & Dot Notation

### Many2one (M2O)

Stores a single foreign key. Most common in pivots and filters.

```python
partner_id = fields.Many2one('res.partner')  # stores integer ID
```

| Usage in spreadsheets | Syntax | Example |
|----------------------|--------|---------|
| Pivot groupBy | `{"fieldName": "partner_id"}` | Groups by partner name |
| Domain filter | `("partner_id", "=", 42)` | Records for partner 42 |
| Domain with `in` | `("partner_id", "in", [1, 5, 9])` | Records for these partners |
| Global filter (relation) | `"modelName": "res.partner"` | Dropdown of partners |
| fieldMatching | `"chain": "partner_id", "type": "many2one"` | Connects filter to pivot |

### Many2many (M2M)

Stores multiple links via relation table.

```python
tag_ids = fields.Many2many('crm.tag')  # relation table
```

| Usage | Works? | Notes |
|-------|--------|-------|
| Pivot groupBy | YES but explosion | Creates row per tag per record (duplicates totals) |
| Domain `in` | YES | `("tag_ids", "in", [1, 2])` = records with ANY of these tags |
| Global filter | YES | `"modelName": "crm.tag"`, multi-select |
| fieldMatching type | `"many2many"` | (v18 uses `"many2one"` even for M2M in some cases) |

**Warning**: Using M2M as pivot groupBy multiplies rows. A record with 3 tags appears 3 times. Totals will be inflated. Use M2M for **filters**, not for groupBy.

### Dot Notation (Chained Relations)

Access fields through relations using dot notation:

```json
// In fieldMatching — filter by partner's country
"chain": "partner_id.country_id", "type": "many2one"

// In domains — filter by related field
"domain": "[(\"partner_id.country_id\", \"=\", 68)]"

// In pivot groupBy — group by related field
{"fieldName": "partner_id.country_id"}
```

**Rules for dot notation:**
- Each segment must be a stored Many2one (or the final segment stored)
- `partner_id.name` works (M2O.Char)
- `partner_id.country_id.name` works (M2O.M2O.Char)
- `order_line_ids.product_id` does NOT work in groupBy (One2many cannot be grouped)
- `tag_ids.name` may work in domains but NOT in pivot groupBy

### One2many (O2M)

Reverse side of Many2one. Virtual field, NOT stored.

```python
order_line_ids = fields.One2many('sale.order.line', 'order_id')
```

| Usage | Works? | Why |
|-------|--------|-----|
| Pivot groupBy | **NO** | Not stored, no SQL GROUP BY possible |
| Pivot measure | **NO** | Not aggregatable |
| Domain filter | Limited | `("order_line_ids.product_id", "=", 5)` works via subquery |
| fieldMatching | **NO** | Cannot connect filter to O2M |

**Alternative**: Use the child model (`sale.order.line`) as pivot source and group by the parent's field (`order_id.partner_id`).

### v18 vs v19

| Aspect | v18 | v19 |
|--------|-----|-----|
| fieldMatching M2M type | `"many2one"` (inconsistent) | `"many2many"` (corrected) |
| Dot notation | Same behavior | Same behavior |
| Company field | `company_id` (M2O) | `company_ids` (M2M) on `spreadsheet.dashboard` |

---

## 5. Date vs Datetime — Granularities & Filter Matching

### Storage Differences

| Type | Python | Database | Example |
|------|--------|----------|---------|
| `fields.Date` | `datetime.date` | `DATE` | `2026-03-20` |
| `fields.Datetime` | `datetime.datetime` | `TIMESTAMP` (UTC) | `2026-03-20 14:30:00` |

### Timezone Handling

- **Date fields**: No timezone. `2026-03-20` is the same everywhere.
- **Datetime fields**: Stored in **UTC**. Odoo converts to user timezone for display. Pivot GROUP BY on datetime uses UTC, which can shift records across day boundaries depending on timezone.

**Practical impact**: A record created at 23:30 CET (22:30 UTC) on March 20 may appear under March 20 in the pivot (UTC grouping), matching user expectation. But a record at 00:30 CET on March 21 (23:30 UTC March 20) appears under March **20** in the pivot.

**Recommendation**: For date-based dashboards, prefer `Date` fields over `Datetime` when available (e.g., `date_order` over `create_date`).

### Granularities in Pivots

Available granularities for date/datetime fields in pivot columns/rows:

| Granularity | Output | Example |
|-------------|--------|---------|
| `day` | `YYYY-MM-DD` | `2026-03-20` |
| `week` | `W12 2026` | Week number |
| `month` | `March 2026` | Month name + year |
| `quarter` | `Q1 2026` | Quarter + year |
| `year` | `2026` | Year only |
| `day_of_week` | `Monday` | Day name (for patterns) |
| `hour_number` | `14` | Hour of day (0-23, datetime only) |

Syntax in pivot definition:
```json
{"fieldName": "date_order", "granularity": "month"}
// or shorthand:
{"fieldName": "date_order:month"}
```

### Filter Type Matching (CRITICAL)

The `type` in fieldMatching **MUST** match the field's actual type:

| Field type | fieldMatching type | Mismatch result |
|-----------|-------------------|-----------------|
| `fields.Date` | `"date"` | OK |
| `fields.Datetime` | `"datetime"` | OK |
| `fields.Date` | `"datetime"` | **Silent failure** — filter ignored |
| `fields.Datetime` | `"date"` | **Silent failure** — filter ignored |

**How to check**: `fields_get` → look at `ttype` value (`"date"` or `"datetime"`).

### v18 vs v19

| Aspect | v18 | v19 |
|--------|-----|-----|
| Date filter relative presets | `this_year`, `last_month`, `last_quarter`, etc. | **Renamed**: `last_12_months`, `last_30_days`, `last_90_days`, `MTD`, `QTD`, `YTD`, `today`, `yesterday` |
| Granularities | Same 7 | Same 7 |
| Timezone behavior | Same | Same |

---

## 6. Domain + Global Filter Interaction

### The AND Rule

Global filters **ADD** conditions to the pivot/list/chart domain using AND. They never replace the existing domain.

```
Final query domain = pivot.domain AND filter1.condition AND filter2.condition AND ...
```

### The Duplication Trap (Most Common Error)

```json
// WRONG: domain already filters by year, filter also filters by year
{
  "domain": "[(\"date_order\", \">=\", \"2026-01-01\"), (\"date_order\", \"<=\", \"2026-12-31\")]",
  "fieldMatching": {
    "year-filter-uuid": {"chain": "date_order", "type": "date"}
  }
}
// If user selects 2025 in filter -> domain says 2026 AND filter says 2025 -> 0 rows
```

```json
// CORRECT: empty domain, filter controls date scope
{
  "domain": "[]",
  "fieldMatching": {
    "year-filter-uuid": {"chain": "date_order", "type": "date"}
  }
}
```

### When to Keep Domain Non-Empty

Use pivot domain ONLY for **invariant** conditions that should never change regardless of filters:

| OK to hardcode in domain | NOT OK (use filter instead) |
|-------------------------|---------------------------|
| `("state", "=", "posted")` — always exclude drafts | `("date", ">=", "2026-01-01")` — user may want other years |
| `("move_type", "in", ["out_invoice", "out_refund"])` — structural filter | `("user_id", "=", 5)` — user may want other salespeople |
| `("active", "=", True)` — exclude archived | `("partner_id", "=", 42)` — user may want other customers |

### v18 vs v19

Same behavior in both versions. The AND combination rule is unchanged.

---

## 7. Multi-Currency in Pivots

### The Problem

`fields.Monetary` fields store amounts in a specific currency. Summing amounts in different currencies produces meaningless totals.

### Detection

```bash
# Check if model has multi-currency records
./scripts/search_records.sh model.name \
  '[]' '["currency_id"]' 10
# If all records return same currency_id -> safe to sum
# If mixed -> need conversion strategy
```

### Strategies

| Strategy | When to use | How |
|----------|------------|-----|
| **Ignore** (most common) | Single-currency company | Just use `amount:sum` directly |
| **Filter by currency** | Multi-currency, separate analysis | Add relation filter for `res.currency` |
| **Convert in formulas** | Multi-currency, unified view | `=PIVOT.VALUE(...) * ODOO.CURRENCY.RATE("USD","EUR")` |
| **Use company currency field** | Model has `amount_company_currency` | Use that field as measure instead |

### Format Strings

| Currency | Format | Example |
|----------|--------|---------|
| EUR | `#,##0.00[$€]` | 1.234,56€ |
| USD | `[$US$]#,##0.00` | US$1,234.56 |
| Generic | `#,##0.00` | 1,234.56 (no symbol) |

### v18 vs v19

No changes in monetary field behavior. The `ODOO.CURRENCY.RATE()` formula works identically in both versions.

---

## 8. Report Models as Pivot Sources

### What Are Report Models?

Odoo provides pre-built SQL view models optimized for GROUP BY queries. They denormalize data from multiple tables into flat, indexed structures.

### Standard Report Models

| Model | Source data | Key measures | Key groupBy fields |
|-------|-----------|-------------|-------------------|
| `sale.report` | Sale orders + lines | `price_subtotal`, `product_uom_qty`, `qty_delivered`, `qty_invoiced` | `date`, `partner_id`, `user_id`, `product_id`, `categ_id`, `state`, `company_id` |
| `account.invoice.report` | Account moves + lines | `price_subtotal`, `price_average`, `residual`, `quantity` | `date`, `partner_id`, `product_id`, `account_id`, `journal_id`, `move_type` |
| `purchase.report` | Purchase orders + lines | `price_total`, `qty_ordered`, `qty_received` | `date_order`, `partner_id`, `product_id`, `user_id` |
| `stock.report` | Stock moves | `quantity`, `product_qty` | `date`, `product_id`, `location_id`, `picking_type_id`, `state` |
| `crm.lead` | CRM pipeline | `expected_revenue`, `prorated_revenue` | `date_open`, `date_closed`, `user_id`, `team_id`, `stage_id` |
| `account.move.line` | Journal items | `debit`, `credit`, `balance`, `amount_currency` | `date`, `account_id`, `partner_id`, `journal_id`, `company_id` |
| `hr.leave.report` | Absences | `number_of_days` | `date_from`, `employee_id`, `department_id`, `holiday_status_id` |

### When to Use Report Model vs Transaction Model

| Situation | Use | Why |
|-----------|-----|-----|
| Sales by month/partner | `sale.report` | Pre-joined with product, partner; all fields stored |
| Individual order details | `sale.order` | Need specific order fields not in report |
| Invoice analysis by account | `account.invoice.report` | Optimized for accounting aggregations |
| Custom computed KPIs | Snapshot model | Report models can't be extended easily |

### v18 vs v19

| Model | v18 | v19 |
|-------|-----|-----|
| `sale.report` | Same fields | Same fields |
| `account.invoice.report` | `invoice_date` field | Same |
| `purchase.report` | Same | Same |
| Display name computation | `name_get()` may exist | `_compute_display_name` only |

Report model structures are stable across v18/v19. The main change is that any custom `name_get()` overrides on these models must be converted to `_compute_display_name` in v19.

---

## 9. Security & Visibility in Dashboards

### How Access Control Affects Pivots

Odoo applies security at the ORM level. Pivots/lists only show records the user can access.

**Three layers**:

1. **Model access** (`ir.model.access`): Can this group read this model at all?
   - Without read access -> pivot shows error, not empty data

2. **Record rules** (`ir.rule`): Which records within the model can this user see?
   - With record rules -> pivot shows only matching records (silently filtered)
   - Example: salesperson sees only their own leads

3. **Field groups** (`groups` attribute on field): Is this field visible to this user?
   - Field hidden from user -> pivot with that measure may error

### Practical Implications

| Scenario | Symptom | Fix |
|----------|---------|-----|
| User has no model access | Error loading dashboard | Add `ir.model.access` for the group |
| Record rule limits visibility | Pivot shows partial data (correct behavior) | Document expected behavior; or use `sudo` in snapshot |
| Field restricted to admin group | Pivot error or empty column | Use different measure or grant field access |
| Multi-company rule active | Only current company's records shown | Add company filter; or use `company_ids` in v19 |

### Dashboard Security Groups

```json
// Dashboard visible to specific group
"group_ids": [[6, 0, [ref("sales_team.group_sale_salesman")]]]

// Dashboard visible to multiple groups
"group_ids": [[6, 0, [ref("sales_team.group_sale_salesman"), ref("account.group_account_user")]]]
```

### v18 vs v19

| Aspect | v18 | v19 |
|--------|-----|-----|
| Group reference field | `groups_id` on `res.users` | **`group_ids`** on `res.users` |
| Group category | `category_id` on `res.groups` | **`privilege_id`** (new `res.groups.privilege` model) |
| Record rule domains | `user.groups_id` | **`user.group_ids`** |
| Dashboard company | `company_id` (M2O) | `company_ids` (M2M) — multi-company dashboards |
| Export security | `base.group_allow_export` | Same |

**Impact**: If custom `ir.rule` domains reference `user.groups_id` in v18, they MUST be updated to `user.group_ids` in v19, or the rule silently fails (grants too much or too little access).

---

## 10. Context Keys Relevant to Dashboards

### Standard Context Keys

| Key | Type | Effect on dashboards |
|-----|------|---------------------|
| `lang` | string | Translates field labels, selection values, headers. **Always use `"es_ES"` for Spanish instances** |
| `tz` | string | Affects datetime display (not storage). Pivot GROUP BY always uses UTC |
| `allowed_company_ids` | list[int] | Filters multi-company records. Dashboard respects this from user session |
| `active_test` | bool | If `False`, includes archived records. Default `True` (excludes archived) |

### Context in Pivot Definitions

```json
{
  "context": {
    "lang": "es_ES"
  }
}
```

This ensures pivot headers and selection labels display in Spanish. Without it, they may show in English or the server's default language.

### v18 vs v19

| Key | v18 | v19 |
|-----|-----|-----|
| `active_id` in views | Available | **Removed** — use `id` instead |
| `lang` | Same behavior | Same behavior |
| `allowed_company_ids` | Same | Same, but dashboard now has `company_ids` M2M field |

---

## Cross-References to Other Skills

These skills provide deeper coverage of specific Odoo internals. The spreadsheet skill should delegate to them rather than duplicate:

| Need | Delegate to | How to invoke |
|------|------------|---------------|
| Introspect model fields (types, store, relations) | **odoo-pilot** | `./scripts/search_records.sh ir.model.fields [domain] [fields] limit` |
| Check model access rights | **odoo-pilot** | `./scripts/search_records.sh ir.model.access [domain] [fields] limit` |
| Verify record rules | **odoo-pilot** | `./scripts/search_records.sh ir.rule [domain] [fields] limit` |
| API changes between v18/v19 (ORM, decorators, views) | **odoo-module-upgrader** | Check `patterns/odoo18_to_19.md` |
| Field naming conventions, model design patterns | **odoo-module-creator** | Delega en la skill `odoo-module-creator` (developer) para guidelines Indaws de modelos |
| Security audit (SQL injection, XSS, access patterns) | **odoo-code-auditor** | Delega en la skill `odoo-code-auditor` (developer) para patrones de seguridad |
| Custom model performance (N+1, inefficient queries) | **odoo-code-auditor** | Delega en la skill `odoo-code-auditor` (developer) para patrones de rendimiento |
| Indaws module conventions, layer architecture | **indaws-core-developer** | Check the `indaws-core-developer` skill (references on indaws layers / code patterns) |
| v19 breaking changes (groups, privileges, ORM) | **indaws-core-developer** | Check the `indaws-core-developer` skill (references on Odoo 18/19 fixes) |
