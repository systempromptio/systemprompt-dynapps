# Ejemplos de Código - Odoo Studio Migration

Ejemplos prácticos para migraciones comunes.

---

## 1. Conexión Básica

```python
import sys
sys.path.insert(0, '/path/to/.claude/skills/odoo-studio-migration/scripts')

from connection import OdooConnection
from operations import OdooOperations
from studio import StudioMigration

# Configurar conexión
conn = OdooConnection(
    url="http://localhost:18000",
    db="mi_base_datos",
    user="admin",
    key="password_o_api_key"
)

# Autenticar (detecta protocolo automáticamente)
if conn.authenticate():
    print(f"✓ Conectado: UID={conn.uid}, Version={conn.version}")

    # Crear operaciones
    ops = OdooOperations(conn)
    studio = StudioMigration(conn)
```

---

## 2. Crear Campo Many2one

```python
# Obtener modelo destino
model_id = studio.get_model_id('mrp.production')

# Crear campo many2one
field_id = studio.create_field(
    model_id=model_id,
    name='x_brand_id',
    field_type='many2one',
    label='Marca',
    relation='product.brand',
    help='Marca del producto para esta orden de producción'
)

# Registrar en Studio
studio.register_field_in_studio(field_id, 'x_brand_id')

print(f"Campo creado: {field_id}")
```

---

## 3. Crear Campo Boolean

```python
model_id = studio.get_model_id('sale.order')

field_id = studio.create_field(
    model_id=model_id,
    name='x_is_gift',
    field_type='boolean',
    label='Es Regalo',
    help='Marcar si este pedido es un regalo'
)

studio.register_field_in_studio(field_id, 'x_is_gift')
```

---

## 4. Crear Campo Selection

```python
model_id = studio.get_model_id('stock.picking')

field_id = studio.create_field(
    model_id=model_id,
    name='x_priority_level',
    field_type='selection',
    label='Nivel Prioridad',
    selection='[("low", "Baja"), ("medium", "Media"), ("high", "Alta")]'
)

studio.register_field_in_studio(field_id, 'x_priority_level')
```

---

## 5. Crear Vista Heredada

```python
# Obtener vista padre
parent_view_id = studio.get_view_id('mrp.mrp_production_form_view')

# Arquitectura de la vista
arch = '''<data>
    <xpath expr="//field[@name='product_id']" position="before">
        <field name="x_brand_id"
               string="Marca"
               options="{'no_create': True, 'no_open': True}"/>
    </xpath>
</data>'''

# Crear vista
view_id = studio.create_inherited_view(
    'mrp.production.form.brand',
    'mrp.production',
    parent_view_id,
    arch
)

# Registrar en Studio
studio.register_view_in_studio(view_id, 'mrp.production.form.brand')
```

---

## 6. Crear Server Action (Botón)

```python
model_id = studio.get_model_id('sale.order')

# Código de la acción
code = '''
# Validar y procesar pedido
if record.state == 'draft':
    # Guardar valores
    record['x_processed'] = True
    record['x_processed_date'] = datetime.date.today()

    # Confirmar pedido
    record.action_confirm()

    log(f"Pedido {record.name} procesado correctamente")
'''

sa_id = studio.create_server_action(
    'Procesar Pedido',
    model_id,
    code,
    binding_type='action'  # Aparece en dropdown de acciones
)

studio.register_in_studio(sa_id, 'ir.actions.server', 'sa_process_order')
```

---

## 7. Crear Automated Action

