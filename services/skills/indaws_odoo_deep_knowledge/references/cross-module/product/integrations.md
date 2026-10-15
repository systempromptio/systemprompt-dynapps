# Integraciones — Producto, Variantes y UoM con otros módulos

Cómo `product.template`/`product.product`/atributos/UoM interactúan con sale, purchase, stock, account, MRP, POS, website, y módulos especializados (delivery, repair, helpdesk).

---

## 1) Sale (sale, sale_management)

### sale.order.line — campos relevantes

| Campo | Tipo | Notas |
|---|---|---|
| `product_id` | M2o product.product, required (salvo display_type) | Variante específica |
| `product_template_id` | M2o computed | Para search/filter UI |
| `product_uom` | M2o uom.uom, store, compute, readonly=False | Default `product_id.uom_id`. Domain limitado a misma categoría. |
| `product_uom_qty` | Float compute, readonly=False | Qty en `product_uom` |
| `product_uom_category_id` | M2o related | Para domain del selector |
| `product_no_variant_attribute_value_ids` | M2M PTAV | Para atributos `no_variant` elegidos en config |
| `product_custom_attribute_value_ids` | O2M product.attribute.custom.value | Para `is_custom` text inputs |
| `product_packaging_id` | M2o product.packaging (v18) | Editable, recalcula qty |
| `product_packaging_qty` | Float | Qty en packagings |

### Configurador de producto en sale (v18)

- Producto con `has_configurable_attributes=True` → al añadirlo a SO se abre el **product configurator wizard** (`sale.product.configurator`).
- El wizard:
  - Lista PTAL (excluyendo `no_variant` cuando solo se elige variante).
  - Para `always`/`dynamic` permite elegir una PTAV por línea.
  - Para `no_variant` permite seleccionar values (que NO crean variante pero se guardan en línea SO).
  - Para `is_custom` permite texto libre.
- Si `create_variant='dynamic'` y la combinación no existe → crea la `product.product` al confirmar.

### Cómo se aplica `price_extra`

`sale.order.line._compute_price_unit`:

1. Resuelve precio base con pricelist sobre `product.product`.
2. Si pricelist usa "based on Sales Price": parte de `product.lst_price` (que YA incluye `price_extra`).
3. Si pricelist usa "based on Cost" o fórmula sin variant extra → debe sumarse explícitamente con `_get_attributes_extra_price`.

**Bug común** (→ `gotchas.md` PLSG13): pricelist con fórmula "100% del coste" ignora `price_extra`. Para incluirlo: pricelist "based on Sales Price" + tipo `percentage`. Caso percentage-sobre-list_price en PLSG8; cómo se acumula `price_extra` en PRG8.

### Sale: dual UoM en línea

Si activado `uom.group_uom`:

- Selector `product_uom` en la línea, limitado a categoría del producto. Domain filtra el dropdown pero NO valida al escribir vía ORM/import (→ `gotchas.md` UOG13).
- Cambiar UoM dispara `_compute_price_unit` que **convierte el precio** vía `uom._compute_price`. Pricelist re-resuelta en la nueva UoM (→ UOG23). Contraste con la factura, donde el mismo cambio NO recalcula precio (→ UOG14).
- `qty_delivered` (entregada) se mantiene en `product_uom` de la línea (no en stock UoM). Stock UoM solo en `stock.move`.

### Combos (type='combo')

- Línea SO con `product_id.type='combo'` muestra picker de combo items.
- Al confirmar: crea N líneas hijas con `combo_item_id` set, precios en cero (excepto extra_price del combo item).
- `is_combo_parent` (Boolean computed) en SO line = True para la línea madre.
- Total = price del combo padre + extra_prices de items.
- **Reportes/queries** que sumen precios de SO line doble-cuentan si no excluyes hijas o padre (→ `gotchas.md` PRG11). Filtrar por `combo_item_id != False` o `not is_combo_parent`.

### sale_loyalty (descuentos)

