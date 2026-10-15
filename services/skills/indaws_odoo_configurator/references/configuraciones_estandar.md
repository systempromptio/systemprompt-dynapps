# Playbook de Configuraciones Estandar Odoo

> Catalogo de configuraciones comunes por app. Cada receta incluye: que es, como se hace, verificacion y precauciones.
> Se amplia con la experiencia — empezamos con las configuraciones mas comunes.

---

## Ventas

### Anticipos (Down Payments)
- **Que**: Configurar porcentaje y cuenta contable de anticipos
- **Como**:
  - Settings > Sales > Invoicing > Down Payments
  - `sale.config.settings` → `deposit_default_product_id`
  - Producto de anticipo: verificar que tiene cuenta de ingresos correcta (NO usar la 700)
  - Cuenta recomendada: 438 (Anticipos de clientes)
- **Verificacion**: Crear SO → Crear factura de anticipo → Verificar asiento con cuenta 438
- **Precauciones**:
  - Si la cuenta de anticipos es incorrecta, los asientos van a la cuenta generica de ingresos
  - Verificar que el producto de anticipo tiene `type = 'service'`

### Tarifas (Pricelists)
- **Que**: Activar y configurar listas de precios
- **Como**:
  - Settings > Sales > Pricing > Pricelists
  - `res.config.settings` → `group_product_pricelist`
- **Verificacion**: Crear tarifa con descuento → Crear SO → Verificar precio calculado
- **Precauciones**:
  - Elegir bien "Multiple prices per product" vs "Advanced rules"
  - En multi-empresa, las tarifas son por empresa

### Unidades de Medida
- **Que**: Activar unidades de medida en ventas/compras
- **Como**:
  - Settings > Sales > Product Catalog > Units of Measure
  - `res.config.settings` → `group_uom`
- **Verificacion**: Verificar que productos permiten seleccionar UdM
- **Precauciones**:
  - Una vez activado y con datos, no desactivar

### Variantes de Producto
- **Que**: Activar atributos y variantes de producto
- **Como**:
  - Settings > Sales > Product Catalog > Variants
  - `res.config.settings` → `group_product_variant`
- **Verificacion**: Crear producto con atributo (ej: Talla) → Verificar variantes generadas
- **Precauciones**:
  - Elegir correctamente generacion de variantes: "Instantly", "Dynamically", "Never"
  - NO desactivar si ya hay variantes creadas

### Presupuestos con Fecha de Expiracion
- **Que**: Configurar dias por defecto de validez de presupuestos
- **Como**:
  - Settings > Sales > Quotations & Orders > Default Quotation Validity
  - `res.config.settings` → `quotation_validity_days`
- **Verificacion**: Crear SO → Verificar que validity_date se calcula automaticamente
- **Precauciones**:
  - Valor 0 = sin expiracion

### Firmas y Pago Online
- **Que**: Activar firma electronica y/o pago online en presupuestos
- **Como**:
  - Settings > Sales > Quotations & Orders > Online Signature / Online Payment
  - `res.config.settings` → `portal_confirmation_sign`, `portal_confirmation_pay`
- **Verificacion**: Enviar presupuesto por email → Abrir link portal → Verificar opciones
- **Precauciones**:
  - Pago online requiere proveedor de pago configurado (Stripe, PayPal, etc.)

---

## Contabilidad

### Localizacion Fiscal Espana
- **Que**: Configurar plan contable espanol y posiciones fiscales
- **Como**:
  - Instalar `l10n_es` → Seleccionar plan contable (PyME o Normal)
  - Settings > Accounting > Fiscal Localization
- **Verificacion**: Verificar que cuentas 100-999 existen, posiciones fiscales creadas
- **Precauciones**:
  - NUNCA cambiar plan contable una vez hay asientos creados
  - Verifactu: si `l10n_es_edi_verifactu` instalado, los asientos son inmutables

