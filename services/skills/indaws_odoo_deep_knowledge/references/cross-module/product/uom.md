# Unidades de Medida (UoM) — Odoo 18.0

Cómo funcionan `uom.uom`, `uom.category`, la conversión, las precisiones, el "dual UoM" sale/purchase/stock, y la integración con todos los módulos. Para los cambios v19 (modelo radicalmente refactorizado): ver `v19_changes.md`.

## Modelo conceptual

```
uom.category  "Unit"             "Length"           "Weight"
              │                  │                  │
              ├─ uom.uom (ref)   ├─ uom.uom (ref)  ├─ uom.uom (ref)
              │  "Units" (1.0)   │  "m" (1.0)      │  "kg" (1.0)
              │                  │                  │
              ├─ uom.uom         ├─ uom.uom         ├─ uom.uom
              │  "Dozen"         │  "cm"            │  "g"
              │  type=bigger     │  type=smaller    │  type=smaller
              │  factor=1/12     │  factor=100      │  factor=1000
              │                  │                  │
              └─ uom.uom         └─ uom.uom         └─ uom.uom
                 "Pack of 6"        "km"               "lb"
                 type=bigger        type=bigger        type=bigger
                 factor=1/6         factor=1/1000      factor=2.205...
```

**Regla invariante**: **solo se pueden convertir UoMs de la misma `category_id`**. Conversión cruzada lanza `UserError`. Sin categoría → sin conversión.

## uom.category

| Campo | Tipo | Notas |
|---|---|---|
| `name` | Char, translate | "Unit", "Length", "Weight", "Working Time"... |
| `uom_ids` | O2M `uom.uom` | Las UoMs de esta categoría |
| `reference_uom_id` | M2o, **store=False** | Dummy: solo para tracking en `_onchange_uom_ids` |

**Constraints automáticas** (`_check_category_reference_uniqueness`):

- Exactamente **1** UoM con `uom_type='reference'` por categoría.
- Si hay UoMs en la categoría, debe haber 1 reference.
- Lanza `ValidationError` si la categoría tiene 2+ references o 0 references.

**`_onchange_uom_ids`** (cambio de reference UoM):

- Si cambias la reference de una categoría, **se recalculan los `factor` de todas las hermanas** para mantener las ratios coherentes con la nueva reference.
- Es magia frágil — operación rara.

## uom.uom

| Campo | Tipo | Notas |
|---|---|---|
| `name` | Char, translate, required | "Units", "kg", "m"... |
| `category_id` | M2o `uom.category`, required, ondelete='restrict' | No se puede borrar la categoría si tiene UoMs |
| `factor` | Float, default=1.0, **digits=0** (NUMERIC unlimited) | Cuántas veces esta UoM es **más pequeña** que la reference: `1 ref = factor × this` |
| `factor_inv` | Float computed | `1.0 / factor`. Cuántas veces esta UoM es **más grande**: `1 this = factor_inv × ref` |
| `rounding` | Float, default=0.01, **digits=0** | Precision del redondeo (granularidad) |
| `active` | Boolean | Archivable |
| `uom_type` | Selection | `bigger` / `reference` / `smaller` |
| `ratio` | Float computed/inverse | Helper UI: si bigger=factor_inv, smaller=factor, reference=1 |
| `color` | Integer computed | UI: 7 (azul) si reference, 0 otherwise |
| `_order` | "factor DESC, id" | Más pequeñas primero |

**SQL constraints**:

- `factor != 0`.
- `rounding > 0`.
- Si `uom_type='reference'` entonces `factor = 1.0`.

### Significado de factor

La fórmula canónica (verificada en `_compute_quantity`):

```python
# Convertir qty de UoM A a UoM B (misma categoría):
amount = qty / A.factor          # qty en términos de la reference
amount = amount * B.factor       # qty en términos de B
amount = float_round(amount, precision_rounding=B.rounding, rounding_method='UP')
```

Ejemplo (categoría "Unit", reference=Units, factor=1):

- Dozen: type=bigger, factor=1/12 ≈ 0.0833 → 1 docena = 12 units (`1 / 0.0833 = 12`)
- Pack of 6: type=bigger, factor=1/6 ≈ 0.1667 → 1 pack = 6 units
- Wait. Si factor < 1, type=bigger. Si factor > 1, type=smaller.
- Ejemplo verdad: g (gramo) en categoría Weight (kg reference): factor=1000, type=smaller. 1 kg = 1000 g.

### precisión y `digits=0`

`factor`, `factor_inv`, `rounding` usan `digits=0` que en Odoo se traduce a **NUMERIC sin precision limit en PostgreSQL**. Esto se hace para evitar pérdida de precisión en factores grandes/pequeños (ej. 1/12 = 0.083333...).

