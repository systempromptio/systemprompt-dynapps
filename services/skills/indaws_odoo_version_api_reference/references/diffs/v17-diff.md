# v17 Diff — What v17 lacks compared to v18

Base reference is v18. This file documents what is DIFFERENT or MISSING in Odoo 17.

## ORM / Python API

### `precompute=True` — available but newer

Introduced in v17.0 (backported from v16.4 development). Works the same as v18. Not available in v16.0.

### `name_get()` — still works

```python
# v17: name_get() is deprecated but functional
# _compute_display_name is the preferred approach (same as v18)
# name_get() will still be called if _compute_display_name is not overridden
```

### `group_operator` — deprecated but works

```python
# v17.0-17.1: group_operator still works
amount = fields.Float(group_operator='sum')

# v17.2+: aggregator is preferred, group_operator still works
amount = fields.Float(aggregator='sum')
```

### `_read_group()` — new signature since v16.3

Same signature as v18. The old signature (v16.0) is gone.

### Product type

```python
# v17: type='product' for storable products
# v18: is_storable=True (type='product' still works but deprecated)
type = fields.Selection([('consu', 'Consumable'), ('service', 'Service'), ('product', 'Storable')])
```

### Command API — same as v18

`Command.create()`, `Command.update()`, etc. all work identically.

---

## Views

### `<tree>` — works (not deprecated yet)

```xml
<!-- v17: <tree> is the standard tag -->
<tree string="Orders" editable="top">
<!-- <list> is NOT valid in v17 -->
```

### `attrs=` / `states=` — REMOVED in v17 (CRITICAL)

The `attrs="{...}"` and `states="..."` view attributes are **gone** in v17. The
conditions move directly onto each affected attribute as a Python expression
evaluated against the record:

```xml
<!-- v16 and earlier -->
<field name="discount" attrs="{'invisible': [('state','!=','draft')],
                                'readonly': [('locked','=',True)]}"/>
<button name="confirm" states="draft,sent"/>

<!-- v17+ -->
<field name="discount"
       invisible="state != 'draft'"
       readonly="locked"/>
<button name="confirm" invisible="state not in ('draft','sent')"/>
```

Expressions use the record's field values directly (`state`, `locked`, …),
`parent.` for the parent record in a one2many, and standard Python operators.
This is one of the most common v16→v17 migration breakages — a leftover `attrs=`
silently does nothing or raises on load depending on the build.

### Search `<group>` — works

```xml
<!-- v17: <group> wrapper is valid and standard -->
<search>
    <group expand="0" string="Group By">
        <filter .../>
    </group>
</search>
```

### Kanban `kanban-box` — standard

```xml
<!-- v17: use kanban-box -->
<t t-name="kanban-box">
```

### Form context `active_id` — standard

```xml
<!-- v17: active_id works normally -->
context="{'search_default_type_id': active_id}"
```

---

## JavaScript

### RPC service — works everywhere

```javascript
// v17: useService("rpc") works in both backend and frontend
this.rpc = useService("rpc");
```

### Tour registration — modern style

```javascript
// v17: registry-based tour registration (same as v18)
import { registry } from "@web/core/registry";
registry.category("web_tour.tours").add("my_tour", { ... });
```

### Test framework — QUnit (NOT Hoot)

```javascript
// v17: QUnit is the test framework
// v18: Hoot replaces QUnit
QUnit.module("my_module");
QUnit.test("my test", async function (assert) { ... });
```

### `odoo.define()` — deprecated but works

```javascript
// v17: legacy module system still functional
odoo.define('module.name', function (require) { ... });
// ES modules (/** @odoo-module **/) are preferred
```

---

## Test Framework (Python)

### `freeze_time` — external package

```python
# v17: requires freezegun external package
from freezegun import freeze_time

# v18: built-in freeze_time
from odoo.tests import freeze_time
```

### Form import

```python
# v17+: from odoo.tests
from odoo.tests import Form

# v16: from odoo.tests.common
from odoo.tests.common import Form
```

---

## What v17 does NOT have (compared to v18)

> **Este fichero es el destino de TODO cambio v17→v18.** No hay `v18-diff.md` —v18 **es** la base—,
> así que un cambio que ocurre *en* v18 solo cabe aquí. Cuando falta, acaba colándose en
> `v19-diff.md`, y ahí lo lee quien migra de 18 a 19 en vez de quien viene de 17. Ya han aparecido
> **6 filas mal ubicadas** en `v19-diff.md` por esta causa (lotes 58 y 59).