### Moneda Extranjera (Multi-Currency)
- **Que**: Activar gestion multi-moneda
- **Como**:
  - Settings > Accounting > Currencies > Multi-Currencies
  - `res.config.settings` → `group_multi_currency`
  - Activar monedas necesarias en Accounting > Configuration > Currencies
  - Configurar proveedor de tipos de cambio automatico
- **Verificacion**: Crear factura en moneda extranjera → Verificar tipo de cambio aplicado
- **Precauciones**:
  - Configurar cuentas de diferencia de cambio (ganancia/perdida)
  - Verificar diario de diferencias de cambio

### Email Saliente (Outgoing Mail)
- **Que**: Configurar servidor SMTP para envio de emails
- **Como**:
  - Settings > Technical > Outgoing Mail Servers (`ir.mail_server`)
  - Campos: smtp_host, smtp_port, smtp_user, smtp_pass, smtp_encryption
- **Verificacion**:
  1. Enviar email de prueba desde Settings > "Test Connection"
  2. Crear factura → Enviar por email → Verificar recepcion
  3. Comprobar `mail.mail` que no hay emails en estado `exception`
- **Precauciones**:
  - En test: NO configurar SMTP real — usar `mail.catchall.domain` vacio
  - Verificar SPF/DKIM del dominio del cliente antes de configurar
  - Gmail requiere "App Password" (no la contrasena normal)

### Email Entrante (Incoming Mail / Fetchmail)
- **Que**: Configurar recepcion de emails para alias
- **Como**:
  - Settings > Technical > Incoming Mail Servers (`fetchmail.server`)
  - Campos: server_type (imap/pop), server, port, user, password, is_ssl
  - Configurar `mail.catchall.domain` y `mail.catchall.alias`
- **Verificacion**: Enviar email al alias → Verificar que se crea registro en el modelo destino
- **Precauciones**:
  - IMAP preferido sobre POP3 (no borra del servidor)
  - Verificar que el alias esta mapeado al modelo correcto

### Posiciones Fiscales
- **Que**: Mapear impuestos y cuentas segun tipo de operacion
- **Como**:
  - Accounting > Configuration > Fiscal Positions
  - `account.fiscal.position`: name, auto_apply, country_id, tax_ids, account_ids
- **Verificacion**: Crear factura a partner con posicion fiscal → Verificar impuestos mapeados
- **Precauciones**:
  - auto_apply + country_id para aplicacion automatica por pais
  - Verificar que el mapeo cubre todos los impuestos necesarios

### Cuentas Analiticas
- **Que**: Activar contabilidad analitica
- **Como**:
  - Settings > Accounting > Analytics > Analytic Accounting
  - `res.config.settings` → `group_analytic_accounting`
- **Verificacion**: Crear cuenta analitica → Crear factura con analitica → Verificar lineas analiticas
- **Precauciones**:
  - Planes analiticos en Odoo 17+ reemplazan etiquetas analiticas

### Diarios Contables
- **Que**: Crear/configurar diarios (ventas, compras, banco, efectivo)
- **Como**:
  - Accounting > Configuration > Journals (`account.journal`)
  - Campos: name, type (sale/purchase/bank/cash/general), code, default_account_id
- **Verificacion**: Crear asiento en el diario → Verificar secuencia y cuenta por defecto
- **Precauciones**:
  - Codigo del diario: maximo 5 caracteres, unico
  - No eliminar diarios con asientos

---

## Inventario

### Multi-Almacen
- **Que**: Activar gestion multi-almacen
- **Como**:
  - Settings > Inventory > Warehouse > Storage Locations
  - `res.config.settings` → `group_stock_multi_locations`
- **Verificacion**: Crear segundo almacen → Verificar rutas y locations creadas
- **Precauciones**:
  - Activar locations ANTES de crear almacenes adicionales
  - Rutas inter-almacen se crean automaticamente

### Trazabilidad (Lotes / Series)
- **Que**: Activar tracking por lote o numero de serie
- **Como**:
  - Settings > Inventory > Traceability > Lots & Serial Numbers
  - `res.config.settings` → `group_stock_lot` (lotes) / `group_stock_production_lot` (series)
