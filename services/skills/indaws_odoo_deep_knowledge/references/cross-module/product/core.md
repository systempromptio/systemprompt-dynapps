<!-- verified: v18.0 @ 2026-06 -->
# Product Core — Odoo 18.0

Modelos `product.template` y `product.product`, ciclo de vida, herencias y campos clave. Las **variantes** se cubren en `variants.md` y las **unidades de medida** en `uom.md`. Para v19: ver `v19_changes.md`.

## Modelo dual: template + product

Odoo usa SIEMPRE dos modelos:

| Modelo | Rol | Identificador típico | Cuándo es 1, cuándo es N |
|---|---|---|---|
| `product.template` | "Producto" funcional. Datos comunes a todas las variantes. | `product_tmpl_id` | Siempre 1 por SKU funcional |
| `product.product` | Variante concreta. Inherits del template (`_inherits`). | `product_id` | 1 si no hay atributos. N si hay variantes |

Regla **invariante**: aunque el template no tenga atributos, **siempre existe al menos un `product.product`** (la variante "única"). Es el que se referencia en `stock.move`, `sale.order.line`, `account.move.line`, etc.

```python
# Acceder al template desde la variante (campo many2one inverso):
product.product_tmpl_id

# Acceder a las variantes desde el template (one2many):
template.product_variant_ids          # todas las variantes
template.product_variant_id           # SOLO la primera variante (compute)
template.product_variant_count        # nº de variantes
```

### _inherits delegación (no herencia clásica)

`product.product._inherits = {'product.template': 'product_tmpl_id'}` significa que:

- Los campos del template se leen/escriben transparentemente desde la variante (`variant.name == variant.product_tmpl_id.name`).
- Si **escribes** un campo del template en una variante, **se escribe en el template** y afecta a TODAS las variantes hermanas.
- Excepciones: campos definidos directamente en `product.product` (sobrescriben los del template). Ejemplo: `barcode`, `default_code`, `image_variant_1920`, `weight`, `volume`, `standard_price` (este último `company_dependent`).

```python
# OJO: esto cambia el peso de TODAS las variantes del template:
variant_S.weight = 2.0   # en realidad escribe en product.template

# Para cambiar SOLO la variante:
variant_S.with_context(...).write({'weight': 2.0})  # idem, mismo problema
# Solución: weight tiene compute con _set_product_variant_field, propaga
# de template -> variante solo si hay UNA variante. Con N variantes, escribir
# en el template NO baja a las variantes existentes.
```

## product.template — campos clave (v18)

### Identificación

- `name` (Char, translate, index='trigram')
- `default_code` (Char, computed→variant) — Internal Reference, único si una variante
- `barcode` (Char, computed→variant) — único si una variante
- `active` (Boolean, default=True)
- `is_favorite` (Boolean) — usado en `_order`

### Tipo

- `type`: `'consu'` (Goods), `'service'`, `'combo'`
  - **v18 cambio**: ya no existe `'product'` (storable) en el `type`. La distinción goods/storable se hace ahora con `is_storable` (en `stock`) o `tracking`. Ver `stock/core.md`.
- `service_tracking`: para servicios, computed/store con `default='no'`. Otros valores se añaden por módulos (`sale_project`, `helpdesk_account_followup`, etc.).
- `combo_ids`: Many2many a `product.combo` cuando `type='combo'`.

### Categorización

- `categ_id` (Many2one `product.category`, required, default=All) — clave para reglas contables (`property_account_income_categ_id`, `property_stock_*`, valoración).
- `product_tag_ids` (Many2many `product.tag`) — etiquetas a nivel template.
- `additional_product_tag_ids` (en `product.product`) — etiquetas adicionales por variante.

### Precio

- `list_price` (Float) — precio de catálogo (precio base de venta, sin IVA, sin variant extra).
- `standard_price` (Float, **company_dependent en product.product**, computed en template) — coste/valoración AVCO. En `product.product` es `company_dependent=True`; en template es computed delegando a la primera variante.
- `currency_id` (Many2one, computed) — moneda de la company del producto, o main_company.
- `cost_currency_id` (Many2one, computed, `depends_context=('company',)`) — moneda de la company activa.

### UoM

- `uom_id` (Many2one `uom.uom`, required, default=`uom.product_uom_unit` ≈ Units) — UoM de stock/inventario.
- `uom_po_id` (Many2one `uom.uom`, computed con `_compute_uom_po_id`, store, readonly=False) — UoM de compras. Auto-fija a `uom_id` cuando cambia la categoría.
- `uom_category_id` (related a `uom_id.category_id`).
- `uom_name` (related a `uom_id.name`).
- Detalle de conversión y dual-UoM: ver `uom.md`.