- `coupon_program` aplica descuentos creando líneas SO adicionales con `reward_id`.
- Estas líneas NO heredan variants ni UoM correctamente; suelen usar producto "Discount" con UoM Units.
- ANG13 (analytic): tampoco heredan `analytic_distribution`. Ver `analytic/gotchas.md`.

### sale_management — vistas y catálogo

- `product.catalog.mixin` añade vista "Catálogo" al SO. Filtros por atributos visibles aquí.
- Filtro por PAV en catálogo: `[('product_template_attribute_value_ids.product_attribute_value_id', '=', pav_id)]`.

---

## 2) Purchase

### purchase.order.line

| Campo | Notas |
|---|---|
| `product_uom` | Default = `product_id.uom_po_id or uom_id`. Domain = misma categoría. |
| `product_qty` | Qty en `product_uom` (ojo: en PO se llama `product_qty`, no `product_uom_qty`) |
| `product_uom_qty` | **Computed/store**: qty convertida a `product.uom_id` (stock UoM). Es el campo que stock usa. |
| `price_unit` | Precio en `product_uom`. Computed desde `product.supplierinfo`. |

### Triple conversión: vendor → PO line → stock

```
Vendor cotiza:    seller.price € / seller.product_uom
PO line:          line.product_uom (puede diferir de seller, ej. compras en otra UoM)
Stock:            stock.move.product_uom_qty (en product.uom_id de stock)
```

`_compute_price_unit`:

```python
po_line_uom = line.product_uom or line.product_id.uom_po_id
price_unit = seller.product_uom._compute_price(seller.price, po_line_uom)
```

Acumula rounding en cada conversión (→ `gotchas.md` UOG12). Para minimizar: vendor + PO line en misma UoM cuando sea posible.

### Recepción → stock move

Al confirmar PO:

- `stock.move` creado con `product_uom_qty = line.product_uom_qty` (en stock UoM).
- `line.product_uom_qty` (PO) ≠ `move.product_uom_qty` (stock). El primero es **qty convertida a stock UoM** del PO line; el segundo es la demanda al stock.
- Al recibir: `stock.move.quantity` (v18) registra qty real entregada.
- Backorder: si `quantity < product_uom_qty`, se crea backorder con el resto.

### purchase.order.line.qty_received

Computed desde `move_ids.quantity` y `invoice_lines.quantity` (en facturas) — todo convertido a `line.product_uom`:

```python
qty += inv_line.product_uom_id._compute_quantity(inv_line.quantity, line.product_uom)
```

Mezcla quantities de stock UoM (move) + invoice UoM (puede ser otra) → todo en PO line UoM. Múltiples conversiones, propenso a rounding errors visibles como "0.001 unidad pendiente" (→ `gotchas.md` UOG12).

### supplierinfo selección

`product._select_seller(partner_id, quantity, date, uom_id, params)`:

1. Filtra `seller_ids` por `partner_id` (si dado) y `date_start/date_end`.
2. Filtra por `product_uom._compute_quantity(quantity, seller.product_uom) >= min_qty`.
3. Ordena por `sequence`, luego `min_qty desc`, devuelve el primero (sequence gana sobre precio, → `gotchas.md` SIG1).

→ Cambiar UoM de la PO line puede cambiar el vendor seleccionado porque min_qty se evalúa en seller UoM (→ SIG2). `supplierinfo.product_uom` es related a `uom_po_id`, no editable por seller (→ SIG3). Cambios en supplierinfo no son retroactivos a POs confirmadas (→ SIG5).

---

## 3) Stock

### stock.move

```python
product_id          # M2o product.product
product_uom         # M2o uom.uom — UoM de la demanda
product_uom_qty     # Float — qty demanda
quantity            # Float — qty hecha (v18 unificó quantity_done)
```

**Reglas**:

- `product_uom` puede diferir de `product.uom_id` (la "stock UoM"). Solo restricción: misma categoría (en v19, mismo árbol `relative_uom_id`, → `gotchas.md` UOG24).
- Al evaluar disponibilidad/reserva, se convierte siempre a `product.uom_id`. Conversión por defecto redondea `UP` (→ UOG4).
- `state` (draft/waiting/confirmed/assigned/done) se actualiza según `quantity` vs `product_uom_qty`.

