# Recetas de Scripts para Odoo Shell

## 1. Reasignar registros a otro usuario / equipo

```python
# Reasignar leads no asignados a un comercial concreto
DRY_RUN = True
DOMAIN = [('user_id', '=', False), ('stage_id.sequence', '<', 10)]
NEW_USER_ID = 42   # ID del comercial destino
BATCH_SIZE = 200

leads = env['crm.lead'].search(DOMAIN)
print(f"[INFO] Leads a reasignar: {len(leads)}")

for i in range(0, len(leads), BATCH_SIZE):
    batch = leads[i:i + BATCH_SIZE]
    with env.cr.savepoint():
        if not DRY_RUN:
            batch.write({'user_id': NEW_USER_ID})
    if not DRY_RUN:
        env.cr.commit()
    print(f"[OK] Batch {i//BATCH_SIZE + 1}: {len(batch)} leads")

if DRY_RUN:
    print("[DRY-RUN] Sin cambios.")
```

## 2. Forzar recompute de campos computados

```python
# Recomputar campo `amount_total` en pedidos de un periodo
DRY_RUN = True
DOMAIN = [('state', 'in', ['sale', 'done']), ('date_order', '>=', '2024-01-01')]
FIELD = 'amount_total'
BATCH_SIZE = 100

orders = env['sale.order'].search(DOMAIN)
print(f"[INFO] Pedidos a recomputar: {len(orders)}")

for i in range(0, len(orders), BATCH_SIZE):
    batch = orders[i:i + BATCH_SIZE]
    if not DRY_RUN:
        with env.cr.savepoint():
            # Invalidar caché fuerza recálculo en el próximo acceso
            batch.invalidate_recordset([FIELD])
            # Acceder al campo fuerza el recompute en ORM
            _ = batch.mapped(FIELD)
        env.cr.commit()
    print(f"[OK] Batch {i//BATCH_SIZE + 1}: {len(batch)} pedidos")

if DRY_RUN:
    print("[DRY-RUN] Sin recompute. Revisar counts.")
```

Para recompute directo sin lectura:
```python
# Método más agresivo: marca el campo para recompute y lo ejecuta.
# add_to_compute exige que FIELD sea store=True + compute (si no, AssertionError:
# "Cannot add to recompute no-store or no-computed field").
env.add_to_compute(env['sale.order']._fields[FIELD], orders)
env.flush_all()   # ejecuta los recomputes pendientes
if not DRY_RUN:
    env.cr.commit()
```

`Model.recompute()` NO existe en v17+ (solo v16) y rompe en runtime. Para forzar el
recálculo de un recordset usar `orders._recompute_recordset([FIELD])`, o tras
`add_to_compute` llamar `env.flush_all()`.

## 3. Corregir estados inválidos

```python
# Corregir facturas en estado 'draft' que tienen move_type = 'out_invoice'
# pero amount_total > 0 y sin líneas (datos corruptos)
DRY_RUN = True
DOMAIN = [
    ('state', '=', 'draft'),
    ('move_type', '=', 'out_invoice'),
    ('invoice_line_ids', '=', False),
    ('amount_total', '>', 0),
]
BATCH_SIZE = 50

invoices = env['account.move'].search(DOMAIN)
print(f"[INFO] Facturas corruptas encontradas: {len(invoices)}")

for inv in invoices:
    print(f"  - ID {inv.id}: {inv.name} | Total: {inv.amount_total}")

if not DRY_RUN:
    for i in range(0, len(invoices), BATCH_SIZE):
        batch = invoices[i:i + BATCH_SIZE]
        with env.cr.savepoint():
            # Cancelar y poner en borrador limpio
            batch.button_cancel()
        env.cr.commit()
    print(f"[OK] {len(invoices)} facturas canceladas.")
else:
    print("[DRY-RUN] Sin cambios.")
```

## 4. Set masivo de campo (actualización de metadatos)

```python
# Asignar categoría a todos los contactos de un país sin categoría
DRY_RUN = True
COUNTRY_ID = 68   # España
CATEGORY_ID = 5   # ID de la categoría destino
DOMAIN = [
    ('country_id', '=', COUNTRY_ID),
    ('category_id', 'not in', [CATEGORY_ID]),
    ('is_company', '=', True),
]
BATCH_SIZE = 500

partners = env['res.partner'].search(DOMAIN)
category = env['res.partner.category'].browse(CATEGORY_ID)
print(f"[INFO] Contactos a categorizar: {len(partners)} | Categoría: {category.name}")

for i in range(0, len(partners), BATCH_SIZE):
    batch = partners[i:i + BATCH_SIZE]
    with env.cr.savepoint():
        if not DRY_RUN:
            # Many2many: [(4, id)] añade sin eliminar los existentes
            batch.write({'category_id': [(4, CATEGORY_ID)]})
    if not DRY_RUN:
        env.cr.commit()
    print(f"[OK] Batch {i//BATCH_SIZE + 1}: {len(batch)} partners")

# Verificación
after = env['res.partner'].search_count(DOMAIN)
print(f"[VERIFY] Partners sin categoría tras ejecución: {after} (esperado: 0)")
if DRY_RUN:
    print("[DRY-RUN] Sin cambios.")
```