- **Verificacion**: Crear producto storable con tracking → Hacer recepcion → Verificar lote asignado
- **Precauciones**:
  - `is_storable=True` en Odoo 18+, NO `type='product'`
  - No se puede cambiar tracking de un producto con stock existente

### Rutas y Reglas de Abastecimiento
- **Que**: Configurar reglas de reabastecimiento, dropship, MTO
- **Como**:
  - Settings > Inventory > Warehouse > Multi-Step Routes
  - `res.config.settings` → `group_stock_adv_location`
  - Configurar rutas en Inventory > Configuration > Routes
- **Verificacion**: Confirmar SO con ruta MTO → Verificar que se genera PO automaticamente
- **Precauciones**:
  - MTO requiere proveedor configurado en producto
  - Dropship requiere modulo `stock_dropshipping`

### Paquetes y Embalaje
- **Que**: Activar gestion de paquetes para envios
- **Como**:
  - Settings > Inventory > Operations > Packages
  - `res.config.settings` → `group_stock_tracking_lot`
- **Verificacion**: En recepcion/envio → Asignar paquete → Verificar contenido
- **Precauciones**:
  - No confundir paquetes con lotes/series

---

## Compras

### Acuerdos de Compra
- **Que**: Activar acuerdos marco y licitaciones
- **Como**:
  - Settings > Purchase > Orders > Purchase Agreements
  - `res.config.settings` → `group_purchase_agreement` (no disponible en todas las versiones, verificar)
  - Modulo `purchase_requisition` puede ser necesario
- **Verificacion**: Crear acuerdo de compra → Seleccionar proveedores → Confirmar
- **Precauciones**:
  - Tipos: "Blanket Order" (acuerdo marco) vs "Purchase Tender" (licitacion)

### Aprobacion de Compras
- **Que**: Activar flujo de aprobacion para ordenes de compra
- **Como**:
  - Settings > Purchase > Orders > Purchase Order Approval
  - `res.config.settings` → `po_double_validation` = 'two_step'
  - Configurar monto minimo: `po_double_validation_amount`
- **Verificacion**: Crear PO por encima del monto → Verificar que requiere aprobacion
- **Precauciones**:
  - Solo aplica a POs que superan el monto configurado

---

## CRM

### Pipeline y Etapas
- **Que**: Configurar etapas del pipeline de ventas
- **Como**:
  - CRM > Configuration > Stages (`crm.stage`)
  - Campos: name, sequence, team_id (null = compartida), is_won
- **Verificacion**: Crear oportunidad → Mover por etapas → Verificar flujo
- **Precauciones**:
  - Si team_id es null, la etapa es compartida entre todos los equipos
  - No eliminar etapas con oportunidades

### Equipos de Ventas
- **Que**: Configurar equipos de ventas con miembros y alias
- **Como**:
  - CRM > Configuration > Sales Teams (`crm.team`)
  - Campos: name, user_id (lider), member_ids, alias_name
- **Verificacion**: Crear equipo → Asignar miembros → Verificar que oportunidades se asignan
- **Precauciones**:
  - El alias de email permite crear oportunidades automaticamente

### Puntuacion de Leads (Lead Scoring)
- **Que**: Activar scoring automatico de leads
- **Como**:
  - Settings > CRM > Lead Scoring
  - Configurar reglas en CRM > Configuration > Lead Scoring Rules
- **Verificacion**: Crear lead con criterios de scoring → Verificar puntuacion asignada
- **Precauciones**:
  - Requiere suficientes datos historicos para ser util

---

## Recursos Humanos

### Tipos de Ausencia
- **Que**: Configurar tipos de ausencia (vacaciones, bajas, etc.)
- **Como**:
  - Time Off > Configuration > Leave Types (`hr.leave.type`)
  - Campos: name, allocation_type, requires_allocation, responsible_ids
- **Verificacion**: Crear tipo → Asignar allocation → Solicitar ausencia → Verificar
- **Precauciones**:
  - `requires_allocation` determina si necesita asignacion previa
  - Verificar politica de aprobacion (hr_responsible, manager, both)

