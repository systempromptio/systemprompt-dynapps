# Troubleshooting - Odoo Studio Migration

Guía completa de problemas y soluciones para migración a Odoo Studio V18/V19.

---

## Errores de Conexión

### Error: "404 Not Found" en /jsonrpc

**Causa:** XML-RPC/JSON-RPC no está habilitado en Odoo 18.

**Solución:** Usar autenticación por sesión web o habilitar en `odoo.conf`:
```
xmlrpc = True
xmlrpc_port = 18000
```

### Error: "Access Denied" con API Key

**Causa:** Módulos de seguridad personalizados (como `indaws_security`) pueden bloquear API keys.

**Solución:** Usar contraseña real del usuario en lugar de API key, o configurar el módulo de seguridad para permitir API keys.

### Error: "Invalid credentials"

**Causa:** Credenciales incorrectas o usuario no existe.

**Solución:**
1. Verificar URL, base de datos, usuario
2. Probar con contraseña real (no API key)
3. Verificar que el usuario tenga permisos de administrador

---

## Errores de Campos

### Problema: Campo no visible en vistas

**Causa:** Campo creado pero no añadido al arch de la vista.

**Solución:** Crear o heredar vista con el campo:
```xml
<xpath expr="//group[@name='general']" position="inside">
    <field name="x_new_field"/>
</xpath>
```

### Problema: "Invalid field 'x_field' on model 'xxx'"

**Causa:** El campo no existe en el modelo o no se ha creado correctamente.

**Solución:**
1. Verificar que el campo tiene `state='manual'`
2. Verificar que model_id es correcto
3. Recargar Odoo para actualizar caché

### Problema: "Invalid field 'default' on model 'ir.model.fields'"

**Causa:** En V18, los campos `default`, `copy`, `translate` fueron eliminados.

**Solución:** No usar estos campos al crear campos:
```python
# ❌ INCORRECTO
{'name': 'x_campo', 'default': 'False', 'copy': False}

# ✅ CORRECTO
{'name': 'x_campo', 'state': 'manual'}
```

### Problema: TypeError con resultados de create()

**Causa:** En V18, `create()` puede devolver `[id]` en lugar de `id`.

**Solución:** Usar función `extract_id()`:
```python
def extract_id(result):
    if isinstance(result, list):
        return result[0] if result else False
    return result

field_id = extract_id(execute('ir.model.fields', 'create', [{...}]))
```

---

## Errores de Vistas

### Problema: Vista tipo 'tree' no encontrada

**Causa:** V18 usa 'list' en lugar de 'tree'.

**Solución:** Cambiar en todas partes:
```xml
<!-- ❌ INCORRECTO -->
<tree string="Model">
    <field name="name"/>
</tree>

<!-- ✅ CORRECTO -->
<list string="Model">
    <field name="name"/>
</list>
```

```python
# En acciones de ventana
'view_mode': 'list,form'  # NO 'tree,form'
```

### Problema: Error con attrs en V18

**Causa:** El atributo `attrs` está obsoleto en V17+.

**Solución:** Usar atributos directos:
```xml
<!-- ❌ V13-V16 -->
<field name="x_state" attrs="{'invisible': [('x_state', '=', False)]}"/>

<!-- ✅ V17+ -->
<field name="x_state" invisible="not x_state"/>
```

### Problema: Columna no visible en lista

**Causa:** El campo tiene `optional="hide"` (oculto por defecto).

**Solución:**
```xml
<field name="x_campo" optional="show" string="Mi Campo"/>
```

### Problema: Vista heredada no aplica cambios

**Causa:** El xpath busca un campo que está en otra vista heredada, no en la vista padre.

**Solución:** Heredar de la vista que CONTIENE el campo:
```python
# Buscar qué vista añade el campo
views = execute('ir.ui.view', 'search_read',
    [[('model', '=', 'stock.move'), ('type', '=', 'list')]],
    {'fields': ['name', 'arch']})

for v in views:
    if 'product_id' in v['arch']:
        parent_view_id = v['id']
        break
```