### Bloqueo de UoMs maestras

`_filter_protected_uoms()`: bloquea modificar/borrar UoMs creadas vía `ir.model.data` con `module='uom'` (las que vienen del módulo base), excepto las "desprotegidas" en `_unprotected_uom_xml_ids`:

- `product_uom_hour` (también protegida cuando hr_timesheet instalado).
- `product_uom_dozen`.

Estas UoMs estándar **no se pueden borrar**, solo archivar.

### Onchange critical fields warning

Si modificas `factor`, `factor_inv`, `uom_type`, `rounding` o `category_id` en una UoM protegida con más de 1 día de antigüedad → warning UI (no error):

> "Some critical fields have been modified... Existing data WON'T be updated by this change. As units of measure impact the whole system, this may cause critical issues. E.g. modifying the rounding could disturb your inventory balance."

**Implicación**: cambiar el factor de una UoM YA usada en stock/SO/PO **NO actualiza retroactivamente** las quantities. Quedan inconsistencias silenciosas.

## API de conversión

### `_compute_quantity(qty, to_unit, round=True, rounding_method='UP', raise_if_failure=True)`

Convierte cantidad de `self` a `to_unit`:

```python
qty_in_dozen = uom_units._compute_quantity(qty=24, to_unit=uom_dozen)
# Returns 2.0
```

**Comportamientos**:

- Si `self` es empty o `qty=0` → devuelve `qty` sin tocar.
- Si `self == to_unit` → devuelve `qty`.
- Si categorías distintas:
  - `raise_if_failure=True` (default) → `UserError`.
  - `raise_if_failure=False` → devuelve `qty` original sin convertir.
- Aplica `float_round(amount, precision_rounding=to_unit.rounding, rounding_method=rounding_method)` si `round=True` y hay `to_unit`.

**`rounding_method`** (de `odoo.tools.float_utils.float_round`):

- `'UP'` (default) — siempre redondea hacia arriba (típico para reservar stock).
- `'DOWN'` — siempre hacia abajo.
- `'HALF-UP'` — al más cercano, 0.5 hacia arriba.

### `_compute_price(price, to_unit)`

Convierte precio (mantiene el "precio total" — fórmula inversa al qty):

```python
# 100 EUR/kg → ?/g
uom_kg._compute_price(price=100, to_unit=uom_g)
# 100 * kg.factor / g.factor = 100 * 1 / 1000 = 0.1 EUR/g  ✓
```

**Diferencia clave con `_compute_quantity`**:

- `_compute_quantity` divide por `self.factor` y multiplica por `to_unit.factor` (cantidad: más cantidad en UoM más pequeña).
- `_compute_price` multiplica por `self.factor` y divide por `to_unit.factor` (precio: menor precio unitario en UoM más pequeña).
- **NO** aplica rounding.
- Si categorías distintas → **devuelve `price` sin convertir** (no error). Confunde a quien espera UserError.

### Helper: precision de comparación

```python
from odoo.tools import float_compare, float_is_zero, float_round

float_compare(qty1, qty2, precision_rounding=uom.rounding)  # -1, 0, 1
float_is_zero(qty, precision_rounding=uom.rounding)
float_round(value, precision_rounding=uom.rounding, rounding_method='HALF-UP')
```

**NUNCA comparar quantities con `==`** — usar `float_compare`. Errores típicos en stock cuando float arrastra epsilon.

## Decimal Precision (`decimal.precision`)

Modelo `decimal.precision` (en `product/models/decimal_precision.py`) define dígitos de precisión por concepto. Está pensado para campos `Float(digits='Nombre')`:

| Nombre (en BD) | Default digits | Uso |
|---|---|---|
| `Product Unit of Measure` | 2 | Quantities (qty, qty_done, qty_invoiced...) |
| `Product Price` | 2 | Precios unitarios |
| `Discount` | 2 | Descuentos |
| `Product Volume` | 2 | Volumen |
| `Stock Weight` | 2 | Peso |
| `Account` | 2 | Importes contables |
| `Percentage Analytic` | 4 | Distribuciones analíticas |

**Diferencia con `uom.rounding`**:

- `decimal.precision` controla **dígitos decimales** de campos Float → afecta display + storage (NUMERIC scale).
- `uom.rounding` controla **granularidad de redondeo** al convertir → afecta el valor calculado.
- Conviven: `Product Unit of Measure` = 2 dígitos significa "guardar con 2 decimales", pero la UoM puede tener `rounding=1.0` (sin decimales, ej. piezas que no se cortan).

Modificar la precisión vía Settings > Technical > Decimal Accuracy:

- Solo afecta campos `Float(digits='Nombre')`.
- **NO afecta** campos `Float('Etiqueta', digits=(precision, scale))` con tupla hardcoded.
- Aumentar la precisión es seguro. Reducirla puede provocar pérdida silenciosa.

