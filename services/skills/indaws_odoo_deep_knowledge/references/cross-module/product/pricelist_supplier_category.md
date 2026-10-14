# Pricelist + Supplierinfo + Category — Odoo 18.0

La cadena **categoría → producto → vendor → pricelist** determina precio, coste, cuenta contable y valoración de cualquier transacción. Este fichero documenta los tres modelos con sus algoritmos de resolución.

---

## A. `product.category` — la fuente de defaults

### Modelo

| Campo | Tipo | Notas |
|---|---|---|
| `name` | Char, index='trigram' | |
| `parent_id` | M2o `product.category`, ondelete='cascade' | **`_parent_store=True`** + `parent_path` indexada |
| `parent_path` | Char, index | Path tipo `'1/3/8/'` para queries con `parent_of`/`child_of` |
| `complete_name` | Char computed/stored, **recursive=True** | "Parent / Sub / Leaf" |
| `child_id` | O2m | Subcategorías |
| `product_count` | Integer computed | Templates con `categ_id child_of self` (recorre toda la rama) |
| `product_properties_definition` | PropertiesDefinition | Schema de campos dinámicos para `template.product_properties` |

**Constraints**:

- `_check_category_recursion` — no se permiten ciclos en `parent_id` (`_has_cycle()`).
- 3 categorías master no se pueden borrar (UserError):
  - `product.product_category_all` — root "All".
  - `product.cat_expense` — "Expenses" (para hr_expense).
  - `product.product_category_1` — "Saleable".
- Categoría hereda `mail.thread` (chatter en categoría).

### Property fields añadidos por otros módulos

`product.category` es un **bus** de propiedades por compañía. Los módulos que tocan facturación/stock/valoración cuelgan campos `company_dependent=True`:

#### Por `account` module

| Campo (en `product.template`) | Cascada |
|---|---|
| `property_account_income_id` (M2o account.account) | Producto → categoría (`property_account_income_categ_id`) → journal default |
| `property_account_expense_id` | Producto → categoría (`property_account_expense_categ_id`) → journal default |

#### Por `stock_account` module (en `product.category`)

| Campo | Tipo | Notas |
|---|---|---|
| `property_cost_method` | Selection: `'standard'` / `'fifo'` / `'average'` | **Coste/valoración** — vive en CATEGORÍA, no en producto. `company_dependent=True` |
| `property_valuation` | Selection: `'manual_periodic'` / `'real_time'` | Valoración manual (sin SVL automáticos) vs Anglo-Saxon en tiempo real |
| `property_stock_journal` | M2o `account.journal` | Diario donde se asientan los SVL en `real_time` |
| `property_stock_account_input_categ_id` | M2o `account.account` | "Stock Input" (cuenta puente entradas) |
| `property_stock_account_output_categ_id` | M2o `account.account` | "Stock Output" (cuenta puente salidas) |
| `property_stock_valuation_account_id` | M2o `account.account` | Cuenta de balance "Stock Valuation" |

#### Por `stock` module (en `product.template`)

| Campo | Notas |
|---|---|
| `property_stock_production` | M2o `stock.location` (default por producto) |
| `property_stock_inventory` | M2o `stock.location` (default por producto) |
| `removal_strategy_id` (en categoría + ubicación) | M2o `stock.removal.strategy` (FIFO/LIFO/FEFO/Closest Location) |

### Resolución de property fields — cascada

Para campos `company_dependent=True` en `product.template` con fallback a categoría:

```
1. product.template.property_X_id                      (override per-product, per-company)
   ↓ si vacío
2. product.template.categ_id.property_X_categ_id       (default category, per-company)
   ↓ si vacío
3. product.template.categ_id.parent_id.property_X_categ_id  (cascade up the tree)
   ↓ si vacío
4. Journal/system default (último recurso)
```

**Implementación real** en `_get_product_accounts()` (account module):

```python
return {
    'income': self.property_account_income_id
              or self._get_category_account('property_account_income_categ_id'),
    'expense': self.property_account_expense_id
               or self._get_category_account('property_account_expense_categ_id'),
}
```

`_get_category_account` recorre la cadena de padres hasta encontrar valor.

### `removal_strategy_id` — dual: categoría + ubicación

