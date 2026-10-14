# Migration Scripts Decision Matrix

Cuándo y qué tipo de migration script necesitas al actualizar un módulo Odoo.

**Prerequisito**: leer `v18-base/modules.md` §Migration Scripts para la estructura de carpetas y templates.

## Execution Order Recap

```
pre-migrate.py  →  ORM update (schema)  →  post-migrate.py  →  [all modules done]  →  end-migrate.py
(old schema)                                (new schema)                                (all new schemas)
```

- **pre-migrate**: raw SQL only (`cr`), ORM no disponible, tablas en schema VIEJO
- **post-migrate**: ORM disponible (`env`), tablas en schema NUEVO
- **end-migrate**: ORM disponible, TODOS los módulos ya actualizados (cross-module)

---

## Decision Matrix

### Cambios de campos

| Cambio | ¿Script? | Tipo | Qué hacer |
|--------|----------|------|-----------|
| Añadir campo nuevo | No | — | ORM añade columna. Si necesita valor en registros existentes → `default=` en campo o `post-migrate` con `UPDATE` |
| Añadir campo computed stored | Depende | `post-migrate` | Si `compute` depende de datos existentes → forzar recompute: `env['model'].search([])._compute_field()` |
| Eliminar campo del código | Opcional | `pre-migrate` | Columna queda huérfana. `DROP COLUMN` si quieres limpiar, pero no es obligatorio |
| **Renombrar campo** | **Sí** | `pre-migrate` | `ALTER TABLE t RENAME COLUMN old TO new` — si no, Odoo crea columna nueva y pierdes datos |
| **Cambiar tipo de campo** (Char→Int, M2O→M2M) | **Sí** | `pre-migrate` | Guardar datos en columna temporal → ORM recrea columna → `post-migrate` restaura datos |
| Cambiar de required=False a required=True | Depende | `pre-migrate` | Si hay NULLs en DB → `UPDATE` con valor default antes de que ORM añada NOT NULL |
| Cambiar campo normal a computed stored | Depende | `post-migrate` | Forzar recompute. Datos originales se pierden al cambiar a computed |
| Cambiar computed stored a normal | Depende | No | ORM mantiene la columna. Valores existentes se conservan |
| **Cambiar Selection values** (eliminar opción) | **Sí** | `post-migrate` | `UPDATE table SET field = 'new_val' WHERE field = 'removed_val'` |
| Añadir Selection value nuevo | No | — | Sin impacto en datos existentes |
| Cambiar campo de company_dependent | **Sí** | `pre-migrate` + `post-migrate` | Migrar datos de JSONB a columna normal o viceversa |

### Cambios de modelos

| Cambio | ¿Script? | Tipo | Qué hacer |
|--------|----------|------|-----------|
| **Renombrar modelo** (`_name`) | **Sí** | `pre-migrate` | `ALTER TABLE old_name RENAME TO new_name` + actualizar `ir_model_data`, `ir_model`, `ir_model_fields`, `ir_model_relation` |
| Cambiar `_inherit` (sin `_name`) a `_name` propio — **extension in-place → modelo nuevo** | **Sí** | `pre-migrate` | Crear tabla nueva, migrar datos desde la tabla del modelo que se extendía. (Decía "delegation→new model": `_inherit` sin `_name` es **extensión**, no delegación — la delegación es `_inherits`. Corregido 2026-08-07) |
| Cambiar `_table` | **Sí** | `pre-migrate` | `ALTER TABLE old_table RENAME TO new_table` |
| Añadir `_sql_constraints` | Depende | `pre-migrate` | Si datos existentes violan la constraint → limpiar ANTES |
| Eliminar `_sql_constraints` | No | — | Odoo las elimina automáticamente |
| Cambiar `_rec_name` | No | — | Solo afecta display, no schema |
| Cambiar `_order` | No | — | Solo afecta queries, no schema |

### Cambios de relaciones

| Cambio | ¿Script? | Tipo | Qué hacer |
|--------|----------|------|-----------|
| **Mover campo de modelo A a modelo B** | **Sí** | `post-migrate` | `UPDATE B SET field = (SELECT field FROM A WHERE ...)` con ORM o SQL |
| M2O → M2M | **Sí** | `pre-migrate` + `post-migrate` | Pre: guardar datos M2O. Post: insertar en tabla rel M2M |
| M2M → M2O | **Sí** | `post-migrate` | Decidir cuál de los múltiples valores mantener |
| Cambiar comodel de M2O | **Sí** | `post-migrate` | Mapear IDs del modelo viejo al nuevo |
| Renombrar tabla rel de M2M | **Sí** | `pre-migrate` | `ALTER TABLE old_rel RENAME TO new_rel` |

### Cambios de XML/datos

| Cambio | ¿Script? | Tipo | Qué hacer |
|--------|----------|------|-----------|
| **Cambiar XML ID de record** | **Sí** | `pre-migrate` | `UPDATE ir_model_data SET name = 'new' WHERE module = 'x' AND name = 'old'` |
| Eliminar XML record | Depende | `pre-migrate` | Si otros módulos lo referencian via `ref()` → actualizar `ir_model_data` primero |
| Cambiar XML ID de vista heredada | **Sí** | `pre-migrate` | Si no, Odoo crea vista duplicada |
| Cambiar noupdate de data | No | — | Solo afecta futuras actualizaciones |