```python
model_id = studio.get_model_id('mrp.production')

# Código a ejecutar
code = '''
# Sincronizar cuenta analítica desde la marca
if record.x_brand_id and record.state == 'draft':
    brand = record.x_brand_id
    if brand.x_analytic_account_id:
        record['x_analytic_account_id'] = brand.x_analytic_account_id.id
'''

# Crear Server Action
sa_id = studio.create_server_action(
    'AA - Sincronizar Analítica',
    model_id,
    code
)

# Obtener campo trigger
trigger_field = ops.search('ir.model.fields', [
    ('model_id', '=', model_id),
    ('name', '=', 'x_brand_id')
], limit=1)

# Crear Automated Action
aa_id = studio.create_automated_action(
    'Sincronizar Cuenta Analítica',
    model_id,
    'on_write',
    sa_id,
    filter_domain="[('x_brand_id', '!=', False), ('state', '=', 'draft')]",
    trigger_field_ids=[trigger_field[0]] if trigger_field else None
)

studio.register_in_studio(aa_id, 'base.automation', 'aa_sync_analytic')
```

---

## 8. Crear Menú

```python
# Buscar menú padre (ej: Manufacturing > Configuration)
parent_id = studio.find_menu_by_name('Configuration', 'Manufacturing')

# Crear acción de ventana
action_id = studio.create_window_action(
    'Mis Productos',
    'product.product',
    'list,form',
    domain="[('x_is_custom', '=', True)]"
)

# Crear menú
menu_id = studio.create_menu(
    'Productos Personalizados',
    parent_id,
    action_id,
    sequence=50
)

studio.register_in_studio(menu_id, 'ir.ui.menu', 'menu_custom_products')
```

---

## 9. Script Completo de Migración

```python
#!/usr/bin/env python3
"""Migración: Añadir campo Marca a Órdenes de Producción"""

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

if not conn.authenticate():
    sys.exit(1)

studio = StudioMigration(conn)

# 1. Crear campo en product.brand (para guardar cuenta analítica)
brand_model_id = studio.get_model_id('product.brand')
analytic_field_id = studio.create_field(
    model_id=brand_model_id,
    name='x_analytic_account_id',
    field_type='many2one',
    label='Cuenta Analítica',
    relation='account.analytic.account'
)
studio.register_field_in_studio(analytic_field_id, 'x_analytic_account_id')

# 2. Crear campo en mrp.production
mrp_model_id = studio.get_model_id('mrp.production')
brand_field_id = studio.create_field(
    model_id=mrp_model_id,
    name='x_brand_id',
    field_type='many2one',
    label='Marca',
    relation='product.brand'
)
studio.register_field_in_studio(brand_field_id, 'x_brand_id')

# 3. Crear campo analítica en mrp.production
analytic_mrp_field_id = studio.create_field(
    model_id=mrp_model_id,
    name='x_analytic_account_id',
    field_type='many2one',
    label='Cuenta Analítica',
    relation='account.analytic.account'
)
studio.register_field_in_studio(analytic_mrp_field_id, 'x_analytic_account_id')

# 4. Crear Server Action
code = '''
if record.x_brand_id and record.state == 'draft':
    if record.x_brand_id.x_analytic_account_id:
        record['x_analytic_account_id'] = record.x_brand_id.x_analytic_account_id.id
'''
sa_id = studio.create_server_action(
    'AA - Sincronizar Analítica',
    mrp_model_id,
    code
)

# 5. Crear Automated Action
trigger_field = studio.ops.search('ir.model.fields', [
    ('model_id', '=', mrp_model_id),
    ('name', '=', 'x_brand_id')
], limit=1)

aa_id = studio.create_automated_action(
    'Sincronizar Analítica desde Marca',
    mrp_model_id,
    'on_write',
    sa_id,
    trigger_field_ids=[trigger_field[0]] if trigger_field else None
)

# 6. Crear vista
parent_view = studio.get_view_id('mrp.mrp_production_form_view')
arch = '''<data>
    <xpath expr="//field[@name='product_id']" position="before">
        <field name="x_brand_id" string="Marca" options="{'no_create': True}"/>
        <field name="x_analytic_account_id" string="Cuenta Analítica"
               readonly="state != 'draft'"/>
    </xpath>
</data>'''
view_id = studio.create_inherited_view(
    'mrp.production.form.brand',
    'mrp.production',
    parent_view,
    arch
)
studio.register_view_in_studio(view_id, 'mrp.production.form.brand')

print("✅ Migración completada")
```