```python
# Resolución (en stock):
strategy = stock.location.removal_strategy_id \
        or product.product_tmpl_id.categ_id.removal_strategy_id \
        or default ('fifo')
```

→ **Ubicación tiene prioridad sobre categoría**. Esto sorprende: si configuras FEFO en categoría "Lácteos" pero la ubicación "Camara fría" tiene FIFO, gana FIFO. Importante revisar ambos.

### Cambio de categoría con stock existente

`product.template.write({'categ_id': X})` puede disparar **reevaluación masiva de stock**:

`stock_account.product_template._product_categ_id_changed()`:

1. Si la nueva categoría tiene `property_cost_method` diferente:
   - Para AVCO → FIFO: crea SVL de "out" para vaciar el coste promedio y "in" con FIFO layers.
   - Para FIFO → AVCO: similar, crea SVL para colapsar layers en average.
2. Si `property_valuation` cambia de `manual_periodic` a `real_time`:
   - Genera asientos contables en `property_stock_journal` para regularizar el stock actual.
3. Si afecta valoración, llama a `_svl_empty_stock_am` + `_svl_replenish_stock_am`.

**Side-effect grave**: mover masivamente productos de categoría puede generar **cientos de SVL + asientos contables** en un solo `write`. Hacerlo en horas no productivas y con backup.

---

## B. `product.supplierinfo` — vendor pricelist

### Modelo

| Campo | Tipo | Notas |
|---|---|---|
| `partner_id` | M2o `res.partner`, ondelete='cascade', required, check_company | Vendor |
| `product_tmpl_id` | M2o `product.template`, ondelete='cascade', check_company, **index** | Required |
| `product_id` | M2o `product.product`, check_company | **Opcional**: si vacío aplica a TODAS las variantes |
| `product_name` | Char | Nombre del producto que aparece en el RFQ (pisa el interno) |
| `product_code` | Char | Código del producto en el vendor |
| `product_uom` | M2o uom, **related** a `product_tmpl_id.uom_po_id` | NO editable directamente; cambias `uom_po_id` del template |
| `min_qty` | Float, digits='Product Unit of Measure' | Cantidad mínima EN UoM DEL VENDOR para que esta línea aplique |
| `price` | Float | Precio en `currency_id`, EN UoM DEL VENDOR |
| `discount` | Float (digits='Discount') | Descuento % |
| `price_discounted` | Float computed | `price * (1 - discount/100)` |
| `currency_id` | M2o res.currency, default=company.currency | Moneda del vendor (puede diferir de company) |
| `delay` | Integer, default=1 | Lead time en días (PO → recepción) |
| `sequence` | Integer, default=1 | **Determina prioridad** (no precio) |
| `date_start` / `date_end` | Date | Validez temporal |
| `company_id` | M2o res.company, default=current | Per-company |
| `product_variant_count` | Integer related | Para UI |

**`_order`** = `'sequence, min_qty DESC, price, id'`

### `_select_seller()` — algoritmo paso a paso

Punto de entrada: `product._select_seller(partner_id, quantity, date, uom_id, ordered_by='price_discounted', params)`.

```python
def _select_seller(self, partner_id=False, quantity=0.0, date=None, uom_id=False,
                   ordered_by='price_discounted', params=False):
    sort_key = ('price_discounted', 'sequence', 'id')
    if ordered_by != 'price_discounted':
        sort_key = (ordered_by, 'price_discounted', 'sequence', 'id')

    sellers = self._get_filtered_sellers(...)  # ver abajo
    res = self.env['product.supplierinfo']
    for seller in sellers:
        if not res or res.partner_id == seller.partner_id:
            res |= seller
    return res and res.sorted(sort_function)[:1]
```

**Paso 1**: `_get_filtered_sellers` filtra el universo de `seller_ids` del producto:

```python
for seller in self.seller_ids.sorted(lambda s: (s.sequence, -s.min_qty, s.price, s.id)):
    # Convertir qty a UoM del seller
    quantity_uom_seller = quantity
    if quantity and uom_id and uom_id != seller.product_uom:
        quantity_uom_seller = uom_id._compute_quantity(quantity, seller.product_uom)

    # Filtros (continue si falla):
    if seller.date_start and seller.date_start > date: continue
    if seller.date_end and seller.date_end < date: continue
    if partner_id and seller.partner_id not in [partner_id, partner_id.parent_id]:
        continue
    if float_compare(quantity_uom_seller, seller.min_qty, precision_digits=...) == -1:
        continue
    # ... otros filtros (multi-currency, multi-company)
```

