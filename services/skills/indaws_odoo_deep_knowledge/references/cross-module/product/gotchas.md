<!-- verified: v18.0 @ 2026-06 -->
# Product / Variants / UoM — Gotchas Odoo 18.0

Comportamientos ocultos y errores comunes alrededor de `product.template`, `product.product`, variantes y unidades de medida. Numerados para referencia cruzada:

- **PRG** = product.template / product.product core
- **VAG** = variants (atributos, PTAL, PTAV, exclusiones)
- **UOG** = UoM y conversión
- **CTG** = product.category
- **PLSG** = product.pricelist + product.pricelist.item
- **SIG** = product.supplierinfo
- **SRVG** = servicios (type='service')

---

## product.template / product.product (PRG)

**PRG1. Siempre existe al menos una `product.product` incluso sin atributos** — Cuando creas un template sin `attribute_line_ids`, `_create_variant_ids()` crea **automáticamente** una variante "única". `template.product_variant_count == 1`. Stock/SO/PO referencian a ESA variante. Si la borras manualmente, próxima escritura en el template intenta recrearla. Nunca asumas "el template ES el producto vendible" — siempre es la variante.

**PRG2. `_inherits` (no `_inherit`) — campos del template escriben en el template** — `product.product._inherits = {'product.template': 'product_tmpl_id'}`. Hacer `variant.name = "X"` cambia el name del **template** (afecta a todas las hermanas). Para campos overridden en `product.product` (`barcode`, `default_code`, `standard_price`, `weight`, `volume`, `image_variant_*`) sí escribe solo en la variante. Confunde a quien viene de `_inherit`.

**PRG3. `standard_price` es `company_dependent` en variant pero computed en template** — En `product.product` se guarda **por empresa**: en v17 como registros `ir.property`; en **v18+ como columna `jsonb`** en la tabla del modelo con `company_id` de clave (`ir.property` eliminado en v18, y `property_value` no existe — era un error de esta ficha). En `product.template` es **computed** desde la primera variante (`_compute_template_field_from_variant_field`). Implicación: en multi-company, cada empresa ve un coste diferente para el mismo producto, y la "vista del template" muestra el de la empresa actual. Cambiar coste en una empresa NO afecta a las demás.

**PRG4. `weight` / `volume` del template solo funcionan si hay 1 sola variante** — `_compute_template_field_from_variant_field('weight', default=False)`: si hay >1 variantes, el campo del template muestra `False`/0.0. El "valor real" vive en cada `product.product`. Imprimir el weight del template en informes con multi-variant da resultado vacío. Iterar `product_variant_ids` y mostrar pesos por variante.

**PRG5. `barcode` y `default_code` son **propios** de la variante** — A diferencia de `weight`, estos NO son computed: son campos propios en `product.product` (que **shadow** al campo computed del template). Cada variante tiene su barcode propio. El template muestra el de la primera (compute).

**PRG6. Barcode único POR EMPRESA (también vs packaging)** — Constraint `_check_barcode_uniqueness` (`product_product.py`) valida que el barcode no exista en otro `product.product` ni en `product.packaging`, **agrupando por company** (`_get_barcodes_by_company` + `_check_duplicated_product_barcodes` / `_check_duplicated_packaging_barcodes`). El docstring lo dice explícito: "Barcodes should only be unique within a company". NO es global como suele asumirse: el mismo barcode puede repetirse en empresas distintas. (En v19 la colisión es contra `product.uom` en vez de `product.packaging`, que desaparece.) Importar productos con barcodes duplicados dentro de la misma empresa falla por uno y deja el resto sin importar. Pre-deduplicar antes. (Nota: la búsqueda usa `read_group` —deprecado— no `_read_group`.)

**PRG7. `product.template.list_price` NO aplica pricelist** — `list_price` es el precio de catálogo "puro". Para precio final, el entry point canónico es **`pricelist._get_product_price(product, quantity, uom=, currency=, date=)`** — se llama sobre el registro `product.pricelist` pasándole el product (`product/models/product_pricelist.py:104`, `Pricelist._get_product_price`), NO `product._get_product_price(...)`. A nivel producto, el helper con contexto es `product._get_contextual_price()` (`product/models/product_product.py:876`, `ProductProduct._get_contextual_price`, lee `pricelist`/`quantity`/`uom` del contexto). Mostrar `list_price` en informes/web sin pricelist da el precio sin descuentos/markups. Bug frecuente: vista QWeb muestra `product.list_price` y luego se pregunta por qué no aplica la promo.

**PRG8. `lst_price` en variant SÍ incluye `price_extra` pero NO pricelist** — `product.product.lst_price = list_price + sum(PTAV.price_extra)`, además es **uom-context-dependent** (`@api.depends_context('uom')`). Es lo que se muestra en kanban de productos. Sigue sin aplicar pricelist.

**PRG9. `categ_id` arrastra cuentas contables via property fields** — `property_account_income_id`, `property_account_expense_id`, `property_stock_*` se resuelven con fallback **producto → categoría → empresa**. Mover un producto a otra categoría puede cambiar las cuentas usadas en facturación si el producto NO tiene override propio. Auditar antes con `default_get` o `fields.get('property_account_income_id')`.

**PRG10. Eliminar template raramente funciona** — `_unlink` falla por FK desde stock.move, account.move.line, sale.order.line, mrp.bom, etc. Odoo lo gestiona con `_unlink_or_archive()` que cae a archive en silencio. Resultado: el template "desaparece" del menú pero sigue en BD con `active=False`. Para "limpiar" hay que borrar primero todos los registros que apuntan a él.

**PRG11. `product.combo` (nuevo tipo v18) descompone en líneas hijas** — Al vender un producto `type='combo'`, en SO se crea 1 línea principal (la del propio producto combo) + N líneas hijas. Cada hija tiene `combo_item_id` set y `linked_line_id` apuntando a la principal; la principal tiene `product_type == 'combo'` y `linked_line_ids` con las hijas (`sale/models/sale_order_line.py:137-156`, `SaleOrderLine.linked_line_id`). NO existe un campo `is_combo_parent`. Reportes/queries sumando precios de SO line **doble-cuentan** si no excluyes uno de los dos niveles. Filtrar las hijas con `combo_item_id != False`, o la principal con `product_type == 'combo'`.

**PRG12. `image_1920` del template es un Image propio (image.mixin); es la VARIANTE la que hace fallback al template** — CORREGIDO 2026-07 (el claim previo estaba invertido): `product.template` hereda `image.mixin` (`product/models/product_template.py:20`, `class ProductTemplate`) → `image_1920` es un campo **Image almacenado propio** (`odoo/addons/base/models/image_mixin.py:13`, `class ImageMixin`), NO computed desde `product_variant_ids[0]`. `image_variant_1920` solo existe en `product.product`. El fallback va al revés de lo que suele creerse: `product.product.image_1920` devuelve la imagen de la variante y, si no la tiene, cae a la del template (`product/models/product_product.py:157-160`, `ProductProduct._compute_image_1920`). Para la foto propia de la variante usa `variant.image_variant_1920` (False si no tiene); para la del template, `template.image_1920`.