## 5. Limpiar registros huérfanos

```python
# Eliminar líneas de analítico sin move_line_id (huérfanas por bug)
DRY_RUN = True
# account.analytic.line sin journal item asociado
DOMAIN = [('move_line_id', '=', False), ('account_id', '!=', False)]
BATCH_SIZE = 200

lines = env['account.analytic.line'].with_context(active_test=False).search(DOMAIN)
print(f"[INFO] Líneas analíticas huérfanas: {len(lines)}")

# Mostrar muestra para revisión humana
for l in lines[:10]:
    print(f"  - ID {l.id}: {l.name} | Cuenta: {l.account_id.name} | Fecha: {l.date}")
if len(lines) > 10:
    print(f"  ... y {len(lines) - 10} más")

if not DRY_RUN:
    input("¿Confirmar eliminación? (Ctrl+C para cancelar, Enter para continuar) ")
    for i in range(0, len(lines), BATCH_SIZE):
        batch = lines[i:i + BATCH_SIZE]
        with env.cr.savepoint():
            batch.unlink()
        env.cr.commit()
        print(f"[OK] Eliminadas {len(batch)} líneas (batch {i//BATCH_SIZE + 1})")
    print(f"[DONE] {len(lines)} líneas huérfanas eliminadas.")
else:
    print("[DRY-RUN] Sin cambios. Revisar la muestra antes de ejecutar.")
```

## 6. Forzar recompute de precio en líneas de pedido

```python
# Actualizar precio de venta en líneas de pedidos borrador
# cuando el precio del producto ha cambiado
DRY_RUN = True
DOMAIN = [('order_id.state', '=', 'draft'), ('product_id.active', '=', True)]
BATCH_SIZE = 300

lines = env['sale.order.line'].search(DOMAIN)
print(f"[INFO] Líneas a actualizar precio: {len(lines)}")

changed = 0
for i in range(0, len(lines), BATCH_SIZE):
    batch = lines[i:i + BATCH_SIZE]
    for line in batch:
        new_price = line.product_id.lst_price
        if line.price_unit != new_price:
            if not DRY_RUN:
                with env.cr.savepoint():
                    line.write({'price_unit': new_price})
            changed += 1
    if not DRY_RUN:
        env.cr.commit()

print(f"[SUMMARY] Revisadas: {len(lines)} | Con precio cambiado: {changed}")
if DRY_RUN:
    print("[DRY-RUN] Sin cambios.")
```

## 7. Editar el arch XML de vistas con lxml

```python
from lxml import etree

DRY_RUN = True
views = env['ir.ui.view'].search([('type', '=', 'form'), ('arch_db', 'ilike', 'old_widget')])
print(f"[INFO] Vistas candidatas: {len(views)}")

changed = 0
for view in views:
    root = etree.fromstring(view.arch.encode('utf-8'))   # arch es str -> encode a bytes
    node = root.find(".//field[@name='foo']")
    if node is None:
        continue
    node.set('widget', 'new_widget')
    # otras utilidades lxml:
    #   parent.replace(node, sub)          mover/sustituir nodo (incluso entre árboles)
    #   etree.strip_tags(root, 't')        desenvuelve <t> conservando su contenido
    #   etree.strip_attributes(node, 'data-oe-id', 'data-oe-xpath')
    new_arch = etree.tostring(root, encoding='unicode')
    if not DRY_RUN:
        # ir.ui.view VALIDA la herencia al escribir: un xpath que no ancla lanza
        # excepción AQUÍ. savepoint por vista para no abortar el lote entero.
        try:
            with env.cr.savepoint():
                view.arch = new_arch
        except Exception as e:
            print(f"[WARN] vista {view.id}: {e}")
            continue
    changed += 1

if not DRY_RUN:
    env.cr.commit()
print(f"[SUMMARY] Vistas modificadas: {changed}")
```

Nota: nunca `env.cr.rollback()` a pelo en un script (tumba toda la transacción); usa `savepoint` por
elemento. Ancla los xpath por `hasclass()`/`name=`, no por posición.