### Variantes

- `attribute_line_ids` (One2many `product.template.attribute.line`, copy=True) — define qué atributos tiene el template.
- `product_variant_ids` (One2many `product.product`).
- `product_variant_id` (Many2one computed) — solo prefetching de la 1ª variante.
- `product_variant_count` (Integer computed).
- `valid_product_template_attribute_line_ids` (M2M computed) — PTAL con al menos un value activo.
- `has_configurable_attributes` (Boolean stored, computed) — `True` si tiene dinámicos, ≥2 values en alguna línea, multi-checkbox, o algún value `is_custom`.
- Mecánica completa en `variants.md`.

### Multi-company

- `company_id` (Many2one, **NO required**) — si vacío, producto compartido entre todas las empresas. Combinado con `_check_company_auto = True` y `_check_company_domain = check_company_domain_parent_of`.
- Restricción: códigos de barras únicos **por empresa** (constraint `_check_barcode_uniqueness` en `product.product`; agrupa por company vía `_get_barcodes_by_company`, docstring "Barcodes should only be unique within a company"). NO es unicidad global. Colisiona también con `product.packaging` (v18) / `product.uom` (v19).

### Documentos, propiedades

- `product_document_ids` (One2many `product.document`, polimórfico) — adjuntos públicos (web/cliente). Hay also internal/internal_default.
- `product_properties` (Properties) — campo dinámico cuyo schema viene de `categ_id.product_properties_definition`.
- `image_1920`, `image_1024`, ... `image_128` — todas computed cuando es la única variante; se almacenan como adjunto.

### Compras

- `seller_ids` (One2many `product.supplierinfo`, `depends_context=('company',)`) — proveedores. Ordenados por sequence; `_select_seller()` filtra por cantidad mínima y fecha.
- `variant_seller_ids` (One2many `product.supplierinfo`) — incluye variants.

### Vista

- `description`, `description_purchase`, `description_sale` — descripciones diferenciadas que se vuelcan a SO/PO/facturas.
- `is_storable` (en `stock`): determina si afecta a inventario. Si `type='consu'` y `is_storable=False`, no aparece en stock.move (pasada salvaje frecuente).

## product.product — campos propios (no del template)

| Campo | Tipo | Notas |
|---|---|---|
| `product_tmpl_id` | M2o template | required, ondelete='cascade', auto_join=True |
| `default_code` | Char, index | **Override** del template (sobrescribe) |
| `barcode` | Char, index='btree_not_null' | **Override**. Único **por empresa** (`product.product` + `product.packaging` v18 / `product.uom` v19), NO global |
| `standard_price` | Float, `company_dependent=True` | **Por empresa** — v17: registros `ir.property`; **v18+: columna `jsonb` en la tabla del modelo** con `company_id` como clave (`ir.property` eliminado en v18; NO existe ningún `property_value`) |
| `weight`, `volume` | Float | **Override** del template (no compute) — valor real de la variante |
| `image_variant_1920` (+ sizes) | Image | Imagen específica de variante; los campos `image_*` del template hacen fallback |
| `product_template_attribute_value_ids` | M2M PTAV | Define la combinación de la variante (clave de unicidad) |
| `product_template_variant_value_ids` | M2M PTAV | **Subset**: solo PTAV con `attribute_line_id.value_count > 1` (excluye single-value lines) |
| `combination_indices` | Char stored, index | Hash IDs de PTAV separados por coma, ordenados — para búsqueda rápida de combinación |
| `price_extra` | Float computed | Suma de `price_extra` de los PTAV de la variante |
| `lst_price` | Float computed (uom-context-dependent) | `list_price + price_extra`, convertido a UoM del contexto si lo hay |
| `partner_ref` | Char computed | Ref proveedor para `partner_id` en contexto |
| `additional_product_tag_ids` | M2M tag | Tags solo de la variante (suma con los del template via `all_product_tag_ids`) |
| `packaging_ids` (v18) | O2m `product.packaging` | Packagings por variante. **En v19 desaparece** (ver v19_changes.md). |

### Combination indices — clave de unicidad

```python
@api.depends('product_template_attribute_value_ids')
def _compute_combination_indices(self):
    for product in self:
        product.combination_indices = product.product_template_attribute_value_ids._ids2str()
```