### Estructura Departamental
- **Que**: Crear departamentos y asignar managers
- **Como**:
  - Employees > Configuration > Departments (`hr.department`)
  - Campos: name, parent_id, manager_id, company_id
- **Verificacion**: Crear departamento → Asignar empleados → Verificar organigrama
- **Precauciones**:
  - manager_id afecta flujos de aprobacion (ausencias, gastos)

---

## Website / eCommerce

### Publicar Productos en Web
- **Que**: Activar venta online de productos
- **Como**:
  - Website > eCommerce > Products
  - `product.template` → `website_published = True`
  - Configurar categorias web: `product.public.category`
- **Verificacion**: Publicar producto → Acceder a la tienda → Verificar que aparece
- **Precauciones**:
  - Verificar precios con impuestos incluidos/excluidos segun configuracion
  - Imagenes: recomendado 1024x1024px minimo

---

## Fabricacion (MRP)

### Lista de Materiales (BOM)
- **Que**: Crear lista de materiales para productos fabricados
- **Como**:
  - Manufacturing > Configuration > Bill of Materials (`mrp.bom`)
  - Campos: product_tmpl_id, bom_line_ids, type (normal/phantom), product_qty
- **Verificacion**: Crear BOM → Crear orden de fabricacion → Verificar componentes
- **Precauciones**:
  - Phantom BOM: se descompone automaticamente en SO/deliveries
  - Verificar UdM de componentes vs producto final

### Centros de Trabajo (Work Centers)
- **Que**: Configurar centros de trabajo para ordenes de trabajo
- **Como**:
  - Manufacturing > Configuration > Work Centers (`mrp.workcenter`)
  - Campos: name, capacity, time_start, time_stop, costs_hour
- **Verificacion**: Crear centro → Asociar a operacion en BOM → Verificar orden de trabajo
- **Precauciones**:
  - Activar "Work Orders" en Settings > Manufacturing primero

---

## Configuracion General

### Multi-Empresa
- **Que**: Activar y configurar multi-empresa
- **Como**:
  - Settings > Companies > Manage multiple companies
  - Crear empresas en Settings > Companies (`res.company`)
  - Asignar empresas a usuarios
- **Verificacion**: Cambiar de empresa en selector → Verificar datos segregados
- **Precauciones**:
  - CRITICO: Configurar reglas inter-empresa cuidadosamente
  - Productos y contactos pueden ser compartidos o por empresa
  - Verificar secuencias y diarios por empresa

### Idiomas
- **Que**: Instalar y configurar idiomas adicionales
- **Como**:
  - Settings > Translations > Languages
  - `res.lang`: activar idiomas necesarios
  - Cargar traducciones: Settings > Translations > Import / Export
- **Verificacion**: Cambiar idioma de usuario → Verificar interfaz traducida
- **Precauciones**:
  - Algunos modulos custom pueden no tener traducciones
  - Traducciones de Studio fields requieren exportar/importar .po

### Usuarios y Permisos
- **Que**: Crear usuarios y asignar grupos de permisos
- **Como**:
  - Settings > Users & Companies > Users (`res.users`)
  - Asignar grupos por app en la ficha del usuario
- **Verificacion**: Login como nuevo usuario → Verificar acceso a menus y datos
- **Precauciones**:
  - NUNCA dar acceso Administration/Settings sin necesidad
  - Verificar reglas de registro (record rules) si hay restricciones custom
  - En produccion: crear usuario test primero

### Datos de la Empresa
- **Que**: Configurar datos fiscales, logo, direccion de la empresa
- **Como**:
  - Settings > Companies > Update Info (`res.company`)
  - Campos: name, street, city, zip, country_id, vat, logo, email, phone
- **Verificacion**: Verificar que aparecen en facturas/presupuestos
- **Precauciones**:
  - Logo: formato PNG con fondo transparente recomendado
  - VAT: formato correcto segun pais (ES: ESB12345678)
