# Cambios en Odoo 18 - Guía de Migración

Resumen de cambios críticos al migrar de V13-V17 a V18.

---

## Cambios en Vistas

### Tipo de Vista

```xml
<!-- V13-V17 -->
<tree string="Modelo">
    <field name="name"/>
</tree>

<!-- V18 -->
<list string="Modelo">
    <field name="name"/>
</list>
```

### Atributos Condicionales

```xml
<!-- V13-V17 -->
<field name="state" attrs="{'invisible': [('state', '=', False)]}"/>
<field name="active" attrs="{'readonly': [('state', '!=', 'draft')]}"/>

<!-- V18 -->
<field name="state" invisible="not state"/>
<field name="active" readonly="state != 'draft'"/>
```

### Widget Statusbar

```xml
<!-- V13-V17 -->
<field name="state" widget="statusbar"/>  <!-- Funciona con char -->

<!-- V18 -->
<field name="state" widget="badge"/>      <!-- Usar badge para char -->
<field name="state" widget="statusbar"/>   <!-- Solo para selection -->
```

---

## Cambios en Campos

### res.partner

| V13-V17 | V18 | Notas |
|---------|-----|-------|
| `is_customer = True` | `customer_rank > 0` | Eliminado |
| `is_supplier = True` | `supplier_rank > 0` | Eliminado |

```python
# V13-V17
partners = env['res.partner'].search([('is_customer', '=', True)])

# V18
partners = env['res.partner'].search([('customer_rank', '>', 0)])
```

### ir.model.fields

| Campo | V13-V17 | V18 |
|-------|---------|-----|
| `default` | ✅ Válido | ❌ ELIMINADO |
| `copy` | ✅ Válido | ❌ ELIMINADO |
| `translate` | ✅ Válido | ❌ ELIMINADO |

```python
# V13-V17
env['ir.model.fields'].create({
    'name': 'x_campo',
    'default': 'False',
    'copy': False,
})

# V18 - NO incluir estos campos
env['ir.model.fields'].create({
    'name': 'x_campo',
    'state': 'manual',
})
```

---

## Cambios en Stock

### stock.move

| Campo | V13-V17 | V18 |
|-------|---------|-----|
| `quantity_done` | ✅ Válido | ❌ ELIMINADO |
| `production_id` | ✅ Válido | Usar `move_id.production_id` desde línea |

```python
# V13-V17
moves = env['stock.move'].search_read([
    ('production_id', '!=', False),
    ('state', '=', 'done'),
], ['product_id', 'quantity_done'])

# V18 - Usar stock.move.line
move_lines = env['stock.move.line'].search_read([
    ('move_id.production_id', '!=', False),
    ('state', '=', 'done'),
], ['product_id', 'quantity'])
```

### product.product

| Campo | V13-V17 | V18 |
|-------|---------|-----|
| `bom_ids` | ✅ Válido (si mrp instalado) | ✅ Válido (si mrp instalado) |

```python
# Verificar MRP instalado antes de usar
module = env['ir.module.module'].search([('name', '=', 'mrp')])
if module and module.state == 'installed':
    products = env['product.product'].search([('bom_ids', '!=', False)])
```

---

## Cambios en Automated Actions

### base.automation

| Campo | V13-V17 | V18 |
|-------|---------|-----|
| `trigger_fields` | ✅ Válido | ❌ ELIMINADO |
| `action_server_id` | ✅ Válido | ❌ ELIMINADO |
| `on_change_field_ids` | N/A | ✅ Nuevo (many2many) |
| `action_server_ids` | N/A | ✅ Nuevo (one2many) |

```python
# V13-V17
env['base.automation'].create({
    'name': 'Auto-sync',
    'trigger_fields': [(6, 0, [field_id])],
    'action_server_id': sa_id,
})

# V18
env['base.automation'].create({
    'name': 'Auto-sync',
    'on_change_field_ids': [(6, 0, [field_id])],
    'action_server_ids': [(6, 0, [sa_id])],
})
```

---

## Cambios en Scheduled Actions

### ir.cron

| Campo | V13-V17 | V18 |
|-------|---------|-----|
| `numbercall` | ✅ Válido | ❌ ELIMINADO |
| `doall` | ✅ Válido | ❌ ELIMINADO |

```python
# V13-V17
env['ir.cron'].create({
    'name': 'Limpiar',
    'numbercall': -1,  # Infinito
    'doall': False,
})

# V18 - Sin estos campos
env['ir.cron'].create({
    'name': 'Limpiar',
    'interval_number': 1,
    'interval_type': 'days',
    'active': True,
})
```

---

## Cambios en Server Actions

### Operaciones Prohibidas

```python
# V13-V17 - Funciona
record.field = value
hasattr(record, 'field')

# V18 - ERROR
record.field = value        # STORE_ATTR prohibido
hasattr(record, 'field')    # NameError

# V18 - CORRECTO
record['field'] = value
try:
    if record.field:
        pass
except Exception:
    pass
```

### Imports Prohibidos

```python
# V13-V17 - Funciona
from datetime import timedelta
from dateutil.relativedelta import relativedelta

# V18 - ERROR
# Todos los imports están bloqueados

# V18 - CORRECTO
relativedelta = dateutil.relativedelta.relativedelta
now = datetime.datetime.now()
```

---

## Cambios en Distribución Analítica

### V13-V17

```python
# Campo directo
record.analytic_account_id = analytic_id
```

### V18

```python
# Campo JSON
record.analytic_distribution = {str(analytic_id): 100.0}
```

---

## Resumen de Migración

### Antes de Migrar

- [ ] Verificar que todos los módulos requeridos están instalados
- [ ] Identificar campos eliminados en V18
- [ ] Revisar vistas que usan `tree` → cambiar a `list`
- [ ] Revisar Server Actions con imports → usar variables globales
- [ ] Revisar Automated Actions → usar nuevos campos

### Durante la Migración

- [ ] Usar `state='manual'` para campos Studio
- [ ] Usar `studio=True` en ir.model.data
- [ ] Cambiar `tree` a `list` en todas las vistas
- [ ] Usar sintaxis de diccionario en Server Actions
- [ ] Verificar que Automated Actions usan campos V18

### Después de Migrar

- [ ] Probar todas las Server Actions
- [ ] Verificar Automated Actions se disparan
- [ ] Comprobar vistas renderizan correctamente
- [ ] Exportar desde Studio y verificar
