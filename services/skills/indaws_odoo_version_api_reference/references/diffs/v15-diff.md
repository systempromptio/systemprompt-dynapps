# v15 Diff — What v15 lacks compared to v18

Base reference is v18. This file documents what is DIFFERENT or MISSING in Odoo 15.

## ORM / Python API

### `Command` class — NOT available

```python
# v18
from odoo.fields import Command
order.write({'line_ids': [Command.create({'product_id': 1})]})

# v15: use tuples
order.write({'line_ids': [(0, 0, {'product_id': 1})]})
```

Tuple syntax reference:
- `(0, 0, vals)` — create
- `(1, id, vals)` — update
- `(2, id, 0)` — delete
- `(3, id, 0)` — unlink (M2M)
- `(4, id, 0)` — link (M2M)
- `(5, 0, 0)` — clear
- `(6, 0, [ids])` — set

### `name_get()` — the ONLY way

```python
# v15: no _compute_display_name at all
def name_get(self):
    return [(rec.id, f"[{rec.code}] {rec.name}") for rec in self]
```

### `group_operator` — standard

```python
# v15: group_operator is the only option
amount = fields.Float(group_operator='sum')
# aggregator does NOT exist
```

### `_read_group()` — old signature only

```python
# v15: ONLY old signature
results = Model.read_group(domain, ['field1:sum'], ['groupby_field'])
# Returns list of dicts
```

### `precompute` — does NOT exist

### `is_storable` — does NOT exist

```python
# v15: type='product' only
type = fields.Selection([('consu', 'Consumable'), ('service', 'Service'), ('product', 'Storable')])
```

### `@api.depends_context` — available

Works the same as v18.

### Translations

```python
# v15: _() requires explicit module import
from odoo import _
_("Hello %s") % name      # printf-style (still common in v15)

# v18: named placeholders preferred
_("Hello %(name)s", name=name)
```

---

## Views

### `<tree>` — standard (only option)

All view syntax is the classic v15 format. No `<list>`, no `card`, no `id` context variable.

### Assets — legacy manifest format

```python
# v15: assets declared differently
'assets': {
    'web.assets_backend': [
        '/module/static/src/js/my_widget.js',
    ],
    'web.assets_qweb': [
        '/module/static/src/xml/my_template.xml',
    ],
}
# v16+: web.assets_qweb merged into web.assets_backend
```

---

## JavaScript

### Module system — `odoo.define()` is standard

```javascript
// v15: odoo.define() is the ONLY module system
odoo.define('module.my_widget', function (require) {
    'use strict';
    const Widget = require('web.Widget');
    const core = require('web.core');
    // ...
});

// ES modules (/** @odoo-module **/) NOT available in v15
```

### OWL — version 1 (or not available)

v15 uses OWL 1 in limited contexts. Most components are still legacy `Widget` class. OWL 2 is NOT available.

### Widget extension

```javascript
// v15: include() pattern for extending widgets
const MyWidget = require('module.Widget');
MyWidget.include({
    start: function () {
        // ...
        return this._super.apply(this, arguments);
    },
});
```

### Tour registration — legacy

```javascript
// v15: legacy tour system
odoo.define('module.tour', function (require) {
    var tour = require('web_tour.tour');
    tour.register('my_tour', {
        url: '/web',
    }, [{
        trigger: '.o_kanban_view',
        content: 'Click here',
    }]);
});
```

Tour step properties differ from v18:
- `run: 'click'` instead of implicit click
- `content` is the tooltip text
- No `isActive` property

---

## Test Framework (Python)

### Test classes

```python
# v15
from odoo.tests.common import SavepointCase, HttpCase, Form
# TransactionCase also available but SavepointCase is more common in v15

# v18: SavepointCase removed, use TransactionCase
```

### `tagged()` — available

```python
from odoo.tests import tagged

@tagged('post_install', '-at_install')
class TestMyModule(SavepointCase):  # SavepointCase, not TransactionCase
    ...
```

### `freeze_time` — external

```python
from freezegun import freeze_time
```

### `assertRecordValues` — available

Works the same as v18.

---

## What v15 does NOT have (compared to v18)

| Feature | v18 | v15 |
|---------|-----|-----|
| `Command` class | Yes | No — use tuples `(0, 0, vals)` |
| `_compute_display_name` | Standard | No — use `name_get()` |
| `aggregator` | Yes | No — use `group_operator` |
| `precompute=True` | Yes | No |
| `is_storable=True` | Yes | No — use `type='product'` |
| `_read_group()` new signature | Yes | No — old signature only |
| `<list>` view tag | Yes | No |
| ES modules | Yes | No — `odoo.define()` only |
| OWL 2 | Yes | No — OWL 1 or legacy Widget |
| Registry-based tours | Yes | No — `tour.register()` |
| Hoot (JS tests) | Yes | No — QUnit |
| Built-in `freeze_time` | Yes | No |
| `Form` in `odoo.tests` | Yes | No — `odoo.tests.common` |
| Named placeholders in `_()` | Preferred | No — use `%s` / `%d` |
| `web.assets_qweb` merged | Yes | No — separate bundle |
| `@api.returns` | Works | Works (still standard) |
| `SavepointCase` | Removed | Standard |

---

## Migration checklist v15 → v18

- [ ] Replace all tuples `(0, 0, vals)` with `Command.create(vals)` etc.
- [ ] Replace `name_get()` with `_compute_display_name()`
- [ ] Replace `group_operator` with `aggregator`
- [ ] Replace `type='product'` with `is_storable=True`
- [ ] Update `_read_group()` to new signature
- [ ] Add `precompute=True` where eligible
- [ ] Replace `SavepointCase` with `TransactionCase`
- [ ] Move `Form` import to `odoo.tests`
- [ ] Replace `freezegun.freeze_time` with built-in
- [ ] Migrate `odoo.define()` to ES modules
- [ ] Migrate `Widget.include()` to OWL 2 components or `patch()`
- [ ] Migrate `tour.register()` to registry-based
- [ ] Merge `web.assets_qweb` into `web.assets_backend`
- [ ] Update `_("text %s") % val` to `_("text %(key)s", key=val)`
- [ ] Remove `@api.returns` if migrating further to v19
