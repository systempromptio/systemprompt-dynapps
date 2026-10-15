<!-- verified: v18.0 @ 2026-06 -->
# Field Types — Simple Fields & Parameters (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/odoo/fields.py` (5443 lines)

## Base Parameters (all fields)

```python
fields.Char(
    string='Label',              # str — UI label
    help='Tooltip text',         # str — help tooltip
    readonly=False,              # bool — readonly in UI
    required=False,              # bool — cannot be empty/False
    index=False,                 # str|bool — 'btree', 'btree_not_null', 'trigram', True, False
    default=value_or_callable,   # value|callable — static or function(self)
    groups='base.group_user',    # str — CSV of group XML IDs
    company_dependent=False,     # bool — value per-company in JSONB
    copy=True,                   # bool — copied on record.copy()
    store=True,                  # bool — stored in DB
    aggregator=None,             # str — 'sum','avg','min','max','count','count_distinct','array_agg','bool_and','bool_or'
    group_expand=None,           # callable|True — expand groupby
    prefetch=True,               # bool — include in prefetch batch
    exportable=True,             # bool — can be exported
)
```

### Important parameter interactions

- `required=True` + `default=...` → default used on create, write with `False` raises ValidationError
- `store=False` → not in DB, recomputed every access. Cannot search unless `search=` defined
- `company_dependent=True` → stored as JSONB (company id → value). Warns if `required` or `translate` set. Allowed types only: char, float, boolean, integer, text, many2one, date, datetime, selection, html (`COMPANY_DEPENDENT_FIELDS`, `odoo/fields.py:64`, `COMPANY_DEPENDENT_FIELDS`). Auto-defaults: `copy=False`, `index='btree_not_null'`, `prefetch='company_dependent'`. Fallbacks come from `ir.default`
- `groups='...'` → invisible to users outside groups. Read returns `False`, write silently ignored
- `index='trigram'` → enables `LIKE`/`ILIKE` with pg_trgm. Only Char/Text
- `index='btree_not_null'` → partial index excluding NULL

---

## Computed Field Parameters

```python
fields.Float(
    compute='_compute_amount',   # str|callable
    precompute=False,            # bool — compute BEFORE insert
    compute_sudo=...,            # bool — default: True if store=True, False if store=False
    recursive=False,             # bool — depends on itself
    inverse='_inverse_amount',   # str|callable — make writable
    search='_search_amount',     # str|callable — domain for store=False search
)
```

### Patterns

```python
# Compute
@api.depends('line_ids.price', 'line_ids.qty')
def _compute_amount(self):
    for record in self:
        record.amount = sum(l.price * l.qty for l in record.line_ids)

# Inverse
def _inverse_amount(self):
    for record in self:
        if record.line_ids:
            record.line_ids[0].price = record.amount

# Search
def _search_amount(self, operator, value):
    return [('line_ids.price', operator, value)]
```

`precompute=True`: use when compute depends ONLY on fields available at create time.

---

## Related Field Parameters

```python
fields.Char(
    related='partner_id.name',   # dot-separated path, auto-computes, auto-depends
    related_sudo=True,           # traverse in superuser mode (default True)
)
```

Inherits string, help, readonly, required, groups, type from target. Default `store=False`.

---

## Simple Fields

### Boolean
```python
fields.Boolean(default=False)
```
DB: `bool`. No default aggregator.

### Integer
```python
fields.Integer(default=0, aggregator='sum')
```
DB: `int4`. Default aggregator: `sum`.

### Float
```python
fields.Float(digits=(16, 2), default=0.0, aggregator='sum')
```
DB: `float8` or `numeric`. `digits='Product Price'` reads from `decimal.precision`.
Use `float_round()`, `float_compare()`, `float_is_zero()` from `odoo.tools`.

### Monetary
```python
fields.Monetary(currency_field='currency_id', aggregator='sum')
```
DB: `numeric`. Default `aggregator='sum'`. `currency_field` defaults to `'currency_id'` (then `'x_currency_id'`) if omitted (`odoo/fields.py:1727-1731`, `Monetary.get_currency_field`). The currency field must exist on the model. Monetary has `write_sequence=10` (vs default 0) so the currency is written before the amount during `write()` — this is a write-ordering mechanism, not a field-declaration-order requirement.

### Char
```python
fields.Char(size=None, trim=True, translate=False)
```
DB: `varchar`. `trim=True` (default) strips whitespace (web client only). `translate=True` stores per-language in JSONB. `size` is deprecated (`odoo/fields.py:1805`, `_String.size`).

### Text
```python
fields.Text(translate=False)
```
DB: `text`. Multi-line. Same params as Char minus `size`/`trim`.

### Html
```python
fields.Html(sanitize=True, sanitize_overridable=False, sanitize_tags=True,
            sanitize_attributes=True, sanitize_style=False, sanitize_form=True,
            strip_style=False, strip_classes=False)
```
DB: `text`. `translate=html_translate` for term extraction.

### Date
```python
fields.Date()
```
DB: `date`. Helpers: `Date.today()`, `Date.context_today(record)`, `Date.to_date(string)`, `Date.start_of(date, 'month')`, `Date.end_of(date, 'quarter')`, `Date.add(date, days=5)`.

### Datetime
```python
fields.Datetime()
```
DB: `timestamp without timezone`. Always UTC. Extra: `Datetime.now()`, `Datetime.context_timestamp(record, dt)`.

### Binary
```python
fields.Binary(attachment=True)
```
`attachment=True` (default): stored in filestore. `bin_size=True` in context returns size.

### Image
```python
fields.Image(max_width=0, max_height=0, verify_resolution=True)
```
Subclass of Binary with auto-resize. Standard sizes: `image_1920`, `image_512`, `image_256`, `image_128`.

### Selection
```python
fields.Selection(selection=[('draft', 'Draft'), ('done', 'Done')])
```
DB: `varchar`. Extend: `selection_add=[('new', 'New')]`. Ondelete: `ondelete={'new': 'set default'}`.

### Json
```python
fields.Json()
```
DB: `jsonb`. NOT searchable via domain operators.

---

## Special Fields

### Properties / PropertiesDefinition
```python
fields.Properties(definition='parent_id.property_definition')
fields.PropertiesDefinition()
```
`definition` is a dotted path `'<m2o_field>.<definition_field>'`; Odoo splits it into the
internal `definition_record` / `definition_record_field` attributes (`odoo/fields.py:3573-3577`, `Properties._setup_definition_attrs`). The
init parameter is `definition=`, NOT `definition_record=`. DB: `jsonb` (Properties) /
the definition is stored on the container record. Allowed sub-field types: boolean, integer,
float, char, date, datetime, many2one, many2many, selection, tags, separator. Used by Studio.

---

## Automatic fields

| Field | Type | Purpose |
|-------|------|---------|
| `id` | Id | Primary key |
| `display_name` | Char(compute) | From `_compute_display_name` |
| `create_uid` | Many2one('res.users') | Creator |
| `create_date` | Datetime | Creation time |
| `write_uid` | Many2one('res.users') | Last modifier |
| `write_date` | Datetime | Last modification |