### Dual UoM en move

En PO con `uom_po_id != uom_id`:

```
PO line product_uom = Box (uom_po_id)
PO line product_qty = 5
PO line product_uom_qty = 60   ← convertido a stock UoM (Units)

stock.move product_uom = Box   ← demanda hereda PO line UoM
stock.move product_uom_qty = 5  ← qty en move's UoM (Box)
```

→ Stock UI muestra "5 Box". `Forecast` y `Reservation` operan internamente en Units. Hay un toggle en kanban inventory para ver stock en UoM del producto vs UoM original del move.

### stock.quant — siempre en `product.uom_id`

`stock.quant.product_uom_id` related al producto, readonly. Las cantidades en stock se almacenan SIEMPRE en stock UoM del producto. Conversiones se hacen on-the-fly al mostrar/operar.

### stock.move.line (reservation lines)

- `product_uom_id` puede diferir del move (al unir varios moves en un picking).
- `qty_done` (v17) → `quantity` (v18) — en `product_uom_id` de la move.line.

### Cambio de UoM stock en producto existente

Tema delicado: cambiar `product.template.uom_id` cuando ya hay quants/moves:

- Constraint blanda: warning UI, pero **NO bloquea** la operación.
- Stock quants quedan con `product_uom_id` apuntando a la **antigua** UoM (no se recalculan).
- Moves históricos siguen en UoM antigua.
- Resultado: stock muestra cantidades coherentes (related al producto, refresh OK) pero hay inconsistencia silenciosa en informes con joins.
- **Buena práctica**: nunca cambiar `uom_id` de un producto con histórico. Crear producto nuevo. Misma lógica que cambiar `factor` de una UoM en uso (→ `gotchas.md` UOG3) y que `uom_po_id` se resetee al cambiar de categoría (→ UOG10).

### Packaging en stock (v18)

`stock.move.product_packaging_id`: si la PO/SO usaba packaging, se hereda al move. Al recibir, el barcode del packaging se puede escanear y se decodifica en qty.

`stock.move._check_qty_per_packaging()`: si `product_packaging_id` set, `product_uom_qty` debe ser múltiplo de `packaging.qty`. UserError si no. El helper de redondeo a múltiplo (`uom._check_qty`) puede devolver qty distinto al input por epsilon de floats (→ `gotchas.md` UOG22). Packaging es ortogonal a la UoM (→ UOG11). En v19 los packagings dejan de ser `product.packaging` y pasan a `product.uom` / `product.template.uom_ids` (ver sección 17).

### Inventory adjustments

`stock.quant._apply_inventory()`: ajusta quants en `product_uom_id` del quant (= stock UoM). UI permite introducir en otra UoM, convierte al guardar.

### Tracking lots/serials

`stock.lot.product_uom_id` related al producto. Las cantidades en `stock.lot.product_qty` (computed) siempre en stock UoM, independiente de la UoM de movimiento.

### Kits (BoM `type='phantom'`)

- Producto kit en SO: `product.template.type='consu'` o `'service'`, `categ_id.is_phantom_categ` o BoM `type='phantom'`.
- Al confirmar SO: el procurement **explota** la BoM phantom en stock moves para los componentes.
- **Variantes del kit**: si el kit tiene atributos, cada variante puede tener BoM phantom propia. Si no, BoM aplica a todas.
- **UoM del kit vs componentes**: cada `mrp.bom.line.product_uom_id` independiente. Conversión automática (→ `gotchas.md` UOG1: solo dentro de misma categoría / mismo árbol en v19).
- Un servicio (`type='service'`) no admite BoM (→ `gotchas.md` SRVG5); la única "fabricación" posible es subcontratación.

---

## 4) Account (Facturación)

### account.move.line

```python
product_id          # M2o product.product
product_uom_id      # M2o uom.uom — UoM de la línea de factura
quantity            # Float en product_uom_id
price_unit          # Precio en product_uom_id
```

### Cuentas contables: cascada producto → categoría → empresa