**PRG13. `is_storable` (v18+) no es `type='storable'` (v17)** — En v18 se eliminó el type `'storable'` del Selection. Ahora `type` es solo `consu`/`service`/`combo`. La distinción "es de inventario" se hace con `is_storable` (Boolean, en stock module). Buscar `product.template type='storable'` falla silenciosamente en v18 (devuelve recordset vacío). Usar `('is_storable', '=', True)`.

**PRG14. `product_variant_id` (singular) es solo la PRIMERA variante** — Computed `_compute_product_variant_id` devuelve `product_variant_ids[:1]`. Útil para prefetching/optimización. NO es "la variante activa" ni "la variante única". En templates con N variantes apunta arbitrariamente a la primera por sort order (`is_favorite desc, default_code, name, id`).

**PRG15. Multi-company: `company_id=False` significa "global", no "main company"** — Si dejas company vacía, el producto es compartido. Asignarlo a una empresa restringe acceso pero también desencadena `_check_company_auto`. Cambiar `company_id` de un producto YA usado en otra empresa lanza error de check_company al primer access.

---

## Variantes (VAG)

**VAG1. `create_variant` NO se puede cambiar tras usar el atributo** — Constraint en `product.attribute.write`: si `number_related_products > 0`, lanza `UserError`. Para cambiar `always`/`dynamic`/`no_variant` hay que borrar (no archivar — borrar) todas las PTAL que referencian al atributo, cambiar el modo, y volver a crear las PTAL. Las variantes históricas se archivan.

**VAG2. Atributo con un solo value se inyecta silenciosamente** — Una PTAL con `value_count=1` NO aparece en el configurador (`_is_configurable() == False`) pero **se aplica a todas las variantes existentes** vía `_create_variant_ids`. Se usa para añadir "Material: Algodón" a productos en producción sin proliferar variantes. Confusión: el atributo "existe pero no se ve".

**VAG3. `display_type='multi'` fuerza `create_variant='no_variant'`** — SQL constraint `check_multi_checkbox_no_variant`. Multi-checkbox permite seleccionar **múltiples values a la vez** — incompatible con variantes (que son combinaciones cerradas). Solo tiene sentido para "extras" sin impacto en stock (toppings, opciones de servicio).

**VAG4. `dynamic` no crea variantes hasta el SO** — Con `create_variant='dynamic'`, `template.product_variant_count == 0` es legítimo y NO bug. Las variantes se crean cuando el cliente las pide vía `_get_variant_for_combination` + `_create_product_variant`. Reportes que asumen "todo producto tiene ≥1 variante" rompen.

**VAG5. Limite hard de 1000 variantes por template (always)** — `ir.config_parameter` `product.dynamic_variant_limit`, default=1000. Combinación que exceda → `UserError("The number of variants to generate is above allowed limit")`. Para catálogos grandes: cambiar a `dynamic` o subir el límite (con cuidado, performance degrada por encima de 10000).

**VAG6. `exclude_for` es direccional pero Odoo valida ambas direcciones** — `A.exclude_for.value_ids = [B]` significa "elegir A excluye B". Internamente `_is_combination_possible` verifica que A no exluya B Y que B no excluya A — simétrico funcionalmente. Pero al **guardar** la regla solo en A puedes confundir a alguien leyendo el form de B (no ve la exclusión). Recomendación: definir en ambos extremos para claridad.

**VAG7. Exclusiones pueden dejar el template sin variantes válidas** — Si tras añadir exclusiones todas las combinaciones quedan imposibles, `_create_variant_ids()` lanza `UserError("This configuration would lead to no possible variant")` y revierte. NO archiva el template — bloquea la operación.

**VAG8. `ptav_active` (no `active`) — los views muestran archived por defecto** — `product.template.attribute.value` usa `ptav_active` en vez de `active` porque las vistas de configuración (line en template form) tienen que mostrar archived values (para poder reactivarlos). `active_test` se desactiva sistemáticamente. Si filtras por `[('active', '=', True)]` en una query → vacío (campo no existe). Usar `('ptav_active', '=', True)`.

**VAG9. `default_extra_price` (PAV) NO sincroniza con `price_extra` (PTAV)** — `default_extra_price` se copia a `price_extra` SOLO al **crear** una PTAV. Cambiar `pav.default_extra_price` después NO afecta a las PTAV existentes. Para propagar: wizard `update.product.attribute.value` con `mode='update_extra_price'` (botón "Update product extra prices" en la UI de PAV).

**VAG10. `ptav_product_variant_ids` es readonly — escribir lanza UserError** — La relación M2M `product_variant_combination` solo se puede modificar desde `product.product.product_template_attribute_value_ids`. Esto fuerza que `_compute_combination_indices` se dispare. Lanza `UserError("You cannot update related variants from the values...")`.

**VAG11. PTAV `display_name` SIEMPRE prefija "Atributo: Value" (sin escape)** — `_compute_display_name` override en `product.template.attribute.value`: `f"{value.attribute_id.name}: {value.name}"` → `"Talla: S"`, no solo `"S"` (`product/models/product_template_attribute_value.py:152-158`, `ProductTemplateAttributeValue._compute_display_name`). Razón: en el form de exclusiones y en reporting el name del value solo es ambiguo. **OJO**: NO existe escape vía contexto en PTAV — el truco `with_context(show_attribute=False)` aplica a `product.attribute.value` (PAV), cuyo `_compute_display_name` SÍ respeta `@api.depends_context('show_attribute')` (`product/models/product_attribute_value.py:60-61`, `ProductAttributeValue._compute_display_name`; el guard, en `product/models/product_attribute_value.py:70`, `ProductAttributeValue._compute_display_name`). En PTAV el prefijo es incondicional.

**VAG12. Cambiar `attribute_id` o `product_tmpl_id` de una PTAL/PTAV está prohibido** — `write` override lanza `UserError`. Mover una línea entre templates → crear nueva PTAL + borrar/archivar la antigua. Esto se hace para preservar la integridad referencial de las variantes ya creadas.

**VAG13. PTAL archivada se REACTIVA en `create` con misma combinación** — `product.template.attribute.line.create()` busca PTALs con `active=False`, mismo `product_tmpl_id` y `attribute_id` (`limit=1`), y las reactiva (write `vals`) en vez de crear nuevas (`product/models/product_template_attribute_line.py:98-114`, `ProductTemplateAttributeLine.create`). Matiz importante: al **archivar** una PTAL, su `write` hace `value_ids=[Command.clear()]` (`:148-150`), así que la PTAL archivada queda **sin values**; la "config antigua" ya no está guardada en la línea. Al reactivarla con `value_ids=[L, XL]`, la línea queda con esos values (no "pisa" nada porque la antigua ya estaba limpia). El efecto práctico sigue siendo el mismo a tener en cuenta: una `create` aparentemente nueva reutiliza el ID de la PTAL archivada en vez de crear una.

