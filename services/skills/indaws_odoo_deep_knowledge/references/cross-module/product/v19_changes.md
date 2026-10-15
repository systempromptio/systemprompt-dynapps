# Cambios v18 → v19 — Producto / Variantes / UoM

Diffs entre Odoo 18.0 y Odoo 19.0 en los modelos `product.template`, `product.product`, `uom.uom`, `uom.category`, `product.packaging`, atributos y variantes.

**Veredicto general**: el módulo `uom` y el área de packaging tienen el refactor más profundo del core en v19. Migraciones requieren atención.

---

## 1) UoM: refactor radical (BREAKING)

### `uom.uom` cambia su modelo de datos por completo

**v18** — modelo basado en categorías:

```python
class UoM(models.Model):
    _name = 'uom.uom'
    _order = "factor DESC, id"

    name = fields.Char(...)
    category_id = fields.Many2one('uom.category', required=True, ondelete='restrict')
    factor = fields.Float('Ratio', default=1.0, digits=0, required=True)
    factor_inv = fields.Float('Bigger Ratio', compute='_compute_factor_inv')
    rounding = fields.Float('Rounding Precision', default=0.01)
    uom_type = fields.Selection([('bigger', ...), ('reference', ...), ('smaller', ...)])
    ratio = fields.Float(compute='_compute_ratio', inverse='_set_ratio')
    active = fields.Boolean(default=True)
```

**v19** — modelo basado en árbol jerárquico (parent/child):

```python
class UomUom(models.Model):
    _name = 'uom.uom'
    _parent_name = 'relative_uom_id'
    _parent_store = True
    _order = 'sequence, relative_uom_id, id'

    name = fields.Char(...)
    sequence = fields.Integer(compute="_compute_sequence", store=True, readonly=False, precompute=True)
    relative_factor = fields.Float('Contains', default=1.0, required=True)
    rounding = fields.Float(compute="_compute_rounding")  # GLOBAL, ya no per-UoM
    active = fields.Boolean(default=True)
    relative_uom_id = fields.Many2one('uom.uom', 'Reference Unit', ondelete='cascade')
    related_uom_ids = fields.One2many('uom.uom', 'relative_uom_id', 'Related UoMs')
    factor = fields.Float('Absolute Quantity', compute='_compute_factor', recursive=True, store=True)
    parent_path = fields.Char(index=True)
```

### Campos ELIMINADOS en v19

| Campo v18 | Reemplazo v19 | Notas migración |
|---|---|---|
| `category_id` | Implícito por `parent_path` | `_has_common_reference()` reemplaza el check de misma categoría |
| `factor_inv` | Eliminado | Calcular: `1 / factor` |
| `uom_type` | Eliminado | Inferible: `relative_uom_id == False` → es "raíz" (era "reference") |
| `ratio` (compute UI) | Eliminado | UI usa `relative_factor` directo |

### Campos NUEVOS en v19

| Campo | Tipo | Uso |
|---|---|---|
| `relative_uom_id` | M2o `uom.uom` (parent) | "Esta UoM es relativa a..." |
| `relative_factor` | Float, required | "Esta UoM contiene N de la parent" |
| `related_uom_ids` | O2m (children) | UoMs derivadas |
| `parent_path` | Char, parent_store | Tree path indexada (estándar Odoo) |
| `sequence` | Integer | Orden display |
| `factor` | Float **stored, computed recursivamente** | Producto acumulado: `relative_factor * parent.factor` |

### `uom.category` — ¿desaparece?

En v19, `uom.category` **sigue existiendo** pero ya no es el discriminador de compatibilidad. La compatibilidad se determina por **árbol parent_path**: dos UoMs son compatibles si comparten algún ancestor (`_has_common_reference`).

`product.template.uom_category_id` (related en v18) **NO existe en v19**.

### `rounding` ahora es GLOBAL

```python
def _compute_rounding(self):
    decimal_precision = self.env['decimal.precision'].precision_get('Product Unit')
    self.rounding = 10 ** -decimal_precision
```

→ Todas las UoMs tienen el **mismo** rounding, derivado de `decimal.precision('Product Unit')`. El default (precision=2) da `rounding=0.01`.

**Migración**: en v18 podías tener "Kg" con rounding=0.001 y "Litro" con rounding=0.01. En v19 ambos heredan del mismo decimal precision. Si necesitabas precisión distinta por UoM → **debes cambiarlo a nivel global** o usar precisión específica del concepto donde se aplica (ej. stock vs facturación).