### Problema: Botón no aparece en lista embebida

**Causa:** La lista está embebida en el formulario y no hereda de vistas externas.

**Solución:** Heredar del formulario contenedor:
```xml
<!-- Heredar de stock.picking.form, no de stock.move.list -->
<xpath expr="//list/field[@name='product_id']" position="after">
    <field name="x_custom_field"/>
</xpath>
```

---

## Errores de Server Actions

### Problema: Import prohibido

**Causa:** La sintaxis `from x import y` está bloqueada en Server Actions.

**Solución:** Usar variables globales directamente:
```python
# ❌ PROHIBIDO
from datetime import timedelta
from dateutil.relativedelta import relativedelta

# ✅ PERMITIDO
relativedelta = dateutil.relativedelta.relativedelta
now = datetime.datetime.now()
```

### Problema: NameError: 'hasattr' no está definido

**Causa:** `hasattr`, `getattr`, `setattr` NO están disponibles.

**Solución:** Usar try/except:
```python
# ❌ INCORRECTO
if hasattr(partner, 'agent_ids') and partner.agent_ids:
    agent_id = partner.agent_ids[0].id

# ✅ CORRECTO
agent_id = False
try:
    if partner.agent_ids:
        agent_id = partner.agent_ids[0].id
except Exception:
    pass
```

### Problema: relativedelta no funciona

**Causa:** Intentando importar relativedelta.

**Solución:**
```python
# ✅ CORRECTO
relativedelta = dateutil.relativedelta.relativedelta
date = date - relativedelta(years=1)
```

### Problema: "forbidden opcode(s) in ... STORE_ATTR"

**Causa:** V18 no permite asignación directa con `.` en Server Actions.

**Solución:** Usar sintaxis de diccionario:
```python
# ❌ INCORRECTO
record.x_field = value

# ✅ CORRECTO
record['x_field'] = value
```

### Problema: Error de widget statusbar

**Causa:** `widget="statusbar"` requiere campo selection, no char.

**Solución:**
```xml
<!-- ❌ INCORRECTO si x_state es char -->
<field name="x_state" widget="statusbar"/>

<!-- ✅ CORRECTO para char fields -->
<field name="x_state" widget="badge"/>
```

---

## Errores de Automated Actions

### Problema: Automated Action no se dispara

**Causa:** Faltan campos requeridos en V18.

**Solución:**
```python
# V18 requiere AMBOS:
'on_change_field_ids': [(6, 0, [trigger_field_id])],
'action_server_ids': [(6, 0, [server_action_id])],
```

### Problema: "Invalid field 'trigger_fields' on model 'base.automation'"

**Causa:** En V18, el campo es `on_change_field_ids`, no `trigger_fields`.

**Solución:**
```python
# ❌ INCORRECTO
'trigger_fields': [(6, 0, [field_id])]

# ✅ CORRECTO
'on_change_field_ids': [(6, 0, [field_id])]
```

---

## Errores de Stock V18

### Problema: "Invalid field 'quantity_done' on model 'stock.move'"

**Causa:** En V18, `quantity_done` ya no existe en `stock.move`.

**Solución:** Usar `stock.move.line`:
```python
# ❌ V13-V17
moves = env['stock.move'].search_read([...], ['quantity_done'])

# ✅ V18
move_lines = env['stock.move.line'].search_read([
    ('move_id.production_id', '!=', False),
    ('state', '=', 'done'),
], ['product_id', 'quantity'])
```

### Problema: "Invalid field 'bom_ids' on model 'product.product'"

**Causa:** El módulo MRP no está instalado.

**Solución:**
```python
# Verificar e instalar MRP
module = env['ir.module.module'].search([('name', '=', 'mrp')])
if module and module.state != 'installed':
    module.button_immediate_install()
```

---

## Errores de Menús

### Problema: Menú creado en ubicación incorrecta

**Causa:** No se encontró el menú padre correcto.