**VAG14. `product_template_variant_value_ids` excluye single-value lines** — En `product.product` hay DOS M2M a PTAV:
  - `product_template_attribute_value_ids`: **todas** las PTAVs de la combinación (incluye single-value lines).
  - `product_template_variant_value_ids`: subset con `attribute_line_id.value_count > 1` (excluye single-value).
  El segundo es el que se muestra en kanban/info de variante. Confusión: una variante con 3 atributos pero 2 single-value muestra solo 1 en kanban.

**VAG15. Variantes archivadas siguen vinculadas a SO/PO/stock antiguos** — Cuando una combinación de atributos cambia (exclusión, eliminación de PAV), las variantes "imposibles" se archivan (`active=False`), no se borran. Las SO/PO/stock.move antiguas siguen apuntando a ellas y funcionan (display correcto). En el configurador nuevo no aparecen. **No `unlink`** las variantes archivadas manualmente.

**VAG16. `combination_indices` puede dar falsos negativos en queries directas** — Es un string `'12,34,56'` (IDs ordenados). Buscar por combinación con `('combination_indices', '=', '34,12,56')` falla por orden. Usar siempre `_get_variant_for_combination(ptav_recordset)` que ordena internamente.

**VAG17. Atributos en BoM/MRP — variantes específicas** — `mrp.bom.product_id` (M2o product.product) es opcional. Si vacío, la BoM aplica a TODAS las variantes del template. Si está set, solo a esa variante. Mismo template con BoMs distintas por variante es soportado. `mrp.bom.byproduct_ids` apunta a variantes específicas. **Confusión típica**: definir BoM en template (sin variant) y esperar que cada variante consuma componentes distintos — no funciona, hace falta una BoM por variante o usar `bom_line.bom_product_template_attribute_value_ids` para condicionar líneas por PTAV.

**VAG18. eCommerce: PAV `image` aparece como swatch solo si display_type compatible** — Para `display_type='color'`, el swatch usa `html_color` (si set) o `image`. Para `pills`/`radio`/`select` no muestra `image`. En website module hay templates específicos por display_type. Si subes una imagen para un value y no aparece en la web, revisar el display_type del atributo.

**VAG19. Variantes en POS — limitación severa** — POS NO soporta el configurador de variantes nativo. Si un template tiene N variantes, cada variante aparece como "producto individual" en la pantalla de POS (no hay popup de "elige talla/color"). Para configurador en POS: módulos OCA `pos_product_variants` o desarrollo custom. POS sí soporta multi-variant a nivel búsqueda (cada variante con barcode/code propio).

**VAG20. is_custom value crea texto libre — pero NO crea variante nueva** — Cuando el cliente introduce texto en un value `is_custom=True`, se guarda en `product.attribute.custom.value` asociado a la SO line. La variante usada es la "base" (sin el texto). Reportar tipo "Customizaciones del mes" requiere unir SO line + custom.value (no aparece en `product.product`).

---

## UoM (UOG)

**UOG1. Solo conversión dentro de la misma categoría** — `_compute_quantity` con `raise_if_failure=True` (default) lanza `UserError` si `self.category_id != to_unit.category_id`. Con `raise_if_failure=False` **devuelve qty SIN convertir** (silencioso, peor). `_compute_price` con categorías distintas **también devuelve sin convertir** y NO lanza error. Confunde: precio "no se convirtió" pero parece OK.

**UOG2. `factor` y `factor_inv` parecen redundantes pero NO son lo mismo** — `factor`: cuántas veces this UoM cabe en 1 reference (`1 ref = factor × this`). `factor_inv` = `1 / factor`: cuántas reference cabe en 1 this. Para `Dozen` (1 docena = 12 unidades): `factor=1/12`, `factor_inv=12`. Editar uno en UI actualiza el otro automáticamente. **NO** confundir `factor` con "ratio" (campo dummy que cambia según `uom_type` para mostrar siempre un número >1 en UI).

**UOG3. Cambiar `factor` de UoM en uso NO actualiza datos históricos** — El warning `_onchange_critical_fields` lo dice explícitamente. Si tenías 100 stock moves con `qty=10 Boxes` y cambias `Box.factor` de 1/12 a 1/24, los moves siguen mostrando "10 Boxes" pero la conversión a stock UoM ahora dice 240 en vez de 120. Recálculo masivo manual requerido. **Nunca cambies factor de UoMs en producción**; crear UoM nueva.

**UOG4. `_compute_quantity` por defecto redondea con `rounding_method='UP'`** — Optimizado para stock (siempre reservar/comprar suficiente). En facturación quieres `'HALF-UP'` típicamente. Override explícito: `uom._compute_quantity(qty, to_unit, rounding_method='HALF-UP')`. Bug frecuente: factura sale por 100.01€ porque qty=33.333 docenas se redondeó UP a 33.34.

**UOG5. `_compute_price` NO redondea** — A diferencia de `_compute_quantity`, no aplica `float_round`. Devuelve el resultado completo. Esto sí está bien (precios suelen requerir precisión máxima), pero ojo al display (digits='Product Price' redondea visualmente).

**UOG6. Precision `digits=0` en `factor`/`rounding` = NUMERIC sin límite** — En Odoo `Float(digits=0)` se traduce a PostgreSQL `NUMERIC` sin precision. Evita pérdida de precisión en 1/3=0.3333... Pero las queries con `WHERE factor = 0.0833333` casi siempre fallan por epsilon. **Comparar factors con `float_compare`** o convertir a fraction.

**UOG7. `rounding` de la UoM vs `decimal.precision('Product Unit of Measure')` son COSAS DISTINTAS** — `rounding` (default 0.01) controla la granularidad del valor calculado al convertir. `decimal.precision` controla los **dígitos** del campo Float. Una UoM con `rounding=1.0` (piezas indivisibles) usada en un campo con precision=2 muestra "5.00" — no "5". Coherencia se mantiene si rounding=10^-precision, pero NO es obligatorio.

**UOG8. UoMs maestras del módulo `uom` no se pueden borrar** — `_unlink_except_master_data` → `_filter_protected_uoms` filtra `ir.model.data` con `module='uom'`, exceptuando los listados en `_unprotected_uom_xml_ids`. En **v18** ese unprotected list es solo `product_uom_hour` y `product_uom_dozen` (`uom/models/uom_uom.py:49-54`, `UoM._unprotected_uom_xml_ids`); en **v19** se añade `product_uom_pack_6` (`odoo19/addons/uom/models/uom_uom.py:24-32`, `UomUom._unprotected_uom_xml_ids`). Las demás solo archivables. Si necesitas "borrar" Units, archive. Side-effect: en multi-language, los nombres traducidos persisten incluso archived.