---

## 10. Patrón Wizard No-Transient

```python
# Studio NO puede crear TransientModel, usar modelo normal + cron

# 1. Crear modelo wizard
wizard_model_id = studio.create_model(
    'Wizard Actualización',
    'x_update_wizard'
)
studio.register_model_in_studio(wizard_model_id, 'x_update_wizard')

# 2. Crear campos
studio.create_field(wizard_model_id, 'x_note', 'text', 'Nota')
studio.create_field(wizard_model_id, 'x_move_id', 'many2one',
                    relation='stock.move', label='Movimiento')

# 3. Server Action para actualizar
code = '''
if record.x_move_id:
    record.x_move_id['x_mrp_note'] = record.x_note
action = {'type': 'ir.actions.act_window_close'}
'''
sa_update_id = studio.create_server_action(
    'Actualizar Nota',
    wizard_model_id,
    code
)

# 3.5 Registrar y obtener XML ID (IMPORTANTE para portabilidad)
studio.register_in_studio(sa_update_id, 'ir.actions.server', 'sa_update_wizard')
sa_xml_id = studio.get_action_xml_id(sa_update_id, 'server')

# 4. Vista del wizard (usar XML ID, NO ID numérico)
arch = f'''<form string="Actualizar Nota">
    <group>
        <field name="x_note" nolabel="1"/>
    </group>
    <footer>
        <button name="{sa_xml_id}" type="action" string="Actualizar" class="btn-primary"/>
        <button string="Cancelar" special="cancel"/>
    </footer>
</form>'''
studio.create_view('x_update_wizard.form', 'x_update_wizard', arch, 'form')

# 5. Scheduled Action para limpiar wizards antiguos
cron_code = '''
cutoff = datetime.datetime.now() - datetime.timedelta(days=1)
old = env['x_update_wizard'].search([('create_date', '<', cutoff)])
if old:
    count = len(old)
    old.unlink()
    log(f'Eliminados {count} wizards antiguos')
'''
studio.ops.create('ir.cron', {
    'name': 'Limpiar Wizards',
    'model_id': wizard_model_id,
    'state': 'code',
    'code': cron_code,
    'interval_number': 1,
    'interval_type': 'days',
    'active': True,
})
```

---

## 11. Verificar Estado de Exportación

```python
# Verificar si un campo es exportable
field_id = 1234  # ID del campo
result = studio.verify_exportable(field_id, 'ir.model.fields')

if result['is_exportable']:
    print(f"✅ Campo exportable: {result['xml_id']}")
else:
    print(f"❌ Campo NO exportable")
    for issue in result['issues']:
        print(f"   - {issue}")
```

---

## 12. Corregir Elemento No Exportable

```python
# Corregir un elemento específico
field_id = 1234

# Verificar y corregir
result = studio.verify_exportable(field_id, 'ir.model.fields')
if not result['is_exportable']:
    print(f"Corrigiendo: {result['issues']}")
    studio.fix_exportable(field_id, 'ir.model.fields')
    print("✅ Corregido")
```

---

## 13. Verificar Todos los Elementos Studio

```python
# Verificar estado de exportación de TODOS los elementos
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
    for issue in status['issues'][:10]:
        print(f"  ❌ {issue['xml_id']}")
        print(f"     Issues: {', '.join(issue['issues'])}")
else:
    print("✅ Todos los elementos son exportables")
```

---

## 14. Corregir Todos los Elementos No Exportables

