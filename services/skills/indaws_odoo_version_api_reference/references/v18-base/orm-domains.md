<!-- verified: v18.0 @ 2026-06 -->
# Domain Language — Complete Reference (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/odoo/osv/expression.py`

Domains are the Odoo query language. A domain is a list of conditions in **prefix notation** combined with logical operators.

## Operators

### Logical operators (prefix)

```python
'&'   # AND (binary) — default between conditions
'|'   # OR (binary)
'!'   # NOT (unary)
```

Implicit `&` between consecutive conditions:
```python
[('state', '=', 'done'), ('date', '>', '2024-01-01')]
# equivalent to:
['&', ('state', '=', 'done'), ('date', '>', '2024-01-01')]
```

### Comparison operators

| Operator | Types | Example | Notes |
|----------|-------|---------|-------|
| `=` | All | `('name', '=', 'John')` | Exact match |
| `!=` | All | `('active', '!=', False)` | Not equal |
| `<` | Num/Date | `('date', '<', '2024-01-01')` | |
| `<=` | Num/Date | `('amount', '<=', 1000)` | |
| `>` | Num/Date | `('date', '>', '2024-01-01')` | |
| `>=` | Num/Date | `('amount', '>=', 0)` | |
| `in` | All | `('state', 'in', ['draft', 'done'])` | Value in list |
| `not in` | All | `('state', 'not in', ['cancel'])` | Value not in list |
| `like` | Str | `('name', 'like', 'ohn')` | Case-sensitive contains — auto-wraps to `%value%` |
| `not like` | Str | `('name', 'not like', 'spam')` | Auto-wraps to `%value%` |
| `ilike` | Str | `('name', 'ilike', 'john')` | Case-insensitive contains — auto-wraps to `%value%` |
| `not ilike` | Str | `('name', 'not ilike', 'spam')` | Auto-wraps to `%value%` |
| `=like` | Str | `('name', '=like', 'J%')` | Literal SQL LIKE (you provide %) |
| `=ilike` | Str | `('name', '=ilike', 'j%')` | Literal SQL ILIKE |
| `=?` | All | `('field', '=?', value)` | Conditional: True if value is False/None, else `=` |
| `child_of` | M2O | `('categ_id', 'child_of', [5])` | Record 5 and all its descendants (uses `_parent_name`) |
| `parent_of` | M2O | `('id', 'parent_of', [1])` | Record 1 and all its ancestors |
| `any` | O2M/M2M | `('line_ids', 'any', [('qty', '>', 0)])` | Records with at least one line matching subdomain |
| `not any` | O2M/M2M | `('line_ids', 'not any', [('qty', '<=', 0)])` | Records with NO lines matching subdomain |

### Key details

- **`ilike`** auto-wraps value with `%value%` (`odoo/osv/expression.py:172`, `WILDCARD_OPERATORS`; wrapping at `odoo/models.py:3256`, `BaseModel._condition_to_sql`): `('name', 'ilike', 'john')` matches `'John Smith'`, `'JOHNSON'`
- **`like`** ALSO auto-wraps with `%value%` (it is in `WILDCARD_OPERATORS` too) — the ONLY difference vs `ilike` is case-sensitivity: `('name', 'like', 'ohn')` matches `'John'` but NOT `'JOHNSON'` (case-sensitive, but still substring via the auto `%`)
- **`=like`/`=ilike`**: NOT wildcard operators — you control the `%` placement: `('name', '=ilike', 'j%')` = starts with j
- **`=?`**: useful for optional filters: `('partner_id', '=?', partner_id)` — if partner_id is falsy, condition is True (ignored)
- **`child_of`/`parent_of`**: work with hierarchical models (those with `_parent_name`, usually `parent_id`)
- **`any`/`not any`**: sub-domain on relational fields. More efficient than `mapped()` + `filtered()`

---

## Complex domain examples

```python
# state='done' AND date > 2024-01-01
[('state', '=', 'done'), ('date', '>', '2024-01-01')]

# state='done' OR state='posted'
['|', ('state', '=', 'done'), ('state', '=', 'posted')]

# state='done' AND (date > 2024-01-01 OR amount > 1000)
['&', ('state', '=', 'done'),
 '|', ('date', '>', '2024-01-01'), ('amount', '>', 1000)]

# NOT state='cancel'
['!', ('state', '=', 'cancel')]
# equivalent and simpler:
[('state', '!=', 'cancel')]

# Orders with at least one line where qty > 0
[('line_ids', 'any', [('qty', '>', 0)])]

# Partners in category 5 or any child category
[('category_id', 'child_of', [5])]

# Optional filter: if partner_id is set, filter by it; if not, match all
[('partner_id', '=?', partner_id)]
```