**UOG9. `product_uom_hour` se protege adicionalmente cuando hr_timesheet está instalado** — Módulo `hr_timesheet` override `_unprotected_uom_xml_ids` quita `product_uom_hour`. Si instalas timesheet y luego intentas archive Hour → UserError. Para módulos que dependen de UoMs específicas, esta es la patrón.

**UOG10. `uom_po_id` se resetea silenciosamente al cambiar `uom_id` a otra categoría** — `_compute_uom_po_id`: si `uom_id.category_id != uom_po_id.category_id` → resetea a `uom_id`. Cambiar `uom_id` de "Unit → Weight" deja `uom_po_id` en Weight automáticamente. Quien tenía configurado "compra en Box (Unit) y stock en Kg (Weight)" pierde la config sin warning.

**UOG11. `product_packaging` (v18) vs UoM — son ortogonales** — Packaging NO es una UoM. Es agrupación física (qty + barcode) expresada en stock UoM. En SO line conviven 3 campos: `product_uom_qty` (en `product_uom`), `product_packaging_id`, `product_packaging_qty`. Constraint `_check_product_uom_qty` verifica que cuadren. **Cambiar packaging en línea SO recalcula product_uom_qty** automáticamente. Detalle full en `sale/core.md`.

**UOG12. Conversión vendor → PO line → stock acumula errores** — `purchase.order.line._compute_price_unit`: `seller.product_uom._compute_price(seller.price, line.product_uom)`. Luego al confirmar, stock.move convierte de `line.product_uom` a `product.uom_id`. Doble conversión + rounding doble. Si vendor cotiza 0.0833 EUR/unidad y compras en docenas, esperas 1 EUR/docena exacto — sale 0.99999... Acumular `_compute_price` antes de redondear evita epsilon.

**UOG13. SO line: el domain `category_id` restringe el selector, NO valida al escribir** — `product_uom` field domain="[('category_id', '=', product_uom_category_id)]" filtra el dropdown. Pero si escribes un ID directamente vía ORM/import sin pasar por la UI → se acepta. Luego `_compute_quantity` falla al sincronizar con stock. Validar en código.

**UOG14. Multi-UoM en facturas: cambiar UoM SÍ recalcula price_unit (corregido v18)** — A diferencia de versiones antiguas, en v18 `account.move.line._compute_price_unit` lleva `@api.depends('product_id', 'product_uom_id')` y al cambiar la UoM re-resuelve el precio vía `product._get_tax_included_unit_price(..., product_uom=line.product_uom_id)` (`account/models/account_move_line.py:850-868`, `AccountMoveLine._compute_totals`). El campo es `store=True, readonly=False`: el recálculo NO es una conversión del `price_unit` existente, sino una **re-resolución del precio de catálogo/contextual** para la nueva UoM, y deja de dispararse en cuanto el usuario edita manualmente el `price_unit` (al ser readonly=False). Matiz a vigilar: si la factura se genera desde una SO, hereda `price_unit` ya fijado por la SO line; un cambio posterior de UoM en la factura re-resuelve por catálogo, no por el precio pactado en la SO. Auditar líneas con override manual de price_unit + UoM no estándar antes de validar.

**UOG15. SII / EDI español: la UoM de la línea va al XML** — Tax authority (AEAT) puede rechazar facturas con UoMs no estándar (claves IVA específicas requieren "kg", "m³", "litros"). Línea con `product_uom = "Pack of 6"` se manda como texto literal — AEAT no entiende. Para SII, mapear a UoM estándar antes (override `_get_sii_uom_code` en l10n_es modules).

**UOG16. `decimal.precision` se aplica a `digits='Nombre'`, NO a `digits=(p, s)`** — Solo los campos `Float(digits='Product Unit of Measure')` (string) leen la precision dinámica de `decimal.precision`. Los `Float(digits=(16, 2))` (tupla) están hardcoded. Cambiar la precision en Settings no afecta a estos. Encontrar campos hardcoded con `grep "digits=(" addons/`.

**UOG17. Reducir `decimal.precision` solo AVISA (no está prohibido)** — Contrario a lo que se suele creer, NO hay constraint que bloquee reducir dígitos. El base `decimal.precision._onchange_digits_warning` (`odoo/addons/base/models/decimal_precision.py:52-66`, `DecimalPrecision._onchange_digits_warning`) solo muestra un **warning** onchange ("The precision has been reduced... existing data WON'T be updated") si `digits < _origin.digits`; el `write` no lo impide. El módulo `product` añade `_onchange_digits` (otro warning, si la nueva precision es menos precisa que el rounding de alguna UoM) y un `_check_main_currency_rounding` que solo aplica a la precision **'Account'** vs la moneda de la empresa (`product/models/decimal_precision.py:11-40`, `DecimalPrecision._check_main_currency_rounding`). Conclusión: reducir es técnicamente posible (con riesgo de inconsistencias en datos NUMERIC ya almacenados), no lanza UserError. No existe ninguna constraint dedicada de validación de precisión que lo bloquee.

**UOG18. `uom.group_uom` desactivado oculta UoMs pero NO las elimina** — Si el grupo "Units of Measure" no está activo, los selectors en SO/PO/stock no aparecen y todo opera en stock UoM. Pero los datos ANTIGUOS con UoM custom siguen ahí. Activar/desactivar el grupo es seguro funcionalmente; **NO** purga datos.

**UOG19. POS no permite cambiar UoM en línea ni vender en packaging** — `pos.order.line.product_uom_id` es related al producto (readonly). Si necesitas "kg" en POS (balanza) hay que usar producto con `uom_id=kg` directamente; no hay conversión en POS UI. Productos con multi-UoM en backend se "aplanan" a stock UoM en POS.

**UOG20. UoM de servicios (hour/day) compite con working schedules en HR** — `uom.product_uom_hour` (cat. Working Time) se usa en `account.analytic.line.unit_amount` para timesheets, en `sale.order.line.product_uom` para servicios facturados por horas, y en `hr.attendance` indirectamente. Cambiar factor entre Hour y Day (default 1 day = 8h) afecta a TODO. Para empresas con jornada distinta a 8h: NO cambiar la UoM Day — crear UoM `Day 7.5h` propia.

**UOG21. Convertir 0 cantidad siempre devuelve 0 sin error** — `_compute_quantity(qty=0, to_unit=...)`: early return `if not qty: return qty`. Útil pero confunde: convertir entre categorías incompatibles con qty=0 NO lanza el UserError. Tests de validación de UoMs deben usar qty > 0.

**UOG22. `_check_qty` (helper de packaging) puede devolver qty distinto al input por floats** — Helper que redondea `product_qty` al múltiplo más cercano de `packaging_qty`. Usa `float_round` en vez de modulo porque `8 % 1.6 = 1.5999...` (epsilon). Llamarlo cambia qty silenciosamente. SO line `_compute_product_uom_qty` lo usa al cambiar packaging.

