# Gestión de Scripts de Migración

> Fuente: Artículo 2602, Sección 15
> Tipos de scripts, estructura de directorios, nomenclatura, idempotencia, dry-run y logs.

---

## Tipos de Scripts

| Tipo | Cuándo se ejecuta | Propósito | Ejemplo |
|------|-------------------|-----------|---------|
| **Pre-migrate** | ANTES del upgrade de Odoo | Preparar datos para que el upgrade no falle | Renombrar campos, limpiar datos inconsistentes, eliminar registros huérfanos |
| **Post-migrate** | DESPUÉS del upgrade de Odoo | Transformar datos a la nueva estructura | Mapear campos renombrados, recalcular valores, migrar datos de modelos eliminados |
| **Cleanup** | Después del post-migrate | Limpieza de datos temporales o obsoletos | Eliminar modelos deprecated, limpiar ir.model.data huérfanos, purgar ir.attachment obsoletos |
| **Validación** | Después de todo el proceso | Verificar integridad de datos | Contar registros, verificar saldos, validar relaciones |

---

## Estructura de Directorios

```
repo/
├── migration_scripts/
│   ├── README.md                    ← Índice con descripción de cada script
│   ├── pre_migrate/
│   │   ├── 001_rename_old_fields.py
│   │   ├── 002_clean_orphan_records.py
│   │   └── ...
│   ├── post_migrate/
│   │   ├── 001_remap_partner_fields.py
│   │   ├── 002_recalculate_totals.py
│   │   └── ...
│   ├── cleanup/
│   │   ├── 001_remove_deprecated_models.py
│   │   └── ...
│   └── validation/
│       ├── 001_count_records.py
│       ├── 002_check_balances.py
│       └── ...
```

---

## Convenciones de Nomenclatura

**Formato:** `NNN_descripcion_breve.py`

- `NNN`: Número secuencial de 3 dígitos (001, 002...) que define el orden de ejecución.
- `descripcion_breve`: Snake_case, máximo 40 caracteres, describe la acción.

**Ejemplos:**
- `001_rename_x_studio_fields.py`
- `002_clean_orphan_analytic_lines.py`
- `003_migrate_custom_sale_fields.py`

---

## Estructura de un Script

```python
"""
Script: NNN_descripcion.py
Tipo: pre_migrate / post_migrate / cleanup / validation
Proyecto: [Nombre del cliente]
Migración: vXX → vYY
Autor: [Nombre]
Fecha: [DD/MM/YYYY]

Descripción:
    [Qué hace este script y por qué es necesario]

Dependencias:
    - [Scripts que deben ejecutarse antes]

Modelos afectados:
    - [model.name]

Reversible: Sí / No
    Si sí: [Cómo revertir]
"""

import xmlrpc.client  # o requests para JSON2

# Configuración — se inyecta desde el runner o se lee de variables de entorno
URL = None
DB = None
UID = None
PASSWORD = None


def execute(url, db, uid, password, dry_run=False):
    """
    Punto de entrada del script.

    Args:
        url: URL del servidor Odoo
        db: Nombre de la base de datos
        uid: ID del usuario autenticado
        password: Contraseña o API key
        dry_run: Si True, solo muestra qué haría sin ejecutar cambios

    Returns:
        dict con resultado: {"status": "ok/error", "records_affected": N, "details": "..."}
    """
    pass


def rollback(url, db, uid, password):
    """Revierte los cambios del script (si es reversible)."""
    pass


if __name__ == "__main__":
    import sys
    # ... argumentos CLI ...
```

---

## Documentación de Scripts (README.md)

El `README.md` del directorio `migration_scripts/` debe mantenerse actualizado:

```markdown
# Scripts de Migración — [Cliente] v[XX] → v[YY]

## Orden de ejecución

### Pre-migrate (ejecutar ANTES del upgrade)
| # | Script | Descripción | Modelos | Reversible | Estado |
|---|--------|-------------|---------|------------|--------|
| 1 | 001_rename_old_fields.py | Renombra campos x_studio_ | res.partner | Sí | Probado |

### Post-migrate (ejecutar DESPUÉS del upgrade)
| # | Script | Descripción | Modelos | Reversible | Estado |
|---|--------|-------------|---------|------------|--------|
| 1 | 001_remap_partner_fields.py | Mapea campos custom | res.partner | Sí | Probado |

### Cleanup
...

### Validación
...
```

---

## Reglas de Ejecución

### Idempotencia obligatoria
Ejecutar 2 veces el mismo script NO debe producir errores ni duplicar datos. Verificar SIEMPRE si la acción ya se ejecutó antes de ejecutarla.

### Modo dry-run obligatorio
Todo script debe poder ejecutarse en modo simulación para verificar qué haría sin modificar datos.

### Control de versiones
No mantener scripts "sueltos" en máquinas locales. Todo en `migration_scripts/` dentro del repositorio.

### Documentar cada ejecución

```
EJECUCIÓN DE SCRIPTS — [Entorno] — [Fecha]

Entorno: [testing / rehearsal / producción]
BD: [nombre_bd]
Ejecutado por: [Nombre]

Pre-migrate:
  001_rename_old_fields.py     → OK (15 registros afectados) — 0.3s
  002_clean_orphan_records.py  → OK (42 registros eliminados) — 1.2s

[Upgrade Odoo ejecutado]

Post-migrate:
  001_remap_partner_fields.py  → OK (230 registros mapeados) — 2.5s

Cleanup:
  001_remove_deprecated_models.py → OK — 0.8s

Validación:
  001_count_records.py → OK (deltas dentro de tolerancia)
  002_check_balances.py → OK (saldos cuadrados)

RESULTADO GLOBAL: OK
TIEMPO TOTAL SCRIPTS: 4.8s
```

---

## Reglas Críticas

1. **NUNCA** ejecutar scripts directamente en producción sin haberlos probado antes en al menos un entorno de testing.
2. **NUNCA** modificar un script después del rehearsal salvo por bugs críticos — en ese caso re-ejecutar el rehearsal.
3. **TODO** script que toque datos contables (account.move, account.move.line) requiere validación del BC y aprobación del TL.
4. Los scripts de **validación se ejecutan SIEMPRE** — no son opcionales.
5. **Guardar los logs de ejecución** en el repositorio o en la tarea Odoo del proyecto.
