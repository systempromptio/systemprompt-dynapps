# Datos de Precondicion por Tipo de Flujo

Referencia de datos necesarios para ejecutar cada tipo de flujo. Utilizado por:
- `verify_data.py` (Modo B/C): verificacion programatica contra Odoo
- LLM (Modo A): documentacion de requisitos de datos

---

## Estructura de cada precondicion

```
Modelo Odoo: nombre del modelo
Campos criticos: campos que deben existir y tener valor
Minimo registros: cantidad minima necesaria
Dominio de busqueda: filtro Odoo para localizar registros validos
Verificacion: como saber que el dato es valido
Sugerencia si falta: que hacer si no existe
```

---

## 1. VENTAS (sale)

### 1.1. Datos comunes a todos los flujos de venta

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `res.partner` | name, vat, country_id, street | name | `[("customer_rank",">",0)]` | 1 | Crear cliente de prueba con `odoo-test-data-generator` modo demo |
| `product.product` | name, list_price, type, taxes_id | name, list_price | `[("sale_ok","=",True),("active","=",True)]` | 2 | Crear productos de prueba |
| `account.journal` | name, type, company_id | name | `[("type","=","sale")]` | 1 | Existe por defecto. Si no, revisar configuracion contable |
| `res.company` | name, vat, currency_id | name | `[("id","=",company_id)]` | 1 | Siempre existe |

### 1.2. Datos adicionales por variante

**SALE_ORDER_ADVANCE** (anticipo):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Producto anticipo | `product.product` con `type="service"` | Configurado en Ajustes > Ventas > Facturacion > Producto anticipo |
| Politica de facturacion | "Cantidades pedidas" en productos | Campo `invoice_policy` = `order` |

**SALE_ORDER_DISCOUNT** (descuento):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Descuentos habilitados | Ajustes > Ventas > Precios | `res.config.settings` campo `group_discount_per_so_line` = True |

**SALE_ORDER_SEND** (envio email):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Email cliente | `res.partner.email` | Campo email no vacio en el partner |
| Servidor de correo saliente | `ir.mail_server` configurado | Al menos 1 servidor activo |

### 1.3. Configuracion de empresa que afecta reversibilidad (ventas)

| Campo `res.company` | Impacto en flujos de venta |
|---------------------|---------------------------|
| `restrict_mode_hash_table` | Si True, las facturas creadas desde pedidos de venta tendran hash — no se pueden eliminar tras publicar |
| `check_account_audit_trail` | Si True, los asientos de las facturas de venta no se pueden eliminar |
| Localizacion fiscal (SII/TBAI/Verifactu) | Si instalado, publicar facturas desde ventas envia datos a AEAT. **Evitar flujo SALE_CREATE_INVOICE en produccion** |
| `fiscalyear_lock_date` / `hard_lock_date` | Facturas de venta deben usar fecha posterior al bloqueo |

---

## 2. COMPRAS (purchase)

### 2.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `res.partner` | name, supplier_rank | name | `[("supplier_rank",">",0)]` | 1 | Crear proveedor de prueba |
| `product.product` | name, standard_price | name | `[("purchase_ok","=",True),("active","=",True)]` | 2 | Crear productos comprables |
| `account.journal` | name, type | name | `[("type","=","purchase")]` | 1 | Existe por defecto |

### 2.2. Datos adicionales por variante