## Dual UoM: stock, sale, purchase

Tres campos distintos en `product.template`:

| Campo | Default | Uso real |
|---|---|---|
| `uom_id` | `uom.product_uom_unit` (Units) | **UoM de stock e inventario**. Sale UoM por defecto. |
| `uom_po_id` | computed = `uom_id` | **UoM de compras**. Editable pero debe estar en la misma `uom_category_id`. |
| (no hay uom_sale_id en CE) | - | Sale usa `uom_id` directamente. Indaws / OCA añaden `uom_sale_id`. |

### `_compute_uom_po_id`

```python
@api.depends('uom_id')
def _compute_uom_po_id(self):
    for template in self:
        if not template.uom_po_id or template.uom_id.category_id != template.uom_po_id.category_id:
            template.uom_po_id = template.uom_id
```

→ Si cambias `uom_id` a una categoría distinta, `uom_po_id` se resetea a `uom_id` (porque sino sería inválido). Si están en la misma categoría, se mantiene.

### Caso típico: Compra en cajas, stock en unidades

```
uom_id = Units
uom_po_id = Box of 12 (factor=1/12 en categoría Unit)
```

Al confirmar una PO de "5 boxes":

- PO line: qty=5, uom=Box of 12.
- Stock move generado: qty=60 units, uom=Units (conversión automática vía `_compute_quantity`).
- Invoice (vendor bill): vendor invoice line por defecto en **uom de la PO line** (Box of 12) — el contable verá "5 cajas a X €/caja" o "60 unidades a Y €/unidad" según ajuste.

### Quantities en stock.move

`stock.move` tiene **dos campos** de UoM y dos de qty:

| Campo | Significado |
|---|---|
| `product_uom_qty` | Cantidad solicitada (demanda), en `product_uom`. |
| `product_uom` | UoM de la demanda (lo que se mostró en PO/SO/MO). |
| `quantity` (v18) | Cantidad realmente hecha/movida, **en `product.uom_id`** (stock UoM). |
| `product_id.uom_id` | Stock UoM (siempre la del template). |

En v17 era `quantity_done` + las reservas en `move_line_ids`. v18 unificó. Detalle en `stock/v19_changes.md` y `stock/core.md`.

### Sale: product_uom override

En `sale.order.line`:

```python
product_uom = fields.Many2one(
    'uom.uom',
    compute='_compute_product_uom', readonly=False, store=True,
    domain="[('category_id', '=', product_uom_category_id)]",
)
product_uom_category_id = fields.Many2one(related='product_id.uom_id.category_id', depends=['product_id'])
```

**Comportamiento**:

- Default = `product_id.uom_id`.
- Editable (readonly=False) pero **limitado a misma categoría** (domain).
- `_compute_product_uom_qty()` recalcula `product_uom_qty` si cambia el packaging.

### Purchase: product_uom default uom_po_id

```python
self.product_uom = self.product_id.uom_po_id or self.product_id.uom_id
```

PO line default a `uom_po_id` (no a `uom_id`).

### Account move line: UoM en factura

`account.move.line.product_uom_id` (M2o uom.uom). En facturas de venta/compra:

- Si la factura viene de SO/PO → hereda la UoM de la línea origen.
- Si se crea factura manual con producto → default es `product_id.uom_id`.
- Editable por el usuario en la línea de factura.

**Impacto en sii / l10n_es**: la UoM de la factura va a XML SII como "Unidad de Medida". Para algunas claves de IVA hay validación AEAT (ej. m³ para gas).

## Quantities en MRP

### BOM Lines (`mrp.bom.line`)

- Cada línea de BoM tiene su propia `product_uom_id` (M2o uom.uom), independiente del producto componente.
- Restricción: debe estar en la categoría de `product_id.uom_id`.
- Conversión a stock UoM al consumir.

### Producto fabricado vs cantidad de BOM

- `mrp.bom.product_uom_id` define la UoM en la que se "produce" la cantidad.
- `mrp.bom.product_qty` es la cantidad fabricada por BoM.
- Ejemplo: "Esta BoM produce 12 Unit per 1 batch" → `product_uom_id=Dozen, product_qty=1` o `product_uom_id=Unit, product_qty=12`. Equivalentes en teoría, distintos en UI.

### Workcenter capacity

- `mrp.workcenter.capacity.product_uom_id` no existe en v18 base — la capacity se expresa por producto, en su UoM de stock.

## Packaging (`product.packaging`) — v18 (eliminado en v19)

`product.packaging`:

| Campo | Tipo |
|---|---|
| `product_id` | M2o product.product |
| `name` | Char (ej. "Box of 10") |
| `qty` | Float (ej. 10.0) |
| `product_uom_id` | related a `product_id.uom_id` |
| `barcode` | Char unique |
| `sales`, `purchase` | Boolean — visible en sale/purchase order |

