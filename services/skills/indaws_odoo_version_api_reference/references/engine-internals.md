# Engine Internals — What Developers Need to Know

Compact reference of Odoo's internal mechanics that affect how you write code.
Not version-specific — these are structural behaviors present in v16-v19.
For deep analysis, see skill `odoo-deep-knowledge` (references/engine/).

## 1. Cache & Prefetch

```
PREFETCH_MAX = 1000  # Records prefetched per batch
INSERT_BATCH_SIZE = 100
UPDATE_BATCH_SIZE = 100
IN_MAX = 1000  # Max items in SQL IN clause
```

**Cache is per-transaction, per-field.** Invalidated on:
- `env.cr.commit()` / `env.cr.rollback()` → full cache clear
- `invalidate_recordset()` → targeted invalidation
- Field write → automatic invalidation of dependents

**Prefetch groups:** All records in the same recordset share a prefetch group. Accessing `record.field` prefetches that field for up to 1000 records in the group. This is WHY iterating a recordset is fast but iterating a list of individual records is N+1.

```python
# GOOD: single prefetch group → 1 query for all names
for record in self.search([]):
    print(record.name)

# BAD: each browse() creates isolated prefetch group → N queries
for id in ids:
    print(self.browse(id).name)
```

## 2. Flush & SQL Timing

**Odoo delays writes.** `write()` updates cache immediately but DB write is deferred. Flush happens:
- Before `search()` / `_read_group()` (needs consistent DB)
- Before `env.cr.execute()` (raw SQL)
- Before `flush_recordset()` / `flush_model()` (explicit)
- At end of RPC call (implicit)

**Critical rule:** If you mix ORM writes + raw SQL reads, call `self.flush_recordset()` between them or you read stale data.

```python
self.field = 'new_value'
# self.env.cr.execute("SELECT field FROM table WHERE id = %s", [self.id])
# ↑ MAY return OLD value — flush not triggered by execute()

self.flush_recordset(['field'])  # Force write to DB
self.env.cr.execute("SELECT field FROM table WHERE id = %s", [self.id])
# ↑ Now returns 'new_value'
```

## 3. Computed Field Engine

### Trigger Graph
Every `@api.depends('field_a', 'relation_id.field_b')` registers a trigger:
- `field_a` change → recompute THIS field
- `relation_id.field_b` change on related model → recompute THIS field

Triggers propagate transitively. Deep chains (`a.b.c.d.e`) create wide recomputation waves.

### store=True vs store=False Decision

| Criteria | store=True | store=False |
|----------|-----------|-------------|
| Need in search/filter | Yes | No (unless add search method) |
| Need in read_group | Yes | No |
| Read frequency >> Write frequency | Yes | Maybe |
| Depends on volatile fields | No (recompute storm) | Yes |
| Cross-model depends depth > 2 | No (performance) | Yes |

### precompute=True
Saves 1 UPDATE on `create()`. Only works if ALL depends fields are in the create vals (same model, not relational).

## 4. Domain → SQL Compilation

Domains compile to SQL WHERE clauses. Performance depends on:

```python
# GOOD: direct field → simple WHERE
[('state', '=', 'done')]  # → WHERE state = 'done'

# GOOD: M2O → simple JOIN
[('partner_id.country_id', '=', ref_id)]  # → JOIN res_partner ON... WHERE country_id = X

# SLOW: O2M/M2M → subquery
[('line_ids.product_id', '=', ref_id)]  # → WHERE id IN (SELECT order_id FROM lines WHERE product_id = X)

# VERY SLOW: deep path through O2M
[('order_id.line_ids.tax_ids.amount', '>', 0)]  # → nested subqueries
```

### Index Strategy
| Pattern | Index Type | When |
|---------|-----------|------|
| `field = value` | `index=True` (btree) | Exact match, ORDER BY |
| `field ILIKE '%value%'` | `index='trigram'` | Name/code search |
| `field = value` but 90% NULL | `index='btree_not_null'` | Sparse M2O (lot_id, etc.) |
| JSON field | GIN (manual) | analytic_distribution |

## 5. Security Layers (3-tier)

```
Layer 1: ACL (ir.model.access) — model-level CRUD booleans per group
  ↓ pass
Layer 2: Record Rules (ir.rule) — row-level domain filters
  - Global (no groups): AND between all global rules
  - Group rules: OR between rules of user's groups
  - Global AND (OR of group rules)
  ↓ pass
Layer 3: Field Groups (groups= on field) — field-level visibility
  - fields.Char(groups='base.group_system') → invisible + excluded from read/write
```

### sudo() Implications
- Bypasses Layer 1 (ACL) and Layer 2 (Record Rules)
- Does NOT bypass `_check_company()` SQL constraints
- Does NOT bypass Python `@api.constrains`
- Use `compute_sudo=True` on field instead of `sudo()` inside compute

## 6. OWL Component Patterns (v18+)

### Service Access
```javascript
import { useService } from "@web/core/utils/hooks";
setup() {
    this.orm = useService("orm");       // ORM calls
    this.action = useService("action"); // Navigate/do_action
    this.dialog = useService("dialog"); // Modals
    this.notification = useService("notification"); // Toasts
    this.rpc = useService("rpc");       // Raw RPC (backend only in v19)
}
```

### Registry System
```javascript
import { registry } from "@web/core/registry";
registry.category("views").add("my_view", MyView);
registry.category("fields").add("my_widget", MyWidget);
registry.category("services").add("my_service", myServiceDef);
// 88+ field widgets registered in fields category
```

### View Architecture
Every view = Controller + Renderer + Model (C/R/M pattern):
- **Controller**: orchestrates, handles actions
- **Renderer**: OWL component tree, renders UI
- **Model**: data loading, grouping, sorting

## 7. QWeb Dual Engine

### Server (Python) — Reports & Emails
- `ir.qweb` compiles template → Python function (cached)
- Expression sandbox: only safe operations (no import, no exec)
- Report: HTML → wkhtmltopdf → PDF
- `t-field` renders with field widget (monetary, date, etc.)

### Client (OWL) — Browser UI
- Templates compiled to vDOM render functions
- `t-on-click`, `t-ref`, `t-slot`, `t-component` directives
- Template inheritance: `t-inherit` with `t-inherit-mode="primary|extension"`

## 8. Cron Internals

```sql
-- Cron acquisition uses advisory lock to prevent double-execution:
SELECT ... FROM ir_cron
WHERE active = true AND nextcall <= NOW()
FOR NO KEY UPDATE SKIP LOCKED
```

- Failed cron (exception) → auto-deactivated after N failures
- `_trigger(at=datetime)` schedules precise one-shot execution
- Batch processing: commit every N records to avoid long transactions
