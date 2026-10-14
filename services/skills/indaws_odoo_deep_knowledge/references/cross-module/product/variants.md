# Variantes de Producto — Odoo 18.0

Cómo funcionan los atributos, valores, líneas de atributo, valores materializados, exclusiones y variantes dinámicas. Modelo "matricial" entre 5 entidades.

## Las 5 entidades

```
product.attribute              "Talla"                   (catálogo global)
  ├── product.attribute.value  "S", "M", "L"             (valores globales)
  │
  └─→ product.template.attribute.line  (PTAL)
         │     "este template usa este atributo con estos values"
         │     product_tmpl_id + attribute_id + value_ids (M2M a PAV)
         │
         └─→ product.template.attribute.value  (PTAV)
                "value materializado para este template"
                attribute_line_id + product_attribute_value_id
                + price_extra
                + exclude_for (O2M a exclusion)

product.template.attribute.exclusion
   "esta PTAV excluye estas otras PTAVs"
   product_template_attribute_value_id + value_ids (M2M PTAV)

product.product                "Camiseta S"
   product_template_attribute_value_ids (M2M PTAV) ← clave de unicidad
   combination_indices (Char stored, índice) ← hash de PTAVs ordenados
```

### Por qué dos capas (PAV vs PTAV)

- `product.attribute.value` (PAV) es **global**: "Rojo", "Azul" — comparten todos los templates que usen el atributo "Color".
- `product.template.attribute.value` (PTAV) es **por template**: el "Rojo" para "Camiseta" tiene `price_extra` propio y participa en exclusiones específicas de "Camiseta".
- PTAV es la **clave funcional** de las variantes — `product.product.product_template_attribute_value_ids` apunta a PTAVs, no a PAVs.
- Una PTAV "lleva" un PAV (`product_attribute_value_id`) más el contexto del template.

## product.attribute

| Campo | Tipo | Notas |
|---|---|---|
| `name` | Char, translate | "Talla" |
| `sequence` | Integer, default=20 | Orden de mostrado |
| `create_variant` | Selection | **`always` / `dynamic` / `no_variant`** — clave de la lógica |
| `display_type` | Selection | `radio` / `pills` / `select` / `color` / `multi` (multi-checkbox) |
| `value_ids` | O2M PAV | Valores disponibles |
| `attribute_line_ids` | O2M PTAL | Templates donde se usa |
| `product_tmpl_ids` | M2M stored | Computed, templates con el atributo |
| `number_related_products` | Integer computed | Para validaciones (no se puede borrar si ≥1) |

### create_variant — los 3 modos

| Modo | Comportamiento | Genera `product.product`? | Uso típico |
|---|---|---|---|
| `always` | **Default**. Cada combinación crea una variante al instante. | Sí, todas las combinaciones | Stock, BOM, precios distintos |
| `dynamic` | Variantes se crean **bajo demanda** cuando se añaden a un SO/PO/quotation. | Sí, solo las que se piden | Catálogos enormes (configuradores) |
| `no_variant` | El atributo **no genera variantes**. Aparece en el configurador pero NO crea `product.product`. | **No** | Personalizaciones (grabados, mensajes), filtros |

**Críticas**:

- Una vez el atributo está usado en un template, `create_variant` **NO se puede cambiar** (constraint `write()` line 117 de `product_attribute.py`). Para cambiar el modo, hay que **borrar todas las PTAL** que lo referencian.
- `display_type='multi'` (multi-checkbox) **fuerza** `create_variant='no_variant'` (SQL constraint `check_multi_checkbox_no_variant`).

### display_type

- `radio` (default) / `pills` / `select` / `color` (con `html_color` en PAV) / `multi` (multi-checkbox).
- En website/eCommerce, los `display_type` controlan el widget renderizado (swatches color, pills, etc.).
- `multi` solo tiene sentido con `no_variant` — permite múltiples values seleccionados a la vez (ej. "Extras" en un menú).