**UOG23. Cambiar `product_uom` en SO line re-resuelve el precio** — En `sale.order.line`, `_compute_price_unit` depende de `product_uom` (`@api.depends('product_id', 'product_uom', 'product_uom_qty')`, `sale/models/sale_order_line.py:561-562`, `SaleOrderLine._compute_price_unit`): cambiar la UoM re-resuelve la pricelist en la nueva UoM. En v18 `account.move.line` se comporta **igual** (ver UOG14 corregido: también depende de `product_uom_id` y re-resuelve), así que ya NO hay el contraste "SO sí / factura no" que se documentaba antes. Diferencia real: la factura re-resuelve por catálogo/contextual, no por el precio pactado de la SO. `qty_delivered` se mantiene en la `product_uom` de la línea SO, no en stock UoM (stock UoM solo vive en `stock.move`).

**UOG24. v19: la conversión deja de ser "misma categoría" y pasa a ser por árbol `relative_uom_id`** — En v19 `uom.uom` elimina `category_id` y usa un árbol (`_parent_name='relative_uom_id'`, `odoo19/addons/uom/models/uom_uom.py::_parent_name`). `factor` ya no se edita: es computed recursivo `relative_factor * relative_uom_id.factor` (`uom/models/uom_uom.py:43`, `class UomUom`, v19; el compute en `uom/models/uom_uom.py:69-75`, `UomUom._compute_factor`, v19). Dos UoMs son convertibles si comparten raíz del árbol. El docstring de `_compute_quantity` (`uom/models/uom_uom.py:159`, `UomUom._compute_quantity`, v19) todavía dice "different UomUom category" pero la implementación (`uom/models/uom_uom.py:169-171`, `UomUom._compute_quantity`, v19) solo multiplica/divide por `factor`. Código que comparaba `uom.category_id == other.category_id` para validar convertibilidad debe migrarse a comprobar la raíz del árbol.

**UOG25. v19: `product_uom` → `product_uom_id` en TODAS las líneas (rename BREAKING)** — El campo UoM se renombra en v19: `sale.order.line.product_uom` → `product_uom_id` (`odoo19/addons/sale/models/sale_order_line.py:132`, `SaleOrderLine.product_uom_id`), `purchase.order.line.product_uom` → `product_uom_id` (`odoo19/addons/purchase/models/purchase_order_line.py:36`, `PurchaseOrderLine.product_uom_id`), `product.supplierinfo.product_uom` → `product_uom_id` (`odoo19/addons/product/models/product_supplierinfo.py:25`, `ProductSupplierinfo.product_uom_id`). `account.move.line.product_uom_id` ya tenía ese nombre en v18. **Impacto**: TODO override Python, vista XML/QWeb, dominio, `@api.depends`, Studio o import CSV que referencie `product_uom` en sale/purchase/supplierinfo rompe en v19 (AttributeError / campo inexistente). Buscar `product_uom\b` (sin `_id`) en custom antes de migrar. La KB v18 (`uom.md`, `v19_changes.md`) todavía usa el nombre antiguo — no fiarse.

**UOG26. v19: `sale.order.line.product_packaging_id`/`product_packaging_qty` ELIMINADOS (no reconvertidos)** — CORRIGE lo que decía `v19_changes.md`: en v19 la sale.order.line **no tiene** `product_packaging_id` ni `product_packaging_qty` (verificado: no existen en `sale/models/sale_order_line.py`). No es que "apunten ahora a uom.uom" — desaparecen. El "packaging" se elige como una UoM más en `product_uom_id`, restringida a `allowed_uom_ids` (ver UOG33). Reportes/queries/vistas que leían `product_packaging_id`/`_qty` en SO deben reescribirse contra `product_uom_id`.

**UOG27. v19: `stock.move` gana campo NUEVO `packaging_uom_id` (NO es `product_packaging_id` reciclado)** — En v19 `stock.move.packaging_uom_id` (M2o `uom.uom`, "Packaging unit from sale or purchase orders", computed store, `stock/models/stock_move.py:194`, `StockMove.product_packaging_id`) es un campo **nuevo**; `product_uom` mantiene su nombre. La KB (`v19_changes.md` §2) afirmaba que `stock.move.product_packaging_id` "pasa a uom.uom" — es incorrecto: el nombre del campo es `packaging_uom_id`, no un rename de product_packaging_id. Adaptar código de albaranes al nombre real.

**UOG28. v19: `_compute_price` TAMBIÉN invierte la fórmula y NO valida árbol** — La KB documentaba solo la inversión de `_compute_quantity`. `_compute_price` en v19 es `amount = price * to_unit.factor / self.factor` (`uom/models/uom_uom.py:196-203`, `UomUom._compute_price`, v19) — invertida respecto a v18 (`price * self.factor / to_unit.factor`) porque `factor` cambió de semántica a "Absolute Quantity". Además **no tiene ningún guard de categoría/árbol**: en v18 devolvía `price` sin convertir si las categorías diferían; en v19 hace la multiplicación/división pase lo que pase → convertir precio entre UoMs de árboles distintos devuelve un número **plausible pero erróneo**, sin error. No hay red de seguridad; validar `_has_common_reference` tú mismo antes de convertir precios.

**UOG29. v19: `raise_if_failure` en `_compute_quantity` es un parámetro MUERTO** — La firma lo mantiene (`uom/models/uom_uom.py:153`, `UomUom._compute_quantity`, v19) y el docstring promete "raise an exception if the conversion is not possible (different UomUom category)", pero el cuerpo del método (`uom/models/uom_uom.py:162-176`, `UomUom._compute_quantity`, v19) **nunca lo lee ni lanza nada** y **no llama a `_has_common_reference`**. Convertir entre UoMs de árboles sin ancestro común NO da UserError (ni con `raise_if_failure=True` ni `False`) — devuelve `qty * self.factor / to_unit.factor` silenciosamente. `product/models/product_pricelist.py:217`, `ProductPricelist._compute_price_rule` (v19) lo llama con `raise_if_failure=False` pero daría lo mismo. Todo código v18 que confiaba en el UserError de conversión cruzada (UOG1) pierde esa protección en v19: falla en silencio con datos corruptos.

**UOG30. v19: `relative_uom_id` es `ondelete='cascade'` → borrar una UoM padre borra el subárbol** — `relative_uom_id = fields.Many2one('uom.uom', ondelete='cascade')` (`uom/models/uom_uom.py:41`, `class UomUom`, v19). En v18 `category_id` era `ondelete='restrict'` (no podías borrar la categoría con UoMs dentro). En v19 borrar una UoM raíz/intermedia **cascade-borra todas sus derivadas** (`related_uom_ids`). Riesgo de pérdida de datos masiva si se hace `unlink` sobre una UoM de referencia no protegida. La protección de master-data (`_filter_protected_uoms`) solo cubre las UoMs del módulo `uom`; UoMs custom raíz no están protegidas → un unlink limpia todo su árbol.

