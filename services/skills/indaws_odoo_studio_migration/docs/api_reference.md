# Referencia de API - Odoo Studio Migration

Referencia rápida de campos, métodos y cambios en V18.

---

## Campos ir.model.fields V18

### Campos Disponibles

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `name` | char | Nombre técnico (ej: x_custom_field) |
| `field_description` | char | Etiqueta visible |
| `ttype` | selection | Tipo: char, text, boolean, integer, float, date, datetime, selection, many2one, one2many, many2many |
| `model_id` | many2one | Modelo al que pertenece |
| `relation` | char | Modelo relacionado (many2one/one2many/many2many) |
| `relation_field` | char | Campo inverso (one2many) |
| `selection` | text | Opciones para selection (JSON) |
| `help` | text | Texto de ayuda |
| `readonly` | boolean | Solo lectura |
| `required` | boolean | Requerido |
| `state` | selection | 'base', 'manual' (usar 'manual' para Studio) |
| `store` | boolean | Almacenar en BD |
| `domain` | char | Dominio para many2one |
| `context` | char | Contexto para many2one |
| `on_delete` | selection | Comportamiento al eliminar (set null, cascade) |

### Campos ELIMINADOS en V18

| Campo | Estado | Alternativa |
|-------|--------|-------------|
| `default` | ❌ ELIMINADO | Automated Action on_create |
| `copy` | ❌ ELIMINADO | Automated Action on_create |
| `translate` | ❌ ELIMINADO | N/A |

---

## Campos base.automation V18

### Campos Disponibles

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `name` | char | Nombre de la automatización |
| `model_id` | many2one | Modelo objetivo |
| `trigger` | selection | on_create, on_write, on_unlink, on_time |
| `filter_domain` | char | Dominio de filtro |
| `on_change_field_ids` | many2many | Campos que disparan (para on_write específico) |
| `action_server_ids` | one2many | Server Actions a ejecutar |
| `active` | boolean | Activo/inactivo |
| `trg_date_id` | many2one | Campo fecha (para on_time) |
| `trg_date_range` | float | Rango de días |
| `trg_date_range_type` | selection | days, hours, minutes |

### Campos ELIMINADOS en V18

| Campo | Estado | Alternativa |
|-------|--------|-------------|
| `trigger_fields` | ❌ ELIMINADO | Usar `on_change_field_ids` |
| `action_server_id` | ❌ ELIMINADO | Usar `action_server_ids` (plural) |
| `state` | ❌ ELIMINADO | Usar `action_server_ids` |

### Estructura Correcta V18

```python
aa_values = {
    'name': 'Nombre Automatización',
    'model_id': model_id,
    'trigger': 'on_write',
    'on_change_field_ids': [(6, 0, [trigger_field_id])],
    'action_server_ids': [(6, 0, [server_action_id])],
    'filter_domain': "[('x_trigger', '=', True)]",
    'active': True,
}
```

---

## Campos ir.cron V18

### Campos Disponibles

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `name` | char | Nombre |
| `model_id` | many2one | Modelo objetivo |
| `state` | selection | Siempre 'code' |
| `code` | text | Código Python |
| `interval_number` | integer | Número del intervalo |
| `interval_type` | selection | minutes, hours, days, weeks, months |
| `active` | boolean | Activo |
| `priority` | integer | Prioridad (default 5) |

### Campos ELIMINADOS en V18

| Campo | Estado | Alternativa |
|-------|--------|-------------|
| `numbercall` | ❌ ELIMINADO | N/A |
| `doall` | ❌ ELIMINADO | N/A |

---

## Campos ir.model.data V18

### Estructura para Studio

```python
{
    'name': 'nombre_unico',           # Nombre del XML ID
    'module': 'studio_customization',  # Siempre este valor para Studio
    'model': 'ir.model.fields',       # Modelo del registro
    'res_id': 12345,                  # ID del registro
    'studio': True,                   # ⚠️ CRÍTICO: Flag para Studio
    'noupdate': False,                # Si puede actualizarse
}
```

---

## Variables Globales en Server Actions

### Disponibles (NO requieren import)

| Variable | Descripción |
|----------|-------------|
| `env` | Entorno de Odoo |
| `model` | Modelo del registro (recordset vacío) |
| `record` | Registro que dispara la acción |
| `records` | Recordset en modo multi |
| `time` | Módulo time de Python |
| `datetime` | Módulo datetime de Python |
| `dateutil` | Módulo dateutil (relativedelta disponible) |
| `timezone` | Módulo timezone de Python |
| `log` | Función: log(message, level='info') |
| `_logger` | Logger: _logger.info(message) |
| `UserError` | Clase de excepción |
| `Command` | Namespace para comandos x2many |

### NO Disponibles

| Función | Alternativa |
|---------|-------------|
| `hasattr()` | try/except |
| `getattr()` | try/except |
| `setattr()` | record['field'] = value |
| `import` | Usar variables globales |

---

## Mapeo de Tipos de Campo