```
1. product.property_account_income_id / property_account_expense_id  (per-product)
   └→ 2. product.category.property_account_income_categ_id / expense_categ_id
       └→ 3. Default journal account_id
```

Si producto y categoría tienen vacíos los property fields, factura cae al default del journal. La resolución recorre `parent_id` de la categoría sin fallback al root "All" (→ `gotchas.md` PRG9, CTG4).

**company_dependent=True** en property fields → cada empresa puede tener cuentas distintas para el mismo producto/categoría. En multi-company SAR, esto es lo normal. Mover `categ_id` de un producto sin override propio cambia las cuentas usadas (→ PRG9).

### UoM en factura desde SO/PO

- Factura desde SO → línea factura hereda `sale_order_line.product_uom`.
- Factura desde PO → línea factura hereda `purchase_order_line.product_uom`.
- Factura manual con producto → default `product_id.uom_id`.

### Cambio de UoM en línea factura — NO recalcula price_unit

Bug abierto y conocido (→ `gotchas.md` UOG14): en `account.move.line`, cambiar `product_uom_id` NO dispara conversión de `price_unit`. El usuario debe ajustar manualmente. Contraste con la SO line, que SÍ re-convierte (→ UOG23). Verificado en `odoo18/addons/account/models/account_move_line.py::_compute_totals` (`_compute_price_unit` depende de `product_id`/`product_uom_id` pero `_compute_product_uom_id`, líneas 807-813, solo asigna UoM desde el producto, sin re-convertir el precio existente). Constraint en línea 1350-1353 solo valida que la UoM sea de la misma categoría que el producto, no el precio.

Side-effect: facturas creadas vía wizards (downpayment, refund) pueden quedar con UoM diferente del precio si no se ha verificado.

### SII / EDI / facturación electrónica

- AEAT (España): UoM de la línea va al campo "Unidad de Medida" del XML.
- Solo acepta valores estándar en algunas claves (ej. para hidrocarburos: "litros"; para gas: "m³").
- Si el producto usa UoM custom (ej. "Pack of 12"), el módulo l10n_es_edi_sii hace mapping a UoM estándar — pero **no** sin override si UoM custom no está en su map (→ `gotchas.md` UOG15).
- OCA `l10n_es_aeat_sii_oca` tiene mapping más amplio.

### Decimal precision en valoración

`Account` (decimal.precision) y `Product Price` controlan el redondeo en cálculo de:

- `move.amount_total` = suma de `line.price_subtotal` redondeado.
- `move.line.price_subtotal` = `quantity * price_unit * (1 - discount/100)` redondeado a `Product Price` o `Account`.

Diferencia entre suma de líneas vs total → casi siempre rounding. Para minimizar usar `decimal.precision('Account')=4` en producción.

---

## 5) MRP (Manufacturing)

### mrp.bom

| Campo | Notas |
|---|---|
| `product_tmpl_id` | M2o template (obligatorio) |
| `product_id` | M2o product.product (opcional — si vacío, BoM para todas las variantes) |
| `product_uom_id` | UoM del producto fabricado |
| `product_qty` | Qty fabricada por batch |
| `bom_line_ids` | Líneas con componentes |
| `byproduct_ids` | Sub-productos generados |
| `operation_ids` | Operaciones (workorders) si MRP Workorder instalado |

### mrp.bom.line

| Campo | Notas |
|---|---|
| `product_id` | M2o product.product (componente) |
| `product_uom_id` | UoM del componente — independiente del producto |
| `product_qty` | Qty consumida por batch (en `product_uom_id`) |
| `bom_product_template_attribute_value_ids` | M2M PTAV — **condicionante por variante** |

### Línea condicionada por variante

`bom_product_template_attribute_value_ids` permite definir "esta línea solo se aplica si la variante producida tiene estos PTAV". Ejemplo:

- Template "Camiseta" con atributo "Color" y values [Rojo, Azul].
- BoM con 1 línea "Tela 1m²" sin condición (aplica a todas).
- 1 línea "Tinta roja" con `bom_product_template_attribute_value_ids = [PTAV(Rojo)]`.
- 1 línea "Tinta azul" con `bom_product_template_attribute_value_ids = [PTAV(Azul)]`.