**Solución:** Buscar por jerarquía de padres:
```python
# Buscar menú por jerarquía
for mid in reporting_menus:
    data = execute('ir.ui.menu', 'read', [mid], ['parent_id'])
    if data and data[0].get('parent_id'):
        parent_data = execute('ir.ui.menu', 'read',
            [data[0]['parent_id'][0]], ['name'])
        if parent_data and 'sale' in parent_data[0]['name'].lower():
            correct_parent_id = mid
            break
```

### Problema: Menús duplicados

**Causa:** Script ejecutado múltiples veces.

**Solución:** Verificar si existe antes de crear:
```python
existing = execute('ir.ui.menu', 'search',
    [[('name', '=', 'Mi Menú'), ('parent_id', '=', parent_id)]])
if not existing:
    execute('ir.ui.menu', 'create', {...})
```

---

## Errores de Exportación Studio

### Problema: Studio no exporta campos creados vía script

**Causa:** Los campos/modelos no tienen XML ID con `studio: True`.

**Solución:**
```python
execute('ir.model.data', 'create', [{
    'name': f'field_{field_name}_{uuid.uuid4().hex[:8]}',
    'module': 'studio_customization',
    'model': 'ir.model.fields',
    'res_id': field_id,
    'studio': True,
}])
```

### Problema: Warnings al exportar - "model_id has no XML id"

**Causa:** El modelo padre no está registrado en Studio.

**Solución:** Registrar el modelo ANTES que sus campos.

### Problema: Error "KeyError: x_studio_field_name"

**Causa:** Se registraron XML IDs para campos related de Studio (state: base).

**Solución:** Solo registrar campos con `state: manual`:
```python
# ✅ CORRECTO
fields = execute('ir.model.fields', 'search',
    [[('name', 'like', 'x_%'), ('state', '=', 'manual')]])
```

---

## Errores de Campos V18

### Problema: Campo 'is_customer' inválido

**Causa:** V18 eliminó `is_customer` e `is_supplier` de res.partner.

**Solución:**
```python
# ❌ V13-V17
domain = [('is_customer', '=', True)]

# ✅ V18
domain = [('customer_rank', '>', 0)]
```

---

## Errores de Wizards

### Problema: Wizard muestra resultados One2many en el formulario

**Causa:** Se añadieron páginas de resultados al wizard.

**Solución:** Los resultados deben mostrarse vía acción de ventana retornada:
```python
action = {
    'name': 'Resultados',
    'type': 'ir.actions.act_window',
    'res_model': 'x_result_model',
    'view_mode': 'list',
    'target': 'current',
    'domain': [('id', 'in', result_ids)],
}
```

### Problema: Wizard con botón guardar innecesario

**Solución:** Eliminar botón guardar. Solo botones de acción y descartar:
```xml
<footer>
    <button name="{sa_action_id}" type="action" string="Generar" class="btn-primary"/>
    <button string="Descartar" special="cancel"/>
</footer>
```

---

## Errores de Scheduled Actions

### Problema: "Invalid field 'numbercall'" en ir.cron

**Causa:** Campos `numbercall` y `doall` eliminados en V18.

**Solución:**
```python
# ✅ CORRECTO para V18
execute('ir.cron', 'create', [{
    'name': 'Limpiar Datos',
    'model_id': model_id,
    'state': 'code',
    'code': '...',
    'interval_number': 1,
    'interval_type': 'days',
    'active': True,
}])
```

---

## Errores de with_delay()

### Problema: with_delay() no funciona en Server Actions

**Causa:** `with_delay()` requiere un método que exista en el modelo.

**Solución:** Usar patrón de campo disparador:
1. Crear campo `x_compute_trigger` (boolean)
2. Crear Automated Action que detecta el trigger
3. Botón ejecuta `with_delay().write({'x_compute_trigger': True})`
4. Automated Action ejecuta el cálculo real

Ver documentación completa en SKILL.md sección "Fase 2b".

---

## Errores de Exportación Studio (CRÍTICO)