---

## Domain helper functions

```python
from odoo.osv.expression import AND, OR, normalize_domain, is_leaf, TRUE_DOMAIN, FALSE_DOMAIN
```

### Combining domains

```python
AND([domain1, domain2, domain3])
# Returns: ['&', '&', ...domain1..., ...domain2..., ...domain3...]

OR([domain1, domain2])
# Returns: ['|', ...domain1..., ...domain2...]
```

Empty list handling (via `combine`, `odoo/osv/expression.py:270`, `def combine`; `[]` normalizes to `[(1,'=',1)]`):
- `AND([])` → `[(1, '=', 1)]` = `TRUE_DOMAIN` (matches everything)
- `OR([])` → `[(0, '=', 1)]` = `FALSE_DOMAIN` (matches nothing)
- `AND([[], domain])` → `domain` (`[]` is the AND identity, dropped)
- `OR([[], domain])` → `[(1, '=', 1)]` (matches everything — `[]` is the OR absorbing element)

### Constants

```python
TRUE_LEAF = (1, '=', 1)       # always True condition
FALSE_LEAF = (0, '=', 1)      # always False condition
TRUE_DOMAIN = [(1, '=', 1)]   # domain matching all records
FALSE_DOMAIN = [(0, '=', 1)]  # domain matching no records
```

### Utility functions

```python
normalize_domain(domain)       # make implicit & operators explicit
distribute_not(domain)         # push NOT inside terms: !('a','=',1) → ('a','!=',1)
is_leaf(element)               # True if element is a valid (field, op, value) tuple
is_operator(element)           # True if element is '&', '|', or '!'
is_boolean(element)            # True if element is TRUE_LEAF or FALSE_LEAF
is_false(model, domain)        # True if domain is logically always false
```

---

## Domain → SQL compilation

Understanding how domains compile to SQL helps write efficient filters:

| Pattern | SQL | Performance |
|---------|-----|-------------|
| `('field', '=', value)` | `WHERE field = value` | Fast (direct column) |
| `('m2o_id.name', 'ilike', 'x')` | `WHERE m2o_id IN (SELECT id FROM ... WHERE name ILIKE '%x%')` | Subquery |
| `('m2o_id.name', 'ilike', 'x')` + `auto_join=True` | `JOIN ... WHERE name ILIKE '%x%'` | JOIN (faster for large tables) |
| `('o2m_ids.field', '=', value)` | `WHERE id IN (SELECT inverse FROM ... WHERE field = value)` | Subquery |
| `('o2m_ids', 'any', domain)` | `WHERE id IN (SELECT inverse FROM ... WHERE ...)` | Subquery (explicit) |
| Deep path `('a.b.c', '=', x)` | Nested subqueries | Slow — avoid depth > 2 |

### Index recommendations

| Domain pattern | Recommended index |
|----------------|-------------------|
| `('field', '=', value)` | `index='btree'` or `index=True` |
| `('field', '=', value)` where field is often NULL | `index='btree_not_null'` |
| `('field', 'ilike', value)` | `index='trigram'` (Char/Text only; builds a GIN trigram index) |
| `('field', 'in', [...])` | `index='btree'` |

---

## Common patterns

### Building dynamic domains

```python
domain = [('active', '=', True)]
if partner_id:
    domain.append(('partner_id', '=', partner_id))
if date_from:
    domain.append(('date', '>=', date_from))
# Works because implicit AND between conditions
```

### Combining multiple optional domains

```python
from odoo.osv.expression import AND, OR

domains = [base_domain]
if filter_a:
    domains.append([('type', '=', 'a')])
if filter_b:
    domains.append([('type', '=', 'b')])
final_domain = AND(domains)  # or OR(domains) for any-match
```

### Searching with negation on O2M

```python
# Orders WITHOUT any line qty=0 (all lines have qty > 0)
[('line_ids', 'not any', [('qty', '=', 0)])]

# Orders with NO lines at all
[('line_ids', '=', False)]
```