→ Al fabricar Camiseta Roja, solo se consume tela + tinta roja.

**Limitación**: no se pueden encadenar condiciones (AND/OR complejas). Cada línea es "estas PTAV obligatorias".

### Workcenter capacity por producto

`mrp.workcenter.capacity` (M2o workcenter, M2o product_id, time_efficiency) — capacity expresada por variante específica, no por template.

### Variantes en MO

- `mrp.production.product_id` (M2o product.product, required).
- Si BoM no especifica `product_id`, el MO aplica BoM del template a esa variante.
- Si BoM especifica `product_id`, MO debe usar esa variante.

### UoM en operaciones

- `mrp.routing.workcenter.time_cycle_manual` en horas.
- `mrp.workorder.duration_expected` en minutos.
- Conversiones hardcoded en el module (no parametrizables vía UoM).

---

## 6) POS (Point of Sale)

### Variantes en POS — limitación

- POS NO tiene configurador de variantes nativo.
- Cada `product.product` aparece como producto individual en pantalla.
- Búsqueda por barcode → encuentra variante específica.
- **No hay UI para "elige talla S/M/L"** en POS.

Workarounds:

- Cada variante con barcode/imagen propios → cajero busca por barcode.
- Módulo OCA `pos_product_variants` (v15-v18) añade picker.
- Custom: módulo "POS Combos" (v18 nativo) parcialmente cubre esto.

### Combos en POS (v18 nativo)

- `product.template.type='combo'` con `combo_ids` set funciona en POS.
- Cajero añade combo → modal pide elegir items.
- POS resuelve el descuento de combo automáticamente.
- En cierre: cada item del combo aparece como línea separada en el journal entry (de POS session).

### UoM en POS

- `pos.order.line.product_uom_id` related, readonly en UI (→ `gotchas.md` UOG19).
- Productos con UoM = "kg" (balanza): integración vía `iot.box` con balanza, qty viene de la báscula.
- POS NO soporta cambio de UoM en línea ni packagings (→ UOG19).
- En reportes POS: qty siempre en stock UoM.

### Atributos en POS

- `display_type='multi'` con `create_variant='no_variant'` SÍ funciona — es la forma de implementar "Toppings": "Pizza Margarita + extra queso + extra olivas".
- Se selecciona en modal al añadir el producto.
- Cada `is_custom=True` value permite texto libre.

---

## 7) Website / eCommerce

### Variantes en eCommerce

- `website_sale` añade vista de configurador en página de producto.
- PTAL con `display_type='color'` → swatches color (rectangulos clickables).
- `pills` → botones tipo "S, M, L".
- `radio` → radio buttons clásicos.
- `select` → dropdown.
- `multi` → checkboxes (sin variante, solo opciones).

### Filtros por atributo en /shop

- Sidebar de `/shop` muestra atributos como filtros.
- Configurable por atributo: `product.attribute.visibility='visible'` (campo en website_sale).
- Filtros usan domain en `product_template_attribute_value_ids.product_attribute_value_id`.

### Imagen de variante en web

- Si variante tiene `image_variant_1920` set → se muestra.
- Sino, fallback a `template.image_1920`.
- En swatches color: si PAV tiene `image` → se usa; si tiene `html_color` → cuadrado coloreado.

### Public pricelist

- `website.pricelist_id` define la pricelist mostrada al visitante anónimo.
- Para mostrar `price_extra` correctamente: pricelist debe ser "based on Sales Price".

### Compatibilidad combo + website

- v18 los combos funcionan en eCommerce con configurador modal.
- Antes (v17 sale_optional_products) era distinto — los optional products eran sugeridos pero no obligados.

### accessory_product_ids / alternative_product_ids

- `product.template.accessory_product_ids` (cross-sell en website_sale): productos sugeridos al añadir al carrito.
- `alternative_product_ids` (substitutes): productos mostrados en página del producto.
- Ambos M2M a template, no a variante específica.

---

## 8) Delivery (Shipping)

