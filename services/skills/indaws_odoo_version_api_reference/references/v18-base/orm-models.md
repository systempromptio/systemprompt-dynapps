<!-- verified: v18.0 @ 2026-06 -->
# BaseModel — Complete Public API (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/odoo/models.py` (7637 lines)

All Odoo models inherit from `BaseModel`. `Model` (persistent), `TransientModel` (temporary), and `AbstractModel` (no table) are thin subclasses.

## Table of Contents

1. [CRUD](#crud)
2. [Search](#search)
3. [Recordset Navigation](#recordset-navigation)
4. [Display Name](#display-name)
5. [Aggregation](#aggregation)
6. [Context & Environment](#context--environment)
7. [Defaults & Metadata](#defaults--metadata)
8. [Access Control](#access-control)
9. [Cache & Flush](#cache--flush)
10. [Data Import/Export](#data-importexport)
11. [Actions & Utilities](#actions--utilities)
12. [Set Operations](#set-operations)
13. [Common Override Points](#common-override-points)

---

## CRUD

### create

```python
@api.model_create_multi
def create(self, vals_list: list[ValuesType]) -> Self
```

Creates one or more records. The bare signature takes `list[dict]`; the `@api.model_create_multi` decorator wraps it so callers may also pass a single dict (normalized to a one-element list). Returns created recordset.

- Handles defaults, precompute fields, constraints, and access checks
- Triggers `@api.constrains` methods after creation
- Calls `_check_company()` if model has `_check_company_auto = True`
- Batch: always pass a list of dicts when creating multiple records (one INSERT)

### read

```python
@api.readonly
def read(self, fields=None, load='_classic_read') -> list[dict]
```

Reads field values for records in self. Returns list of dicts `{field_name: value}`.

- `load='_classic_read'` (default): Many2one returns `[id, display_name]`. `load=None`: Many2one returns just the `id` (skips loading m2o display names)
- Checks access rights before reading
- Raises `MissingError` if a record doesn't exist

### write

```python
def write(self, vals: dict) -> True
```

Updates all records in self with the provided values. Always returns `True`.

- Triggers `@api.constrains` methods if constrained fields change
- Triggers `@api.onchange` side effects are NOT executed (those are UI-only)
- Cannot write on computed fields unless `inverse` is defined

### unlink

```python
def unlink(self) -> True
```

Deletes all records in self. Calls `@api.ondelete` methods before deletion.

- Cascades to One2many fields with `ondelete='cascade'`
- Raises `UserError` if `@api.ondelete` methods raise
- Returns `True`

### copy

```python
@api.returns('self')
def copy(self, default=None) -> Self
```

Duplicates the records in self (one copy per record). `default` overrides specific field values on the copies.

- Fields with `copy=False` are excluded (except if in `default`)
- One2many fields with `copy=True` are deep-copied
- Translations are copied
- In v18 `copy()` is **batch-capable**: it calls `copy_data()` (a dict per record), then `create()` on the list, and copies translations per record. It does NOT call `ensure_one()`. (In older versions copy was single-record.)

### copy_data

```python
def copy_data(self, default=None) -> list[dict]
```

Returns the values dict that `copy()` would use to create the duplicate. One dict per record in self. Useful when you need to modify values before actually creating the copy.

---

## Search

### search

```python
@api.model
@api.readonly
@api.returns('self')
def search(self, domain, offset=0, limit=None, order=None) -> Self
```

Returns recordset matching domain. Access rules are applied.

- `domain`: list of tuples — see orm-domains.md
- `order`: string like `'name asc, date desc'`. Defaults to model's `_order`
- `offset`/`limit`: for pagination
- Returns empty recordset if nothing matches

### search_fetch

```python
@api.model
@api.readonly
@api.returns('self')
def search_fetch(self, domain, field_names, offset=0, limit=None, order=None) -> Self
```

Like `search()` but also prefetches the given fields in the same query. More efficient than `search()` + accessing fields separately.

### search_count

```python
@api.model
@api.readonly
def search_count(self, domain, limit=None) -> int
```

Returns count of records matching domain. `limit` caps the count (useful for "has more than N" checks).

### search_read

```python
@api.model
@api.readonly
def search_read(self, domain=None, fields=None, offset=0, limit=None, order=None, **read_kwargs) -> list[dict]
```

Performs a `search_fetch()` followed by `_read_format()`. Returns list of dicts. Convenient for RPC calls. Extra `**read_kwargs` are forwarded to the read step (e.g. `load`).

### browse

```python
def browse(self, ids=()) -> Self
```

Returns a recordset for the given ids. Does NOT check if records exist — that happens on field access. Pass a single int or an iterable of ints.

### exists

```python
@api.returns('self')
def exists(self) -> Self
```

Returns the subset of self that actually exists in the database. Use after `browse()` if you're not sure the records are still there.

---

## Recordset Navigation

### mapped

```python
def mapped(self, func) -> list | Self
```

Applies `func` to each record (or dot-path string). If `func` returns recordsets, they're unioned. If it returns values, returns a list.

```python
partners = orders.mapped('partner_id')           # recordset
names = orders.mapped('partner_id.name')          # list of strings
amounts = orders.mapped(lambda o: o.amount_total) # list of floats
```

### filtered

```python
def filtered(self, func) -> Self
```

Returns records where `func(record)` is truthy. `func` can be a callable or a field name string.

```python
active = records.filtered('active')                   # field name
big = records.filtered(lambda r: r.amount > 1000)     # callable
```

### filtered_domain

```python
def filtered_domain(self, domain) -> Self
```

Returns records in self matching the domain. Preserves order of self. Does NOT query the database — filters in Python against cached values.

### sorted

```python
def sorted(self, key=None, reverse=False) -> Self
```

Returns self sorted by `key`. `key` can be a callable, field name string, or `None` (uses model's `_order`).

```python
by_date = records.sorted('date', reverse=True)
by_custom = records.sorted(lambda r: (r.priority, r.name))
```

### grouped

```python
def grouped(self, key) -> dict
```

Groups records by key (callable or field name). Returns `{key_value: recordset}`.

```python
by_state = orders.grouped('state')
# {'draft': order(1,3), 'done': order(2,4)}
```

### concat

```python
def concat(self, *args) -> Self
```

Concatenates self with other recordsets. May contain duplicates (same record appears twice). O(1) operation.

### union

```python
def union(self, *args) -> Self
```

Union of self with other recordsets. Removes duplicates, preserves first-occurrence order.

### Set operators

```python
records_a | records_b   # union
records_a & records_b   # intersection
records_a - records_b   # difference
records_a + records_b   # concatenation (may have duplicates)
```

---

## Display Name

### _compute_display_name

```python
@api.depends(lambda self: (self._rec_name,) if self._rec_name else ())
def _compute_display_name(self)
```

Computes the `display_name` field. Override this to customize how records appear in Many2one dropdowns and breadcrumbs.

```python
def _compute_display_name(self):
    for rec in self:
        rec.display_name = f"[{rec.code}] {rec.name}"
```

### _search_display_name

```python
@api.model
def _search_display_name(self, operator, value) -> list
```

Returns a domain for searching by display name. Override when display_name is composed of multiple fields.

```python
def _search_display_name(self, operator, value):
    return ['|', ('name', operator, value), ('code', operator, value)]
```

### name_search

```python
@api.model
@api.readonly
def name_search(self, name='', args=None, operator='ilike', limit=100) -> list[tuple[int, str]]
```

Searches records by display_name. Returns `[(id, display_name), ...]`. Used by relational field widgets for autocomplete. Calls `_search_display_name` internally.

### name_create

```python
@api.model
def name_create(self, name) -> tuple[int, str] | False
```

Creates a record with just a display name. Returns `(id, display_name)` or `False`. Used by "quick create" in Many2one dropdowns. Only works if `_rec_name` is set and writable.

---

## Aggregation

### _read_group

```python
@api.model
def _read_group(self, domain, groupby=(), aggregates=(), having=(), offset=0, limit=None, order=None)
```

Low-level grouping. Returns list of tuples matching `(groupby_values..., aggregate_values...)`.

```python
# Count orders per state
results = Order._read_group(
    [('date', '>=', '2024-01-01')],
    groupby=['state'],
    aggregates=['amount_total:sum', 'id:count'],
)
# [(state_value, sum_amount, count), ...]
```

Aggregate specs: `'field_name:agg_function'` where agg is `sum`, `avg`, `min`, `max`, `count`, `count_distinct`, `array_agg`, `recordset`, `bool_and`, `bool_or` (the `__count` pseudo-aggregate is also accepted). `recordset` post-processes an `array_agg` of ids back into a recordset.

### read_group

```python
def read_group(self, domain, fields, groupby, offset=0, limit=None, orderby=False, lazy=True)
```

Higher-level grouping. Returns list of dicts with `__domain`, `__count` (or `field_count`), and aggregated values. `lazy=True` groups by first field only.

---

## Context & Environment

### with_context

```python
def with_context(self, *args, **kwargs) -> Self
```

Returns self with an extended context. Merges `kwargs` into existing context.

```python
records.with_context(lang='es_ES', force_company=1)  # merge kwargs into current context
records.with_context({'lang': 'es_ES'})              # positional dict REPLACES the base context
records.with_context({'lang': 'es_ES'}, active_test=False)  # replace base, then merge kwargs on top
```

A positional dict argument replaces the current context entirely; any `**kwargs` are then merged onto that provided dict.

**Warning**: returns a NEW recordset. Does NOT modify self.

### with_company

```python
def with_company(self, company) -> Self
```

Returns self with the given company set as main company. Also adds it to `allowed_company_ids` if missing.

### with_user

```python
def with_user(self, user) -> Self
```

Returns self in the given user's environment. Disables superuser mode.

### with_env

```python
def with_env(self, env) -> Self
```

Returns self attached to a completely different environment.

### with_prefetch

```python
def with_prefetch(self, prefetch_ids=None) -> Self
```

Returns self with modified prefetch scope. Pass `()` to disable prefetching.

### sudo

```python
def sudo(self, flag=True) -> Self
```

Returns self in superuser mode. Bypasses access rights (ACL) and record rules. Does NOT bypass `@api.constrains`, `_check_company`, or Python-level checks.

```python
# Read restricted data
total = order.sudo().amount_total

# Minimal scope: get value, then continue without sudo
partner_name = record.sudo().partner_id.name
```

### new

```python
@api.model
def new(self, values=None, origin=None, ref=None) -> Self
```

Creates a record in memory (not persisted). Used internally by `onchange` and Form testing. `origin` links to a real record for comparison.

---

## Defaults & Utilities

### default_get

```python
@api.model
def default_get(self, fields_list) -> dict
```

Returns default values for the given fields. Sources: field `default=` attribute, `ir.default` records, context keys (`default_field_name`).

### ensure_one

```python
def ensure_one(self) -> Self
```

Asserts self contains exactly one record. Raises `ValueError` if empty or multiple.

### toggle_active / action_archive / action_unarchive

```python
def toggle_active(self)        # flips active field
def action_archive(self)       # sets active=False
def action_unarchive(self)     # sets active=True
```

### get_base_url

```python
def get_base_url(self) -> str
```

Returns the base URL for the record. Reads `web.base.url` system parameter.

### is_transient

```python
def is_transient(cls) -> bool
```

Returns whether model is a TransientModel (wizard).

---

## Common Override Points

| Method | When to override |
|--------|-----------------|
| `_compute_display_name()` | Custom display name format |
| `_search_display_name()` | Custom search-by-name logic |
| `_check_company()` | Custom multi-company validation |
| `_order` | Default sort order (class attribute) |
| `_rec_name` | Which field is the "name" (class attribute) |
| `_sql_constraints` | Database-level constraints (class attribute) |
| `_compute_access_url()` | Portal URL for the record |
| `_track_subtype()` | Mail tracking subtypes (with mail.thread) |

---

For access control, cache/flush, import/export, and metadata methods see `orm-models-advanced.md`.