Es el equivalente al "hash" de combinación. Usado por `_get_variant_for_combination()` para encontrar la variante exacta sin recorrer todos los `product.product`. **Índice DB**: `combination_indices` está indexado para esto.

## Mixins heredados

| Mixin | Modelo | Aporta |
|---|---|---|
| `mail.thread` | template + variant | Chatter, tracking de campos clave (list_price tracking=True) |
| `mail.activity.mixin` | template + variant | Actividades |
| `image.mixin` | template | Cascade de tamaños 1920/1024/512/256/128 |
| `product.catalog.mixin` | sale/purchase order | Catálogo de productos en SO/PO |

## Ciclo de vida — creación de un producto

```python
# 1) Crear template
tmpl = self.env['product.template'].create({
    'name': "Camiseta",
    'list_price': 25.0,
    'type': 'consu',
})
# Side-effect: se crea automáticamente 1 product.product via _create_variant_ids()

# 2) Añadir atributos => crea PTAL/PTAV => recrea variantes
tmpl.write({
    'attribute_line_ids': [(0, 0, {
        'attribute_id': talla.id,
        'value_ids': [(6, 0, [s.id, m.id, l.id])],
    })],
})
# Side-effect: _update_product_template_attribute_values() crea 3 PTAV,
# y luego _create_variant_ids() crea 3 product.product (S/M/L)
# y archiva la variante "única" previa si no encaja.

# 3) Acceder
tmpl.product_variant_count       # 3
tmpl.product_variant_ids         # recordset de 3 product.product
```

## Archivado vs eliminación

- `template.action_archive()` → archiva también todas sus variantes en cascada (lógica en `product.template.toggle_active`).
- Eliminar un template directamente raramente funciona: hay constraints desde `stock.move`, `account.move.line`, `sale.order.line`, etc. El sistema cae a "archive" automáticamente en muchos lugares.
- Eliminar una variante con `_unlink_or_archive()`: intenta `unlink()`, si falla por FK la archiva.

## product.category — herencia jerárquica

- `parent_id` + `parent_path` (parent_store) — soporta tree.
- `property_*` fields (`product.template.property_account_income_id`, `property_account_expense_id`, etc.) son `company_dependent=True` y se resuelven con fallback **producto → categoría → categoría padre → empresa**.
- Asignar cuentas en categoría es la práctica estándar (no producto a producto).

## product.tag

- M2M con template (`product_tag_ids`) + M2M con product.product (`additional_product_tag_ids`).
- `all_product_tag_ids` (computed en variant) → unión de ambos.
- Permite filtros transversales sin tocar `categ_id`.

## product.combo / product.combo.item (Combos v18)

- Nuevo type `'combo'` introducido en v18 (refactor de los "optional products" de venta y reemplazo del config-product de POS).
- `product.combo`: agrupa choices.
- `product.combo.item`: cada opción seleccionable (referencia a `product.product`, extra_price, sequence).
- Cuando se vende un combo, se descompone en líneas hijas en SO (`combo_item_id` en `sale.order.line`).
- En POS reemplaza completamente la lógica histórica de "configurable products".

## product.pricelist + product.pricelist.item — referencia rápida

(Detalle completo en `sale/core.md` y `sale/gotchas.md`.)

- `pricelist.item.applied_on`: `'3_global'` / `'2_product_category'` / `'1_product'` / `'0_product_variant'`. **Las 4 letras DEBEN ordenarse por especificidad descendente** — el resolver pricelist usa la primera regla que matchee con mayor especificidad. Confusión común.
- `pricelist.item.compute_price`: `'fixed'` / `'percentage'` / `'formula'`.
- `_get_product_price()` en **`product.pricelist`** (`plist._get_product_price(product, quantity, uom=, ...)`) es el punto de entrada canónico para precios; a nivel producto, `product._get_contextual_price()`. **No mezclar** con `lst_price` (que no aplica pricelist).

## Helpers críticos