### Problema: Studio no exporta campos/modelos creados vía script

**Causa:** Los elementos NO cumplen los 3 requisitos de exportación.

**Requisitos CRÍTICOS en ir.model.data:**
| Campo | Valor Requerido | Consecuencia |
|-------|-----------------|--------------|
| `module` | `studio_customization` | NO usar `studio` |
| `studio` | `True` | **CRÍTICO** - Sin esto NO se exporta |
| `noupdate` | `False` | Si es True, NO se exporta |

**Diagnóstico:**
```python
# Verificar un elemento específico
result = studio.verify_exportable(field_id, 'ir.model.fields')
print(f"Exportable: {result['is_exportable']}")
print(f"Issues: {result['issues']}")

# Verificar TODOS los elementos
status = studio.verify_all_exportable()
print(f"Exportables: {status['exportable']}/{status['total']}")
for issue in status['issues']:
    print(f"  ❌ {issue['xml_id']}: {issue['issues']}")
```

**Solución - Corregir un elemento:**
```python
# Corregir elemento específico
studio.fix_exportable(field_id, 'ir.model.fields')
```

**Solución - Corregir TODOS:**
```python
# Corregir todos los elementos no exportables
results = studio.fix_all_exportable()
print(f"Corregidos por modelo: {results}")
```

### Problema: Elemento tiene prefijo `studio.` en lugar de `studio_customization.`

**Causa:** El registro en ir.model.data tiene `module='studio'`.

**Diagnóstico:**
```sql
-- Buscar registros con módulo incorrecto
SELECT name, module, model, res_id
FROM ir_model_data
WHERE module = 'studio' AND model LIKE 'ir.model%';
```

**Solución:**
```python
# Actualizar módulo
ops.write('ir.model.data', [xml_id], {
    'module': 'studio_customization',
    'studio': True,
    'noupdate': False,
})
```

### Problema: ir.model.access no se exporta

**Causa:** Los accesos creados manualmente tienen `studio=False`.

**Diagnóstico:**
```python
# Verificar accesos de un modelo
accesses = ops.search_read('ir.model.access', [
    ('model_id.model', '=', 'x_custom_model')
], ['name', 'group_id'])

for acc in accesses:
    xml = ops.search_read('ir.model.data', [
        ('model', '=', 'ir.model.access'),
        ('res_id', '=', acc['id'])
    ], ['module', 'studio', 'noupdate'])

    if not xml or not xml[0].get('studio'):
        print(f"❌ {acc['name']}: NO exportable")
```

**Solución:**
```python
# Registrar acceso en Studio
studio.register_access_in_studio(access_id, 'x_custom_model')
```

### Problema: Acciones de ventana no se exportan

**Causa:** Las ir.actions.act_window no tienen XML ID registrado.

**Diagnóstico:**
```python
# Buscar acciones sin XML ID
actions = ops.search_read('ir.actions.act_window', [
    ('res_model', 'like', 'x_%')
], ['name', 'res_model'])

for action in actions:
    xml = ops.search_read('ir.model.data', [
        ('model', '=', 'ir.actions.act_window'),
        ('res_id', '=', action['id'])
    ], ['name', 'studio'])

    if not xml:
        print(f"❌ {action['name']}: SIN XML ID")
```

**Solución:**
```python
# Registrar acción en Studio
studio.register_action_in_studio(action_id, 'act_window', 'mi_accion')
```

### Problema: Automated Action no tiene Server Action asociada

**Causa:** El campo `action_server_ids` está vacío.

**Diagnóstico:**
```python
# Buscar AAs sin server action
aas = ops.search_read('base.automation', [], ['name', 'action_server_ids'])
for aa in aas:
    if not aa['action_server_ids']:
        print(f"❌ {aa['name']}: SIN SERVER ACTION")
```

**Solución:**
```python
# Asociar Server Action a Automated Action
ops.write('base.automation', [aa_id], {
    'action_server_ids': [(6, 0, [sa_id])]
})
```