### Cambios de módulo/manifest

| Cambio | ¿Script? | Tipo | Qué hacer |
|--------|----------|------|-----------|
| Bump versión sin cambios schema | No | — | Solo bump en `__manifest__` |
| Cambiar dependencias | No | — | Odoo resuelve el grafo automáticamente |
| **Merge 2 módulos en 1** | **Sí** | `pre-migrate` + `post-migrate` | Pre: `UPDATE ir_model_data SET module = 'new' WHERE module = 'old'`. Post: limpiar `ir_module_module` |
| **Split 1 módulo en 2** | **Sí** | `pre-migrate` | Mover `ir_model_data` al módulo nuevo. Registrar módulo nuevo en `ir_module_module` |
| Renombrar módulo técnico | **Sí** | `pre-migrate` | Actualizar `ir_model_data.module`, `ir_module_module.name`, renombrar carpeta |

### Migración entre versiones Odoo

| Cambio | ¿Script? | Tipo | Qué hacer |
|--------|----------|------|-----------|
| `name_get` → `_compute_display_name` | No | — | Solo código Python, no hay datos que migrar |
| `group_operator` → `aggregator` | No | — | Solo código Python |
| `type='product'` → `is_storable=True` | No | — | Odoo migra esto en core |
| `tree` → `list` en vistas (v19) | No | — | Solo XML, sin datos |
| Campo movido de modelo en core | **Sí** | `post-migrate` | Consultar release notes para mapeo exacto |

---

## Template con Idempotency Guards

```python
# migrations/18.0.2.0.0/pre-migrate.py

import logging

_logger = logging.getLogger(__name__)


def migrate(cr, version):
    if not version:
        return

    # Guard: check column exists before renaming
    cr.execute("""
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'my_table' AND column_name = 'old_field'
    """)
    if cr.fetchone():
        cr.execute("ALTER TABLE my_table RENAME COLUMN old_field TO new_field")
        _logger.info("Renamed my_table.old_field → new_field")
```

```python
# migrations/18.0.2.0.0/post-migrate.py

import logging
from odoo import api, SUPERUSER_ID

_logger = logging.getLogger(__name__)


def migrate(cr, version):
    if not version:
        return
    env = api.Environment(cr, SUPERUSER_ID, {})

    # Guard: only update records that need it
    records = env['my.model'].search([('field', '=', 'old_value')])
    if records:
        records.write({'field': 'new_value'})
        _logger.info("Migrated %d records: field old_value → new_value", len(records))
```

---

## Utility Functions (openupgradelib)

Si el proyecto usa `openupgradelib` (OCA), estos helpers evitan SQL manual:

```python
from openupgradelib import openupgrade

# Renombrar columna
openupgrade.rename_columns(cr, {'my_table': [('old_col', 'new_col')]})

# Renombrar modelo
openupgrade.rename_models(cr, [('old.model', 'new.model')])

# Renombrar módulo
openupgrade.update_module_names(cr, [('old_module', 'new_module')])

# Renombrar XML IDs
openupgrade.rename_xmlids(cr, [('old_module.old_id', 'new_module.new_id')])

# Mover campo de modelo
openupgrade.move_field_m2o(cr, 'source.model', 'field_name', 'm2o_field', 'target.model', 'target_field')

# Check column exists
openupgrade.column_exists(cr, 'table_name', 'column_name')
```

> **Nota**: `openupgradelib` no viene con Odoo. Hay que instalarla (`pip install openupgradelib`) o copiar las funciones necesarias como SQL directo.

---

## Versioning: Cuándo crear la carpeta

El nombre de la carpeta debe coincidir con la **nueva versión** del manifest:

```python
# ANTES: version '18.0.1.0.0'
# DESPUÉS: version '18.0.2.0.0' (bump porque renombraste un campo)
# → carpeta: migrations/18.0.2.0.0/pre-migrate.py
```

Odoo ejecuta scripts de TODAS las versiones entre la versión instalada y la nueva, en orden ascendente. Si un usuario instala fresh, no ejecuta ningún migration script.

---

## Checklist antes de hacer bump

- [ ] ¿Algún campo renombrado/eliminado/cambiado de tipo? → `pre-migrate`
- [ ] ¿Algún modelo renombrado/fusionado? → `pre-migrate`
- [ ] ¿Selection values eliminados? → `post-migrate`
- [ ] ¿Campos moved between models? → `post-migrate`
- [ ] ¿Computed stored fields que necesitan recompute? → `post-migrate`
- [ ] ¿XML IDs cambiados? → `pre-migrate`
- [ ] ¿SQL constraints nuevas con datos que violan? → `pre-migrate`
- [ ] ¿Scripts son idempotentes? (guards con `column_exists`, `fetchone`)
- [ ] ¿Version en carpeta coincide con nueva version en manifest?
- [ ] ¿Logger info para cada operación? (debugging en producción)