```python
# Precio resuelto con UoM y currency (pero SIN pricelist):
template._price_compute(price_type='list_price', uom=other_uom, currency=other_cur)
# Firma: _price_compute(price_type, uom=None, currency=None, company=None, date=False)
# Devuelve {tmpl_id: price}.

# Precio CON pricelist (canónico) — se llama sobre la PRICELIST, no sobre el producto:
plist._get_product_price(product, quantity=1.0, uom=other_uom, currency=..., date=...)
# Owner real: product.pricelist._get_product_price (product_pricelist.py:104).
# A nivel producto, helper con contexto: product._get_contextual_price()
# (product_product.py:876, lee pricelist/quantity/uom del contexto).

# Buscar variante por combinación:
template._get_variant_for_combination(combination)   # (product_template.py:1133)
# Devuelve product.product o empty recordset.

# Verificar si una combinación es válida (no excluida):
template._is_combination_possible(combination, parent_combination=None, ignore_no_variant=False)
# (product_template.py:1080) Considera exclusiones y archived values.

# Crear/archivar variantes según attribute_line_ids actual:
template._create_variant_ids()
# Llamado automáticamente al cambiar PTAL/PTAV/exclusions.
```

## Servicios — matriz de configuración

`type='service'` activa campos extra (en módulos sale/sale_project/sale_timesheet/helpdesk). Las combinaciones válidas no son todas — depende de qué módulos hay instalados.

### Campos clave para servicios

| Campo | Módulo | Selection (en CE base) |
|---|---|---|
| `invoice_policy` | sale | `'order'` (al confirmar) / `'delivery'` (qty entregada) |
| `service_type` | sale | `'manual'` (qty manual). Extensible. |
| `service_tracking` | product | `'no'`. Extensible. |
| `service_policy` | sale_project / sale_subscription | `'ordered_prepaid'` / `'delivered_milestones'` / `'delivered_manual'` / `'delivered_timesheet'` |

### Cómo se desbloquean valores (cascada de módulos)

| Módulo instalado | Añade a `service_tracking` | Añade a `service_type` | Añade a `service_policy` |
|---|---|---|---|
| `sale` | — | `'manual'` | `'ordered_prepaid'` |
| `sale_project` | `'task_in_project'`, `'project_only'` | — | `'delivered_milestones'`, `'delivered_manual'` |
| `sale_timesheet` | `'task_in_project'` (refuerza) | `'timesheet'` | `'delivered_timesheet'` |
| `helpdesk_sale_timesheet` | `'task_global_project'` | — | — |
| `sale_subscription` | — | — | `'recurring'` (subscription) |
| `mrp` (con type=service para combos) | — | — | — |

### Combinaciones válidas — matriz simplificada

| service_tracking | service_policy | service_type | Comportamiento |
|---|---|---|---|
| `'no'` | `'ordered_prepaid'` | `'manual'` | **Default**. Factura al confirmar SO, qty manual. No crea tarea. |
| `'no'` | `'delivered_manual'` | `'manual'` | Factura solo qty marcada como "entregada" manualmente. |
| `'task_in_project'` | `'delivered_timesheet'` | `'timesheet'` | **Servicio facturable por horas**. Crea task; qty entregada viene de timesheets. |
| `'task_in_project'` | `'delivered_milestones'` | `'manual'` | Crea task; factura cuando se marcan milestones completados. |
| `'project_only'` | `'delivered_timesheet'` | `'timesheet'` | Crea proyecto completo (no solo task). Útil para proyectos grandes. |
| `'task_global_project'` | `'delivered_timesheet'` | `'timesheet'` | Tasks van a un proyecto helpdesk global (no per-SO). |

### Reglas implícitas

- `service_policy='delivered_timesheet'` **requiere** `service_tracking != 'no'` (necesita una tarea/proyecto donde imputar timesheets).
- `service_type='timesheet'` aparece SOLO si `sale_timesheet` instalado.
- `service_policy='delivered_milestones'` crea `sale.order.line.milestone_ids` (1+ hitos), factura al marcar `reached=True`.
- `service_tracking='no'` con `service_policy='delivered_manual'`: el usuario edita `qty_delivered` a mano en SO line.

### Productos service y consu mezclados

Un combo (`type='combo'`) puede contener tanto `consu` como `service`. El SO descompone en líneas hijas y cada hija se procesa según su tipo (service crea task, consu crea stock move).

### Servicios y stock

- `type='service'` por defecto NO tiene `is_storable=True` (no aparece en informes de stock).
- Excepción: servicios "delivery" (carriers) — son `type='service'`, `sale_ok=False`, generan stock pickings indirectamente via `delivery.carrier`.

### Servicios y MRP

- `type='service'` NO puede tener BoM (MRP es para tangibles).
- Excepción: subcontratación — un servicio "Manufactura subcontratada" puede actuar como vendor BoM (`type='service'` + supplierinfo).

### Frequencia del olvido

**Gotcha común**: cambiar `service_tracking` cuando ya hay SOs/tareas confirmadas:

- Cambiar `'no' → 'task_in_project'`: SOs futuras crean tasks. Las existentes NO.
- Cambiar `'task_in_project' → 'no'`: SOs nuevas no crearán tasks. Las existentes mantienen su task.
- Cambiar `service_policy` retroactivamente: precios de SO line NO se recalculan, pero `qty_to_invoice` SÍ (puede generar facturas inesperadas).

---

## Cantidades de stock — semántica de los `*_qty` fields

Solo aplica si el producto tiene `is_storable=True` (en `stock`). Para `type='service'` o `consu` no storable, todos devuelven 0.0.

Definidos en `stock/models/product.py`:

| Campo | Fórmula | Cuándo lo necesitas |
|---|---|---|
| `qty_available` | Suma de `stock.quant.quantity` en ubicaciones internas | "Stock físico actual" |
| `free_qty` | `qty_available - sum(stock.quant.reserved_quantity)` | "Stock disponible para reservar nuevos pedidos" |
| `incoming_qty` | Suma de `stock.move.product_uom_qty` planificados (state in [waiting, confirmed, partially_available, assigned]) hacia ubicaciones internas | "Pendiente de recibir" |
| `outgoing_qty` | Suma de moves planificados desde ubicaciones internas | "Pendiente de salir" |
| `virtual_available` | `qty_available - outgoing_qty + incoming_qty` | "Stock futuro previsto" |

**Todos en `product.uom_id`** (stock UoM). NO son store=True (se calculan al leer).

### Context que afecta el cálculo

```python
# Restringir a una ubicación:
product.with_context(location=stock_location_id).qty_available

# Restringir a un warehouse:
product.with_context(warehouse=warehouse_id).qty_available

# Stock en una fecha pasada:
product.with_context(to_date='2024-12-31').qty_available
# usa stock.move históricos para "rewind"

# Filtrar por lot/owner/package:
product.with_context(lot_id=lot.id).qty_available
product.with_context(owner_id=partner.id).qty_available
product.with_context(package_id=package.id).qty_available
```

### Performance

- `_compute_quantities` itera con `quants_res = stock.quant._read_group(['product_id'], ['quantity', 'reserved_quantity'])` — eficiente para batches.
- Para 1 producto: 4-6 queries.
- Para 100 productos: similar (read_group agrupa).
- **NO** son store → cada acceso re-ejecuta. Cachear si lees varias veces (`with_prefetch` o variable local).

### Mostrar en template vs variant

- En `product.template`: estas qty son **sumas de todas las variantes**.
- En `product.product`: qty de esa variante específica.

### Diferencia clave: `free_qty` vs `virtual_available`

- `free_qty` = "qué puedo reservar AHORA con stock REAL".
- `virtual_available` = "qué tendré en el futuro contando entradas/salidas planificadas".

Caso real: tienes 10 unidades en stock, 5 reservadas, 3 incoming, 0 outgoing:

- `qty_available` = 10
- `free_qty` = 5  (10 - 5 reservadas)
- `incoming_qty` = 3
- `outgoing_qty` = 0
- `virtual_available` = 13  (10 + 3 - 0)

→ Para `virtual_available` NO se descuentan las reservas. Es solo "stock + entradas - salidas".

### `nbr_reordering_rules` / `reordering_min_qty` / `reordering_max_qty`

Computados desde `stock.warehouse.orderpoint`. Si producto tiene una reordering rule (min/max), estos campos exponen los valores.

**Gotcha**: si producto tiene N reordering rules (una por warehouse), `nbr_reordering_rules` los suma pero `reordering_min_qty` da el min del primero (no la suma de mins). Para ver todos: `orderpoint_ids`.

---

## Cross-references

- **Atributos, variantes, PTAL, PTAV, exclusiones, dynamic**: ver `variants.md`.
- **UoM, conversión, dual UoM, precision**: ver `uom.md`.
- **Categoría, Pricelist, Supplierinfo**: ver `pricelist_supplier_category.md`.
- **Integración con sale/purchase/stock/account/pos/website**: ver `integrations.md`.
- **Cambios v19**: ver `v19_changes.md`.
- **Gotchas numerados**: ver `gotchas.md`.
- **Valoración (SVL, AVCO, FIFO)**: ver `stock/account.md`.
- **Property fields multi-company**: ver `account/operations.md` y `cross-module/security/gotchas.md`.
- **Servicios + project/timesheet**: ver `project/core.md`, `enterprise/timesheet/core.md`.