### Checklist de Verificación Pre-Exportación

Antes de exportar desde Studio, verificar:

```python
# Script de verificación completa
status = studio.verify_all_exportable()

print("=" * 60)
print("REPORTE DE EXPORTACIÓN STUDIO")
print("=" * 60)
print(f"Total elementos: {status['total']}")
print(f"Exportables: {status['exportable']}")
print(f"No exportables: {status['not_exportable']}")
print()

if status['not_exportable'] > 0:
    print("Por modelo:")
    for model, counts in status['by_model'].items():
        if counts['not_exportable'] > 0:
            print(f"  {model}: {counts['not_exportable']} problemas")

    print("\nProblemas detallados:")
    for issue in status['issues'][:10]:  # Mostrar primeros 10
        print(f"  ❌ {issue['xml_id']}")
        print(f"     Issues: {', '.join(issue['issues'])}")
else:
    print("✅ Todos los elementos son exportables")
```

### Script de Corrección Automática

```python
# Corregir todos los elementos no exportables
results = studio.fix_all_exportable()

if results:
    print("Correcciones realizadas:")
    for model, count in results.items():
        print(f"  {model}: {count} elementos")
else:
    print("No se requirieron correcciones")
```

---

## Errores de Botones con IDs de Acción en Vistas

### Problema: Exportación no portable - botones con IDs numéricos

**Causa:** Cuando se crean vistas con botones `type="action"`, el script interpola el ID
numérico de la acción (ej: `name="738"`) en el `arch_db`. El código de exportación de
Studio serializa el arch tal cual, produciendo XML no portable entre bases de datos.

**Síntoma:** Al importar el ZIP de Studio en otra base de datos, los botones no funcionan
o ejecutan una acción incorrecta, porque el ID numérico no corresponde a la misma
acción en el destino.

**Raíz del problema:** Studio UI intenta usar XML IDs (`action.xml_id or str(actionId)`)
en `web_studio/controllers/main.py:1153`, `WebStudioController._operation_add_button_action`, pero los scripts de migración no hacen lo mismo.

**Diagnóstico:**
```python
affected = studio.find_button_numeric_action_ids()

print(f"Vistas con IDs numéricos en botones: {len(affected)}")
for v in affected:
    print(f"  Vista: {v['view_name']} (ID: {v['view_id']})")
    print(f"  Modelo: {v['model']}")
    print(f"  IDs numéricos: {v['numeric_ids']}")
```

**Solución - Corregir vistas existentes:**
```python
result = studio.fix_button_action_ids()

print(f"Vistas corregidas: {result['views_fixed']}")
print(f"Botones corregidos: {result['buttons_fixed']}")

if result['errors']:
    for err in result['errors']:
        print(f"  ERROR: {err}")
```

**Solución - Prevenir en scripts nuevos:**
```python
# Opción 1: Obtener XML ID antes de crear la vista
sa_id = studio.create_server_action('Mi Acción', model_id, code)
studio.register_in_studio(sa_id, 'ir.actions.server', 'mi_accion')
xml_id = studio.get_action_xml_id(sa_id, 'server')

arch = f'''<form>
    <footer>
        <button name="{xml_id}" type="action" string="Ejecutar" class="btn-primary"/>
    </footer>
</form>'''

# Opción 2: Resolver automáticamente antes de crear vista
arch = f'''<form>
    <footer>
        <button name="{sa_id}" type="action" string="Ejecutar" class="btn-primary"/>
    </footer>
</form>'''
portable_arch = studio.resolve_arch_action_ids(arch)
view_id = studio.create_view('mi.vista', 'mi.modelo', portable_arch)
```

**IMPORTANTE:** La acción debe estar registrada en Studio (con XML ID) ANTES
de poder obtener su XML ID. El orden correcto es:
1. Crear la acción (`create_server_action`)
2. Registrar en Studio (`register_in_studio`)
3. Obtener XML ID (`get_action_xml_id`)
4. Usar XML ID en el arch de la vista