### Nuevos métodos públicos en v19

```python
uom.round(value, rounding_method='HALF-UP')
# Redondea value con precision 'Product Unit'

uom.compare(value1, value2)
# float_compare con precision 'Product Unit'. Devuelve -1, 0, 1

uom.is_zero(value)
# float_is_zero con precision 'Product Unit'

uom._has_common_reference(other_uom)
# Bool — comparten ancestor (vía parent_path)
```

### `_compute_quantity` ya NO lanza UserError por categoría

```python
# v18: si categorías distintas, raise UserError (o devuelve qty sin convertir si raise_if_failure=False)
# v19: la firma mantiene raise_if_failure pero la categoría YA NO EXISTE.
#      La verificación se hace por _has_common_reference. Si no comparten ancestor,
#      el comportamiento depende del path.
```

En la práctica v19 maneja la conversión asumiendo que las UoMs están en el árbol. Si llamas con UoMs de árboles distintos, el comportamiento es indefinido (mejor: validar con `_has_common_reference` antes).

⚠️ **Verificado**: `raise_if_failure` es un **parámetro muerto** en v19 — la firma lo mantiene pero el cuerpo (`uom/models/uom_uom.py:162-176`, `UoM.create`) NUNCA lo lee ni lanza, y NO llama a `_has_common_reference`. Conversión cruzada = número erróneo silencioso, nunca UserError. Ver `gotchas.md` UOG29.

⚠️ **`_compute_price` TAMBIÉN se invirtió** (`uom/models/uom_uom.py:196-203`, `UomUom._compute_price`, v19): v18 `price * self.factor / to_unit.factor` → v19 `price * to_unit.factor / self.factor`. Y **sin guard de categoría/árbol** (v18 devolvía price sin convertir si diferían; v19 calcula siempre). Ver UOG28.

### Constraints y ondelete cambian (verificado)

- `relative_uom_id` es `ondelete='cascade'` (`uom/models/uom_uom.py:41`, `class UomUom`, v19) — borrar una UoM padre **borra todo el subárbol**. En v18 `category_id` era `restrict`. Ver UOG30.
- Nueva `_check_factor`: raíz (sin `relative_uom_id`) debe tener `relative_factor==1.0`. Desaparece la unicidad de reference por categoría (se permiten varios árboles). Desaparece SQL `rounding > 0` (rounding pasa a computed). Solo queda `CHECK (relative_factor!=0)`. Ver UOG31.

### `_compute_quantity` también cambió la fórmula

**v18**:
```python
amount = qty / self.factor
amount = amount * to_unit.factor
```

**v19**:
```python
amount = qty * self.factor
amount = amount / to_unit.factor
```

→ **`factor` cambió de semántica**: en v18 era "cuántas veces más pequeña" (relativo a 1 reference). En v19 es "Absolute Quantity" — producto acumulado vía parent_path. Por eso la fórmula se invierte.

**Migración de overrides**: cualquier código custom que use `factor` directamente debe re-evaluarse. El nuevo `factor` es **el absoluto** desde la raíz.

### `_check_qty` (nuevo helper packaging-like)

```python
uom._check_qty(product_qty, uom_id, rounding_method="HALF-UP")
# Redondea product_qty a múltiplo de la conversion uom→uom_id
```

Reemplaza parte de la lógica que en v18 vivía en `product.packaging`.

### Cambios en symptoms / cheatsheet ya hechos

`_index/cheatsheet.md` ya documenta el cambio `float_compare()` → `uom.compare()` para v19.

---

## 2) `product.packaging` — ELIMINADO en v19

### v18: modelo dedicado

```python
class ProductPackaging(models.Model):
    _name = 'product.packaging'
    name = fields.Char(...)
    product_id = fields.Many2one('product.product', ...)
    qty = fields.Float(...)
    product_uom_id = fields.Many2one(related='product_id.uom_id')
    barcode = fields.Char(...)
```

### v19: reemplazado por `product.uom`

Nuevo modelo `product.uom` (en `product/models/product_uom.py`):