### product.template.type='service' + carrier integration

- `delivery.carrier.product_id` (M2o product.product) — producto que se factura como gasto de envío.
- UoM del carrier product = Units típicamente.
- Pricing por carrier: peso (`weight`) y volumen (`volume`) del producto enviado se computan en delivery → tarifa.

### Peso/volumen propagación

- `product.template.weight/volume` → `product.product.weight/volume` (override per variant).
- `stock.picking.weight` = sum de `move_id.product_id.weight * quantity`.
- Conversión UoM: `weight_uom_id` global vía `ir.config_parameter` `product.weight_in_lbs` (default kg).

### Variantes y delivery

- Cada variante puede tener peso/volumen propio.
- Carriers (Correos, UPS, DHL) reciben peso real de variantes en el manifiesto.
- Si variante tiene weight=0 (no rellenado) → delivery puede fallar o usar default.

---

## 9) Repair (Reparaciones)

### repair.order / repair.line

- `repair.line.product_id` (M2o product.product), `product_uom_id` (M2o uom).
- Componentes consumidos en reparación. UoM independiente del producto.
- Variantes funcionan normalmente.

### repair v18 → integración con stock unificada

- Stock moves creados al consumir/devolver piezas.
- UoM del repair.line se mantiene; conversión a stock UoM en la move.

---

## 10) Helpdesk (Enterprise)

### Productos en tickets (refunds, replacements)

- `helpdesk.ticket.product_id` (M2o product.product) — para gestión RMA.
- Replacements crean stock moves o invoices según workflow.
- No hay configurador de variantes en ticket — variante específica obligatoria.

---

## 11) Subscription (Enterprise)

### sale.order.line en subscription

- Suscripciones renovables crean SOs recurrentes.
- Cada SO line usa una variante específica.
- Cambios de variante en plan no re-generan SOs antiguos.
- `recurrence_id` (period) y `subscription_state` añaden lifecycle.

---

## 12) Project (sale_project)

### Servicios → tareas/proyectos

- `product.template.type='service'`, `service_tracking='task_in_project'` o `'project_only'`. Requiere `sale_project`/`sale_timesheet` (→ `gotchas.md` SRVG1); cambiar `service_tracking` no es retroactivo (→ SRVG2); `project_only` crea proyecto entero, no solo task (→ SRVG6).
- Al confirmar SO con producto service → crea `project.task` o `project.project`.
- UoM = Hour habitualmente.
- `service_policy='delivered_timesheet'` → qty entregada viene de `account.analytic.line.unit_amount` (requiere `sale_timesheet`, → SRVG3); `delivered_milestones` usa `milestone_ids` y requiere `sale_project` (→ SRVG4).

### Conversión Hour ↔ Day

- UoM Hour es reference; Day = 8 hours (factor=1/8, configurable).
- Timesheets en hours; sale lines pueden ser en Days.
- Bug (→ `gotchas.md` UOG20): empresas con jornada 7.5h o 6h dejan UoM Day=8h y todo cuadra **mal**. Crear UoM custom (NO cambiar el factor de la Day estándar, → UOG3). `product_uom_hour` está protegida contra borrado y, con `hr_timesheet`, también contra archive (→ UOG8, UOG9).

---

## 13) Inventory / Analytic

### Variantes y analytic distribution

- `product.product.analytic_distribution_model_ids` define reglas de distribución por variante.
- Pero `_get_analytic_distribution` en stock.move devuelve `{}` por defecto (sale_stock override añade desde SO line).
- Manual moves o MRP components: distribución vacía.

Ver `analytic/gotchas.md` ANG10.

---

## 14) Inventory valuation (AVCO/FIFO/Standard)

### Variantes — valoración independiente