**UOG31. v19: constraints de UoM cambian por completo** — (1) Desaparece `_check_category_reference_uniqueness` (v18 exigía exactamente 1 reference por categoría). En v19 se permiten **múltiples árboles independientes** (varias UoMs con `relative_uom_id=False`). (2) Nueva `_check_factor` (`uom/models/uom_uom.py:97-101`, `UomUom._check_factor`, v19): una UoM raíz (sin `relative_uom_id`) DEBE tener `relative_factor == 1.0`, si no → `UserError("Reference unit of measure is missing")`. (3) Desaparece la SQL constraint `rounding > 0` de v18 porque `rounding` ahora es **computed, no almacenado** (`_compute_rounding`, global vía `decimal.precision('Product Unit')`). Solo queda `_factor_gt_zero` (`CHECK (relative_factor!=0)`, `uom/models/uom_uom.py:46-49`, `class UomUom`, v19). Migración: código que creaba UoMs con `factor`/`uom_type`/`category_id` explícitos rompe; hay que usar `relative_uom_id` + `relative_factor`.

**UOG32. v19: cambiar `product.uom_id` hace swap de NOMBRE 1:1, no conversión por factor** — Al cambiar `uom_id` de un producto, el onchange avisa literal: "will apply a conversion 1 old = 1 new... by replacing the unit name" (`product/models/product_product.py:426-428`, `ProductProduct._unlink_or_archive`). `product.template.write` llama a `_update_uom` con `skip_uom_conversion=True` (`product/models/product_template.py:586-588`, `ProductTemplate._search_display_name`). El override de stock (`stock/models/product.py:770-794`, `ProductTemplate.compute_is_storable`) **BLOQUEA con UserError** si el producto tiene algún `stock.move`/`stock.move.line` en una UoM distinta a la del template ("please archive the product and create a new one"); si todos los moves están en la UoM del template, **reescribe `product_uom`/`product_uom_id` de TODOS los moves a la nueva UoM sin tocar las cantidades** (10 "Units" pasan a ser 10 "kg"). `_trigger_uom_warning` (`product/models/product_product.py:419`, `ProductProduct._trigger_uom_warning`, v19; override en `stock/models/product.py:807`) dispara el aviso si el producto tiene cualquier move. Nunca cambiar uom_id de un producto con histórico esperando reconversión numérica: o lo bloquea, o reinterpreta cantidades bajo otro nombre.

**UOG33. v19: `allowed_uom_ids` limita el selector de UoM en líneas al producto, no al árbol** — Nuevo campo computed `allowed_uom_ids = product.uom_id | product.uom_ids` en sale.order.line (v19 `sale/models/sale_order_line.py:537-539`, `SaleOrderLine._compute_allowed_uom_ids`) y purchase.order.line (v19 `purchase/models/purchase_order_line.py:397`, `PurchaseOrderLine._compute_allowed_uom_ids`); el dominio de `product_uom_id` es `[('id','in',allowed_uom_ids)]`. En v18 el dominio era por categoría (cualquier UoM de la categoría). En v19 **solo** puedes elegir la UoM principal del producto o las de `uom_ids` (las "packagings"). Migración crítica: SO/PO/moves históricos con UoMs que NO estén en `product.uom_ids` quedan fuera del selector; el script de migración debe poblar `uom_ids` con las UoMs usadas en el histórico o rompen ediciones/recomputes.

**UOG35. v19: muchas `uom.uom` estándar vienen ARCHIVADAS + los M2M aplican `active_test` al leer** — En v19 varias UoM de `uom_data.xml` se cargan con `active=False` (ej. `product_uom_dozen`, `product_uom_pack_6` y otras). Añadir una UoM archivada a un M2M (ej. `product.template.uom_ids`) **persiste la fila** en la tabla relación, pero **al LEER el campo el ORM aplica `active_test`** y la excluye → `template.uom_ids` (y por tanto `allowed_uom_ids`) **no la muestra**, aunque el `INSERT` en `*_rel` sí ocurrió (verificado por SQL directo: la fila existe, el getter la oculta). Síntoma típico: "añado la Docena al producto y no aparece en el selector de la línea". Fix: activar la UoM (`active=True`) o usar/crear una UoM activa. Afecta a tests (`assertIn(uom, tmpl.uom_ids)` con dozen falla porque dozen viene archivada) y a datos. No confundir con un fallo de escritura: la escritura funciona, el filtro es en la lectura. (Descubierto 2026-07-18 desarrollando `indaws_uom`.)

**UOG34. v19: `product.uom` (modelo nuevo) exige `barcode` obligatorio y único global** — El modelo que reemplaza a `product.packaging` (`product/models/product_uom.py`) tiene `barcode` **required** con `_barcode_uniq = unique(barcode)` y además un `_check_barcode_uniqueness` que cruza contra los barcodes de `product.product` (compatibilidad GS1, `product/models/product_uom.py:20-26`, `class ProductUom`). Consecuencia: no puedes registrar una UoM adicional de producto (ex-packaging) sin barcode, y un barcode ya usado por otro producto/packaging lanza ValidationError. Migración de packagings v18 sin barcode → hay que generarles uno. `uom_id`/`product_id` son `ondelete='cascade'`.

---

---

## product.category (CTG)

**CTG1. `property_cost_method` vive en categoría, no en producto** — Las property fields de valoración (`property_cost_method`, `property_valuation`, `property_stock_journal`, `property_stock_account_*`) están definidas en `product.category`, no en `product.template`. **Standard/FIFO/AVCO afecta a TODA la categoría**. Si quieres methods distintos en productos de "la misma familia", deben estar en sub-categorías diferentes. Bug típico: cambiar `standard_price` en producto Standard cost esperando que ajuste valoración — no lo hace si la categoría es FIFO/AVCO.

**CTG2. Cambiar `categ_id` de producto con stock dispara reevaluación masiva** — `stock_account.product_template._product_categ_id_changed` recalcula SVLs cuando cambia `property_cost_method` o `property_valuation` entre categoría vieja/nueva. Puede generar **cientos de SVLs + asientos** en un solo write. Si haces import masivo cambiando categoría → bloqueo de BD durante minutos. Programar fuera de horas pico, con backup.

**CTG3. `removal_strategy_id` — ubicación gana a categoría** — Resolución: `stock.location.removal_strategy_id` (si set) → `category.removal_strategy_id` → default FIFO. **NO** es categoría > ubicación como sugiere la intuición. Si configuras FEFO en categoría "Frescos" pero la ubicación es FIFO, gana FIFO. Auditar ambos al diagnosticar "el sistema no respeta FEFO".

**CTG4. Cuentas resueltas con cascada Producto → Categoría → Padre** — `_get_product_accounts` resuelve `property_account_income_id` (producto) → `categ.property_account_income_categ_id` (categoría) → recursivo por `parent_id` hasta encontrar valor. **NO** hay fallback al "All" root automático — si todos los padres vacíos, queda vacío y el journal default toma el control. Configurar cuentas en categorías ALTAS de la jerarquía maximiza herencia.

