# Patrones de Migración: Odoo 16 → 17

Cambios breaking que aplican al salto v16→v17. Esta es la transición más destructiva
desde v10→v11 por la eliminación de `attrs` y `states`.

## XML/Views — CRÍTICOS

### 1. attrs dict → atributos separados (BREAKING)
El cambio más invasivo. En v17 se eliminó el atributo `attrs` de las vistas.

```xml
<!-- BEFORE v16 -->
<field name="price" attrs="{'invisible': [('type', '=', 'service')], 'readonly': [('state', '!=', 'draft')], 'required': [('type', '=', 'product')]}"/>

<!-- AFTER v17 -->
<field name="price"
    invisible="type == 'service'"
    readonly="state != 'draft'"
    required="type == 'product'"/>
```

**Reglas de conversión de dominio a expresión Python:**
| Domain v16 | Expression v17 |
|---|---|
| `[('field', '=', value)]` | `field == value` |
| `[('field', '!=', value)]` | `field != value` |
| `[('field', 'in', [a, b])]` | `field in (a, b)` |
| `[('field', 'not in', [a, b])]` | `field not in (a, b)` |
| `[('field', '=', True)]` | `field` |
| `[('field', '=', False)]` | `not field` |
| `['&', cond1, cond2]` | `expr1 and expr2` |
| `['|', cond1, cond2]` | `expr1 or expr2` |
| `[('field', '=', 1)]` (M2O) | `field` (truthy check) |

**Gotchas attrs:**
- `'&'` es implícito en domain — si hay múltiples tuplas sin operador, es AND
- `'1'` (string) vs `1` (int): en expresiones v17 ambos funcionan, pero cuidado con IDs
- Campos computed en expresiones: deben estar presentes en la vista (como `<field invisible="1"/>`)
- attrs con context variables (`context.get('...')`) → convertir a `context.get('...', False)`

### 2. states attribute → invisible expression (BREAKING)
```xml
<!-- BEFORE v16 -->
<button name="action_confirm" states="draft"/>
<field name="partner_id" states="draft,sent"/>

<!-- AFTER v17 -->
<button name="action_confirm" invisible="state != 'draft'"/>
<field name="partner_id" invisible="state not in ('draft', 'sent')"/>
```

**Detección:**
```bash
grep -rn 'states=' --include="*.xml" | grep -v '__manifest__'
```

### 3. tree → list — **NO es de 16→17. Es de 17→18.**

Corregido 2026-08-07. Esta sección decía "completado en v17" y es falso:

| Versión | `ir.ui.view.type` |
|---|---|
| 16.0, **17.0** | `('tree', 'Tree')` — `odoo17/odoo/addons/base/models/ir_ui_view.py:163`, `View.type`. **`<list>` no existe** |
| 18.0, 19.0 | `('list', 'List')` — `odoo18/…:153`, `odoo19/…:149` |

O sea: al migrar **16→17 las vistas siguen siendo `<tree>`** y convertirlas rompe el módulo. La
conversión va en el paso 17→18 (ver `patterns/odoo17_to_18.md`). El `re.sub` de
`patterns/common_patterns.md:171-172` solo debe correr en esa transición.
- Eliminar atributo `edit="1"` (ya no existe)
- `decoration-*` attrs se mantienen igual

### 4. force_save → no longer needed en algunos casos
En v17 algunos campos que antes necesitaban `force_save="1"` ya no lo necesitan si
el campo es editable en el estado actual. Revisar caso por caso.

## Python — BREAKING

### 5. name_get() deprecated → display_name compute
```python
# BEFORE v16
def name_get(self):
    result = []
    for rec in self:
        name = f"[{rec.code}] {rec.name}"
        result.append((rec.id, name))
    return result

# AFTER v17
def _compute_display_name(self):
    for rec in self:
        rec.display_name = f"[{rec.code}] {rec.name}"
```

**Gotcha:** Si el modelo tiene `_rec_name` definido Y un `name_get()` custom,
en v17 necesitas `_compute_display_name`. Si solo usa `_rec_name` sin override, no cambiar.

### 6. _name_search cambio de firma
```python
# BEFORE v16
def _name_search(self, name='', args=None, operator='ilike', limit=100, name_get_uid=None):
    args = args or []
    domain = ['|', ('name', operator, name), ('code', operator, name)]
    return self._search(domain + args, limit=limit, access_rights_uid=name_get_uid)

# AFTER v17
def _name_search(self, name='', domain=None, operator='ilike', limit=None, order=None):
    domain = domain or []
    domain = ['|', ('name', operator, name), ('code', operator, name)] + domain
    return self._search(domain, limit=limit, order=order)
```
- `args` → `domain`
- `name_get_uid` → eliminado
- Añadido `order` parameter

### 7. read_group cambios
```python
# BEFORE v16
def read_group(self, domain, fields, groupby, offset=0, limit=None, orderby=False, lazy=True):

# AFTER v17 — usar _read_group
# La firma cambió, ahora retorna un generador de grupos con aggregate specs
```

### 8. fields.Monetary currency_field
```python
# BEFORE v16 — currency_field implícito si existe 'currency_id'
amount = fields.Monetary()

# AFTER v17 — explícito recomendado
amount = fields.Monetary(currency_field='currency_id')
```

### 9. Command syntax (preferencia, no breaking)
```python
# BEFORE v16
line_ids = [(0, 0, vals), (6, 0, ids)]

# AFTER v17 — preferido
from odoo import Command
line_ids = [Command.create(vals), Command.set(ids)]
```

## JavaScript

### 10. Legacy widgets → OWL migration acelerada
En v17 muchos widgets legacy se convirtieron a OWL. Si el módulo extiende
widgets legacy con `Widget.extend()`, verificar si el widget base cambió a OWL.

### 11. @odoo-module obligatorio
Todo fichero JS sin `/** @odoo-module **/` o `/** @odoo-module alias=... **/`
será ignorado en v17.

## Detección Completa

```bash
# attrs (el más invasivo)
grep -rn "attrs=" --include="*.xml" | wc -l

# states attribute
grep -rn 'states=' --include="*.xml" | grep -v '__manifest__' | wc -l

# tree views
grep -rn '<tree' --include="*.xml" | wc -l

# name_get override
grep -rn 'def name_get' --include="*.py" | wc -l

# _name_search old signature
grep -rn 'name_get_uid' --include="*.py" | wc -l

# old Command syntax
grep -rn '(0, 0,' --include="*.py" | wc -l
grep -rn '(6, 0,' --include="*.py" | wc -l
```