- `product.category.property_cost_method` (`'standard'`, `'fifo'`, `'average'`) aplica a todas las variantes del template (categoría es del template).
- `product.category.property_cost_method` es de la categoría, no del producto; cambiar Standard/FIFO/AVCO afecta a toda la categoría (→ `gotchas.md` CTG1). En `product.template`, `cost_method`/`valuation` son related read-only a la categoría (→ CTG10).
- `standard_price` es **company_dependent en `product.product`** → cada variante tiene coste propio por empresa; en el template es computed desde la primera variante (→ PRG3).
- AVCO: coste promedio por variante (no por template), calculado vía `stock.move.value` y `stock.valuation.layer` (SVL).
- FIFO: layers per variante.
- Cambiar `categ_id` de un producto con stock dispara reevaluación masiva de SVLs (→ CTG2).

### Reevaluación

- `wizard.product.update.standard.price` opera en variant.
- Cambio masivo de templates → escribe en variantes (via _inherits). Cuidado: `name` en variant escribe en el template (→ `gotchas.md` PRG2).
- En multi-company, hacer en cada empresa por separado (`standard_price` es company_dependent, → PRG3).

---

## 15) Indaws / OCA — extensiones comunes

### Indaws

- `indaws_product` y `indaws_product_extension` añaden:
  - Campos custom (multi-supplier mejorado, alias).
  - `uom_sale_id` (M2o uom.uom) — UoM de venta independiente de stock (parche a la limitación CE).
- `indaws_l10n_es_*` mapean UoMs a códigos AEAT.

### OCA

- `product_variant_default_code` — generación automática de `default_code` por variante.
- `product_variant_attribute_name_manager` — variantes con sufijo automático.
- `product_supplierinfo_for_customer` — supplierinfo para clientes (lista de precios B2B).
- `purchase_order_secondary_unit` — segunda UoM en PO line (qty principal + qty secundaria).

---

## 16) Datos maestros / import

### Importación productos con variantes

Patrón estándar:

```csv
name,type,attribute_line_ids/attribute_id/id,attribute_line_ids/value_ids/id
Camiseta,consu,product.product_attribute_talla,product.product_attribute_value_S
Camiseta,consu,product.product_attribute_talla,product.product_attribute_value_M
```

Odoo agrupa por `name` o `External ID` y crea la PTAL con M2M de values.

**Gotcha**: si importas en CE sin variants module activo, los atributos pasan a `no_variant`. Activar `product.group_product_variant`. Nota: el `type='storable'` de v17 ya no existe (→ `gotchas.md` PRG13); en v18+ importar productos de inventario requiere `is_storable=True`, no `type`.

### Importación con UoMs

- `uom_id/id` debe ser external ID (`uom.product_uom_kgm`).
- `uom_po_id/id` igual.
- Si UoMs en categorías distintas → `uom_po_id` se resetea silenciosamente (→ `gotchas.md` UOG10).
- Barcodes duplicados (entre productos y, en v18, packaging / en v19 `product.uom`) fallan por uno y abortan el resto del lote (→ PRG6). Pre-deduplicar.

---

## 17) Cambios v19 que afectan a las integraciones de producto

Verificado contra `odoo19/addons`. Detalle conceptual en `v19_changes.md`; aquí solo el impacto cross-module.

### UoM: split `factor` ↔ árbol `relative_uom_id` (→ `gotchas.md` UOG24)

- `uom.uom` elimina `category_id` y pasa a árbol (`_parent_name='relative_uom_id'`, `product/models/uom_uom.py:20`, `UoM._onchange_rounding`; `_order='sequence, relative_uom_id, id'`, línea 22).
- Nuevos campos: `relative_factor` (Float, editable, `uom/models/uom_uom.py:36-40`, `UomUom.relative_factor`, v19) y `relative_uom_id` (M2o self, "Reference Unit", `uom/models/uom_uom.py:41`, `UomUom.relative_uom_id`). `related_uom_ids` es el O2M inverso (`uom/models/uom_uom.py:42`, `UomUom.related_uom_ids`).
- `factor` ya **NO se edita**: es computed recursivo `relative_factor * relative_uom_id.factor` (el campo, `uom/models/uom_uom.py:43`, `UomUom.factor`; el cómputo, `uom/models/uom_uom.py:69-75`, `UomUom._compute_factor`, v19 — `store=True, recursive=True`).
- Convertibilidad: dos UoMs son convertibles si comparten raíz del árbol. `_compute_quantity` (`uom/models/uom_uom.py:147-176`, `UomUom._compute_quantity`, v19) solo multiplica/divide por `factor`; el docstring (`uom/models/uom_uom.py:158-160`, `UomUom._compute_quantity`) todavía menciona "category" pero es texto obsoleto.
- **Impacto cross-module**: cualquier override que comparase `uom.category_id == other.category_id` (en sale/purchase/stock domains o validaciones) debe migrar a la raíz del árbol. Los domains `[('category_id','=', product_uom_category_id)]` de SO/PO/stock line (secciones 1-3) cambian de base en v19.

