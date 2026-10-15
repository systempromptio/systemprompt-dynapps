<!-- verified: v18.0 @ 2026-06 -->
# Environment, Transaction & Cache (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/odoo/api.py` (1581 lines)

The `Environment` is the central context object in Odoo. Every ORM operation happens through an environment.

## Environment Attributes

```python
env = self.env

env.cr          # Cursor — database cursor for SQL
env.uid         # int — current user ID
env.context     # frozendict — immutable context dict
env.su          # bool — superuser mode flag
env.registry    # Registry — model registry for current DB
env.cache       # Cache — ORM record cache
env.transaction # Transaction — current transaction

# Convenience (computed from above)
env.user        # res.users recordset — current user (sudoed)
env.company     # res.company — current company
env.companies   # res.company recordset — all enabled companies
env.lang        # str|None — current language code (e.g., 'es_ES')
```

## Environment Methods

### Model access

```python
env['res.partner']              # returns empty recordset of model
'res.partner' in env            # True if model exists in registry
```

### Record by XML ID

```python
env.ref('base.res_partner_12')  # returns record by XML ID
env.ref('base.missing', raise_if_not_found=False)  # returns None instead of raising
```

### User checks

```python
env.is_superuser()   # True if in superuser mode (env.su=True)
env.is_admin()       # True if user has "Access Rights" group (base.group_erp_manager)
env.is_system()      # True if user has "Settings" group (base.group_system)
```

### New environment

```python
env(cr=new_cr)                    # new env with different cursor
env(user=other_user)              # new env with different user
env(context={'lang': 'es_ES'})    # new env with replaced context
env(su=True)                      # new env in superuser mode
```

### Cache management

```python
env.flush_all()                   # write all pending changes to DB (all models)
env.invalidate_all(flush=True)    # clear all caches (optionally flush first)
env.clear()                       # clear caches + pending computations

env.is_protected(field, record)   # True if field protected from recomputation
env.protected(field)              # recordset of protected records for field
```

### Protecting fields from recomputation

```python
with env.protecting([field1, field2], records):
    # field1 and field2 won't be recomputed on records
    # inside this block, even if dependencies change
    records.write({'dependency': new_value})
```

### Pending computations

```python
env.fields_to_compute()           # set of fields with pending computations
env.records_to_compute(field)     # recordset needing computation for field
env.is_to_compute(field, record)  # True if field needs computing on record
env.add_to_compute(field, records)    # mark for computation
env.remove_to_compute(field, records) # mark as computed
```

### Direct SQL execution

```python
# Execute and fetch as list of tuples
results = env.execute_query(SQL("SELECT id, name FROM res_partner WHERE active = %s", True))
# [(1, 'Admin'), (2, 'Demo')]

# Execute and fetch as list of dicts
results = env.execute_query_dict(SQL("SELECT id, name FROM res_partner LIMIT 5"))
# [{'id': 1, 'name': 'Admin'}, ...]
```

Both methods auto-flush related models before executing. Use `SQL()` from `odoo.tools` for safe parameter binding.

### Translation

```python
env._("Hello %(name)s", name="World")  # translate string with current env.lang
```

### Cache key (for context-dependent fields)

```python
env.cache_key(field)  # returns tuple of context values relevant for this field
```

---

## Transaction

```python
env.transaction  # Transaction object
```

The Transaction manages the database cursor lifecycle and savepoints. Developers rarely interact with it directly — the framework handles commit/rollback.

Key behavior:
- One transaction per HTTP request or cron job
- `cr.commit()` is called by the framework at the end of a successful request
- Manual `cr.commit()` is **prohibited** — it breaks the transaction boundary
- Savepoints are created by the test framework for rollback between tests

---

## Cache

```python
env.cache  # Cache object
```

The ORM cache stores field values in memory to avoid redundant DB queries.

### How prefetch works

When you access `record.field`, Odoo doesn't just fetch that record — it fetches up to `PREFETCH_MAX` (1000) records from the same prefetch group in one query. This makes looping over recordsets efficient.

```python
for record in self:           # triggers ONE query fetching 1000 records
    name = record.name        # served from cache for most iterations
```

### When to flush

The cache is "lazy write" — `write()` updates the cache immediately but may defer the DB UPDATE. You MUST flush before:

1. **Raw SQL reads** — `env.cr.execute("SELECT ...")` won't see unflushed writes
2. **`search()`** — automatically flushes relevant fields
3. **`_read_group()`** — automatically flushes
4. **Direct cursor use** — always call `self.flush_recordset()` first

```python
# BAD: SQL may not see the write
record.amount = 100
env.cr.execute("SELECT amount FROM table WHERE id = %s", (record.id,))

# GOOD: flush first
record.amount = 100
record.flush_recordset(['amount'])
env.cr.execute("SELECT amount FROM table WHERE id = %s", (record.id,))
```

### When to invalidate

After raw SQL that modifies data, the cache is stale:

```python
env.cr.execute("UPDATE table SET amount = 200 WHERE id = %s", (record.id,))
record.invalidate_recordset(['amount'])  # or invalidate_model() for broader scope
# now record.amount will re-read from DB
```

---

## API Decorators

Source: `$ODOO_SRC/odoo18/odoo/api.py`

### Method decorators

| Decorator | Purpose | When to use |
|-----------|---------|-------------|
| `@api.model` | Method operates on model, not specific records. `self` is empty recordset | Class-level operations, search, defaults |
| `@api.readonly` | Method can use read-only DB connection | Search, read, report methods |
| `@api.private` | Method cannot be called via RPC | Internal methods that must not be externally accessible |
| `@api.model_create_multi` | Wraps `create()` to handle single dict or list of dicts | Only on `create()` override |
| `@api.model_create_single` | Legacy single-dict create wrapper | Migration only, prefer `model_create_multi` |
| `@api.returns(model)` | Specifies return type for API conversion | Methods returning recordsets to be consumed by legacy code |
| `@api.autovacuum` | Called by daily vacuum cron | Cleanup methods (method name must start with `_`) |
| `@api.ondelete(at_uninstall=False)` | Called before `unlink()`. Raise to prevent deletion | Business rules that prevent record deletion |

### Field decorators

| Decorator | Purpose | Example |
|-----------|---------|---------|
| `@api.depends('field1', 'field2.sub')` | Declares dependencies for compute | `@api.depends('line_ids.price')` |
| `@api.depends_context('lang', 'company')` | Context keys that affect computation | Non-stored fields varying by context |
| `@api.constrains('field1', 'field2')` | Validation after create/write | Raise `ValidationError` on invalid state |
| `@api.onchange('field1')` | UI-only reaction to field change | Set defaults, show warnings in form view |

### Key differences

- `@api.constrains` runs on **server** (after DB write) — raises rollback
- `@api.onchange` runs on **client** (before save) — for UX only, no guarantees
- `@api.depends` → triggers recomputation when dependency changes
- `@api.depends_context` → different cache entry per context value

---

## Type Aliases

```python
from odoo.api import DomainType, ContextType, ValuesType, IdType

DomainType = list[str | tuple[str, str, Any]]
ContextType = Mapping[str, Any]
ValuesType = dict[str, Any]
IdType = int | NewId
```

## Constants

```python
from odoo import SUPERUSER_ID  # = 1 (defined in odoo/__init__.py)
```
