# Patrones de Migracion Codigo → Studio

## Prefijo Obligatorio

Todos los campos personalizados DEBEN usar prefijo `x_`:
```python
# Original V13
is_custom = fields.Boolean(string="Es Personalizado")

# Studio V18
x_is_custom = fields.Boolean(string="Es Personalizado")
```

## Server Actions — Variables Disponibles

```python
# DISPONIBLES (sin import)
env, model, record, records, time, datetime, dateutil, timezone, log, UserError, Command

# NO DISPONIBLES
import, hasattr, getattr, setattr, __builtins__, exec, eval

# SINTAXIS CORRECTA V18
record['x_field'] = value           # NO: record.x_field = value
try:                                # NO: if hasattr(record, 'field')
    if record.field: pass
except: pass
```

## Mapeo de Tipos V13-V17 → V18

| Original | Studio V18 |
|----------|-----------|
| `tree` view | `list` view |
| `attrs="{'invisible': [...]}"` | `invisible="condition"` |
| `widget="statusbar"` (char) | `widget="badge"` (char) / `statusbar` (selection) |
| `is_customer` | `customer_rank > 0` |
| `quantity_done` (stock.move) | `stock.move.line.quantity` |

Ver `docs/v18_changes.md` para lista completa.

---

## Patron: Onchange → Automated Action

> **Antes de migrar a Studio, comprueba con `odoo-classification-guide` si esta logica es trivial.** Un onchange con logica de negocio condicional (como el del ejemplo) suele ser **codigo gobernado** que conviene mantener como modulo, no convertir en Automated Action ingobernable. Este patron es el *como* tecnico cuando el veredicto ya es Studio, no una recomendacion de mover toda la logica a Studio.

```python
# Original
@api.onchange('brand_id')
def onchange_brand(self):
    if self.brand_id and self.state == 'draft':
        self.analytic_account_id = self.brand_id.analytic_account_id

# Studio V18: Automated Action con trigger on_write en campo x_brand_id
code = '''
if record.x_brand_id and record.state == 'draft':
    if record.x_brand_id.x_analytic_account_id:
        record['x_analytic_account_id'] = record.x_brand_id.x_analytic_account_id.id
'''
```

**Regla:** NUNCA migrar onchange a `on_change` trigger (es sincrono, bloquea UI). Usar `on_write`.

## Patron: Boton → Server Action

```python
# Server Action que retorna accion
code = '''
wizard = record
result_ids = []
# ... calculo ...
action = {
    'name': 'Resultados',
    'type': 'ir.actions.act_window',
    'res_model': 'x_result',
    'view_mode': 'list',
    'target': 'current',
    'domain': [('id', 'in', result_ids)],
}
'''
```

## Patron: with_delay() → Campo Disparador

```python
# 1. Campo trigger
studio.create_field(model_id, 'x_compute_trigger', 'boolean', 'Disparar')

# 2. Boton encola trabajo
code = '''record.write({'x_compute_trigger': True})'''

# 3. Automated Action detecta trigger (on_write en x_compute_trigger)
code = '''
if record.x_compute_trigger:
    record['x_compute_trigger'] = False
    # ... calculo pesado ...
'''
```

## Patron: Boton Action con XML ID Portable

```python
# INCORRECTO - ID numerico no portable
arch = f'<button name="{sa_id}" type="action" string="Ejecutar"/>'

# CORRECTO - XML ID
studio.register_in_studio(sa_id, 'ir.actions.server', 'mi_accion')
xml_id = studio.get_action_xml_id(sa_id, 'server')
arch = f'<button name="{xml_id}" type="action" string="Ejecutar"/>'

# ALTERNATIVA - Resolver automaticamente despues
arch = f'<button name="{sa_id}" type="action" string="Ejecutar"/>'
portable_arch = studio.resolve_arch_action_ids(arch)
```

## Patron: Modelo Nuevo → Opciones Built-in

En vez de crear campos uno a uno, usar las opciones de `studio_model_create()`:

| Si el modulo tiene... | Usar opcion |
|----------------------|-------------|
| Chatter/actividades | `use_mail` |
| Campo active/archivado | `use_active` |
| Orden drag-drop | `use_sequence` |
| Campo responsable (user) | `use_responsible` |
| Relacion con cliente | `use_partner` |
| Multi-company | `use_company` |
| Notas/descripcion HTML | `use_notes` |
| Campo fecha | `use_date` |
| Rango de fechas | `use_double_dates` |
| Campo monetario | `use_value` |
| Imagen | `use_image` |
| Workflow por etapas | `use_stages` (crea modelo secundario) |
| Tags/etiquetas | `use_tags` (crea modelo secundario) |
| Lineas de detalle | `lines` (crea modelo secundario) |

Cada opcion genera campos con nombres estandar (`x_studio_*`) + vistas auto-generadas.

## Orden de Creacion (CRITICO)

```
1. Verificar modulos instalados
2. Crear modelos (state='manual') + registrar en Studio
3. Crear campos M2O hijos
4. Crear campos basicos (char, boolean, selection, etc.)
5. Crear campos O2M padres (necesitan M2O inverso existente)
6. Crear Server Actions
7. Crear vistas (referencian IDs de acciones)
8. fix_button_action_ids() (XML IDs portables)
9. Crear Automated Actions
10. Crear menus
```

## Buenas Practicas

### Obligatorias
1. Prefijo `x_` para campos y modelos
2. `state='manual'` para todos los campos Studio
3. Registrar en Studio con `studio=True` para exportacion
4. Usar `list` no `tree` en V18
5. No usar campos eliminados V18 (default, copy, numbercall)
6. Sintaxis diccionario en SA: `record['field'] = value`

### Recomendadas
7. Scripts idempotentes (verificar si existe antes de crear)
8. Usar try/except en lugar de hasattr (bloqueado en safe-eval)
9. Verificar modulos instalados antes de usar sus campos
10. Crear indices SQL para M2O criticos post-migracion