### Packaging → `product.uom` (el modelo `product.packaging` desaparece)

- v18 tenía `product/models/product_packaging.py`; v19 lo elimina y crea `product/models/product_uom.py` (`class ProductUom`, `_name='product.uom'`, `_description='Link between products and their UoMs'`, `l10n_co_edi/models/product_uom.py:8-13`, `class ProductUom`).
- `product.uom` es el enlace producto↔UoM↔barcode: campos `uom_id` (M2o uom.uom), `product_id` (M2o product.product), `barcode` (Char required), `company_id` (`product/models/product_uom.py:13-16`, `class ProductUom`).
- Constraint de unicidad de barcode entre productos y packagings se mantiene, ahora en `product.uom._check_barcode_uniqueness` (`product/models/product_uom.py:20-25`, `class ProductUom`) y `product.product._check_duplicated_packaging_barcodes` (`product/models/product_product.py:277-290`, `class ProductProduct`).
- `product.product.product_uom_ids` = O2M a `product.uom` ("Unit Barcode", `product/models/product_product.py:48`, `class ProductProduct`).
- `product.template.uom_ids` = **M2M a `uom.uom`** llamado "Packagings" (`product/models/product_template.py:122`, `ProductTemplate.uom_name`), que son packagings adicionales vendibles. `product.template._get_contained_uoms` devuelve `uom_id | uom_ids` (`product/models/product_template.py:1580`, `ProductTemplate._get_available_uoms`).
- **Impacto cross-module**: SO/PO/stock que en v18 leían `product_packaging_id`/`product_packaging_qty` (secciones 1, 2, 3) pasan a operar sobre `uom_ids`/`product.uom`. La validación de "múltiplo de packaging" sigue en `uom._check_qty` (→ UOG22). El barcode del packaging para escaneo en recepción ahora vive en `product.uom.barcode`.

### Tipos de producto / `is_storable`

- v18 ya había eliminado `type='storable'` a favor de `is_storable` (→ `gotchas.md` PRG13); v19 lo mantiene. `type` sigue siendo `consu`/`service`/`combo`.

### Pricelist

- `product.pricelist._compute_price_rule` (`product/models/product_pricelist.py:210-223`, `Pricelist._compute_price_rule`) sigue convirtiendo la qty a la UoM del producto antes de evaluar `min_quantity` (→ PLSG2); la conversión usa `raise_if_failure=False` (línea 217), que en v19 devuelve la qty sin convertir si no comparten árbol.

---

## Cross-references

- Modelos base: `core.md`
- Variantes en detalle: `variants.md`
- UoM en detalle: `uom.md`
- Cambios v19: `v19_changes.md`
- Gotchas: `gotchas.md` (prefijos: PRG core, VAG variantes, UOG UoM, CTG categoría, PLSG pricelist, SIG supplierinfo, SRVG servicios)
- Stock: `stock/core.md`, `stock/advanced.md`, `stock/rules_deep.md`
- Sale: `sale/core.md`, `sale/loyalty.md`
- Purchase: `purchase/core.md`
- Account: `account/core.md`, `account/operations.md`
- MRP: `cross-module/mrp/production.md`, `cross-module/mrp/integrations.md`
- POS: `pos/core.md`, `pos/gotchas.md`
- Website: `website/ecommerce.md`, `website/ecommerce_deep.md`
- Analytic: `cross-module/analytic/gotchas.md`
- Delivery: `delivery/core.md`