```python
class ProductUom(models.Model):
    _name = 'product.uom'
    _description = 'Link between products and their UoMs'
    _rec_name = 'barcode'

    uom_id = fields.Many2one('uom.uom', required=True, ondelete='cascade')
    product_id = fields.Many2one('product.product', required=True, ondelete='cascade')
    barcode = fields.Char(required=True)
    company_id = fields.Many2one('res.company', default=...)
```

→ Unifica el concepto "packaging" con "UoM secundaria del producto":

- Cada producto puede tener N `product.uom` registros adicionales.
- Cada uno apunta a una UoM (que puede ser `Box of 6`, `Pack of 12`, etc.) **con un barcode propio**.
- `product.template.uom_ids` (M2M) lista todas las UoMs disponibles para el producto (además de `uom_id`).

### Migración de packaging → product.uom

Script de migración estándar Odoo (en `addons/product/migrations/19.0.x.x/`):

```
product.packaging (v18)         →   product.uom (v19)
─────────────────────────────       ────────────────────
name                                  (perdido o → uom_id.name)
product_id                            product_id
qty                                   uom_id (UoM creada con factor=qty)
barcode                               barcode
sales / purchase / etc.               (no equivalente directo)
```

Datos antiguos:

- Si tenías packagings simples (qty + barcode) → se crean UoMs nuevas en árbol.
- Si tenías muchos packagings idénticos (mismos qty) → se reutiliza una UoM.
- Si tenías `sales=True` / `purchase=True` flags → se pierden (en v19 todas las UoMs del producto son utilizables en cualquier sitio).

### Campos relacionados que cambian

| v18 | v19 |
|---|---|
| `product.template.packaging_ids` (O2m) | **Eliminado**; usar `uom_ids` (M2M `uom.uom`) |
| `product.product.packaging_ids` (O2m) | **Eliminado** |
| `sale.order.line.product_packaging_id` | ⚠️ **ELIMINADO por completo** (no "apunta a uom.uom"). El packaging es ahora una UoM más en `product_uom_id`. Ver `gotchas.md` UOG26 |
| `sale.order.line.product_packaging_qty` | ⚠️ **ELIMINADO** (no renombrado). Ver UOG26 |
| `stock.move.product_packaging_id` | ⚠️ NO es rename: v19 añade campo NUEVO `stock.move.packaging_uom_id` (M2o uom.uom). Ver UOG27 |
| `stock.picking.show_check_availability` | Posible cambio mínimo |

> ⚠️ **Corrección verificada contra fuente v19** (los nombres de campo de abajo eran especulativos): el campo UoM de línea se **renombra** `product_uom` → `product_uom_id` en sale/purchase/supplierinfo (UOG25); los campos de packaging en sale.order.line **desaparecen** (UOG26); stock.move recibe un campo **nuevo** `packaging_uom_id` (UOG27).

---

## 3) `product.template` — diffs

### Campos eliminados

| Campo v18 | Razón eliminación | Reemplazo |
|---|---|---|
| `uom_po_id` | El concepto "purchase UoM" unificado en `uom_ids` (puedes elegir cualquier UoM del producto en PO) | Elegir en PO line entre `uom_id` y `uom_ids` |
| `uom_category_id` (related) | Categoría ya no existe como discriminador | Usar `_has_common_reference` |
| `packaging_ids` (O2m product.packaging) | Modelo eliminado | `uom_ids` (M2M `uom.uom`) |

### Campos nuevos

| Campo | Tipo | Notas |
|---|---|---|
| `uom_ids` | M2M `uom.uom` | "Additional packagings/UoMs for this product which can be used for sales" |
| `import_attribute_values` | Char | Para importación masiva: paso intermedio antes de crear PTAL |
| `is_dynamically_created` | Boolean compute | True si el template fue creado por configurador dinámico (no manual) |
| `pricelist_rule_ids` | O2m | Renombrado/extendido de `pricelist_item_count`. Muestra reglas de pricelist aplicables directamente |

### `uom_name` cambia su `string`

v18: `string='Unit of Measure Name'`. v19: `string='Unit Name'`.

---

## 4) Atributos / Variantes — diffs menores

### Cambios en model behavior

**v18 → v19** en `product.attribute`, `product.attribute.value`, `product.template.attribute.line`, `product.template.attribute.value`, `product.template.attribute.exclusion`: **modelos casi estables**; misma lógica de generación de variantes. Cambios verificados contra fuente:

Cambios puntuales:

- **`product.attribute.display_type` gana la opción `('image','Image')`** (v19, `product/models/product_attribute.py:44`, `class ProductAttribute`). En v18 la imagen de un valor solo se usaba dentro de `display_type='color'`; v19 tiene un tipo "Image" dedicado. (ATTRCAP17, ATTRCAP46)
- **`_sql_constraints` (lista) → `models.Constraint`**: `check_multi_checkbox_no_variant` pasa a `_check_multi_checkbox_no_variant = models.Constraint(...)` (mismo CHECK SQL). (ATTRCAP45)
- `action_open_product_template_attribute_lines` marcada `@api.readonly` (v19, sin cambio funcional). (ATTRCAP47)
- **`product.attribute.value`**: `action_add_to_products` y `action_update_prices` marcadas `@api.readonly` (v19). Sin cambio de campos/lógica. (PAVCAP43)
- **`product.template.attribute.value`**: `attribute_value_unique` pasa de `_sql_constraints` a `_attribute_value_unique = models.Constraint(...)`; `write(self, values)` renombrado a `write(self, vals)` con alias. Sin cambio funcional. (PAVCAP44)
- `_compute_is_dynamically_created` en `product.template` (nuevo flag).
- Refactor del configurador frontend (JS) — `website_sale` y `sale` reorganizan controladores. Para overrides custom, revisar las nuevas hooks.
- En `website_sale`, variant_mixin.js renombra/reestructura (ver `website/v19_changes.md`).

### Bridge entre import_attribute_values y attribute_line_ids

`import_attribute_values` (Char) permite importar variantes vía un formato tipo:

```csv
name,import_attribute_values
Camiseta,"Color: Rojo, Azul; Talla: S, M, L"
```

→ Al post-procesar, el campo string se parsea y crea las PTAL/PTAV correspondientes. Reemplaza el patrón de imports v18 con múltiples filas.

---

## 5) Inventory / Stock — UoM en moves

`stock.move.product_uom`:

- v18: domain `[('category_id', '=', product_id.uom_id.category_id)]`.
- v19: domain ahora basado en tree (`_has_common_reference`). En la práctica `domain="[('id', 'in', product_id.uom_ids + product_id.uom_id)]"`.

→ **La UoM de la move ya no puede ser cualquier UoM de la categoría**, sino solo las explícitamente listadas en `product.template.uom_ids` + la principal `uom_id`. Esto reduce errores pero **rompe migraciones** donde clientes tenían SO/PO/moves con UoMs custom no listadas.

Script de migración añade automáticamente UoMs usadas en histórico a `product.uom_ids`.

---

## 6) Sale — UoM en línea (verificado contra fuente)

`sale.order.line.product_uom` → **renombrado `product_uom_id`** (`sale/models/sale_order_line.py:132`, `SaleOrderLine.product_uom`). Ver UOG25.

- v18: field `product_uom`, domain por categoría.
- v19: field `product_uom_id`, domain `[('id','in',allowed_uom_ids)]` donde `allowed_uom_ids = product.uom_id | product.uom_ids` (`:537-539`). No es "cualquier UoM de la categoría" sino solo las listadas en el producto. Ver UOG33.

`sale.order.line.product_packaging_id` / `product_packaging_qty`:

- v18: M2o `product.packaging` + qty en packagings.
- v19: **ELIMINADOS por completo** (no existen en el modelo). El "packaging" se elige como una UoM más en `product_uom_id`. Ver UOG26.

Esto **simplifica** la lógica de SO line — un solo "qty + uom" en vez de qty_uom + qty_packaging + packaging_id.

---

## 7) Purchase — paralelos a sale (verificado)

- `purchase.order.line.product_uom` → **renombrado `product_uom_id`** (`purchase/models/purchase_order_line.py:36`, `PurchaseOrderLine.product_id`), domain `[('id','in',allowed_uom_ids)]`. Nuevo campo `price_unit_product_uom` (`:42`). Ver UOG25.
- `product.supplierinfo.product_uom` → **renombrado `product_uom_id`** (`product/models/product_supplierinfo.py:25`, `SupplierInfo.product_name`, required). Ver UOG25.
- `uom_po_id` eliminado → vendor pricelist sin UoM "preferida"; el usuario elige libremente entre las disponibles (`allowed_uom_ids`).

---

## 8) MRP — diffs

