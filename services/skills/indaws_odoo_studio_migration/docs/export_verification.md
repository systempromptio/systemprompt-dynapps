# Guía de Verificación de Exportación Studio

Esta guía explica cómo verificar y corregir problemas de exportación en módulos de Studio.

---

## Requisitos de Exportación

Para que Studio exporte un elemento, DEBE cumplir **3 requisitos** en `ir.model.data`:

| Campo | Valor Requerido | Consecuencia si incorrecto |
|-------|-----------------|------------------------|
| `module` | `studio_customization` | NO se exportará |
| `studio` | `True` | **NO se exportará** |
| `noupdate` | `False` | No se actualiza en importación |

---

## Scripts de Verificación

### Verificar Estado de Todos los Elementos

```python
import sys
sys.path.insert(0, '/path/to/.claude/skills/odoo-studio-migration/scripts')

from connection import OdooConnection
from studio import StudioMigration

# Conectar
conn = OdooConnection(
    url="http://localhost:18000",
    db="odoo_18",
    user="admin",
    key="admin"
)

if conn.authenticate():
    studio = StudioMigration(conn)

    # Verificar todos los elementos
    status = studio.verify_all_exportable()

    print("=" * 60)
    print("REPORTE DE EXPORTACIÓN STUDIO")
    print("=" * 60)
    print(f"Total elementos: {status['total']}")
    print(f"Exportables: {status['exportable']}")
    print(f"No exportables: {status['not_exportable']}")

    if status['not_exportable'] > 0:
        print("\nPor modelo:")
        for model, counts in status['by_model'].items():
            if counts['not_exportable'] > 0:
                print(f"  {model}:")
                print(f"    Exportables: {counts['exportable']}")
                print(f"    Problemas: {counts['not_exportable']}")

        print("\nDetalles de problemas:")
        for issue in status['issues'][:20]:  # Mostrar primeros 20
            print(f"\n  {issue['xml_id']}")
            print(f"    Modelo: {issue['model']}")
            print(f"    Issues: {', '.join(issue['issues'])}")
    else:
        print("\n✅ Todos los elementos son exportables")
```

### Verificar un Elemento Específico
```python
# Verificar un campo específico
field_id = 12345  # ID del campo
result = studio.verify_exportable(field_id, 'ir.model.fields')

print(f"\nCampo ID {field_id}:")
print(f"  Exportable: {result['is_exportable']}")
print(f"  XML ID: {result['xml_id']}")

if result['issues']:
    print(f"  Problemas: {result['issues']}")
```

---

## Scripts de Corrección

### Corregir Todos los Elementos No Exportables
```python
# Corregir automáticamente todos los elementos con problemas
results = studio.fix_all_exportable()

if results:
    print("Correcciones realizadas:")
    total = sum(results.values())
    print(f"Total elementos corregidos: {total}")

    for model, count in results.items():
        print(f"  {model}: {count} elementos")
else:
    print("✅ No se requirieron correcciones")
```

### Corregir un Elemento Específico
```python
# Corregir un campo específico
field_id = 12345
xml_id = studio.fix_exportable(field_id, 'ir.model.fields', 'mi_campo')

print(f"Campo corregido. XML ID: studio_customization.{xml_id}")
```

### Flujo Completo de Corrección
```python
# 1. Verificar estado actual
status = studio.verify_all_exportable()

if status['not_exportable'] > 0:
    print(f"Se encontraron {status['not_exportable']} elementos no exportables")

    # 2. Corregir
    results = studio.fix_all_exportable()

    # 3. Verificar nuevamente
    status = studio.verify_all_exportable()

    if status['not_exportable'] == 0:
        print("✅ Todos los elementos ahora son exportables")
    else:
        print(f"⚠️ Aún qued {status['not_exportable']} elementos con problemas")
```

---

## Diagnóstico Manual

### Consultas SQL Directas
```sql
-- Verificar elementos no exportables
SELECT
   imd.module,
    imd.name,
    imd.model,
    imd.studio,
    imd.noupdate
FROM ir_model_data imd
WHERE imd.module = 'studio_customization'
AND (imd.studio = false OR imd.noupdate = true);

-- Contar por modelo
SELECT
    imd.model,
    COUNT(*) as total,
    SUM(CASE WHEN studio = true AND noupdate = false THEN 1 ELSE 0 END) as exportable
FROM ir_model_data imd
WHERE imd.module = 'studio_customization'
GROUP BY imd.model;
```

### Verificar Acciones de Ventana
```python
# Las acciones de ventana también necesitan XML ID para exportarse
actions = ops.search_read(
    'ir.actions.act_window',
    [('res_model', 'like', 'x_%')],
    ['id', 'name', 'res_model']
)

for action in actions:
    # Verificar si tiene XML ID
    xml_id = ops.search('ir.model.data', [
        ('model', '=', 'ir.actions.act_window'),
        ('res_id', '=', action['id'])
    ])

    if not xml_id:
        print(f"⚠️ Acción sin XML ID: {action['name']} ({action['res_model']})")
        # Registrar
        studio.register_action_in_studio(action['id'], 'act_window', action['name'].replace(' ', '_').lower())
```

### Verificar Accesos de Modelo
```python
# Los accesos deben recomendación: tener grupo asignado
accesses = ops.search_read(
    'ir.model.access',
    [('name', 'like', 'access_x_%')],
    ['id', 'name', 'group_id', 'model_id']
)

for access in accesses:
    if not access['group_id']:
        print(f"⚠️ Acceso sin grupo: {access['name']}")
        # Asignar grupo Internal User (ID 1)
        ops.write('ir.model.access', [access['id']], {'group_id': 1})

        # Registrar en Studio si no tiene XML ID
        xml_id = ops.search('ir.model.data', [
            ('model', '=', 'ir.model.access'),
            ('res_id', '=', access['id']),
            ('module', '=', 'studio_customization')
        ])

        if not xml_id:
            studio.register_access_in_studio(access['id'], 'custom_model')
```

---

## Verificar Botones con IDs Numéricos de Acción

Los botones `type="action"` en vistas Studio deben usar XML IDs en el atributo `name`,
no IDs numéricos. Los IDs numéricos no son portables entre bases de datos.

```python
# Buscar vistas con botones que usan IDs numéricos
affected = studio.find_button_numeric_action_ids()

if affected:
    print(f"Encontradas {len(affected)} vistas con IDs numéricos:")
    for v in affected:
        print(f"  Vista: {v['view_name']} (ID: {v['view_id']})")
        for nid in v['numeric_ids']:
            xml_id = studio.get_action_xml_id(nid, 'server')
            if not xml_id:
                xml_id = studio.get_action_xml_id(nid, 'act_window')
            if xml_id:
                print(f"    ID {nid} -> {xml_id} (corregible)")
            else:
                print(f"    ID {nid} -> SIN XML ID (registrar primero)")

    # Corregir automáticamente
    result = studio.fix_button_action_ids()
    print(f"\nResultado: {result['views_fixed']} vistas, {result['buttons_fixed']} botones corregidos")
else:
    print("No hay botones con IDs numéricos")
```

---

## Checklist Pre-Exportación

- [ ] Todos los campos tienen `studio=True` en ir.model.data
- [ ] Todos los campos tienen `noupdate=False` en ir.model.data
- [ ] Todas las vistas tienen `studio=True`
- [ ] Todas las acciones de ventana tienen XML ID
- [ ] Todos los accesos tienen grupo asignado
- [ ] Las Automated Actions tienen server action asociada
- [ ] Los módulos `state='manual'` están registrados
- [ ] Los botones `type="action"` en vistas usan XML IDs, no IDs numéricos