**CTG5. `parent_path` SQL prefix matching** — Para queries tipo "productos en categoría X y sus hijas" se usa `parent_path` (string `'1/3/8/'`). `('categ_id.parent_path', 'like', X.parent_path + '%')` es la forma correcta. `('categ_id', 'child_of', X.id)` también funciona pero más lento. **NO usar `('categ_id', 'in', X.ids)`** — solo da la categoría exacta, sin hijas.

**CTG6. 3 categorías master no se pueden borrar** — `product.product_category_all`, `product.cat_expense`, `product.product_category_1`. `unlink()` lanza UserError. Si quieres "reorganizar" la jerarquía y eliminar la root genérica → imposible sin override.

**CTG7. `product_properties_definition` (categoría) define schema dinámico** — `template.product_properties` (Properties field) hereda su schema de la categoría. **Cambiar de categoría con properties activas**: las properties que no existen en la nueva categoría **se conservan en el JSON pero quedan invisibles**. Si vuelves a la antigua, reaparecen. No es bug pero confunde en migraciones.

**CTG8. `product.category._compute_product_count` cuenta toda la rama** — Incluye productos de subcategorías. Si quieres solo productos directos: `search_count([('categ_id', '=', categ.id)])` (sin child_of).

**CTG9. Mover categoría con `parent_id` change valida recursión** — `_check_category_recursion` lanza ValidationError si crearías un ciclo. Pero NO valida invariantes funcionales: mover "Electrónica" a "Servicios" puede dejar productos con `property_account_income_categ_id` heredado de categoría que ya no aplica al tipo de producto. No hay constraint que lo evite.

**CTG10. `cost_method` y `valuation` en `product.template` son related a categoría** — `stock_account` añade `template.cost_method = related('categ_id.property_cost_method')` y `template.valuation = related('categ_id.property_valuation')`. Son **read-only** (no se puede escribir en producto, hay que cambiar la categoría). Esto sorprende a quien viene de v13 donde se podían poner en producto.

---

## product.pricelist + product.pricelist.item (PLSG)

**PLSG1. `applied_on` lexicográfico determina precedencia** — El `_order` `'applied_on, min_quantity desc, ...'` ordena por **string**. Los selectores `'0_product_variant'`, `'1_product'`, `'2_product_category'`, `'3_global'` están diseñados para que orden lex = especificidad descendente. Si custom code crea un valor adicional `'4_brand'`, **viene DESPUÉS de global** y nunca se aplica. Para insertarlo entre variant y product → `'0_5_brand'` o reorganizar.

**PLSG2. `min_quantity` en UoM del PRODUCTO, no del SO line** — `_compute_price_rule` convierte la qty del SO line a `product.uom_id` antes de comparar con `min_quantity`. Si SO line vende en "Pack of 12" qty=2 y el producto tiene `uom_id=Unit` → comparación con min_quantity=20 (Units) → matches. Confuso: en la UI parece que "min_quantity es la qty de la línea". Documentarlo o usar UoM del producto consistente.

**PLSG3. Pricelist archivada (`active=False`) SIGUE matcheando** — `_get_applicable_rules` usa `with_context(active_test=False)` para items archivados. Para "apagar" una regla temporalmente: NO archives, usa `date_end` en el pasado. Archivar la pricelist entera sí funciona (pero rompe el partner.property_product_pricelist si está set a ella).

**PLSG4. Pricelist congelada en SO line al confirmar** — `sale.order.line.pricelist_item_id` se store al confirm. Cambios posteriores en la pricelist NO actualizan precios de SOs confirmadas. Para refrescar: botón "Actualizar precios" o método `_recompute_prices()`. Esto es por diseño (precio "pactado"), pero confunde si cambias mucho la pricelist.

**PLSG5. `fixed_price` se interpreta en `product.uom_id`** — Si producto vende en Units y SO en Dozen, `fixed_price=10` se convierte a 120 por docena vía `_compute_price` (UoM, no redondeo). Si quieres "fijo en la UoM del SO", no se puede directamente.

**PLSG6. Orden en fórmula es crítico: discount → round → surcharge → margin clamp** — En modo `formula`: primero descuenta, luego redondea, luego suma surcharge, luego aplica min/max margin. Si esperas "primero margin, luego descuento" → error. Para precios complejos puede requerir pricelists encadenadas (base = otra pricelist).

**PLSG7. `base_pricelist_id` permite recursión, constraint solo evita auto-ref directa** — `_check_pricelist_recursion` evita `pricelist == base_pricelist_id`. NO evita ciclos largos (A→B→C→A). Si construyes ciclo a mano → `_compute_base_price` entra en recursión infinita y RecursionError en Python. Bug abierto.

**PLSG8. `price_extra` (de variantes) se suma al base price ANTES de la regla** — Si la regla es `compute_price='percentage'` con base=`list_price`, el cálculo es `(list_price + sum(PTAV.price_extra)) * (1 - percent/100)`. Para que el descuento NO afecte al extra: usar regla con base=`standard_price` + override custom.

**PLSG9. Multi-currency: conversion en `_compute_base_price` con `round=False`** — Mantiene precisión, luego redondea al final. Si tu reporte muestra "10.00000099€" en vez de "10€" → casi siempre conversión multi-currency sin redondeo intermedio. Aplicar `currency.round()` explícito en la línea de display.

**PLSG10. Pricelist por country_group resuelve por país del partner** — `_get_partner_pricelist_multi` busca pricelist con `country_group_ids.country_ids in partner.country_id`. Si partner no tiene país → cae a default. Si tiene country pero ninguna pricelist matcha → cae al `ir.config_parameter` global. Bug común: nuevo partner sin país tiene la pricelist "incorrecta" porque no entra en ninguna country_group.

**PLSG11. `applied_on` se re-asigna automáticamente desde el contenido (NO hay `_onchange_rule_content`)** — En v18 el método se llama distinto: en la UI, `_onchange_display_applied_on` (`product/models/product_pricelist_item.py:365`, `class PricelistItem`) deriva `applied_on` desde `display_applied_on` + presencia de `product_tmpl_id`/`categ_id`; el `name` lo recalcula `_compute_name` (`:162-174`) según `applied_on`. Hay además `_check_product_consistency` (`:321-329`, ValidationError) que exige coherencia: `applied_on='2_product_category'` requiere `categ_id`, `'1_product'` requiere `product_tmpl_id`, etc. Resultado práctico: imports con `applied_on` incoherente con los campos set se "corrigen" o fallan por la constraint. No existe `_onchange_rule_content`.

**PLSG12. `currency_id` per pricelist != currency_id per company** — Una pricelist puede tener currency distinta de la company. El precio resultante está en pricelist currency, no en company currency. Las SO confirmadas guardan currency del pricelist en `sale.order.currency_id`. **Si cambias** la currency de una pricelist con SOs confirmadas → cálculo de conversiones futuras puede romper. Crear pricelist nueva en vez de cambiar.