| Feature | v18 | v17 |
|---------|-----|-----|
| `is_storable=True` on products | Yes | No — use `type='product'` (`stock/models/product.py:661-663`, `ProductTemplate`, v17 @3e1233ed1 — el `selection_add` de `type`; en v18 el campo nuevo es `stock/models/product.py:704`, `ProductTemplate.is_storable`) |
| Built-in `freeze_time` | Yes | No — use `freezegun` package |
| Hoot test framework (JS) | Yes (`web/static/lib/hoot/`) | No — use QUnit (**0 imports de `@odoo/hoot` en v17**) |
| `<list>` view tag | Yes | No — use `<tree>` |
| `_compute_display_name` as sole path | Preferred | Works, but `name_get()` fallback still active |
| `aggregator` field param | Yes (`odoo/fields.py:480-482`, `Field._get_attrs`, avisa del viejo) | **No en 17.0**: es `group_operator` (`odoo/fields.py:323`, `Field.group_operator`, v17); **0 ocurrencias de `aggregator`** |

### Rupturas duras de v17 → v18 (esto NO avisa, revienta)

Añadidas 2026-08-17 tras medirlas contra CE17 `@3e1233ed1` y CE18 `@99f5a33c64`. Son las que hacen
que un módulo v17 **no arranque** o **cambie de comportamiento en silencio** al pasar a 18:

| Qué | v17 | v18 | Cómo se manifiesta |
|---|---|---|---|
| **`ir.cron.numbercall` / `doall`** | existen (`odoo/addons/base/models/ir_cron.py:67`, `ir_cron`, v17) | **0 ocurrencias** | `Invalid field 'numbercall'` → **el módulo no carga**. Sustituto de `numbercall=1`: `_notify_progress(..., deactivate=True)` (`odoo/addons/base/models/ir_cron.py:734-749`, `ir_cron._notify_progress`, v18). Y v18 añade auto-desactivación por **5 fallos + 7 días** (`odoo/addons/base/models/ir_cron.py:24`, `MIN_FAILURE_COUNT_BEFORE_DEACTIVATION`, y `:25`, `MIN_DELTA_BEFORE_DEACTIVATION`, v18) |
| **`auth='none'` es `readonly=True` por defecto** | no existe el mecanismo (`readonly` es clave interna a `False`, `odoo/http.py:880`, `_generate_routing_rules`, v17) | `default_mode = …get('readonly', default_auth == 'none')` (`odoo/http.py:890`, `_check_and_complete_route_definition`, v18) | Un endpoint sin auth que **escribe** deja de escribir. **Silencioso.** Fix: `readonly=False` explícito |
| **Guardián del cursor en tests** | **no existe** | `commit`, `rollback` **y `close`** parcheados (`odoo/tests/common.py:1059-1065`, `setUpClass.forbidden`) | Un test v17 que hacía `cr.commit()` pasa a `AssertionError`. En v17 no fallaba: **corrompía el cursor en silencio** |
| **`account.account.company_id` → `company_ids`** | `company_id` m2o (`account/models/account_account.py:102`, `AccountAccount.company_id`, v17) | `company_ids` m2m (`account/models/account_account.py:106`, `AccountAccount.company_ids`, v18) | Dominios y `search` por `company_id` fallan. Ojo: sigue habiendo un `company_id` en ese fichero (`account/models/account_account.py:1455`, `AccountGroup.company_id`) pero es de **`account.group`** |
| **`kanban-box` deprecado** | estándar, sin aviso | avisa en log (`odoo/addons/base/models/ir_ui_view.py:1515-1517`, `View._validate_tag_kanban`) y `card` ya funciona | No rompe, pero un módulo v18 **nuevo** con `kanban-box` nace deprecado — y en v19 falla |

---

## Migration checklist v17 → v18

**Rupturas duras primero** (si no las haces, no arranca o cambia de comportamiento):

- [ ] **Quitar `numbercall` y `doall` de todo XML de cron** → `Invalid field`. Si necesitas «una
      pasada», usar `_notify_progress(..., deactivate=True)`
- [ ] **Revisar los `@http.route(auth='none')` que ESCRIBAN** → añadir `readonly=False`
- [ ] **Buscar `cr.commit()` / `cr.rollback()` en los tests** → savepoint, o cursor aparte
- [ ] **Cambiar los dominios sobre `account.account.company_id`** → `company_ids` (m2m)

Luego lo que solo deprecia o mejora:

- [ ] Replace `type='product'` with `is_storable=True` for storable products
- [ ] Replace `group_operator` with `aggregator` (en 17.0 no existe `aggregator`; v18 acepta el
      viejo con `DeprecationWarning`, `odoo/fields.py:480-482`, `Field._get_attrs`)
- [ ] Replace `freezegun.freeze_time` with `odoo.tests.freeze_time`
- [ ] Migrate JS tests from QUnit to Hoot
- [ ] Ensure `_compute_display_name` overrides exist (remove `name_get()` reliance)
- [ ] Consider using `<list>` tag (optional in v18, mandatory in v19)
- [ ] Cambiar `kanban-box` por `card` (v18 avisa en log; v19 lo exige)