## product.attribute.value (PAV)

| Campo | Tipo | Notas |
|---|---|---|
| `name` | Char, translate | "Rojo" |
| `sequence` | Integer, index | Orden |
| `attribute_id` | M2o attribute, required, ondelete=cascade | |
| `default_extra_price` | Float | Precio extra **default** copiado al crear nuevos PTAV. Cambiarlo NO actualiza PTAV existentes. |
| `is_custom` | Boolean | Permite al cliente introducir texto libre (ej. "Talla a medida") |
| `html_color` | Char | Para `display_type='color'`. Hex (#ff0000) |
| `color` | Integer | Color UI (kanban) |
| `image` | Image | Imagen "swatch" (alternativa a html_color) |
| `active` | Boolean | Archivable |
| `pav_attribute_line_ids` | M2M PTAL | Inverso, dónde se usa |
| `is_used_on_products` | Boolean computed | Bloquea borrar si True |
| `default_extra_price_changed` | Boolean computed | UI: indica si el price difiere de los PTAV |

**Constraints**:

- No se puede cambiar `attribute_id` si está usado en productos.
- No se puede borrar si está usado; se archiva en su lugar (`unlink()` → archive si hay variantes inactivas).
- Eliminar con cascade desde attribute borra los PAVs.

## product.template.attribute.line (PTAL)

| Campo | Tipo | Notas |
|---|---|---|
| `active` | Boolean | Archivable. Al archivar, **clear de value_ids** |
| `product_tmpl_id` | M2o template, ondelete=cascade, index | No se puede mover entre templates (constraint en `write`) |
| `attribute_id` | M2o attribute, ondelete=restrict, index | No se puede cambiar (constraint en `write`) |
| `value_ids` | M2M PAV | Domain: `attribute_id` debe coincidir |
| `value_count` | Integer computed/store | Para optimizar `_get_combination_name` |
| `product_template_value_ids` | O2M PTAV | Inverso |
| `sequence` | Integer, default=10 | Orden visible (1ª = primera columna del configurador) |

**Comportamientos clave**:

- `create()` **reactiva PTALs archivadas** con misma combinación template+attribute en vez de crear nuevas. Esto preserva las variantes históricas y sus referencias en SO/PO antiguas.
- `write({'active': False})` hace `value_ids = [Command.clear()]` automáticamente.
- Cualquier cambio en `value_ids`/`active`/`attribute_id` dispara `_update_product_template_attribute_values()` (crea/archiva PTAVs) y finalmente `template._create_variant_ids()`.
- `_is_configurable()`: True si `len(value_ids) >= 2`, o display_type='multi', o algún value `is_custom`. Una PTAL con un solo value **NO se muestra en el configurador** pero **se aplica silenciosamente a todas las variantes** (sirve para filtros, ej. "Material: Algodón" forzado).

## product.template.attribute.value (PTAV)

Es el **value materializado por template**.

| Campo | Tipo | Notas |
|---|---|---|
| `ptav_active` | Boolean, default=True | NO se llama `active` — porque las vistas filtran por `active_test` y queremos ver archived |
| `name` | Char, related a PAV.name | |
| `product_attribute_value_id` | M2o PAV, required, ondelete=cascade, index | No se puede cambiar después |
| `attribute_line_id` | M2o PTAL, required, ondelete=cascade, index | No se puede mover a otra PTAL |
| `price_extra` | Float | **Extra price específico de este template** (copiado de `pav.default_extra_price` al crear) |
| `currency_id` | M2o related | Moneda del template |
| `exclude_for` | O2M exclusion | Las exclusiones se almacenan aquí |
| `product_tmpl_id` | M2o related, stored, index | Para queries directas |
| `attribute_id` | M2o related, stored, index | |
| `ptav_product_variant_ids` | M2M product.product, **readonly** | Relation: `product_variant_combination` |
| `html_color`, `is_custom`, `display_type`, `image` | related a PAV | |
| `color` | Integer, default random 1-11 | UI |

**Constraints**:

- Unicidad: `(attribute_line_id, product_attribute_value_id)` única (SQL constraint).
- Cambiar `product_attribute_value_id` o `product_tmpl_id` está **prohibido**.
- Modificar `ptav_product_variant_ids` directamente lanza UserError — la única forma de actualizar es escribiendo en `product.product.product_template_attribute_value_ids` (que dispara `_compute_combination_indices`).

**Display name**:

```python
ptav.display_name  # "Talla: S"  (siempre prefija con atributo, salvo en config form)
ptav._get_combination_name()  # "S, Rojo"  (excluye single-value lines y no_variant)
```

**Borrado**:

- Si la PTAL solo tiene este value → se quita de las variantes y luego se intenta borrar.
- Si la PTAV está usada en SO/PO/MO antiguas → `unlink()` falla → se archiva (`ptav_active=False`).

## product.template.attribute.exclusion

Modelo de exclusiones entre values.

| Campo | Tipo | Notas |
|---|---|---|
| `product_template_attribute_value_id` | M2o PTAV, ondelete=cascade | "Cuando elijo esta PTAV..." |
| `product_tmpl_id` | M2o template, ondelete=cascade, required | Redundante (el PTAV ya tiene template) pero permite búsquedas directas |
| `value_ids` | M2M PTAV (relation `product_attr_exclusion_value_ids_rel`) | "...estas otras PTAVs están excluidas" |

**Semántica direccional**:

- `A.exclude_for.value_ids = [B]` significa "si elijo A, B no puede aparecer".
- Es **direccional pero simétrica en la práctica**: Odoo internamente verifica ambas direcciones al validar una combinación. Sin embargo, almacenas la regla solo en una dirección.

**Side-effects en create/write/unlink**:

- Cualquier cambio dispara `template._create_variant_ids()` → regenera variantes para descartar combinaciones imposibles.
- Si la exclusión deja al template sin **ninguna** combinación posible, `_create_variant_ids()` lanza `UserError("This configuration would lead to no possible variant")`.

**Limitaciones**:

- Las exclusiones operan **dentro de un mismo template**.
- Para excluir entre productos distintos (ej. accessory products en eCommerce), hay un mecanismo separado (`accessory_product_ids`/`optional_product_ids` en sale/website).

## Creación y actualización de variantes — `_create_variant_ids()`

El método más importante. Llamado automáticamente cuando cambia:

- `attribute_line_ids` o `value_ids` de una PTAL.
- `exclude_for` de una PTAV.
- `ptav_active`.
- Manualmente vía botón "Configure Variants" o tras operaciones masivas.

### Algoritmo simplificado

```python
def _create_variant_ids(self):
    for template in self:
        lines_without_no_variants = template.valid_PTAL._without_no_variant_attributes()

        # 1. Single-value lines se aplican a TODAS las variantes existentes
        single_value_lines = lines_without_no_variants.filtered(
            lambda ptal: len(ptal.PTAV._only_active()) == 1
        )
        for variant in template.product_variant_ids:
            # añade PTAVs single-value a la combinación

        existing = {variant.PTAVs: variant for variant in all_variants}

        if not template.has_dynamic_attributes():
            all_combinations = itertools.product(*[ptal.PTAV._only_active() for ptal in ...])
            for combo in template._filter_combinations_impossible_by_config(all_combinations):
                if combo in existing:
                    variants_to_activate += existing[combo]
                else:
                    variants_to_create.append({'PTAVs': combo})
                    # límite: ir.config_parameter 'product.dynamic_variant_limit' (default 1000)

        variants_to_unlink = all_variants - variants_to_activate
        variants_to_unlink._unlink_or_archive()
```

### Reglas críticas

1. **Always**: genera **todas** las combinaciones posibles inmediatamente. Límite hard de 1000 variantes (config `product.dynamic_variant_limit`).
2. **Dynamic**: si **al menos un atributo del template** es `dynamic`, NO se generan combinaciones nuevas — solo se gestionan las existentes (las que han sido creadas vía `_get_variant_for_combination()` previa).
3. **No_variant**: completamente ignorado por `_create_variant_ids` (`_without_no_variant_attributes()` filtra fuera).
4. **Single-value PTAL**: NO genera nuevas combinaciones, pero **se inyecta en todas las variantes existentes**. Es la forma de añadir un atributo a un producto ya en producción sin proliferar variantes.
5. **Exclusiones**: combinaciones excluidas NO se crean. Si todas se excluyen → error.

### Activación de variantes archivadas

- Si una combinación nueva ya existía como variante **archived**, se **reactiva** (no se crea una nueva).
- Esto preserva FK history en SO/PO/MO.

## Modo dinámico — flujo completo

```python
# 1) Atributo dinámico
talla = self.env['product.attribute'].create({
    'name': "Talla", 'create_variant': 'dynamic',
    'value_ids': [(0, 0, {'name': s}) for s in ['S', 'M', 'L', 'XL']],
})

# 2) Template con la PTAL — NO se crean product.product de las 4 tallas
tmpl = ...create...
# tmpl.product_variant_count == 0 (en este modo es legítimo)
# tmpl.product_variant_ids es vacío

# 3) En SO se selecciona "Talla: M"
ptav_m = ...   # la PTAV de Talla:M
combination = ptav_m  # + cualquier otra PTAV elegida

# 4) Buscar / crear bajo demanda:
variant = tmpl._get_variant_for_combination(combination)
if not variant:
    variant = tmpl._create_product_variant(combination)
# A partir de ahora variant existe y está en product_variant_ids
```

**Helpers**:

- `tmpl._get_variant_for_combination(ptav_recordset)` → devuelve product.product o empty
- `tmpl._create_product_variant(combination, log_warning=True)` → crea si la combinación es válida
- `tmpl._get_possible_combinations(parent_combination=None)` → iterador de combinaciones válidas (excluye no posibles)
- `tmpl._is_combination_possible(combination, parent_combination=None, ignore_no_variant=False)` → bool

## price_extra — extra price por variante

Tres niveles:

1. **PAV.default_extra_price** — valor "plantilla" para nuevos templates.
2. **PTAV.price_extra** — valor real aplicado a las variantes de **este** template (se copia de `default_extra_price` al crear la PTAV pero **NO se sincroniza** después).
3. **product.product.price_extra** — computed = suma de PTAV.price_extra de la combinación de la variante.

```python
@api.depends('product_template_attribute_value_ids.price_extra')
def _compute_product_price_extra(self):
    for product in self:
        product.price_extra = sum(product.product_template_attribute_value_ids.mapped('price_extra'))
```

### lst_price = list_price + price_extra (en moneda template, UoM template)

`product.product.lst_price` es computed y **uom-context-dependent**:

```python
@api.depends_context('uom')
def _compute_product_lst_price(self):
    to_uom = self.env['uom.uom'].browse(self.env.context.get('uom'))
    for product in self:
        if to_uom:
            list_price = product.uom_id._compute_price(product.list_price, to_uom)
        else:
            list_price = product.list_price
        product.lst_price = list_price + product._compute_price_extra_in_uom(...)
```

**NO incluye pricelist**. Para precio resuelto con pricelist usar `_get_product_price()`.

### Wizard de actualización masiva de price_extra

`update.product.attribute.value` (wizard, action_update_prices) — recorre todas las PTAV asociadas a un PAV y sobrescribe `price_extra` con `default_extra_price`.

## is_custom — texto libre del cliente

- Flag en PAV (`is_custom=True`).
- Cuando una PTAV con `is_custom=True` se selecciona en SO, el cliente puede escribir texto en `product.attribute.custom.value` (modelo separado, asociado a la SO line).
- Útil para grabados, mensajes, "Otro: [escribir]".
- Funciona con cualquier `create_variant` mode pero típicamente con `no_variant` o `dynamic`.

## Validez de combinaciones — _is_combination_possible

```python
template._is_combination_possible(combination, parent_combination=None, ignore_no_variant=False):
```

Verifica:

1. Las PTAVs cubren todas las PTAL `valid_product_template_attribute_line_ids` del template (excluyendo no_variant si `ignore_no_variant=True`).
2. Cada PTAV pertenece al template.
3. Cada PTAV está **activa** (`ptav_active=True`).
4. Ninguna PTAV está en la lista de exclusiones de otra PTAV de la combinación.
5. Si hay `parent_combination` (productos opcionales/accessory), verifica exclusiones cruzadas.

`_filter_combinations_impossible_by_config()` aplica esto a iteradores.

## Single-value PTAL — caso especial muy común

Si una PTAL tiene **un solo value** seleccionado:

- No aparece en el configurador (`_is_configurable() == False`).
- SE INYECTA en todas las variantes existentes (`_create_variant_ids` lo añade silenciosamente).
- Útil para añadir un campo "Material: Algodón" a un producto ya creado sin proliferar variantes.
- `_get_combination_name()` lo OMITE del nombre mostrado de la variante.
- En reporting se considera "atributo del producto" pero no "atributo de variante".

## Cambios y archivado

### Cambiar atributos de un producto en producción

Patrón recomendado:

1. **Archivar PTAL antigua** en vez de borrar → preserva las variantes existentes con `ptav_active=False`.
2. **Crear PTAL nueva** con la nueva configuración → genera variantes nuevas.
3. Las SO/PO antiguas siguen referenciando las variantes archivadas (no rompen).

### Archivar un PAV/PTAV global

- `pav.active=False` o `ptav.ptav_active=False` → las variantes que lo usan también se archivan o no se crean.
- En SO antigua, la línea sigue referenciando la variante archived (display funciona).
- En el configurador (nuevo SO), el value desaparece.

### Atributo con create_variant cambiando

`UserError` si el atributo está usado en ≥1 template. Para cambiar:

1. Borrar todas las PTAL que lo usan (archivar no es suficiente, hay que `unlink`).
2. Cambiar `create_variant`.
3. Re-crear las PTAL.
4. Las variantes antiguas se archivan; **no se reactivan automáticamente**.

## Constraints clave (resumen)

| Constraint | Modelo | Mensaje |
|---|---|---|
| `attribute_value_unique` | PTAV (SQL) | "Each value should be defined only once per attribute per product." |
| `check_multi_checkbox_no_variant` | attribute (SQL) | display_type='multi' obliga create_variant='no_variant' |
| `_check_valid_values` | PTAL (Python) | PTAL activa debe tener ≥1 value; values deben matchear `attribute_id` |
| `_check_valid_values` | PTAV (Python) | PTAV activa: PAV debe estar en `attribute_line_id.value_ids` |
| `_unlink_except_used_on_product` | attribute, PAV | No borrar si usado en templates activos |
| `unlink` PTAL | PTAL | Fallback a archive si hay FK |
| `unlink` PTAV | PTAV | Fallback a archive (ptav_active=False) si hay FK |
| `unlink` PAV | PAV | Fallback a archive si hay variantes archived que lo usan |

## Cross-references

- **Cómo afecta a stock (BOM, kits, rutas)**: ver `mrp/integrations.md` y `stock/advanced.md` (kits).
- **Configurador en sale**: ver `sale/core.md` y `sale/v19_changes.md`.
- **Variantes en eCommerce (swatches, filtros, attribute search)**: ver `website/ecommerce_deep.md`.
- **Combos (nuevo type v18)**: ver `core.md` y `pos/v19_changes.md`.
- **POS sin variantes (limitaciones)**: ver `pos/gotchas.md`.
- **Gotchas numerados**: ver `gotchas.md`.
- **Cambios v19**: ver `v19_changes.md`.