**PLSG13. Regla "based on Cost" / fórmula sobre coste ignora el `price_extra` de variantes** — Distinto de PLSG8 (que es `percentage` sobre `list_price`). Cuando la regla usa `base='standard_price'` (Cost) o una fórmula que parte del coste, el `sum(PTAV.price_extra)` NO entra en el cálculo: `_compute_price_rule` parte del coste de la variante, y `price_extra` solo se acumula sobre el precio de venta (`product.product.lst_price`, ver PRG8). Síntoma: el cliente configura "precio = 100% del coste + margen" y todas las variantes salen al mismo precio aunque tengan `price_extra` distinto. Para incluir el extra: pricelist `based on Sales Price` (`percentage`) o sumar el extra explícitamente con `_get_attributes_extra_price` en código.

---

## product.supplierinfo (SIG)

**SIG1. `sequence` gana sobre precio en `_select_seller`** — Aunque la doc dice "selecciona el mejor", el algoritmo agrupa por **primer vendor que aparece en sort `(sequence, -min_qty, price, id)`**. Vendor con sequence=10 a 100€ gana sobre sequence=20 a 80€. Si quieres "siempre el más barato": pon todos a misma sequence.

**SIG2. `min_qty` evaluado en UoM del vendor** — `_get_filtered_sellers` convierte qty de la PO line a `seller.product_uom` antes de comparar. Cambiar UoM de la PO line puede excluir vendors. Bug típico: PO en "Pallet" (= 100 units), supplierinfo con `min_qty=50` en "Unit" → matches (porque 1 Pallet = 100 Units > 50). Pero si supplierinfo `min_qty=200` → NO matches.

**SIG3. `product_uom` es related a `uom_po_id`, NO editable per-supplierinfo** — `product.supplierinfo.product_uom = related('product_tmpl_id.uom_po_id')`. Para tener un supplierinfo con UoM distinta del `uom_po_id` general del producto: imposible sin override custom. Limitación frecuente: "este vendor cotiza en kg pero compramos en cajas" → no hay solución vanilla en v18.

**SIG4. `product_id=False` aplica a TODAS las variantes** — Patrón típico: precios base a nivel template (`product_id=False`) + overrides por variante. El sort en `_get_filtered_sellers` evalúa ambos, gana el de menor sequence/min_qty/price. Si quieres "solo aplicar override de variante e ignorar base": dejar base con `date_end` pasado.

**SIG5. Cambios en supplierinfo NO retroactivos** — POs ya confirmadas mantienen su `price_unit`. Cambiar el precio en supplierinfo afecta solo a POs futuras (o re-trigger `_compute_price_unit` en draft). En MTO automático, supplierinfo vigente en el momento del procurement se usa.

**SIG6. `delay` (lead time) suma al `date_planned`** — `_run_buy` calcula `date_planned = date_needed - timedelta(days=seller.delay)`. Si `delay` mal (ej. 0 cuando debería ser 7), POs salen muy tarde para llegar a tiempo. Auditar lead times reales vs configurados periodicamente.

**SIG7. `_sanitize_vals` rellena `product_tmpl_id` desde `product_id` pero NO valida coherencia** — Si das `product_id=A.id` y `product_tmpl_id=B.id` (template distinto), no se lanza error. Queda guardado un supplierinfo "huérfano" (variante A asociada a template B). `_get_filtered_sellers` puede dar resultados inesperados. Validar con: `seller.product_id.product_tmpl_id == seller.product_tmpl_id` en checks.

**SIG8. Multi-company: `company_id=False` significa "todas las companies"** — Como pricelists. Útil para precios globales, pero combinarlo con `product.template.company_id` set restringe acceso. En multi-company SAR, mejor poner `company_id` explícito en cada supplierinfo para evitar leaks de precio entre empresas.

---

## Servicios (SRVG)

**SRVG1. `service_tracking != 'no'` requiere módulo (`sale_project` o `sale_timesheet`)** — Sin uno de los dos, el Selection solo tiene `'no'`. Si importas datos con `service_tracking='task_in_project'` y los módulos no están instalados → ValidationError o silent revert a `'no'`.

**SRVG2. Cambiar `service_tracking` retroactivo no afecta SOs existentes** — Si pasas de `'no' → 'task_in_project'`, SOs ya confirmadas siguen sin tener task asociada. Solo nuevas SOs crean task. Para retroactivo manual: bucle script generando `project.task` por cada `sale.order.line` afectada.

**SRVG3. `service_policy='delivered_timesheet'` requiere `sale_timesheet`** — Sin él, la opción no aparece en Selection. Si el módulo está pero `service_tracking='no'` → al guardar Odoo puede silenciosamente cambiar a `'ordered_prepaid'` (no hay donde imputar timesheets).

**SRVG4. `service_policy='delivered_milestones'` crea `sale.order.line.milestone_ids`** — Requiere `sale_project`. Cada milestone es un hito con `quantity_percentage` (% de la qty total). Marcar `reached=True` actualiza `qty_delivered` proporcional. Si milestones no suman 100% → línea nunca se factura del todo.

**SRVG5. Servicios y BoM — incompatibles en MRP nativo** — `type='service'` no permite `bom_ids` (constraint). Subcontratación es la única forma de "fabricar" un servicio (vendor con type=service + supplierinfo + PO automática).

**SRVG6. `service_tracking='project_only'` crea proyecto completo, NO solo task** — Diferencia con `task_in_project`. Útil para "Cada SO genera un proyecto entero" (consultoría grande). Pero crea proyectos con un solo task → ruido en `/odoo/projects/` si se usa indiscriminadamente.

**SRVG7. Servicios y stock — generalmente sin `is_storable`** — Por defecto `type='service'` no tiene `is_storable=True`. Si activas `is_storable` en service → aparece en stock pero las moves no tienen sentido físico (no hay qty real). Solo casos especiales (créditos prepagados, "horas en bolsa").

**SRVG8. `invoice_policy='delivery'` en service** — Activable también en services (no solo consu). Cambia el momento de facturación: solo se factura qty marcada como entregada/timesheet/milestone. Para subscriptions, `service_policy='ordered_prepaid'` es lo habitual (factura por adelantado).

---

## Cross-references

- Núcleo de modelos: `core.md`
- Mecánica de variantes: `variants.md`
- API de UoM: `uom.md`
- Categoría, Pricelist, Supplierinfo: `pricelist_supplier_category.md`
- Integraciones con otros módulos: `integrations.md`
- Cambios v19 (UoM refactor, packaging removido): `v19_changes.md`
- Stock dual UoM: `stock/gotchas.md` STKG61
- Stock valoración (decimal precision crítica): `stock/account.md`
- Sale pricelist (interacción con UoM): `sale/gotchas.md`
- Purchase vendor price (triple conversión): `purchase/core.md`
- Multi-company y property fields: `cross-module/security/gotchas.md`
- Servicios + project/timesheet: `project/core.md`, `enterprise/timesheet/core.md`