`mrp.bom.product_uom_id` domain por `uom_ids` del producto.

`mrp.bom.line.product_uom_id` igual.

**Migración**: BoMs con UoMs custom requieren añadir esas UoMs a `product.uom_ids` antes del upgrade.

---

## 9) Frontend / JS

### Configurador de variantes

- v18: `variant_mixin.js` (en `sale`).
- v19: refactor a `productConfigurator` (Owl component nuevo) en `sale`. Compatible con eCommerce.

### eCommerce

- Refactor de `/shop/product/<id>` page templates.
- Swatches usan los nuevos `display_type` correctamente.
- Configurator AJAX al cambiar variant — sin recarga.

Más detalle en `website/v19_changes.md`.

---

## 10) Compatibilidad de migración — riesgos top

| # | Riesgo | Mitigación |
|---|---|---|
| 1 | Datos en packagings se pierden si script de migración falla | Backup, test en staging, verificar `product.uom` creados |
| 2 | UoMs custom no listadas en `uom_ids` desaparecen de selectors | Pre-migración: poblar `uom_ids` con UoMs usadas en histórico |
| 3 | Overrides que usan `factor` con semántica v18 dan resultados incorrectos | Auditar todo código custom que toca `uom.factor` |
| 4 | `decimal.precision('Product Unit')` ahora controla rounding global de UoMs | Verificar precision suficiente (recomendado: 3+) |
| 5 | Cambios en `_compute_quantity` (firma estable pero comportamiento diferente con categorías cruzadas) | Tests de regresión en código que convierte entre UoMs |
| 6 | `uom_po_id` eliminado: SO/PO que asumían "purchase UoM" diferente de stock UoM rompen | Migración: copiar `uom_po_id` antiguo a `uom_ids` del producto |
| 7 | `uom_category_id` removed en related — vistas custom rompen | Buscar refs en QWeb/Studio y reemplazar |
| 8 | `sale.order.line.product_packaging_id` deja de ser `product.packaging` | Adaptar reportes/queries |

---

## 11) Quick reference v18 ↔ v19

```python
# COMPATIBILIDAD ENTRE UoMs
v18: uom_a.category_id.id == uom_b.category_id.id
v19: uom_a._has_common_reference(uom_b)

# CONVERSIÓN (sin cambio de firma)
v18 / v19: uom_a._compute_quantity(qty, uom_b)
# pero internamente la fórmula cambia (factor tiene semántica distinta)

# REDONDEO
v18: tools.float_round(value, precision_rounding=uom.rounding)
v19: uom.round(value)   # método nuevo, usa precision global

# COMPARACIÓN
v18: tools.float_compare(v1, v2, precision_rounding=uom.rounding)
v19: uom.compare(v1, v2)

# UoM POR DEFECTO COMPRAS
v18: product.uom_po_id
v19: product.uom_ids (M2M, elegir en PO line; sin "default" automático)

# PACKAGINGS
v18: product.packaging_ids (O2m product.packaging)
v19: product.uom_ids (M2M uom.uom; la UoM = el packaging)
v19: product.uom (modelo nuevo: link producto-UoM con barcode)
```

---

## 12) Pendiente investigar (placeholders)

- ✅ **RESUELTO** — Convertir entre UoMs no ancestrales: `_compute_quantity`/`_compute_price` NO validan nada, hacen la aritmética con `factor` igual → resultado erróneo silencioso, sin UserError (`raise_if_failure` muerto). Ver UOG28/UOG29.
- Si el wizard de creación de variantes acepta packaging en v19.
- POS y UoM v19: si el modelo `product.uom` se sincroniza con POS frontend.
- SII v19: si el mapping de UoMs externas se actualiza para usar nuevo árbol.
- Cómo se comporta `_check_qty` con el nuevo `factor` absoluto (helper packaging-like, `uom/models/uom_uom.py:178-194`, `UomUom._check_qty`, v19) — usado por sale/purchase al ajustar a múltiplos de UoM.

---

## Cross-references

- Modelos v18 base: `core.md`, `variants.md`, `uom.md`
- Stock v19: `stock/v19_changes.md`
- Sale v19: `sale/v19_changes.md`
- Purchase v19: `purchase/v19_changes.md`
- Website v19: `website/v19_changes.md`
- POS v19: `pos/v19_changes.md`
- Repair v19: `repair/v19_changes.md`
