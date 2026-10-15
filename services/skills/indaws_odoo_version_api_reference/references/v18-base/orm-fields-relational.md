<!-- verified: v18.0 @ 2026-06 -->
# Field Types — Relational Fields & Command Class (Odoo 18.0)

## Many2one

```python
fields.Many2one(
    comodel_name='res.partner',  # str — target model (required)
    ondelete='set null',         # str — 'set null', 'restrict', 'cascade'
    auto_join=False,             # bool — SQL JOIN instead of subquery
    delegate=False,              # bool — delegation inheritance (_inherits)
    check_company=False,         # bool — validate same company
    domain=[],                   # list|callable — filter available records
)
```
DB: `int4` foreign key. Python: recordset of 0 or 1 record.

- Value is a recordset: `record.partner_id.name` (not `record.partner_id[0].name`)
- `False` when empty (not `None`)
- `ondelete='restrict'`: prevent deletion of referenced record
- `check_company=True`: validates via `_check_company()` — add on cross-model M2O fields

## One2many

```python
fields.One2many(
    comodel_name='sale.order.line',  # str — target model
    inverse_name='order_id',         # str — M2O on target pointing back
    domain=[],
    auto_join=False,
)
```
Not stored in DB. Python: recordset of 0-N records.

- `copy=False` by default. Set `copy=True` to deep-copy lines
- Write using `Command` class (see below)

## Many2many

```python
fields.Many2many(
    comodel_name='res.users',
    relation='module_model_users_rel',   # str — junction table name
    column1='model_id',                  # str — FK for this model
    column2='user_id',                   # str — FK for comodel
    domain=[],
)
```
DB: junction table. Python: recordset of 0-N records.

- `relation`: explicitly name junction table to avoid collisions
- `copy=True` by default (inherits base Field default; M2M does NOT override `copy` unlike O2M). Copying a record copies the M2M links (the linked records themselves are not duplicated)
- `ondelete` must be `'cascade'` or `'restrict'` (default `'cascade'`); `'set null'` raises ValueError (`odoo/fields.py:4958-4963`, `Many2many.setup_nonrelated`)

## Many2oneReference

```python
fields.Many2oneReference(model_field='res_model')
```
DB: `int4`. Stores record ID where model name is in another field.

- `model_field`: name of the `Char` field where the model name is stored (`odoo/fields.py:3438`, `class Many2oneReference`)
- Subclass of `Integer`. Used by `mail.message`, `ir.attachment` for polymorphic references

## Reference

```python
fields.Reference(selection=[('res.partner', 'Partner'), ('res.users', 'User')])
```
DB: `varchar`. Subclass of `Selection`. Stores `'model_name,record_id'`. Not flagged deprecated
in v18 source, but `Many2oneReference` (FK-style int) is the modern, more efficient pattern.

---

## Command Class

`from odoo.fields import Command`

Used to write One2many and Many2many fields:

| Command | Syntax | Effect |
|---------|--------|--------|
| `Command.create(vals)` | `(0, 0, vals)` | Create new record linked to parent |
| `Command.update(id, vals)` | `(1, id, vals)` | Update existing linked record |
| `Command.delete(id)` | `(2, id, 0)` | Remove the record from DB and drop its relation (M2M delete may be blocked if still linked elsewhere) |
| `Command.unlink(id)` | `(3, id, 0)` | Drop the relation only. On O2M: deletes the record if inverse is `ondelete='cascade'`, else sets inverse to False. On M2M: removes the junction row |
| `Command.link(id)` | `(4, id, 0)` | Add a relation to an existing record (works on O2M and M2M) |
| `Command.clear()` | `(5, 0, 0)` | Remove all relations (like `unlink` on every record) |
| `Command.set(ids)` | `(6, 0, ids)` | Replace all relations with given ids |

```python
from odoo.fields import Command

order.write({
    'line_ids': [
        Command.create({'product_id': 1, 'qty': 5}),
        Command.update(line_id, {'qty': 10}),
        Command.delete(old_line_id),
    ],
    'tag_ids': [
        Command.link(tag_id),
        Command.set([tag1_id, tag2_id]),
    ],
})
```

---

## Field Utilities

### Translate mechanism

```python
name = fields.Char(translate=True)           # entire value per language
description = fields.Html(translate=html_translate)  # extract terms
arch = fields.Text(translate=xml_translate)   # XML term extraction
```

Storage: JSONB `{lang_code: value}`. Fallback: requested → en_US → any.

### write_sequence

Internal ordering for `write()`. Monetary has `write_sequence=10` to ensure currency_id (default 0) is set before amount.