**Paso 2**: del set filtrado, toma **el primer vendor** y todos sus `supplierinfo` (es decir, agrupa por `partner_id`).

**Paso 3**: ordena por `price_discounted` y devuelve **el primero**.

### Por qué la documentación oficial dice "selección por sequence" es engañosa

El **PRIMER FILTRO** (sellers que pasan) ya está ordenado por `(sequence, -min_qty, price, id)` antes de filtrar. El **primer vendor** que aparezca (por sequence) "captura" el resultado.

→ Si tienes:

```
Vendor A: sequence=10, min_qty=1,  price=100
Vendor B: sequence=20, min_qty=1,  price=80
```

Para qty=1: aparecen ambos, A primero (sequence menor). `res = A` después de la iteración (porque al ver B con partner distinto, se ignora). Resultado: **A** aunque B sea más barato.

Si quieres "elegir más barato siempre": eliminar sequence diferenciador (poner todos a 1) y dejar que el `sorted(sort_function)[:1]` final decida por precio.

### Lo que realmente cambia el resultado

1. **`min_qty` evaluado en UoM del vendor** — cambiar la UoM de compra puede dejar a un vendor "por debajo de min_qty" y excluirlo.
2. **Validez de fechas** — supplierinfos antiguos siguen activos si no tienen `date_end`. Cron típico de limpieza no existe.
3. **`partner_id` parent** — si el partner principal de la PO es una sucursal/contacto, también valida supplierinfos del partner padre (`partner_id.parent_id`).
4. **Multi-currency** — `price_discounted` se convierte a moneda de la company para comparar entre vendors en distintas monedas.

### Cascada variant → template

Si `seller.product_id` está vacío, aplica a TODAS las variantes del `product_tmpl_id`. Si está set, SOLO a esa variante.

**Patrón típico**: precios "base" en supplierinfo a nivel template (`product_id=False`) y **overrides** para variantes específicas (`product_id=variante.id`).

`_get_filtered_sellers` evalúa AMBOS y deja que el sort final elija.

### Bridge `_sanitize_vals`

```python
def _sanitize_vals(self, vals):
    if vals.get('product_id') and not vals.get('product_tmpl_id'):
        product = self.env['product.product'].browse(vals['product_id'])
        vals['product_tmpl_id'] = product.product_tmpl_id.id
```

→ Si creas un supplierinfo solo con `product_id`, Odoo rellena `product_tmpl_id` automáticamente. Si das `product_id` y `product_tmpl_id` distintos (variante de otro template), **NO valida** — bug latente.

### En `purchase` module — uso real

`purchase.order.line._compute_price_unit`:

```python
seller = line.product_id._select_seller(
    partner_id=line.partner_id,
    quantity=line.product_qty,
    date=line.date_order,
    uom_id=line.product_uom,
)
if seller:
    price_unit = seller.product_uom._compute_price(seller.price, line.product_uom)
    line.price_unit = price_unit
```

→ Al crear PO line, busca seller con la qty/UoM/fecha de la línea. Cambiar qty puede cambiar vendor (porque otro vendor cumple min_qty ahora). Cambiar UoM puede excluir/incluir vendors.

`_run_buy` en `procurement.group` también usa `_select_seller` para crear POs automáticas en MTO.

---

## C. `product.pricelist` y `product.pricelist.item`

### Modelo `product.pricelist`

| Campo | Tipo | Notas |
|---|---|---|
| `name` | Char, translate | |
| `active` | Boolean | Archivable. Reglas archivadas siguen activas (ver gotcha PLSG abajo) |
| `currency_id` | M2o, required | Moneda de la pricelist. Conversión automática desde currency del producto |
| `company_id` | M2o, default=current | Si vacío → multi-company |
| `country_group_ids` | M2M `res.country.group` | Auto-selección por país de partner |
| `sequence` | Integer, default=16 | Para ordenar en `res.partner._get_partner_pricelist_multi` |
| `item_ids` | O2m `product.pricelist.item` | **Domain filtra productos activos** (`product_tmpl_id.active=True`) |