| V13-V17 | Studio V18/V19 |
|---------|----------------|
| `fields.Char` | `char` with `x_` prefix |
| `fields.Text` | `text` with `x_` prefix |
| `fields.Boolean` | `boolean` with `x_` prefix |
| `fields.Integer` | `integer` with `x_` prefix |
| `fields.Float` | `float` with `x_` prefix |
| `fields.Date` | `date` with `x_` prefix |
| `fields.Datetime` | `datetime` with `x_` prefix |
| `fields.Selection` | `selection` (JSON format) |
| `fields.Many2one` | `many2one` with `relation` |
| `fields.One2many` | `one2many` with `relation` + `relation_field` |
| `fields.Many2many` | `many2many` with `relation` |
| `fields.Monetary` | `monetary` with `currency_field` |

---

## Mapeo de Vistas

| V13-V17 | V18 |
|---------|-----|
| `tree` | `list` |
| `form` | `form` |
| `kanban` | `kanban` |
| `search` | `search` |
| `calendar` | `calendar` |
| `pivot` | `pivot` |
| `graph` | `graph` |

---

## Mapeo de Atributos de Vista

| V13-V17 | V18 |
|---------|-----|
| `attrs="{'invisible': [...]}"` | `invisible="condition"` |
| `attrs="{'readonly': [...]}"` | `readonly="condition"` |
| `attrs="{'required': [...]}"` | `required="condition"` |
| `widget="statusbar"` (char) | ❌ Usar `widget="badge"` |
| `optional="hide"` | `optional="hide"` |
| `optional="show"` | `optional="show"` |

---

## Métodos OdooConnection

| Método | Descripción |
|--------|-------------|
| `authenticate()` | Autenticación automática (detecta protocolo) |
| `execute(model, method, args, kwargs)` | Ejecutar método en modelo |
| `is_connected()` | Verificar conexión activa |
| `get_context()` | Obtener contexto de conexión |

---

## Métodos OdooOperations

| Método | Descripción |
|--------|-------------|
| `search(model, domain, limit, offset, order)` | Buscar IDs |
| `read(model, ids, fields)` | Leer registros |
| `search_read(model, domain, fields, ...)` | Buscar y leer |
| `create(model, values)` | Crear registro |
| `write(model, ids, values)` | Actualizar registros |
| `unlink(model, ids)` | Eliminar registros |
| `copy(model, id, default)` | Duplicar registro |
| `name_search(model, name, args, ...)` | Buscar por nombre |
| `name_get(model, ids)` | Obtener display_name |
| `count(model, domain)` | Contar registros |
| `execute_method(model, method, *args)` | Ejecutar método arbitrario |

---

## Métodos StudioMigration

| Método | Descripción |
|--------|-------------|
| `get_model_id(model_name)` | Obtener ID de modelo |
| `create_model(name, model, description)` | Crear modelo Studio |
| `create_field(model_id, name, type, ...)` | Crear campo con state='manual' |
| `register_in_studio(res_id, res_model, prefix)` | Registrar en ir.model.data |
| `create_view(name, model, arch, ...)` | Crear vista |
| `create_inherited_view(...)` | Crear vista heredada |
| `create_window_action(...)` | Crear acción de ventana |
| `create_server_action(...)` | Crear Server Action |
| `create_automated_action(...)` | Crear Automated Action |
| `create_menu(name, parent_id, action_id)` | Crear menú |
| `find_menu_by_name(name, parent_name)` | Buscar menú |
| `create_access_right(...)` | Crear permisos |
| `verify_modules_installed(modules)` | Verificar módulos |
| `install_module(module_name)` | Instalar módulo |
| `get_xml_id(res_id, res_model)` | Obtener XML ID completo de un registro |
| `get_action_xml_id(action_id, action_type)` | Obtener XML ID de una acción |
| `find_button_numeric_action_ids(view_id)` | Detectar botones con IDs numéricos en vistas |
| `fix_button_action_ids(view_id)` | Corregir IDs numéricos en botones de vistas |
| `resolve_arch_action_ids(arch)` | Reemplazar IDs numéricos en arch string antes de crear vista |

---

## Patrones de Código Comunes

### Asignación de campos

```python
# ✅ CORRECTO en Server Actions V18
record['x_field'] = value

# ❌ INCORRECTO
record.x_field = value  # Genera STORE_ATTR error
```

### Acceso a campos many2one

```python
# ✅ CORRECTO
if record.x_partner_id:
    partner_name = record.x_partner_id.name

# ❌ INCORRECTO
if hasattr(record, 'x_partner_id'):
    partner_name = record.x_partner_id.name
```

### Fechas y tiempos

```python
# ✅ CORRECTO
now = datetime.datetime.now()
today = datetime.date.today()
relativedelta = dateutil.relativedelta.relativedelta
yesterday = today - relativedelta(days=1)
```

### Logging

```python
# ✅ CORRECTO
log(f"Procesando registro {record.id}")
log("Error en proceso", level='error')
_logger.info(f"Procesando {len(records)} registros")
```

### Comandos x2many

```python
# ✅ CORRECTO
record['x_line_ids'] = [
    Command.create({'name': 'Línea 1'}),
    Command.create({'name': 'Línea 2'}),
]

# O con tuples
record['x_line_ids'] = [
    (0, 0, {'name': 'Línea 1'}),
    (0, 0, {'name': 'Línea 2'}),
]
```