**Diferencia con UoM**:

- UoM es un sistema de conversión **simétrico** (Units ↔ Dozen).
- Packaging es **asimétrico**: representa una agrupación física (caja, palet) **expresada en la stock UoM**. No es una UoM nueva.
- En SO line: `product_packaging_id` + `product_packaging_qty` (qty de packagings) + `product_uom_qty` (qty en UoM unitaria).
- Constraint `_check_product_uom_qty`: `product_uom_qty == product_packaging_qty * packaging.qty` (en la UoM del packaging, redondeado).

**En v19**: `product.packaging` se ELIMINA. Su rol pasa a `product.uom` (un modelo nuevo, link entre producto y UoM con barcode). Ver `v19_changes.md`.

## Conversiones en compras con vendor pricelist

`product.supplierinfo` tiene:

- `product_uom`: UoM del precio del vendor (puede ser DIFERENTE de uom_po_id, ej. vendor cotiza en Pack 6 pero compramos en Pack 12).
- `price`: precio en esa UoM.
- `min_qty`: cantidad mínima en esa UoM.

En `purchase.order.line._compute_price_unit`:

```python
po_line_uom = line.product_uom or line.product_id.uom_po_id
price_unit = seller.product_uom._compute_price(seller.price, po_line_uom)
```

→ Triple conversión potencial: vendor UoM → PO line UoM → stock UoM al confirmar.

## Configuración: grupo "Unidades de Medida"

Setting: **Inventory > Settings > "Units of Measure"** (también en Sales/Purchase).

- Grupo XML: `uom.group_uom`.
- Si **desactivado**: los campos UoM se ocultan en SO/PO/stock.move, todo asume "stock UoM" del producto.
- Si **activado**: aparecen los selectors de UoM en líneas.

**Práctica común**: activarlo solo en empresas que realmente usan multi-UoM. Activarlo en BD existente es seguro; **desactivarlo** después puede esconder problemas (líneas en UoMs no estándar quedan ocultas pero siguen activas).

## Hour / Working Time UoM

Categoría especial usada por hr_timesheet:

- `uom.product_uom_hour` (reference de la categoría "Working Time").
- `uom.product_uom_day` (factor relativo a hour).
- `uom.product_uom_week`, etc.

**Protección extra cuando hr_timesheet instalado**: `_unprotected_uom_xml_ids()` ya no incluye `product_uom_hour` (override del módulo) → no se puede ni borrar ni desactivar el "hour".

Es la UoM por defecto de servicios facturables (`service_policy='delivered_timesheet'`).

## POS y UoM

POS tiene **soporte limitado** de UoMs:

- `pos.order.line.product_uom_id` related a producto (NO editable en POS UI v18).
- Si el producto tiene `uom_id != Units`, la UoM se muestra junto a la qty en el ticket.
- POS **no permite cambiar UoM en línea** ni vender en packaging — siempre stock UoM.
- Productos que se venden por peso (`balance`/`scale=True` en POS): vienen del backend con UoM "kg" o similar — se introducen vía balanza o `available_in_pos` + integración.

Detalle en `pos/gotchas.md` y `pos/core.md`.

## Helpers de uso común

```python
# Convertir 5 docenas a unidades:
uom_dozen._compute_quantity(5, uom_units)
# → 60.0

# Convertir precio 100 EUR/unidad a EUR/docena:
uom_units._compute_price(100, uom_dozen)
# → 1200.0  (100 * units.factor / dozen.factor = 100 * 1 / (1/12) = 1200)

# Verificar si dos UoMs son compatibles:
uom_a.category_id == uom_b.category_id

# Redondeo seguro:
from odoo.tools import float_round
qty_rounded = float_round(qty, precision_rounding=uom.rounding, rounding_method='UP')

# Comparación segura (NUNCA ==):
from odoo.tools import float_compare
if float_compare(qty1, qty2, precision_rounding=uom.rounding) > 0:
    ...

# Cero seguro:
from odoo.tools import float_is_zero
if float_is_zero(qty, precision_rounding=uom.rounding):
    ...
```

## Cross-references

- **Gotchas numerados de UoM**: ver `gotchas.md` (UOG*).
- **Cambios v19 (refactor radical)**: ver `v19_changes.md`.
- **Integración stock/MRP/sale/purchase**: ver `integrations.md`.
- **Decimal precision en valoración**: ver `stock/account.md`.
- **UoM en facturación electrónica (SII, EDI)**: ver `account/operations.md` y `ecosystem/oca-accounting/`.
- **product.packaging completo**: este fichero + `stock/gotchas.md` STKG61 (dual UoM stock vs purchase).