**Constraint borrado**:

- No se puede borrar una pricelist usada como `base_pricelist_id` en items de otras pricelists (UserError).

### Selección de pricelist para un partner

`res.partner.property_product_pricelist` (computed/stored).

`pricelist._get_partner_pricelist_multi(partner_ids)`:

```
1. partner.specific_property_product_pricelist        (override directo, M2o stored)
   ↓ si vacío
2. pricelist con country_group_ids matching partner.country_id
   ↓ si no hay match
3. ir.config_parameter 'res.partner.property_product_pricelist_<company_id>'
   ↓ si vacío
4. ir.config_parameter 'res.partner.property_product_pricelist'
   ↓ si vacío
5. Primera pricelist activa de la company
```

**Gotcha frecuente**: cuando un cliente "no aparece con la pricelist correcta", el override `specific_property_product_pricelist` puede estar set sin saberlo (residuo de import o copia). Comprobar `partner.with_context(prefetch_fields=False).specific_property_product_pricelist`.

### Modelo `product.pricelist.item`

| Campo | Tipo | Notas |
|---|---|---|
| `pricelist_id` | M2o pricelist, ondelete='cascade', required | |
| `applied_on` | Selection: `'3_global'` / `'2_product_category'` / `'1_product'` / `'0_product_variant'` | **Discriminador clave** |
| `display_applied_on` | Selection: `'1_product'` / `'2_product_category'` | UI helper |
| `categ_id` | M2o product.category | Required si `applied_on='2_product_category'` |
| `product_tmpl_id` | M2o product.template | Required si `applied_on='1_product'` |
| `product_id` | M2o product.product | Required si `applied_on='0_product_variant'` |
| `min_quantity` | Float, digits='Product Unit of Measure', default=0 | **En UoM DEL PRODUCTO** (no del SO line) |
| `date_start`/`date_end` | Datetime | Validez |
| `base` | Selection: `'list_price'` / `'standard_price'` / `'pricelist'` | Precio base |
| `base_pricelist_id` | M2o pricelist | Required si `base='pricelist'` |
| `compute_price` | Selection: `'fixed'` / `'percentage'` / `'formula'` | |
| `fixed_price` | Float | Precio fijo (modo fixed) |
| `percent_price` | Float | % descuento (modo percentage) |
| `price_discount` | Float | % descuento (modo formula) — positivo = descuento, negativo = markup |
| `price_markup` | Float computed/inverse | `= -price_discount` (helper UI) |
| `price_round` | Float | Redondeo al múltiplo de este valor (modo formula) |
| `price_surcharge` | Float | Importe fijo añadido tras descuento (modo formula) |
| `price_min_margin`/`price_max_margin` | Float | Límites de margen sobre `base_price` (modo formula) |

**`_order`** = `'applied_on, min_quantity desc, categ_id desc, id desc'`.

Esto **DETERMINA LA PRECEDENCIA**: orden lexicográfico de `applied_on` (`'0_'` < `'1_'` < `'2_'` < `'3_'`) → **variante > producto > categoría > global**. Luego `min_quantity DESC` (cantidad mayor gana → "5+ unidades a 90€" gana sobre "1+ unidades a 100€" cuando qty=10).

### `_get_applicable_rules` — dominio SQL

```python
[
    ('pricelist_id', '=', self.id),
    '|', ('categ_id', '=', False), ('categ_id', 'parent_of', products.categ_id.ids),
    '|', ('product_tmpl_id', '=', False), templates_domain,
    '|', ('product_id', '=', False), products_domain,
    '|', ('date_start', '=', False), ('date_start', '<=', date),
    '|', ('date_end', '=', False), ('date_end', '>=', date),
]
```

→ `categ_id` se evalúa con **`parent_of`** (no exact match): una regla en categoría "Electrónica" aplica a producto en "Electrónica/Smartphones". Es la única jerarquía respetada en pricelist.

**`active_test=False`**: las reglas archivadas SIGUEN considerándose. Solo se descartan vía `domain` arriba. Si quieres "apagar" temporalmente una regla, NO archives — añade `date_end` en el pasado.