```python
# Corregir automáticamente TODOS los elementos con problemas
print("Corrigiendo elementos no exportables...")
results = studio.fix_all_exportable()

if results:
    print("Correcciones realizadas:")
    total = 0
    for model, count in results.items():
        print(f"  {model}: {count} elementos")
        total += count
    print(f"\nTotal corregidos: {total}")

    # Verificar nuevamente
    status = studio.verify_all_exportable()
    print(f"\nEstado final: {status['exportable']}/{status['total']} exportables")
else:
    print("No se requirieron correcciones")
```

---

## 15. Registrar Acción de Ventana para Exportación

```python
# Crear acción de ventana
action_id = studio.create_window_action(
    'Mi Acción',
    'x_custom_model',
    'list,form'
)

# Registrar en Studio para exportación (CRÍTICO)
studio.register_action_in_studio(action_id, 'act_window', 'mi_accion')
print(f"✅ Acción registrada para exportación")
```

---

## 16. Registrar Acceso de Modelo para Exportación

```python
# Crear acceso con grupo
group_user_id = ops.search('res.groups', [('name', '=', 'Internal User')], limit=1)[0]

access_id = studio.create_access_right(
    model_id=model_id,
    name='x_custom_model_user',
    group_id=group_user_id,
    perm_read=True,
    perm_write=True,
    perm_create=True,
    perm_unlink=False
)

# Registrar en Studio para exportación (CRÍTICO)
studio.register_access_in_studio(access_id, 'x_custom_model')
print(f"Acceso registrado para exportación")
```

---

## 17. Corregir IDs Numéricos en Botones de Vistas

```python
# Detectar y corregir botones que usan IDs numéricos en lugar de XML IDs.
# Los IDs numéricos no son portables entre bases de datos.

# Paso 1: Detectar vistas afectadas
affected = studio.find_button_numeric_action_ids()

if not affected:
    print("No hay vistas con IDs numéricos en botones")
else:
    print(f"Encontradas {len(affected)} vistas afectadas:")
    for v in affected:
        print(f"  {v['view_name']}: IDs {v['numeric_ids']}")

    # Paso 2: Corregir automáticamente
    result = studio.fix_button_action_ids()

    print(f"\nVistas corregidas: {result['views_fixed']}")
    print(f"Botones corregidos: {result['buttons_fixed']}")

    if result['errors']:
        print("\nErrores (acción sin XML ID):")
        for err in result['errors']:
            print(f"  {err}")

    # Paso 3: Mostrar detalle de reemplazos
    for detail in result['details']:
        print(f"\n  Vista: {detail['view_name']}")
        for old_id, new_xml_id in detail['replacements'].items():
            print(f"    {old_id} -> {new_xml_id}")
```

---

## 18. Crear Vista con Botón Portable (Prevención)

```python
# Crear una vista con botones que usen XML IDs desde el principio,
# evitando el problema de IDs numéricos no portables.

model_id = studio.get_model_id('x_update_wizard')

# 1. Crear Server Action
code = '''
if record.x_move_id:
    record.x_move_id['x_mrp_note'] = record.x_note
action = {'type': 'ir.actions.act_window_close'}
'''
sa_id = studio.create_server_action('Actualizar Nota', model_id, code)

# 2. Registrar en Studio ANTES de obtener XML ID
studio.register_in_studio(sa_id, 'ir.actions.server', 'sa_update_note')

# 3. Obtener XML ID
xml_id = studio.get_action_xml_id(sa_id, 'server')
print(f"XML ID de la acción: {xml_id}")

# 4. Usar XML ID en la vista (PORTABLE)
arch = f'''<form string="Actualizar Nota">
    <group>
        <field name="x_note" nolabel="1"/>
    </group>
    <footer>
        <button name="{xml_id}" type="action" string="Actualizar" class="btn-primary"/>
        <button string="Cancelar" special="cancel"/>
    </footer>
</form>'''

view_id = studio.create_view('x_update_wizard.form', 'x_update_wizard', arch, 'form')
studio.register_view_in_studio(view_id, 'x_update_wizard.form')
print(f"Vista creada con botón portable")
```