**PURCHASE_ORDER_APPROVAL** (aprobacion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Aprobacion habilitada | Ajustes > Compra | `res.config.settings` campo `purchase_order_approval` = True |
| Umbral de aprobacion | Importe minimo para aprobar | Campo `company_id.po_double_validation_amount` |

### 2.3. Configuracion de empresa que afecta reversibilidad (compras)

| Campo `res.company` | Impacto en flujos de compra |
|---------------------|----------------------------|
| `restrict_mode_hash_table` | Si True, las facturas de proveedor publicadas tendran hash — no se pueden eliminar |
| `check_account_audit_trail` | Si True, los asientos de facturas de proveedor no se pueden eliminar |
| `fiscalyear_lock_date` / `hard_lock_date` | Facturas de proveedor deben usar fecha posterior al bloqueo |

---

## 3. INVENTARIO (stock)

### 3.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `stock.warehouse` | name, code, lot_stock_id | name | `[("active","=",True)]` | 1 | Existe por defecto |
| `product.product` | name, type="product" | name, type | `[("type","=","product")]` | 1 | Productos almacenables |
| `stock.location` | name, usage | name | `[("usage","=","internal")]` | 2 | Para traslados internos |

### 3.2. Datos adicionales por flujo

**STOCK_RECEPTION** (recepcion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Proveedor | `res.partner` con `supplier_rank > 0` | Necesario para origen |
| Tipo de operacion | `stock.picking.type` recepcion | `[("code","=","incoming")]` |

**STOCK_DELIVERY** (expedicion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Pedido de venta confirmado | `sale.order` en estado `sale` | Genera picking automatico |
| Stock disponible | `stock.quant` con `quantity > 0` | Productos en stock |

**STOCK_INTERNAL_TRANSFER** (traslado):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Ubicacion origen | `stock.location` interna | `[("usage","=","internal")]` |
| Ubicacion destino | `stock.location` interna diferente | ID distinto al origen |
| Stock en origen | `stock.quant` en ubicacion origen | `quantity > 0` |

**STOCK_RETURN** (devolucion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Albaran validado | `stock.picking` en estado `done` | Necesario para devolver |

**STOCK_RETURN_FULL** (devolucion completa desde venta):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Partner cliente | `res.partner` con `customer_rank > 0` | Para la venta |
| Producto almacenable con stock | `product.product` con `type="product"` y `stock.quant` con `quantity > 0` | Necesario para entregar |
| Almacen con tipo de operacion salida | `stock.picking.type` con `code="outgoing"` | Existe por defecto en el almacen |

**STOCK_INVENTORY** (inventario fisico):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Productos almacenables | `product.product` con `type="product"` | Al menos 2 para ajustar |

### 3.3. Precondiciones de stock (disponibilidad)

Antes de ejecutar flujos de inventario, verificar disponibilidad de stock:

| Verificacion | Metodo API | Descripcion |
|-------------|-----------|-------------|
| Verificar stock disponible | `stock.quant._get_available_quantity(product_id, location_id)` | Retorna cantidad disponible (descontando reservas). Usar antes de entregas y traslados |
| Crear stock de prueba | `stock.quant._update_available_quantity(product_id, location_id, qty)` | Actualiza la cantidad disponible directamente. Usar via API para crear stock de prueba sin recepcion |
| Consultar quants existentes | `stock.quant` con dominio `[("product_id","=",pid),("location_id","=",lid),("quantity",">",0)]` | Para verificar existencias antes de flujos |

**Importante:** Si el stock es insuficiente para los flujos de prueba, usar `_update_available_quantity` via API para crear stock de prueba en la ubicacion interna del almacen.

### 3.4. Configuracion de almacen (delivery_steps)

La configuracion de `delivery_steps` del almacen afecta al numero de pickings creados por pedido de venta:

| Valor `delivery_steps` | Descripcion | Pickings creados |
|------------------------|-------------|------------------|
| `ship_only` | 1 paso: Envio directo | 1 picking (OUT) |
| `pick_ship` | 2 pasos: Pick + Envio | 2 pickings (PICK + OUT) |
| `pick_pack_ship` | 3 pasos: Pick + Pack + Envio | 3 pickings (PICK + PACK + OUT) |

| Modelo | Campo | Verificacion |
|--------|-------|--------------|
| `stock.warehouse` | `delivery_steps` | Leer para saber cuantos pickings esperar por pedido de venta |
| `stock.warehouse` | `reception_steps` | `one_step`, `two_steps`, `three_steps` — afecta recepciones de compra |

**Impacto en flujos:** Si el almacen tiene 2 o 3 pasos, confirmar un pedido de venta generara multiples pickings encadenados. Los flujos de entrega deben validar TODOS los pickings en orden.

### 3.5. Configuracion de empresa que afecta reversibilidad (stock)

| Campo `res.company` | Impacto en flujos de stock |
|---------------------|---------------------------|
| `restrict_mode_hash_table` | Si True, las facturas generadas desde entregas tendran hash inalterable — no se pueden eliminar |
| `check_account_audit_trail` | Si True, los asientos contables generados por valoracion de stock no se pueden borrar |

---

## 4. FABRICACION (mrp)

### 4.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `product.product` | name, type="product" | name | Producto fabricable | 1 | Crear producto terminado |
| `mrp.bom` | product_tmpl_id, bom_line_ids | product_tmpl_id | `[]` | 1 | Crear lista de materiales |
| `product.product` (componentes) | name, type="product" | name | Componentes de la BOM | 2+ | Crear componentes |
| `stock.quant` | product_id, quantity | - | Stock de componentes | - | Crear stock de componentes |

### 4.2. Datos adicionales por variante

**MRP_PRODUCTION_WORKORDER** (con ordenes de trabajo):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Centros de trabajo | `mrp.workcenter` | Al menos 1 centro activo |
| Operaciones en BOM | `mrp.routing.workcenter` | BOM con operaciones configuradas |

---

## 5. CRM (crm)

### 5.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `crm.team` | name, use_opportunities | name | `[("use_opportunities","=",True)]` | 1 | Existe por defecto "Ventas" |
| `crm.stage` | name, is_won | name | `[]` | 3+ | Existen por defecto |
| `res.partner` | name | name | `[]` | 0 | Puede crearse durante el flujo |

### 5.2. Datos adicionales

**CRM_OPPORTUNITY_CYCLE** (con actividades):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Tipos de actividad | `mail.activity.type` | Existen por defecto (Llamada, Email, Reunion) |

---

## 6. CONTABILIDAD (account)

### 6.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `res.partner` | name, vat, property_account_receivable_id | name | `[("customer_rank",">",0)]` | 1 | Crear partner con datos fiscales |
| `product.product` | name, list_price | name | `[]` | 1 | Crear producto facturable |
| `account.journal` | name, type | name | `[("type","=","sale")]` | 1 | Existe por defecto |
| `account.account` | code, name | code | Cuentas contables | - | Existen con plan contable |

### 6.2. Datos adicionales por flujo

**ACCOUNT_CREDIT_NOTE** (rectificativa):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Factura publicada | `account.move` en estado `posted` | Necesaria como base de la rectificativa |

**ACCOUNT_RECONCILIATION** (conciliacion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Extracto bancario | `account.bank.statement` con lineas | Al menos 1 linea sin conciliar |
| Facturas pendientes | `account.move` publicadas sin pagar | Para conciliar con extracto |

**ACCOUNT_PAYMENT** (pago directo):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Diario de banco | `account.journal` tipo banco | `[("type","=","bank")]` |

### 6.3. Precondiciones fiscales y de empresa

Campos de `res.company` que afectan la ejecucion y reversibilidad de flujos contables:

| Campo `res.company` | Tipo | Impacto en flujos |
|---------------------|------|-------------------|
| `restrict_mode_hash_table` | Boolean | Si True, las facturas publicadas reciben un hash inalterable (`inalterable_hash`). NO se pueden eliminar ni modificar. Solo se pueden revertir con rectificativa |
| `check_account_audit_trail` | Boolean | Si True, los asientos publicados no se pueden eliminar ni pasar a borrador. Activa la trazabilidad de auditoria |
| `quick_edit_mode` | Selection | Modo de edicion rapida en diarios. Valores: `out_invoices`, `in_invoices`, `out_and_in_invoices`, `False` |

**Fechas de bloqueo contable:**

| Campo `res.company` | Tipo | Impacto |
|---------------------|------|---------|
| `fiscalyear_lock_date` | Date | Fecha de cierre fiscal. Asientos con fecha <= esta fecha NO se pueden crear/modificar (excepto asesores contables) |
| `tax_lock_date` | Date | Fecha de bloqueo fiscal. Asientos que afectan impuestos con fecha <= esta fecha NO se pueden crear/modificar |
| `hard_lock_date` | Date | Fecha de bloqueo estricto. NADIE puede crear/modificar asientos con fecha <= esta fecha (sin excepciones) |

**Regla critica:** Si hay fechas de bloqueo configuradas, todas las entradas de prueba DEBEN usar fechas POSTERIORES a la fecha de bloqueo mas reciente. Si no se respeta, los flujos de creacion/publicacion de facturas fallaran.

**Modulos de localizacion fiscal que afectan facturas:**

| Modulo | Efecto |
|--------|--------|
| `l10n_es_edi_sii` | Envia datos a AEAT en tiempo real al publicar facturas. **NO publicar facturas de prueba** |
| `l10n_es_edi_tbai` | Envia datos a hacienda foral (Pais Vasco/Navarra) al publicar. **NO publicar facturas de prueba** |
| `l10n_es_edi_verifactu` | Sistema Verifactu — establece `inalterable_hash` en facturas publicadas. Las facturas publicadas NO se pueden eliminar |
| `l10n_es_edi_facturae` | Factura electronica — puede requerir certificado XML-DSig valido para publicar |

### 6.4. Precondiciones de diarios (journals)

Verificar que los diarios necesarios existen y estan correctamente configurados:

| Tipo de diario | Dominio | Verificacion | Uso |
|---------------|---------|--------------|-----|
| `sale` | `[("type","=","sale")]` | Debe tener `default_account_id` | Facturas de cliente |
| `purchase` | `[("type","=","purchase")]` | Debe tener `default_account_id` | Facturas de proveedor |
| `bank` | `[("type","=","bank")]` | Debe tener `bank_account_id` o `default_account_id` | Pagos bancarios, extractos |
| `cash` | `[("type","=","cash")]` | Debe tener `default_account_id` | Pagos en efectivo |
| `general` | `[("type","=","general")]` | Debe tener `default_account_id` | Asientos varios, amortizaciones |

| Verificacion adicional | Campo | Descripcion |
|------------------------|-------|-------------|
| Cuenta por defecto | `default_account_id` | Si esta vacio, no se podra crear asientos automaticamente |
| Divisa del diario | `currency_id` | Si difiere de la divisa de la empresa, las facturas se crean en divisa del diario |
| Secuencia | `sequence_id` | Genera la numeracion automatica de asientos |

### 6.5. Configuracion de empresa que afecta reversibilidad (contabilidad)

Resumen de campos de `res.company` que afectan la posibilidad de deshacer/limpiar datos de prueba:

| Escenario | Condicion | Se puede eliminar factura publicada? | Alternativa |
|-----------|-----------|--------------------------------------|-------------|
| Estandar sin hash | `restrict_mode_hash_table = False` | SI (pasar a borrador primero) | — |
| Con hash | `restrict_mode_hash_table = True` | NO | Crear rectificativa |
| Con audit trail | `check_account_audit_trail = True` | NO | Crear rectificativa |
| Con localizacion fiscal (SII/TBAI/Verifactu) | Modulo instalado | NO + envia datos a AEAT | **NO ejecutar flujos de publicacion** |
| Con fecha de bloqueo | `hard_lock_date` o `fiscalyear_lock_date` reciente | Depende de la fecha del asiento | Usar fechas posteriores al bloqueo |

---

## 7. RECURSOS HUMANOS (hr)

### 7.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `hr.department` | name | name | `[]` | 1 | Crear departamento basico |
| `hr.job` | name | name | `[]` | 1 | Crear puesto de trabajo |

### 7.2. Datos adicionales por flujo

**HR_LEAVE_REQUEST** (ausencia):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Empleado activo | `hr.employee` | `[("active","=",True)]` |
| Tipo de ausencia | `hr.leave.type` | Al menos 1 tipo activo |
| Asignacion de dias | `hr.leave.allocation` | Dias disponibles para el tipo |
| Manager | Empleado con `leave_manager_id` | Para aprobacion |

**HR_EXPENSE** (gasto):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Empleado activo | `hr.employee` | `[("active","=",True)]` |
| Producto de gasto | `product.product` | `[("can_be_expensed","=",True)]` |
| Manager | Empleado con `expense_manager_id` | Para aprobacion |

**HR_CONTRACT_LIFECYCLE** (ciclo de vida de contrato):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Departamento | `hr.department` | `[("active","=",True)]` — al menos 1 departamento |
| Puesto de trabajo | `hr.job` | Al menos 1 puesto para asignar al empleado |
| Tipo de estructura salarial (si payroll) | `hr.payroll.structure.type` | Solo si `hr_payroll` instalado |
| Divisa de la empresa | `res.company` con `currency_id` | Necesaria para campo `wage` del contrato |

**HR_LEAVE_ALLOCATION_FULL** (asignacion y ausencia completa):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Empleado activo con calendario | `hr.employee` con `resource_calendar_id` configurado | `[("active","=",True)]` |
| Tipo de ausencia con asignacion fija | `hr.leave.type` con `requires_allocation = "yes"` | Tipo que requiere asignacion previa |
| Manager de ausencias | `hr.employee` campo `leave_manager_id` | Para aprobacion de solicitudes |
| Calendario de trabajo | `resource.calendar` con `attendance_ids` | Dias laborables definidos para el empleado |

---

## 8. PROYECTO (project)

### 8.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `project.project` | name | name | `[("active","=",True)]` | 0 | Se crea en el flujo |

### 8.2. Datos adicionales

**PROJECT_TIMESHEET** (registro de horas):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Proyecto con hojas de horas | `project.project` | `allow_timesheets = True` |
| Tarea existente | `project.task` | Al menos 1 tarea en el proyecto |
| Empleado | `hr.employee` vinculado al usuario | Para registrar horas |

---

## 9. PUNTO DE VENTA (pos)

### 9.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `pos.config` | name, journal_id, picking_type_id | name | `[("active","=",True)]` | 1 | Existe por defecto al instalar POS. Si no, crear desde Punto de Venta > Configuracion > Puntos de venta |
| `pos.payment.method` | name, journal_id | name | `[]` | 1 | Se crean automaticamente (Efectivo, Banco). Si no, crear desde PdV > Configuracion > Metodos de pago |
| `product.product` | name, list_price, available_in_pos | name, list_price | `[("available_in_pos","=",True),("sale_ok","=",True),("active","=",True)]` | 2 | Crear productos con `available_in_pos = True` o habilitar desde ficha del producto |
| `account.journal` | name, type | name | `[("type","in",["sale","general"])]` | 1 | Diario POS existe por defecto |

### 9.2. Datos adicionales por flujo

**POS_SESSION_BASIC** (abrir sesion, vender, cerrar):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Metodo de pago configurado en POS | `pos.config.payment_method_ids` | Al menos 1 metodo de pago asignado al POS |
| Sesion no abierta | No debe existir `pos.session` abierta para el mismo config | `[("config_id","=",config_id),("state","!=","closed")]` — si hay una abierta, cerrarla primero |

**POS_SESSION_CATEGORY** (ventas con categorias):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Categoria POS | `pos.category` | Al menos 1 categoria con `name` definido |
| Productos con categoria | `product.product` con `pos_categ_ids` | Productos asignados a la categoria POS |

**POS_SESSION_INVOICE** (venta con factura desde POS):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Cliente con datos fiscales | `res.partner` con `vat`, `street`, `country_id` | Necesario para emitir factura |
| Diario de facturacion | `pos.config.invoice_journal_id` | Diario tipo `sale` configurado en el POS |

---

## 10. EXTRACTO BANCARIO Y CONCILIACION (account — bank)

### 10.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `account.journal` | name, type, bank_account_id | name, type | `[("type","=","bank")]` | 1 | Crear diario tipo banco desde Contabilidad > Configuracion > Diarios |
| `res.partner.bank` | acc_number, partner_id | acc_number | `[("partner_id","=",company_partner_id)]` | 1 | Crear cuenta bancaria de la empresa en Contabilidad > Configuracion > Cuentas bancarias |
| `account.account` | code, name, account_type | code | `[("account_type","=","asset_cash")]` | 1 | Existe con plan contable. Si no, crear cuenta tipo "Banco y efectivo" |

### 10.2. Datos adicionales por flujo

**BANK_STATEMENT_IMPORT** (importar extracto):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Diario de banco con cuenta bancaria | `account.journal` con `bank_account_id` no vacio | Necesario para importar extracto |
| Fichero de extracto | Archivo OFX, CAMT.053 o CSV | Formato compatible con el modulo de importacion instalado |

**BANK_STATEMENT_RECONCILE** (conciliar extracto):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Lineas de extracto sin conciliar | `account.bank.statement.line` con `is_reconciled = False` | Al menos 1 linea pendiente |
| Facturas publicadas pendientes | `account.move` con `state = "posted"` y `payment_state != "paid"` | Para emparejar con lineas del extracto |
| Cuentas outstanding | `account.journal` campo `company_id.account_journal_payment_debit_account_id` | Cuentas de pagos pendientes configuradas en la empresa |

**BANK_STATEMENT_CREATE_MANUAL** (crear extracto manual):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Diario de banco | `account.journal` tipo `bank` | `[("type","=","bank")]` |
| Partner para lineas | `res.partner` | Al menos 1 para asociar a las lineas del extracto |

**BANK_STATEMENT_FULL** (extracto completo con conciliacion y limpieza):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Diario de banco con cuenta bancaria | `account.journal` tipo `bank` con `bank_account_id` | `[("type","=","bank")]` — necesario para crear extracto |
| Factura publicada pendiente de pago | `account.move` con `state = "posted"` y `payment_state != "paid"` | Para conciliar con linea del extracto |
| Partner de la factura | `res.partner` | Mismo partner en la factura y en la linea del extracto |

---

## 11. ACTIVOS FIJOS (account_asset — Enterprise)

### 11.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `account.asset` (modelo) | name, state="model", method, method_number, account_asset_id, account_depreciation_id, account_depreciation_expense_id, journal_id | name, state | `[("state","=","model")]` | 1 | Crear modelo de activo desde Contabilidad > Configuracion > Modelos de activos |
| `account.journal` | name, type="general" | name | `[("type","=","general")]` | 1 | Diario de amortizacion. Existe por defecto ("Operaciones varias") |
| `account.account` (activo) | code, name | code | `[("account_type","=","asset_non_current")]` | 1 | Cuenta de activo fijo (ej. 2130). Existe con plan contable |
| `account.account` (amortizacion) | code, name | code | `[("account_type","=","asset_non_current"),("deprecated","=",False)]` | 1 | Cuenta de amortizacion acumulada (ej. 2813). Existe con plan contable |
| `account.account` (gasto) | code, name | code | `[("account_type","=","expense")]` | 1 | Cuenta de gasto de amortizacion (ej. 6813). Existe con plan contable |

### 11.2. Datos adicionales por flujo

**ASSET_CREATE** (crear activo desde factura):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Factura de proveedor publicada | `account.move` con `move_type = "in_invoice"` y `state = "posted"` | La factura contiene la compra del activo |
| Modelo de activo vinculado | `account.asset` con `state = "model"` configurado en la cuenta contable | Campo `asset_model` en `account.account` o seleccion manual |
| Grupo de activo (opcional) | `account.asset.group` | Para organizar activos por grupo |

**ASSET_DEPRECIATE** (computar y publicar amortizacion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Activo en estado "running" | `account.asset` con `state = "open"` | `[("state","=","open")]` |
| Lineas de amortizacion pendientes | Entradas de amortizacion no publicadas | `depreciation_entries_count < total_depreciation_entries_count` |

**ASSET_SELL_DISPOSE** (venta o baja de activo):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Activo en estado "running" | `account.asset` con `state = "open"` | Necesario para dar de baja |
| Factura de venta (si venta) | `account.move` con `move_type = "out_invoice"` | Solo si se vende el activo (no para baja simple) |

---

## 12. FABRICACION EXTENDIDA (mrp — avanzado)

### 12.1. Datos comunes (adicionales a seccion 4)

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `mrp.bom` | product_tmpl_id, bom_line_ids, type | product_tmpl_id | `[]` | 1 | Crear lista de materiales con tipo "Fabricar este producto" |
| `mrp.workcenter` | name, costs_hour, time_start, time_stop | name | `[("active","=",True)]` | 1 | Crear centro de trabajo desde Fabricacion > Configuracion > Centros de trabajo |
| `stock.quant` (componentes) | product_id, quantity, location_id | product_id | `[("quantity",">",0),("location_id.usage","=","internal")]` | 2+ | Crear stock de componentes via inventario fisico o recepcion |
| `stock.location` (produccion) | name, usage="production" | name | `[("usage","=","production")]` | 1 | Existe por defecto ("Produccion"). Si no, revisar configuracion de almacen |

### 12.2. Datos adicionales por flujo

**MRP_WORKORDER** (fabricacion con ordenes de trabajo):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Centros de trabajo | `mrp.workcenter` activos | `[("active","=",True)]` — al menos 1 |
| BOM con operaciones | `mrp.bom` con `operation_ids` (campo `mrp.routing.workcenter`) | BOM debe tener al menos 1 operacion configurada |
| Habilitar ordenes de trabajo | Ajustes > Fabricacion > Ordenes de trabajo | `res.config.settings` campo `group_mrp_routings` = True |

**MRP_SUBCONTRACTING** (subcontratacion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| BOM tipo subcontratacion | `mrp.bom` con `type = "subcontract"` | `[("type","=","subcontract")]` |
| Subcontratista | `res.partner` con `supplier_rank > 0` | Asignado en campo `subcontractor_ids` de la BOM |

**MRP_UNBUILD** (desmontar producto):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Producto fabricado en stock | `stock.quant` del producto terminado | `quantity > 0` en ubicacion interna |
| BOM del producto | `mrp.bom` | Para saber como desmontar |

---

## 13. NOMINAS (hr_payroll — Enterprise)

### 13.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `hr.payroll.structure` | name, code, type_id, rule_ids | name, type_id | `[("active","=",True)]` | 1 | Crear estructura salarial o instalar localizacion de nomina del pais |
| `hr.payroll.structure.type` | name | name | `[]` | 1 | Existe por defecto ("Empleado"). Si no, crear tipo de estructura |
| `hr.salary.rule` | name, code, category_id, struct_id | name, code | `[]` | 3+ | Existen con la estructura. Si no, crear reglas basicas (BASIC, GROSS, NET) |
| `hr.employee` | name, contract_id | name | `[("active","=",True),("contract_id","!=",False)]` | 1 | Crear empleado con contrato activo |
| `hr.contract` | name, employee_id, wage, state, structure_type_id, date_start | employee_id, wage, state | `[("state","=","open")]` | 1 | Crear contrato en estado "En progreso" con salario y estructura |
| `hr.work.entry.type` | name, code | name, code | `[]` | 1+ | Existen por defecto (Asistencia, Horas extra, etc.). Instalados con `hr_work_entry` |

### 13.2. Datos adicionales por flujo

**PAYSLIP_COMPUTE** (calcular nomina):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Empleado con contrato activo | `hr.contract` con `state = "open"` y `date_start <= hoy` | Contrato debe cubrir el periodo de la nomina |
| Estructura salarial con reglas | `hr.payroll.structure` con `rule_ids` | Al menos reglas BASIC, GROSS, NET |
| Entradas de trabajo generadas | `hr.work.entry` para el periodo | Generar via `hr.employee.generate_work_entries()` si no existen |

**PAYSLIP_BATCH** (lote de nominas):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Multiples empleados con contrato | `hr.employee` con contrato activo | Al menos 2 empleados para lote |
| Estructura salarial comun | `hr.payroll.structure` | Misma estructura o tipo para todos |
| Periodo definido | Fecha inicio y fin del lote | Tipicamente 1er y ultimo dia del mes |

**PAYSLIP_REFUND** (rectificacion de nomina):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Nomina confirmada | `hr.payslip` en estado `done` | `[("state","=","done")]` |
| Diario de nominas | `account.journal` | Configurado en la estructura salarial |

---

## 14. SUSCRIPCIONES (sale_subscription — Enterprise)

### 14.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `sale.subscription.plan` | name, billing_period_value, billing_period_unit | name | `[("active","=",True)]` | 1 | Crear plan recurrente desde Suscripciones > Configuracion > Planes de suscripcion |
| `product.product` | name, list_price, recurring_invoice | name, list_price | `[("recurring_invoice","=",True),("active","=",True)]` | 1 | Crear producto con `recurring_invoice = True` o habilitarlo en ficha del producto |
| `sale.subscription.pricing` | plan_id, product_template_id, price | plan_id | `[]` | 1 | Crear precio recurrente vinculado al plan y al producto |
| `res.partner` | name | name | `[("customer_rank",">",0)]` | 1 | Crear cliente para la suscripcion |

### 14.2. Datos adicionales por flujo

**SUBSCRIPTION_CREATE** (crear suscripcion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Plan de suscripcion | `sale.subscription.plan` activo | Al menos 1 plan con periodo de facturacion definido |
| Producto recurrente | `product.product` con `recurring_invoice = True` | Pricing configurado para el plan elegido |

**SUBSCRIPTION_RENEW** (renovar suscripcion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Suscripcion activa | `sale.order` con `is_subscription = True` y `subscription_state = "3_progress"` | Suscripcion en progreso |
| Plan con renovacion | `sale.subscription.plan` con `user_extend = True` | Permite al cliente renovar |

**SUBSCRIPTION_UPSELL** (ampliar suscripcion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Suscripcion activa | `sale.order` con `subscription_state = "3_progress"` | En progreso |
| Productos opcionales | `product.product` con `recurring_invoice = True` | Productos adicionales disponibles |

**SUBSCRIPTION_CLOSE** (cerrar suscripcion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Suscripcion activa | `sale.order` con `subscription_state = "3_progress"` | En progreso |
| Motivo de cierre | `sale.order.close.reason` | Al menos 1 motivo configurado |

---

## 15. INVENTARIO EXTENDIDO (stock — avanzado)

### 15.1. Datos adicionales por flujo

**STOCK_SCRAP** (desecho de productos):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Ubicacion de desecho | `stock.location` con `scrap_location = True` | `[("scrap_location","=",True)]` — existe por defecto ("Desechos virtuales") |
| Producto almacenable en stock | `stock.quant` con `quantity > 0` | Producto disponible para desechar |
| Motivo de desecho (opcional) | `stock.scrap.reason.tag` | Opcional pero recomendado para trazabilidad |

**STOCK_MULTI_WAREHOUSE** (traslados entre almacenes):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Multiples almacenes | `stock.warehouse` activos | `[("active","=",True)]` — al menos 2 almacenes |
| Tipo de operacion interna | `stock.picking.type` con `code = "internal"` | `[("code","=","internal")]` — existe por almacen |
| Ruta entre almacenes | Rutas de abastecimiento configuradas | Configurar en Inventario > Configuracion > Almacenes > Rutas |
| Stock en almacen origen | `stock.quant` en ubicacion del almacen origen | `quantity > 0` |

**STOCK_REORDER_RULE** (reglas de reabastecimiento):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Regla de reabastecimiento | `stock.warehouse.orderpoint` con `product_id`, `product_min_qty`, `product_max_qty` | `[]` — al menos 1 regla |
| Producto almacenable | `product.product` con `type = "product"` | Producto con ruta de abastecimiento (compra o fabricacion) |
| Ruta de abastecimiento | `product.product.route_ids` o `stock.warehouse.orderpoint.route_id` | Ruta tipo "Comprar" o "Fabricar" configurada |
| Proveedor (si ruta compra) | `product.supplierinfo` con `partner_id` y `price` | Al menos 1 proveedor configurado en el producto |
| Ubicacion | `stock.warehouse.orderpoint.location_id` | Ubicacion de stock del almacen |

**STOCK_LOT_TRACKING** (trazabilidad por lote/serie):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Producto con tracking | `product.product` con `tracking = "lot"` o `tracking = "serial"` | `[("tracking","in",["lot","serial"])]` |
| Lotes/series existentes (para salidas) | `stock.lot` | Al menos 1 lote si se va a hacer salida |

---

## 16. EMAIL MARKETING (mass_mailing)

### 16.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `mailing.list` | name, active | name | `[("active","=",True)]` | 1 | Crear lista de correo desde Email Marketing > Listas de correo |
| `mailing.contact` | name, email, list_ids | name, email | `[("email","!=",False)]` | 5 | Crear contactos con email valido. Asociar a la lista via `subscription_list_ids` |
| `ir.mail_server` | name, smtp_host | name | `[("active","=",True)]` | 1 | Configurar servidor SMTP saliente en Ajustes > Servidores de correo saliente |

### 16.2. Datos adicionales por flujo

**MAILING_SEND** (enviar campana de email):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Lista de correo con contactos | `mailing.list` con `contact_count > 0` | Al menos 1 lista con contactos suscritos |
| Plantilla de email o cuerpo HTML | Contenido del mailing | Campo `body_html` no vacio o `body_arch` definido |
| Modelo de destinatarios | `mailing.mailing` campo `mailing_model_id` | Por defecto `mailing.list` pero puede ser `res.partner`, `crm.lead`, etc. |
| Servidor de correo configurado | `ir.mail_server` activo | Necesario para enviar. Al menos 1 servidor SMTP |

**MAILING_AB_TEST** (test A/B):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Mailing con A/B test habilitado | `mailing.mailing` con `ab_testing_enabled = True` | Configurado en la campana |
| Lista con suficientes contactos | `mailing.list` con `contact_count >= 10` | Minimo razonable para test A/B |

**MAILING_AUTOMATION** (automatizacion — requiere marketing_automation):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Campana de automatizacion | `marketing.campaign` | Modelo de `marketing_automation` |
| Actividades de campana | `marketing.activity` | Al menos 1 actividad (email, SMS, accion servidor) |
| Modelo de destinatarios | Configurado en la campana | `res.partner`, `crm.lead`, etc. |

---

## Datos transversales (aplican a multiples flujos)

### Moneda y fiscal

| Modelo | Verificacion | Nota |
|--------|--------------|------|
| `res.currency` | Moneda de la empresa configurada | EUR por defecto en Espana |
| `account.tax` | Impuestos configurados (IVA 21%, 10%, 4%) | Vienen con localizacion espanola |
| `account.fiscal.position` | Posiciones fiscales para UE/Extracomunitario | Opcionales pero recomendadas |

### Secuencias

| Modelo | Verificacion | Nota |
|--------|--------------|------|
| `ir.sequence` | Secuencias de SO, PO, INV configuradas | Existen por defecto |

### Usuarios y permisos

| Verificacion | Nota |
|--------------|------|
| Usuario con permisos de la app | Necesario para ejecutar el flujo |
| Multicompania | Si aplica, verificar empresa activa |

---

## Formato JSON para verify_data.py

El script `verify_data.py` espera las precondiciones en este formato:

```json
{
  "flow_id": "SALE_ORDER_STANDARD",
  "preconditions": [
    {
      "model": "res.partner",
      "domain": "[(\"customer_rank\",\">\",0)]",
      "fields": ["name", "vat", "country_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "product.product",
      "domain": "[(\"sale_ok\",\"=\",True),(\"active\",\"=\",True)]",
      "fields": ["name", "list_price"],
      "min_records": 2,
      "critical": true
    }
  ]
}
```

Campos del JSON:
- `model`: Modelo Odoo a consultar
- `domain`: Dominio de busqueda (formato Python/Odoo)
- `fields`: Campos que deben tener valor
- `min_records`: Minimo de registros necesarios
- `critical`: Si es `true`, el flujo no puede ejecutarse sin estos datos

### Ejemplos JSON para nuevos flujos

**POS_SESSION_BASIC:**
```json
{
  "flow_id": "POS_SESSION_BASIC",
  "preconditions": [
    {
      "model": "pos.config",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name", "journal_id", "picking_type_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "pos.payment.method",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "product.product",
      "domain": "[(\"available_in_pos\",\"=\",True),(\"sale_ok\",\"=\",True),(\"active\",\"=\",True)]",
      "fields": ["name", "list_price"],
      "min_records": 2,
      "critical": true
    }
  ]
}
```

**BANK_STATEMENT_RECONCILE:**
```json
{
  "flow_id": "BANK_STATEMENT_RECONCILE",
  "preconditions": [
    {
      "model": "account.journal",
      "domain": "[(\"type\",\"=\",\"bank\")]",
      "fields": ["name", "bank_account_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "account.bank.statement.line",
      "domain": "[(\"is_reconciled\",\"=\",False)]",
      "fields": ["date", "amount"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "account.move",
      "domain": "[(\"state\",\"=\",\"posted\"),(\"payment_state\",\"!=\",\"paid\"),(\"move_type\",\"in\",[\"out_invoice\",\"in_invoice\"])]",
      "fields": ["name", "amount_residual"],
      "min_records": 1,
      "critical": false
    }
  ]
}
```

**ASSET_CREATE:**
```json
{
  "flow_id": "ASSET_CREATE",
  "preconditions": [
    {
      "model": "account.asset",
      "domain": "[(\"state\",\"=\",\"model\")]",
      "fields": ["name", "account_asset_id", "account_depreciation_id", "account_depreciation_expense_id", "journal_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "account.journal",
      "domain": "[(\"type\",\"=\",\"general\")]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**PAYSLIP_COMPUTE:**
```json
{
  "flow_id": "PAYSLIP_COMPUTE",
  "preconditions": [
    {
      "model": "hr.employee",
      "domain": "[(\"active\",\"=\",True),(\"contract_id\",\"!=\",False)]",
      "fields": ["name", "contract_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "hr.contract",
      "domain": "[(\"state\",\"=\",\"open\")]",
      "fields": ["employee_id", "wage", "structure_type_id", "date_start"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "hr.payroll.structure",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name", "type_id"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**SUBSCRIPTION_CREATE:**
```json
{
  "flow_id": "SUBSCRIPTION_CREATE",
  "preconditions": [
    {
      "model": "sale.subscription.plan",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name", "billing_period_value", "billing_period_unit"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "product.product",
      "domain": "[(\"recurring_invoice\",\"=\",True),(\"active\",\"=\",True)]",
      "fields": ["name", "list_price"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "res.partner",
      "domain": "[(\"customer_rank\",\">\",0)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**MAILING_SEND:**
```json
{
  "flow_id": "MAILING_SEND",
  "preconditions": [
    {
      "model": "mailing.list",
      "domain": "[(\"active\",\"=\",True),(\"contact_count\",\">\",0)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "mailing.contact",
      "domain": "[(\"email\",\"!=\",False)]",
      "fields": ["name", "email"],
      "min_records": 5,
      "critical": true
    },
    {
      "model": "ir.mail_server",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name", "smtp_host"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**STOCK_REORDER_RULE:**
```json
{
  "flow_id": "STOCK_REORDER_RULE",
  "preconditions": [
    {
      "model": "stock.warehouse.orderpoint",
      "domain": "[]",
      "fields": ["product_id", "product_min_qty", "product_max_qty", "location_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "product.product",
      "domain": "[(\"type\",\"=\",\"product\")]",
      "fields": ["name", "route_ids"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**MRP_CREATE:**
```json
{
  "flow_id": "MRP_CREATE",
  "preconditions": [
    {
      "model": "product.product",
      "domain": "[(\"type\",\"=\",\"consu\"),(\"active\",\"=\",True)]",
      "fields": ["name", "uom_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "mrp.bom",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["product_tmpl_id", "bom_line_ids"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "stock.quant",
      "domain": "[(\"quantity\",\">\",0),(\"location_id.usage\",\"=\",\"internal\")]",
      "fields": ["product_id", "quantity"],
      "min_records": 1,
      "critical": false
    }
  ]
}
```

**LEAVE_REQUEST_CREATE:**
```json
{
  "flow_id": "LEAVE_REQUEST_CREATE",
  "preconditions": [
    {
      "model": "hr.employee",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name", "leave_manager_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "hr.leave.type",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "hr.leave.allocation",
      "domain": "[(\"state\",\"=\",\"validate\")]",
      "fields": ["employee_id", "holiday_status_id", "number_of_days"],
      "min_records": 1,
      "critical": false
    }
  ]
}
```

**EXPENSE_CREATE:**
```json
{
  "flow_id": "EXPENSE_CREATE",
  "preconditions": [
    {
      "model": "hr.employee",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name", "expense_manager_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "product.product",
      "domain": "[(\"can_be_expensed\",\"=\",True)]",
      "fields": ["name", "standard_price"],
      "min_records": 1,
      "critical": false
    }
  ]
}
```

**APPLICANT_CREATE:**
```json
{
  "flow_id": "APPLICANT_CREATE",
  "preconditions": [
    {
      "model": "hr.job",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": false
    },
    {
      "model": "hr.recruitment.stage",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**VEHICLE_CREATE:**
```json
{
  "flow_id": "VEHICLE_CREATE",
  "preconditions": [
    {
      "model": "fleet.vehicle.model",
      "domain": "[]",
      "fields": ["name", "brand_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "fleet.vehicle.model.brand",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "fleet.vehicle.state",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": false
    }
  ]
}
```

**VEHICLE_LOG_SERVICE:**
```json
{
  "flow_id": "VEHICLE_LOG_SERVICE",
  "preconditions": [
    {
      "model": "fleet.vehicle",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name", "model_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "fleet.service.type",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**MAINTENANCE_CREATE:**
```json
{
  "flow_id": "MAINTENANCE_CREATE",
  "preconditions": [
    {
      "model": "maintenance.team",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "maintenance.stage",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "maintenance.equipment",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name"],
      "min_records": 0,
      "critical": false
    }
  ]
}
```

**REPAIR_CREATE:**
```json
{
  "flow_id": "REPAIR_CREATE",
  "preconditions": [
    {
      "model": "product.product",
      "domain": "[(\"type\",\"=\",\"consu\"),(\"active\",\"=\",True)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "stock.picking.type",
      "domain": "[(\"code\",\"=\",\"repair_operation\")]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "stock.location",
      "domain": "[(\"usage\",\"=\",\"internal\")]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**BANK_STATEMENT_FULL:**
```json
{
  "flow_id": "BANK_STATEMENT_FULL",
  "preconditions": [
    {
      "model": "account.journal",
      "domain": "[(\"type\",\"=\",\"bank\")]",
      "fields": ["name", "bank_account_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "account.move",
      "domain": "[(\"state\",\"=\",\"posted\"),(\"payment_state\",\"!=\",\"paid\"),(\"move_type\",\"=\",\"out_invoice\")]",
      "fields": ["name", "partner_id", "amount_residual"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "res.partner",
      "domain": "[(\"customer_rank\",\">\",0)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**STOCK_RETURN_FULL:**
```json
{
  "flow_id": "STOCK_RETURN_FULL",
  "preconditions": [
    {
      "model": "res.partner",
      "domain": "[(\"customer_rank\",\">\",0)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "product.product",
      "domain": "[(\"type\",\"=\",\"product\"),(\"sale_ok\",\"=\",True),(\"active\",\"=\",True)]",
      "fields": ["name", "list_price"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "stock.quant",
      "domain": "[(\"quantity\",\">\",0),(\"location_id.usage\",\"=\",\"internal\")]",
      "fields": ["product_id", "quantity"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**HR_CONTRACT_LIFECYCLE:**
```json
{
  "flow_id": "HR_CONTRACT_LIFECYCLE",
  "preconditions": [
    {
      "model": "hr.department",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "hr.job",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": false
    },
    {
      "model": "res.company",
      "domain": "[]",
      "fields": ["name", "currency_id"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**LEAVE_ALLOCATION_FULL:**
```json
{
  "flow_id": "LEAVE_ALLOCATION_FULL",
  "preconditions": [
    {
      "model": "hr.employee",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name", "leave_manager_id", "resource_calendar_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "hr.leave.type",
      "domain": "[(\"active\",\"=\",True),(\"requires_allocation\",\"=\",\"yes\")]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "resource.calendar",
      "domain": "[]",
      "fields": ["name", "attendance_ids"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**TICKET_CREATE:**
```json
{
  "flow_id": "TICKET_CREATE",
  "preconditions": [
    {
      "model": "helpdesk.team",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "helpdesk.stage",
      "domain": "[]",
      "fields": ["name", "fold"],
      "min_records": 2,
      "critical": true
    }
  ]
}
```

**QC_CHECK_CREATE:**
```json
{
  "flow_id": "QC_CHECK_CREATE",
  "preconditions": [
    {
      "model": "quality.alert.team",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "quality.point.test_type",
      "domain": "[]",
      "fields": ["name", "technical_name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "product.product",
      "domain": "[(\"type\",\"=\",\"consu\"),(\"active\",\"=\",True)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": false
    }
  ]
}
```

**PLANNING_CREATE:**
```json
{
  "flow_id": "PLANNING_CREATE",
  "preconditions": [
    {
      "model": "resource.resource",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "planning.role",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 0,
      "critical": false
    },
    {
      "model": "hr.employee",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name", "resource_id"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**APPROVAL_CREATE:**
```json
{
  "flow_id": "APPROVAL_CREATE",
  "preconditions": [
    {
      "model": "approval.category",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name", "approval_minimum"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "res.users",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["name"],
      "min_records": 2,
      "critical": true
    }
  ]
}
```

---

## 17. FABRICACION EXTENDIDA — Ordenes de Produccion (mrp)

### 17.1. Datos comunes (complementa seccion 4)

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `product.product` | name, type="consu" | name | `[("type","=","consu"),("active","=",True)]` | 1 | Crear producto fabricable (storable). En Odoo 18, type="consu" con is_storable=True |
| `mrp.bom` | product_tmpl_id, bom_line_ids | product_tmpl_id | `[("active","=",True)]` | 1 | Crear lista de materiales con componentes |
| `product.product` (componentes) | name, type="consu" | name | Componentes de la BOM | 2+ | Crear productos componente |
| `stock.quant` | product_id, quantity | product_id | `[("quantity",">",0),("location_id.usage","=","internal")]` | 2+ | Crear stock de componentes via ajuste de inventario |
| `stock.picking.type` | code="mrp_operation" | code | `[("code","=","mrp_operation")]` | 1 | Existe por defecto al instalar mrp |
| `uom.uom` | name | name | `[]` | 1 | Existe por defecto (Unidades) |

### 17.2. Datos adicionales por flujo

**MRP_CREATE** (crear OP):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| BOM valida con componentes | `mrp.bom` con `bom_line_ids` no vacio | Al menos 1 componente definido |
| Producto con BOM | `product.product` con BOM asociada | `mrp.bom` con `product_tmpl_id` = template del producto |

**MRP_PRODUCE** (marcar como hecho):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Stock de componentes | `stock.quant` con cantidad suficiente | Para cada componente de la BOM: `quantity >= qty_needed` |
| OP confirmada | `mrp.production` en estado `confirmed` o `progress` | `[("state","in",["confirmed","progress"])]` |
| Lote del producto (si tracking) | `lot_producing_id` asignado | Si producto tiene `tracking != 'none'` |

**MRP_SCRAP** (desguazar desde OP):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| OP en progreso | `mrp.production` en estado `confirmed` o `progress` | `[("state","in",["confirmed","progress"])]` |
| Ubicacion de desecho | `stock.location` con `scrap_location = True` | `[("scrap_location","=",True)]` |

---

## 18. RRHH — Ausencias (hr_holidays)

### 18.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `hr.employee` | name, leave_manager_id | name | `[("active","=",True)]` | 1 | Crear empleado con manager de ausencias |
| `hr.leave.type` | name | name | `[("active","=",True)]` | 1 | Existen por defecto (Vacaciones, Enfermedad, etc.) al instalar hr_holidays |
| `resource.calendar` | name | name | `[]` | 1 | Existe por defecto (horario 40h/semana) |

### 18.2. Datos adicionales por flujo

**LEAVE_REQUEST_CREATE** (solicitar ausencia):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Asignacion de dias | `hr.leave.allocation` en estado `validate` | Dias disponibles para el tipo de ausencia y empleado |
| Manager de ausencias | `hr.employee.leave_manager_id` configurado | Para que se pueda aprobar |

**LEAVE_REQUEST_APPROVE** (aprobar ausencia):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Solicitud pendiente | `hr.leave` en estado `confirm` | `[("state","=","confirm")]` |
| Usuario aprobador | HR Officer o `leave_manager_id` del empleado | Permisos de grupo `hr_holidays.group_hr_holidays_responsible` |

**LEAVE_ALLOCATION_CREATE** (asignar dias):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Tipo con asignacion | `hr.leave.type` que permita asignacion | Campo `requires_allocation` o similar activo |
| Empleado activo | `hr.employee` activo | `[("active","=",True)]` |

---

## 19. RRHH — Gastos (hr_expense)

### 19.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `hr.employee` | name, expense_manager_id | name | `[("active","=",True)]` | 1 | Crear empleado con manager de gastos |
| `product.product` | name, can_be_expensed | name | `[("can_be_expensed","=",True)]` | 1 | Crear producto de tipo gasto (ej: "Comida", "Transporte") |
| `account.journal` | name, type | name | `[("type","=","purchase")]` | 1 | Existe por defecto |

### 19.2. Datos adicionales por flujo

**EXPENSE_SUBMIT** (enviar informe):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Gasto sin informe | `hr.expense` sin `sheet_id` | `[("sheet_id","=",False)]` |
| Manager de gastos | `hr.employee.expense_manager_id` configurado | Para aprobacion |

**EXPENSE_APPROVE** (aprobar informe):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Informe enviado | `hr.expense.sheet` en estado `submit` | `[("state","=","submit")]` |
| Usuario aprobador | HR Officer, Manager o approver del empleado | Permisos adecuados |

**EXPENSE_POST** (contabilizar):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Informe aprobado | `hr.expense.sheet` en estado `approve` | `[("state","=","approve")]` |
| Cuentas contables | `account.account` para gastos configuradas | Cuentas de gasto asociadas a los productos |

---

## 20. RECLUTAMIENTO (hr_recruitment)

### 20.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `hr.job` | name | name | `[]` | 1 | Crear puesto de trabajo desde Reclutamiento > Configuracion > Puestos de trabajo |
| `hr.recruitment.stage` | name | name | `[]` | 3+ | Existen por defecto (Nuevo, Cualificacion inicial, Primera entrevista, etc.) |
| `hr.department` | name | name | `[]` | 1 | Crear departamento |

### 20.2. Datos adicionales por flujo

**APPLICANT_CREATE** (crear candidatura):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Puesto de trabajo | `hr.job` existente | Al menos 1 puesto activo |
| Etapas configuradas | `hr.recruitment.stage` | Al menos 3 etapas para el flujo completo |

**APPLICANT_HIRE** (contratar):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Candidatura activa | `hr.applicant` no archivada | `[("active","=",True)]` |
| Departamento | `hr.department` | Para asignar al nuevo empleado |
| Puesto de trabajo | `hr.job` | Para vincular al empleado |

---

## 21. FLOTA (fleet)

### 21.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `fleet.vehicle.model.brand` | name | name | `[]` | 1 | Crear marca de vehiculo (ej: "Toyota", "Renault"). Existen demos si se cargan datos de ejemplo |
| `fleet.vehicle.model` | name, brand_id | name, brand_id | `[]` | 1 | Crear modelo de vehiculo (ej: "Corolla", "Megane") vinculado a la marca |
| `fleet.vehicle.state` | name | name | `[]` | 2+ | Existen por defecto (Nuevo, Activo, En reparacion, Vendido) |
| `fleet.service.type` | name | name | `[]` | 2+ | Existen por defecto (Cambio aceite, Revision, ITV, etc.) |

### 21.2. Datos adicionales por flujo

**VEHICLE_CREATE** (crear vehiculo):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Modelo de vehiculo | `fleet.vehicle.model` con marca | Al menos 1 modelo configurado |
| Conductor (opcional) | `hr.employee` o `res.partner` | Para asignar conductor al vehiculo |

**VEHICLE_LOG_SERVICE** (registrar servicio):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Vehiculo activo | `fleet.vehicle` activo | `[("active","=",True)]` |
| Tipo de servicio | `fleet.service.type` | Al menos 1 tipo configurado |
| Proveedor (opcional) | `res.partner` con `supplier_rank > 0` | Para asignar taller/proveedor del servicio |

---

## 22. MANTENIMIENTO (maintenance)

### 22.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `maintenance.team` | name | name | `[]` | 1 | Existe por defecto al instalar modulo. Si no, crear equipo desde Mantenimiento > Configuracion > Equipos |
| `maintenance.stage` | name, done | name | `[]` | 3+ | Existen por defecto (Nuevo, En progreso, Reparado). Al menos 1 debe tener `done = True` |
| `maintenance.equipment.category` | name | name | `[]` | 0 | Opcional. Categorias de equipo (ej: "Informatica", "Vehiculos", "Maquinaria") |
| `maintenance.equipment` | name | name | `[("active","=",True)]` | 0 | Opcional. Crear equipo desde Mantenimiento > Equipos |

### 22.2. Datos adicionales por flujo

**MAINTENANCE_CREATE** (crear solicitud):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Equipo de mantenimiento | `maintenance.team` | Al menos 1 equipo activo |
| Equipo (opcional) | `maintenance.equipment` | Para vincular la solicitud a un equipo especifico |

**MAINTENANCE_DONE** (completar solicitud):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Etapa completada | `maintenance.stage` con `done = True` | `[("done","=",True)]` — debe existir al menos 1 |
| Solicitud abierta | `maintenance.request` no archivada | `[("archive","=",False)]` |

---

## 23. REPARACION (repair)

### 23.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `product.product` | name, type="consu" | name | `[("type","=","consu"),("active","=",True)]` | 1 | Producto storable a reparar |
| `stock.picking.type` | code="repair_operation" | name | `[("code","=","repair_operation")]` | 1 | Existe por defecto al instalar modulo repair |
| `stock.location` | name, usage="internal" | name | `[("usage","=","internal")]` | 1 | Ubicacion interna de stock |
| `res.partner` | name | name | `[]` | 0 | Cliente para la reparacion (opcional) |

### 23.2. Datos adicionales por flujo

**REPAIR_CREATE** (crear orden):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Producto reparable | `product.product` storable | Producto disponible en stock (preferiblemente) |
| Tipo de operacion reparacion | `stock.picking.type` con `code = "repair_operation"` | Existe por defecto |

**REPAIR_DONE** (completar reparacion):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Orden confirmada | `repair.order` en estado `confirmed` | `[("state","=","confirmed")]` |
| Componentes de repuesto (opcional) | `product.product` para componentes | Si la reparacion requiere recambios |
| Stock de componentes | `stock.quant` de componentes | Si hay lineas de componentes en la orden |

---

## 24. HELPDESK — Tickets (Enterprise)

### 24.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `helpdesk.team` | name | name | `[("active","=",True)]` | 1 | Crear equipo desde Helpdesk > Configuracion > Equipos Helpdesk |
| `helpdesk.stage` | name, fold | name | `[]` | 3+ | Existen por defecto (Nuevo, En progreso, Solucionado, Cancelado). Al menos 1 debe tener `fold = True` para cierre |
| `res.users` | name | name | `[("active","=",True)]` | 1 | Agente para asignar tickets |

### 24.2. Datos adicionales por flujo

**TICKET_CREATE** (crear ticket):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Equipo helpdesk | `helpdesk.team` con etapas asignadas | `team_ids` no vacio en helpdesk.stage |
| Cliente (opcional) | `res.partner` | Para vincular al ticket |

**TICKET_CLOSE** (cerrar ticket):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Etapa de cierre | `helpdesk.stage` con `fold = True` | `[("fold","=",True),("team_ids","in",[team_id])]` |
| Ticket abierto | `helpdesk.ticket` sin `close_date` | `[("close_date","=",False)]` |

---

## 25. CALIDAD — Controles (Enterprise: quality_control)

### 25.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `quality.alert.team` | name | name | `[]` | 1 | Crear equipo de calidad desde Calidad > Configuracion > Equipos de calidad |
| `quality.point.test_type` | name, technical_name | name | `[]` | 1+ | Existen por defecto (pass_fail, measure, picture, etc.) al instalar quality_control |
| `product.product` | name | name | `[("type","=","consu"),("active","=",True)]` | 1 | Producto storable para controlar |
| `quality.point` | name, product_ids, test_type_id | name | `[]` | 0 | Puntos de control (generan checks automaticos). Opcional para checks manuales |

### 25.2. Datos adicionales por flujo

**QC_CHECK_CREATE** (crear control manual):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Equipo de calidad | `quality.alert.team` | Al menos 1 equipo |
| Tipo de test | `quality.point.test_type` | Al menos 1 tipo (pass_fail existe por defecto) |

**QC_CHECK_PASS / QC_CHECK_FAIL** (aprobar/rechazar):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Control pendiente | `quality.check` con `quality_state = 'none'` | `[("quality_state","=","none")]` |

---

## 26. PLANNING — Planificacion (Enterprise)

### 26.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `hr.employee` | name, resource_id | name | `[("active","=",True)]` | 1 | Crear empleado. El resource_id se genera automaticamente |
| `resource.resource` | name | name | `[("active","=",True)]` | 1 | Se crea automaticamente con hr.employee |
| `planning.role` | name | name | `[]` | 0 | Opcional. Crear rol desde Planning > Configuracion > Roles |
| `planning.slot.template` | name, start_time, duration | name | `[]` | 0 | Opcional. Plantillas de turno predefinidas |

### 26.2. Datos adicionales por flujo

**PLANNING_CREATE** (crear turno):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Recurso/empleado | `resource.resource` activo vinculado a `hr.employee` | `[("active","=",True)]` |
| Fechas validas | Fecha inicio < fecha fin | Periodo coherente |

**PLANNING_PUBLISH** (publicar):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Turnos en draft | `planning.slot` con `state = 'draft'` | `[("state","=","draft")]` |
| Empleado con email | `hr.employee` con `work_email` | Para enviar notificacion |

---

## 27. APROBACIONES (Enterprise: approvals)

### 27.1. Datos comunes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `approval.category` | name, approval_minimum | name | `[("active","=",True)]` | 1 | Crear categoria desde Aprobaciones > Configuracion > Tipos de aprobacion. Configurar campos visibles y aprobadores |
| `res.users` | name | name | `[("active","=",True)]` | 2 | Al menos 2 usuarios: 1 solicitante + 1 aprobador |

### 27.2. Datos adicionales por flujo

**APPROVAL_CREATE** (crear solicitud):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Categoria de aprobacion | `approval.category` activa con aprobadores | `approval_minimum >= 1` |
| Aprobadores configurados | Usuarios asignados en la categoria | En `approval.category` campo `approver_ids` |

**APPROVAL_SUBMIT** (enviar):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Solicitud nueva | `approval.request` con `request_status = 'new'` | `[("request_status","=","new")]` |
| Campos obligatorios completos | Segun configuracion de la categoria | Verificar campos con `has_*` != 'no' y required en la categoria |

**APPROVAL_APPROVE** (aprobar):
| Dato extra | Valor | Verificacion |
|------------|-------|--------------|
| Solicitud pendiente | `approval.request` con `request_status = 'pending'` | `[("request_status","=","pending")]` |
| Usuario aprobador | Usuario en `approval.approver` para la solicitud | `[("request_id","=",request_id),("status","!=\",\"approved\")]` |

---

## 28. CONTABILIDAD EXTENDIDA

### 28.1. Datos para asientos recurrentes

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `account.journal` | name, type="general" | name | `[("type","=","general")]` | 1 | Diario de operaciones varias. Existe por defecto |
| `account.account` | code, name | code | `[]` | 2 | Al menos 2 cuentas contables para debe y haber |

### 28.2. Datos para cierre fiscal

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `account.account` | code, account_type="equity" | code | `[("account_type","=","equity")]` | 1 | Cuenta de resultados del ejercicio (ej: 1290). Existe con plan contable |
| `account.account` | code, account_type in income/expense | code | `[("account_type","in",["income","expense"])]` | - | Cuentas de P&L a saldar |
| `account.move` | state="posted" | - | `[("state","=","posted")]` | - | Todos los asientos del periodo contabilizados |

---

## 29. INVENTARIO EXTENDIDO — Flujos Avanzados

### 29.1. Datos para multi-step receipt

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `stock.warehouse` | name, reception_steps | name | `[("active","=",True)]` | 1 | Configurar `reception_steps` = "two_steps" o "three_steps" |
| `stock.picking.type` | code="incoming" | name | `[("code","=","incoming")]` | 1 | Existe por defecto |
| `stock.picking.type` | code="internal" | name | `[("code","=","internal")]` | 1 | Se genera al configurar multi-step |
| `purchase.order` | state="purchase" | - | `[("state","=","purchase")]` | 1 | OC confirmada para generar recepciones |

### 29.2. Datos para trazabilidad por serie

| Modelo | Campo | Requerido | Dominio | Min | Sugerencia si falta |
|--------|-------|-----------|---------|-----|---------------------|
| `product.product` | tracking="serial" | name | `[("tracking","=","serial")]` | 1 | Crear producto con tracking serial |
| `stock.lot` | name, product_id | name | `[]` | 0 | Se crean automaticamente en recepciones via `lot_name` |

---

## Datos transversales nuevos (complementarios)

### Modulos Enterprise — Verificacion de instalacion

| Modelo | Verificacion | Nota |
|--------|--------------|------|
| `ir.module.module` con name="helpdesk" y state="installed" | Helpdesk Enterprise instalado | Necesario para flujos TICKET_* |
| `ir.module.module` con name="quality_control" y state="installed" | Quality Control Enterprise instalado | Necesario para flujos QC_* |
| `ir.module.module` con name="planning" y state="installed" | Planning Enterprise instalado | Necesario para flujos PLANNING_* |
| `ir.module.module` con name="approvals" y state="installed" | Approvals Enterprise instalado | Necesario para flujos APPROVAL_* |
| `ir.module.module` con name="mrp" y state="installed" | Fabricacion instalado | Necesario para flujos MRP_* |
| `ir.module.module` con name="hr_holidays" y state="installed" | Ausencias instalado | Necesario para flujos LEAVE_* |
| `ir.module.module` con name="hr_expense" y state="installed" | Gastos instalado | Necesario para flujos EXPENSE_* |
| `ir.module.module` con name="hr_recruitment" y state="installed" | Reclutamiento instalado | Necesario para flujos APPLICANT_* |
| `ir.module.module` con name="fleet" y state="installed" | Flota instalado | Necesario para flujos VEHICLE_* |
| `ir.module.module` con name="maintenance" y state="installed" | Mantenimiento instalado | Necesario para flujos MAINTENANCE_* |
| `ir.module.module` con name="repair" y state="installed" | Reparacion instalado | Necesario para flujos REPAIR_* |

### Ejemplos JSON para nuevos flujos

**MRP_PRODUCE:**
```json
{
  "flow_id": "MRP_PRODUCE",
  "preconditions": [
    {
      "model": "mrp.production",
      "domain": "[(\"state\",\"in\",[\"confirmed\",\"progress\"])]",
      "fields": ["product_id", "product_qty", "state"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "stock.quant",
      "domain": "[(\"quantity\",\">\",0),(\"location_id.usage\",\"=\",\"internal\")]",
      "fields": ["product_id", "quantity"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**LEAVE_REQUEST_APPROVE:**
```json
{
  "flow_id": "LEAVE_REQUEST_APPROVE",
  "preconditions": [
    {
      "model": "hr.leave",
      "domain": "[(\"state\",\"=\",\"confirm\")]",
      "fields": ["employee_id", "holiday_status_id", "number_of_days"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**EXPENSE_POST:**
```json
{
  "flow_id": "EXPENSE_POST",
  "preconditions": [
    {
      "model": "hr.expense.sheet",
      "domain": "[(\"state\",\"=\",\"approve\")]",
      "fields": ["name", "employee_id", "total_amount"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "account.account",
      "domain": "[(\"account_type\",\"=\",\"expense\")]",
      "fields": ["code", "name"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**APPLICANT_HIRE:**
```json
{
  "flow_id": "APPLICANT_HIRE",
  "preconditions": [
    {
      "model": "hr.applicant",
      "domain": "[(\"active\",\"=\",True)]",
      "fields": ["partner_name", "job_id", "stage_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "hr.department",
      "domain": "[]",
      "fields": ["name"],
      "min_records": 1,
      "critical": false
    }
  ]
}
```

**REPAIR_DONE:**
```json
{
  "flow_id": "REPAIR_DONE",
  "preconditions": [
    {
      "model": "repair.order",
      "domain": "[(\"state\",\"=\",\"confirmed\")]",
      "fields": ["product_id", "state"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**MAINTENANCE_DONE:**
```json
{
  "flow_id": "MAINTENANCE_DONE",
  "preconditions": [
    {
      "model": "maintenance.request",
      "domain": "[(\"archive\",\"=\",False)]",
      "fields": ["name", "stage_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "maintenance.stage",
      "domain": "[(\"done\",\"=\",True)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**TICKET_CLOSE:**
```json
{
  "flow_id": "TICKET_CLOSE",
  "preconditions": [
    {
      "model": "helpdesk.ticket",
      "domain": "[(\"close_date\",\"=\",False)]",
      "fields": ["name", "team_id", "stage_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "helpdesk.stage",
      "domain": "[(\"fold\",\"=\",True)]",
      "fields": ["name"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**PLANNING_PUBLISH:**
```json
{
  "flow_id": "PLANNING_PUBLISH",
  "preconditions": [
    {
      "model": "planning.slot",
      "domain": "[(\"state\",\"=\",\"draft\")]",
      "fields": ["resource_id", "start_datetime", "end_datetime"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "hr.employee",
      "domain": "[(\"active\",\"=\",True),(\"work_email\",\"!=\",False)]",
      "fields": ["name", "work_email"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```

**APPROVAL_APPROVE:**
```json
{
  "flow_id": "APPROVAL_APPROVE",
  "preconditions": [
    {
      "model": "approval.request",
      "domain": "[(\"request_status\",\"=\",\"pending\")]",
      "fields": ["name", "category_id", "request_owner_id"],
      "min_records": 1,
      "critical": true
    },
    {
      "model": "approval.approver",
      "domain": "[(\"status\",\"!=\",\"approved\")]",
      "fields": ["user_id", "request_id"],
      "min_records": 1,
      "critical": true
    }
  ]
}
```