### `_is_applicable_for` — chequeo por producto + qty

```python
def _is_applicable_for(self, product, qty_in_product_uom):
    if self.min_quantity and qty_in_product_uom < self.min_quantity:
        return False  # qty insuficiente

    if self.applied_on == "2_product_category":
        # Verifica categoría con parent_path (no solo igualdad)
        if (product.categ_id != self.categ_id
            and not product.categ_id.parent_path.startswith(self.categ_id.parent_path)):
            return False
    elif self.applied_on == "1_product":
        # match por template
    elif self.applied_on == "0_product_variant":
        # match por variant
    return True
```

**`min_quantity` se evalúa en `product.uom_id`**, NO en la UoM de la SO line. En `_compute_price_rule`:

```python
if target_uom != product_uom:
    qty_in_product_uom = target_uom._compute_quantity(quantity, product_uom, raise_if_failure=False)
```

→ Si SO line: qty=12 Dozen (= 144 unidades en stock UoM = Units), `min_quantity=10` (en stock UoM) sí matchea. **`raise_if_failure=False`** silencia errores de categoría incompatible (caso raro).

### `_compute_price` — el motor

```python
def _compute_price(self, product, quantity, uom, date, currency=None):
    product_uom = product.uom_id
    if product_uom != uom:
        convert = lambda p: product_uom._compute_price(p, uom)
    else:
        convert = lambda p: p

    if self.compute_price == 'fixed':
        price = convert(self.fixed_price)  # fixed_price está en product_uom
    elif self.compute_price == 'percentage':
        base_price = self._compute_base_price(product, quantity, uom, date, currency)
        price = base_price - (base_price * (self.percent_price / 100))
    elif self.compute_price == 'formula':
        base_price = self._compute_base_price(...)
        discount = self.price_discount if self.base != 'standard_price' else -self.price_markup
        price = base_price - (base_price * (discount / 100))
        if self.price_round:
            price = float_round(price, precision_rounding=self.price_round)
        if self.price_surcharge:
            price += convert(self.price_surcharge)
        if self.price_min_margin:
            price = max(price, price_limit + convert(self.price_min_margin))
        if self.price_max_margin:
            price = min(price, price_limit + convert(self.price_max_margin))
    return price
```

**Orden de operaciones (modo formula)**: `(base × (1 - discount/100))` → `round` → `+ surcharge` → `clamp(min_margin, max_margin)`.

**`fixed_price` se interpreta en `product.uom_id`** (stock UoM). Si SO usa otra UoM, se convierte vía `uom._compute_price` (que NO redondea — precio exacto).

### `_compute_base_price` — el origen

```python
if self.base == 'pricelist' and self.base_pricelist_id:
    price = base_pricelist_id._get_product_price(product, quantity, currency, uom, date)
    src_currency = base_pricelist_id.currency_id
elif self.base == 'standard_price':
    price = product._price_compute('standard_price', uom=uom, date=date)
    src_currency = product.cost_currency_id
else:  # 'list_price'
    price = product._price_compute('list_price', uom=uom, date=date)
    src_currency = product.currency_id

if src_currency != currency:
    price = src_currency._convert(price, currency, env.company, date, round=False)
```

**Recursión soportada**: pricelist B basada en pricelist A → `_get_product_price` recursivo. Constraint `_check_pricelist_recursion` evita auto-referencia directa.

**Multi-currency**: `_convert` con `round=False` mantiene precisión hasta el último paso.

### `price_extra` de variantes — interacción crítica

`product._price_compute('list_price')` (en `product.template`):

```python
elif price_type == 'list_price':
    price += template._get_attributes_extra_price()
```

→ `_get_attributes_extra_price` lee `current_attributes_price_extra` del context (set por `_get_product_price_context`). Es decir: **el price_extra de la variante SE APLICA AL BASE PRICE antes de la regla**.

Resultado: `list_price=100 + variant_extra=20` → base = 120 → regla "10% descuento" → 108.

Si la base es `'standard_price'`, los `price_extra` NO se aplican (los attributes no afectan al coste).

### Resolución completa — flujo end-to-end

