# v16 Diff — What v16 lacks compared to v18

Base reference is v18. This file documents what is DIFFERENT or MISSING in Odoo 16.

## ORM / Python API

### `_read_group()` — signature changed mid-version

```python
# v16.0-16.2: OLD signature
results = Model.read_group(domain, ['field1:sum'], ['groupby_field'])
# Returns list of dicts with __domain, __count

# v16.3+: NEW signature (same as v18)
results = Model._read_group(domain, groupby=['field'], aggregates=['amount:sum'])
# Returns list of tuples
```

Both signatures coexist in v16 depending on patch level. Target the OLD signature for v16.0 compatibility.

### `precompute=True` — NOT available

```python
# v18: saves an UPDATE after INSERT
amount = fields.Float(compute='_compute', store=True, precompute=True)

# v16: precompute does not exist — omit it
amount = fields.Float(compute='_compute', store=True)
```

### `aggregator` — NOT available

```python
# v18
amount = fields.Float(aggregator='sum')

# v16: use group_operator
amount = fields.Float(group_operator='sum')
```

### `name_get()` — standard and required

```python
# v16: name_get() is THE way to customize display names
def name_get(self):
    return [(rec.id, f"[{rec.code}] {rec.name}") for rec in self]

# _compute_display_name EXISTE desde <=v14; en v16 su default delega en name_get()
```

### `_compute_display_name` — NO es "available late": existe desde <=v14

Corregido 2026-08-07. Decía *"available from v16.4+. Not available in v16.0-16.3"* y es falso. El
método existe al menos desde v14 (`odoo14/odoo/models.py:1727`, `BaseModel._compute_display_name`, `odoo15:1829`, `odoo16:1541`). Lo que
cambia no es la existencia, es **de qué depende el default**, y eso sí decide dónde hay que
sobreescribir:

| Versión | Default de `_compute_display_name` | Punto de override efectivo |
|---|---|---|
| ≤16 | **delega en `name_get()`** — `names = dict(self.name_get())` (`odoo16/odoo/models.py:1553`, `BaseModel._compute_display_name`; docstring: *"In general `display_name` is equal to calling `name_get()[0][1]`"*) | `name_get()` |
| 17 | **deja de delegar**: computa de `record[self._rec_name]` (`odoo17/odoo/models.py:1672`, `BaseModel._compute_display_name`) | `_compute_display_name` (y `name_get` queda deprecado) |
| 18+ | igual | `_compute_display_name` (`name_get` **eliminado**) |

Así que para un módulo v16, sobreescribir `_compute_display_name` "funciona" pero es inútil si lo que
consume el valor pasa por `name_get`; el punto correcto en v16 sigue siendo `name_get()`. Eso es lo que
alguien quiso decir con "no standard practice", y es una afirmación sobre la práctica, no sobre la
disponibilidad.

### Product type

```python
# v16: type='product' is the only way
type = fields.Selection([('consu', 'Consumable'), ('service', 'Service'), ('product', 'Storable')])
# is_storable does NOT exist
```

### Command API — available

`Command.create()` etc. work in v16. BUT tuple syntax `(0, 0, vals)` also works and is more common in v16 codebases.

### `slug` import location

```python
# v16: direct import
from odoo.addons.http_routing.models.ir_http import slug

# v18: import moved
# Use request.env['ir.http']._slug(value)
```

---

## Views

### `<tree>` — standard (only option)

```xml
<!-- v16: <tree> is the only valid tag -->
<tree string="Orders">
```

### All v18/v19 view changes — NOT applicable

`<list>`, `<group>` removal, `kanban-box`→`card`, `active_id`→`id`, `numbercall` removal — none of these apply to v16. Use the v16 standard syntax.

---

## JavaScript

### Module system — transition period

```javascript
// v16: both systems coexist
// Legacy (still common in v16):
odoo.define('module.name', function (require) {
    const Widget = require('web.Widget');
    ...
});

// Modern (available in v16, preferred):
/** @odoo-module **/
import { Component } from "@odoo/owl";
```

### OWL — version 2 (transition from legacy widgets)

v16 is the transition version: legacy `Widget` class still works, OWL 2 components are the future. Many core components are still legacy in v16.

### Tour registration

```javascript
// v16: legacy tour system
odoo.define('module.tour', function (require) {
    const tour = require('web_tour.tour');
    tour.register('my_tour', { ... }, [steps]);
});

// v17+: registry-based (NOT available in v16)
```

### Test framework — QUnit

```javascript
// v16: QUnit (same as v17)
QUnit.module("my_module");
QUnit.test("my test", async function (assert) { ... });
```

---

## Test Framework (Python)

### Form class location

```python
# v16: from odoo.tests.common
from odoo.tests.common import Form

# v17+: from odoo.tests
from odoo.tests import Form
```

### `SavepointCase` — available

```python
# v16: SavepointCase still exists (renamed to TransactionCase in v17)
from odoo.tests.common import SavepointCase

# v17+: use TransactionCase (SavepointCase removed)
```

### `freeze_time` — external package

```python
# v16: requires freezegun
from freezegun import freeze_time
```

---

## What v16 does NOT have (compared to v18)

| Feature | v18 | v16 |
|---------|-----|-----|
| `precompute=True` | Yes | No |
| `aggregator` field param | Yes | No — use `group_operator` |
| `is_storable=True` | Yes | No — use `type='product'` |
| `_compute_display_name` | Standard (único punto de override) | **Existe** desde ≤v14, pero en v16 su default delega en `name_get()` → el override efectivo es `name_get` |
| `_read_group()` new signature | Yes | Only from 16.3+ |
| `<list>` view tag | Yes (obligatorio desde v18) | No — en v16 y v17 es `<tree>` |
| Hoot (JS tests) | Yes | No — QUnit |
| Built-in `freeze_time` | Yes | No — freezegun |
| Registry-based tours | No (v17+) | No — `tour.register()` |
| `Form` in `odoo.tests` | Yes | No — `odoo.tests.common` |
| ES modules as default | Yes | No — `odoo.define()` still standard |
| `slug()` from ir.http | Yes | No — `from http_routing.models.ir_http` |

---

## Migration checklist v16 → v18

- [ ] Replace `name_get()` with `_compute_display_name()`
- [ ] Replace `group_operator` with `aggregator`
- [ ] Replace `type='product'` with `is_storable=True`
- [ ] Add `precompute=True` where eligible
- [ ] Update `_read_group()` to new signature (if using old style)
- [ ] Replace `SavepointCase` with `TransactionCase`
- [ ] Move `Form` import from `odoo.tests.common` to `odoo.tests`
- [ ] Replace `freezegun.freeze_time` with `odoo.tests.freeze_time`
- [ ] Migrate `odoo.define()` to ES modules (`/** @odoo-module **/`)
- [ ] Migrate `tour.register()` to `registry.category("web_tour.tours").add()`
- [ ] Migrate QUnit tests to Hoot (optional for v18, required for v19)
- [ ] Update `slug()` import
- [ ] Replace `Command` tuples `(0, 0, vals)` with `Command.create(vals)` (optional, improves readability)