```
SO line.price_unit = pricelist._get_product_price(product, qty, currency, uom, date)
  → pricelist._compute_price_rule(product, qty, uom, currency, date)
    → rules = _get_applicable_rules(product, date)  ← SQL: filtra por categ/tmpl/product/fechas
    → for rule in rules (ordered by applied_on, min_qty desc, ...):
        if rule._is_applicable_for(product, qty_in_product_uom):
            suitable_rule = rule; break
    → suitable_rule._compute_price(product, qty, uom, date, currency)
        → _compute_base_price (resuelve 'base' = list_price/standard_price/another pricelist)
        → aplica fixed/percentage/formula
```

### Frozen at confirm

`sale.order.line.pricelist_item_id` (M2o, store) **se congela al confirmar la SO**. Cambios posteriores en la pricelist NO afectan a SO confirmadas.

Recálculo manual: botón "Actualizar precios" o `_recompute_prices()` en `sale.order` (que llama a `_compute_price_unit` con la pricelist actual).

### Constraints

- `_check_pricelist_recursion` — `pricelist_id != base_pricelist_id` directamente.
- `_check_base_pricelist_id` — `base='pricelist'` requiere `base_pricelist_id`.
- `_check_date_range` — `date_start < date_end`.
- `_check_margin` — `price_min_margin <= price_max_margin`.
- `_check_product_consistency` — `applied_on` debe tener su campo correspondiente set.

### Auto-cleanup en write/create

`_onchange_rule_content` y `create` override:

- Si `product_id` set → `applied_on='0_product_variant'`, otros campos a None.
- Si `product_tmpl_id` set → `applied_on='1_product'`.
- Si `categ_id` set (y no es "All") → `applied_on='2_product_category'`.
- Sino → `applied_on='3_global'`.

→ Imports de pricelist.item con `applied_on` mal set se "corrigen" automáticamente. A veces no como esperas.

---

## D. Integración cruzada: Categoría + Pricelist + Vendor

### Caso real: PO automática (MTO/reorder)

Flujo:

```
1. SO confirm → procurement → _run_buy()
2. _run_buy busca seller con _select_seller (qty, partner, date)
3. seller.price (en seller.currency_id, seller.product_uom) → PO line.price_unit
4. price_unit se convierte a line.product_uom (que default es uom_po_id)
5. La PO no usa pricelists (purchases tienen vendor pricing directo)
```

→ **Vendor pricing y customer pricing son universos disjuntos**. Pricelists son SOLO para venta. Para compras: supplierinfo.

### Caso real: facturación

```
1. SO line.price_unit (frozen, en SO currency)
2. Invoice line hereda price_unit
3. Si fiscal_position aplica, puede remapar cuentas (NO precios)
4. Cuenta contable = producto.property_account_income_id 
                  → categ.property_account_income_categ_id
                  → journal default
```

### Caso real: stock valuation

```
1. stock.move done → stock.valuation.layer (SVL)
2. SVL.value = qty × unit_cost (según property_cost_method)
3. Si property_valuation='real_time':
    - asiento contable con:
      - stock_valuation account (categ.property_stock_valuation_account_id)
      - stock_input/output (categ.property_stock_account_*)
    - en journal: categ.property_stock_journal
```

→ **Cambiar la categoría de un producto cambia todo**: costing method, journal de SVL, cuentas. Es la operación más peligrosa en productos.

---

## E. Gotchas resumen (extendidos en `gotchas.md`)

Ver `gotchas.md`:

- **CTG1-CTG10**: category cascade, cost_method, removal_strategy, property fields
- **PLSG1-PLSG12**: pricelist resolution, min_qty UoM, frozen-at-confirm, formula edge cases
- **SIG1-SIG8**: supplierinfo, _select_seller pitfalls

## Cross-references

- `core.md` — producto base
- `variants.md` — `price_extra` y su interacción con pricelist
- `uom.md` — conversiones UoM en pricelist/supplierinfo
- `gotchas.md` — gotchas numerados
- `integrations.md` — uso desde sale/purchase/account
- `stock/account.md` — SVL, valuation, cost methods
- `account/operations.md` — fiscal positions, tax mapping
- `purchase/core.md` — vendor selection en PO
