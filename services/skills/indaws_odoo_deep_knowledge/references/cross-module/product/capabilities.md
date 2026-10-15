# Product — Capacidades (eje consultor: "¿Odoo PUEDE hacer X y a qué coste?")

> Complemento del eje implementación (`variants.md`, `core.md`) y del eje debugging (`gotchas.md`).
> Cada fila = una pregunta real de cliente respondida **contra fuente v18/v19**, con veredicto
> (**nativo** / **config** / **desarrollo** / **no** / **no encontrado**), complejidad y ancla.
> Verificado contra `odoo18/addons/product` y `odoo19/addons/product` (+ `mrp` para kits).
>
> Prefijos: `ATTRCAP` = atributos · `PAVCAP` = valores de atributo · `VARCAP` = variantes · `UOMCAP` = unidades de medida · kits → `mrp/capabilities.md`.

---

## TANDA 1 — Atributos (`product.attribute`)

### A. Modelo, catálogo y alcance

**ATTRCAP1. ¿Los atributos son globales o por producto?** — NATIVO (global) · baja · `product_attribute.py::ProductAttribute` · El `product.attribute` ("Talla") es un catálogo **global**; se conecta a cada template vía `product.template.attribute.line` (PTAL). Un mismo atributo se reusa en N templates.

**ATTRCAP2. ¿Puedo usar el mismo atributo "Color" en cientos de productos?** — NATIVO · baja · `attribute_line_ids` (O2M PTAL) · Sí; cada template añade una PTAL que apunta al atributo global. Los `value_ids` (PAV) también se comparten.

**ATTRCAP3. ¿Cuántos atributos puede tener un producto?** — NATIVO (sin límite duro) · baja · `product_template_attribute_line` · No hay límite de nº de PTAL por template; el límite práctico es el **producto cartesiano de variantes** (`product.dynamic_variant_limit`=1000 en modo `always`, ver VAG5).

**ATTRCAP4. ¿Cuántos valores puede tener un atributo?** — NATIVO (sin límite) · baja · `value_ids` · Sin límite en el modelo; impacta al nº de combinaciones (multiplica).

**ATTRCAP5. ¿Puedo saber en qué productos se usa un atributo?** — NATIVO · baja · `product_tmpl_ids` (compute store) + `number_related_products` · Campo computed que lista templates; `action_open_product_template_attribute_lines` abre las PTAL. `number_related_products` solo cuenta templates **activos** (`product_tmpl_id.active=True`).

**ATTRCAP6. ¿Los atributos se ven siempre en la ficha de producto?** — CONFIG · baja · `group_product_variant` · La pestaña "Atributos y Variantes" solo aparece si está activado el ajuste **Ventas → Variantes** (`product.group_product_variant`, `product_views.xml`). Sin él, no hay UI de atributos aunque el modelo exista.

**ATTRCAP7. ¿Puedo tener atributos sin activar "Variantes" en ajustes?** — NO (UI) / NATIVO (datos) · baja · `group_product_variant` · La UI está gated; por ORM/import sí se pueden crear, pero el usuario no los verá sin el grupo.

### B. Modos de creación de variante (`create_variant`)

**ATTRCAP8. ¿Qué modos de generación de variante existen?** — NATIVO · baja · `create_variant` selection (`product/models/product_attribute.py:27-39`, `class ProductAttribute`) · Tres: `always` (Instantly), `dynamic` (Dynamically), `no_variant` (Never).

**ATTRCAP9. ¿Cuál es el modo por defecto?** — NATIVO · baja · `default='always'` · `always`: cada combinación crea `product.product` al instante.

**ATTRCAP10. ¿Puedo tener un atributo que NO genere variantes (solo opción visual)?** — NATIVO · baja · `create_variant='no_variant'` · Aparece en el configurador pero no crea `product.product` (grabados, mensajes, extras).

**ATTRCAP11. ¿Puedo diferir la creación de variantes a cuando se venden (catálogo enorme)?** — NATIVO · baja · `create_variant='dynamic'` · Variantes bajo demanda vía `_get_variant_for_combination`/`_create_product_variant`; `product_variant_count==0` es legítimo (VAG4).

**ATTRCAP12. ¿Puedo cambiar el modo de un atributo ya usado en productos?** — NO (bloqueado) · alta · `write` override (`product/models/product_attribute.py:117-125`, `ProductAttribute.write`) · `UserError` si `number_related_products>0`. Hay que borrar (unlink, no archivar) todas las PTAL, cambiar el modo y re-crearlas (VAG1).

**ATTRCAP13. ¿Puedo mezclar modos en el mismo producto (un atributo always + otro dynamic)?** — NATIVO (con efecto) · media · `_create_variant_ids` (`has_dynamic_attributes`) · Sí, pero **si al menos un atributo es `dynamic`, el template entero se comporta como dinámico** (no pre-genera combinaciones). Efecto no obvio.

**ATTRCAP14. ¿`no_variant` y `dynamic` se pueden combinar en un producto?** — NATIVO · media · `_without_no_variant_attributes` · Sí; `no_variant` se ignora en la generación y `dynamic` difiere el resto.

### C. Tipo de visualización (`display_type` / widgets del configurador)

**ATTRCAP15. ¿Qué widgets de selección hay para un atributo?** — NATIVO · baja · `display_type` (`product/models/product_attribute.py:40-50`, `class ProductAttribute`) · v18: `radio`, `pills`, `select`, `color`, `multi` (multi-checkbox). **v19 añade `image`** (`product/models/product_attribute.py:44`, `class ProductAttribute` v19).

**ATTRCAP16. ¿Puedo mostrar swatches de color?** — NATIVO · baja · `display_type='color'` + `html_color` en PAV · Hex `#rrggbb` por valor; si el value tiene `image`, la imagen tiene prioridad sobre el color (`product/views/product_attribute_views.xml:65`, `product_attribute_view_form`).

**ATTRCAP17. ¿Puedo usar imágenes como swatch (no solo color)?** — NATIVO (v18 parcial / v19 explícito) · baja · PAV `image` + `display_type='image'` (v19) · En v18 el `image` del value se usa dentro de `display_type='color'` como alternativa al hex. En **v19 hay un `display_type='image'` dedicado**.

**ATTRCAP18. ¿Puedo cambiar el display_type después de crear el atributo?** — CONFIG · baja · `display_type` sin readonly ni write guard: siempre editable (`product/views/product_attribute_views.xml:45`, `product_attribute_view_form`) · Única restricción: SQL constraint `CHECK(display_type != 'multi' OR create_variant = 'no_variant')` (`product/models/product_attribute.py:17`, `class ProductAttribute`) + onchange que fuerza `create_variant='no_variant'` al pasar a multi. El readonly de `product/views/product_attribute_views.xml:49`, `product_attribute_view_form` (`number_related_products != 0 or display_type == 'multi'`) aplica a `create_variant`, NO a `display_type`. Verificado 2026-07-15.

**ATTRCAP19. ¿El display_type afecta a las variantes generadas?** — NO (solo UI) · baja · `display_type` · Es puramente presentación en el configurador/web; no cambia qué `product.product` se crean (salvo `multi`, que fuerza `no_variant`).

**ATTRCAP20. ¿`radio`/`pills`/`select` cambian la lógica de negocio?** — NO · baja · — · Idénticos funcionalmente; solo estética del configurador.

### D. Multi-checkbox

**ATTRCAP21. ¿Puedo permitir seleccionar varios valores del mismo atributo a la vez (toppings)?** — NATIVO · baja · `display_type='multi'` · Multi-checkbox permite marcar varios values simultáneos.

**ATTRCAP22. ¿Multi-checkbox puede generar variantes?** — NO (prohibido) · baja · SQL/`Constraint` `check_multi_checkbox_no_variant` (`product/models/product_attribute.py:14-20`, `class ProductAttribute` v18 / `:14-17` v19) · `multi` **obliga** `create_variant='no_variant'`; incompatible con variantes (VAG3).

**ATTRCAP23. ¿El onchange fuerza no_variant al elegir multi?** — NATIVO · baja · `_onchange_display_type` (`product/models/product_attribute.py:102-105`, `class ProductAttribute`) · Solo si `number_related_products==0`; si ya tiene productos, el constraint bloquea el guardado.

### E. Traducción, orden, secuencia

**ATTRCAP24. ¿El nombre del atributo es traducible?** — NATIVO · baja · `name = fields.Char(translate=True)` · Sí.

**ATTRCAP25. ¿Puedo ordenar los atributos en el configurador?** — NATIVO · baja · `sequence` (default=20, `index=True`) · `_order='sequence, id'`.

**ATTRCAP26. ¿Reordenar atributos invalida caches?** — NATIVO (side-effect) · baja · `write` (`product/models/product_attribute.py:126-132`, `ProductAttribute.write`) · Cambiar `sequence` fuerza `flush_all()`+`invalidate_all()` para resecuenciar O2M prefetch (p.ej. `attribute_line_ids`).

**ATTRCAP27. ¿El orden de los valores dentro del atributo se respeta?** — NATIVO · baja · PAV `sequence` (index) · Sí, `product.attribute.value` tiene `sequence`.

### F. Archivado, borrado, duplicado

**ATTRCAP28. ¿Puedo archivar un atributo?** — NATIVO (condicionado) · baja · `action_archive` (`product/models/product_attribute.py:148-154`, `ProductAttribute.action_archive`) · Solo si `number_related_products==0`; si hay productos → `UserError`.

**ATTRCAP29. ¿Puedo borrar un atributo usado en productos?** — NO · baja · `_unlink_except_used_on_product` (`@api.ondelete`, `:135-144`) · `UserError` listando los productos. Hay que quitar las PTAL primero.

**ATTRCAP30. ¿Duplicar un atributo copia sus valores?** — NATIVO · baja · `value_ids = fields.One2many(..., copy=True)` · Sí, `copy=True` en `value_ids` (`product/models/product_attribute.py:56`, `class ProductAttribute`).

**ATTRCAP31. ¿`active=False` oculta el atributo sin borrarlo?** — NATIVO · baja · `active` field (`:23-26`) · Sí, con el gate de `action_archive`.

### G. Constraints y edición segura

**ATTRCAP32. ¿Puedo renombrar un atributo en producción?** — NATIVO · baja · `name` sin constraint de write · Sí; propaga a `display_name` de PTAV (prefijo "Atributo: Value", VAG11).

**ATTRCAP33. ¿Hay riesgo al cambiar el modo por script/ORM saltándome el aviso?** — SÍ (riesgo) · alta · `write` override · El override valida igual por ORM; no hay bypass salvo SQL directo (desaconsejado: deja combinaciones inconsistentes).

**ATTRCAP34. ¿El constraint multi/no_variant se valida a nivel BD?** — NATIVO · baja · `models.Constraint` CHECK · Sí, es un CHECK SQL; imposible violarlo ni por SQL de escritura normal.

### H. Reutilización y semántica cross-módulo (dónde impacta)

**ATTRCAP35. ¿Los atributos afectan a precios?** — NATIVO · baja · vía PAV `default_extra_price`→PTAV `price_extra` · El extra price vive en el **valor**, no en el atributo (ver Tanda 2 / VAG9).

**ATTRCAP36. ¿Puedo excluir combinaciones (talla XL no en color Rojo)?** — NATIVO · media · `product.template.attribute.exclusion` · Sí, a nivel PTAV; direccional pero validado en ambos sentidos (VAG6, VAG7).

**ATTRCAP37. ¿Los atributos se usan en pricelists por variante?** — CONFIG · baja · `applied_on='0_product_variant'` (`product_pricelist_item_views.xml`) · Sí, reglas de precio a nivel variante (gated por `group_product_variant`).

**ATTRCAP38. ¿Los atributos condicionan líneas de BoM (MRP)?** — NATIVO · media · `bom_line.bom_product_template_attribute_value_ids` · Una línea de BoM puede aplicar solo a ciertas PTAV (VAG17).

**ATTRCAP39. ¿Los atributos funcionan en POS?** — SÍ (configurador nativo) · alta · v18/v19 · POS trae configurador de atributos nativo (`ProductConfiguratorPopup`): resuelve atributos no_variant/dynamic y valores custom (price_extra incluido) en el punto de venta. Los `create_variant=always` generan `product.product` que aparecen como productos individuales en la rejilla, pero el popup también los cubre. No requiere OCA/custom para el flujo estándar. Verificado 2026-07-15.

**ATTRCAP40. ¿Los atributos se usan como filtros/facetas en eCommerce?** — NATIVO · media · website_sale (`display_type` swatches) · Sí; el `display_type` controla el widget web (VAG18).

### I. Custom / texto libre (a nivel atributo→valor)

**ATTRCAP41. ¿Puedo permitir que el cliente escriba texto libre (grabado)?** — NATIVO · baja · PAV `is_custom` + `product.attribute.custom.value` · El flag vive en el valor; el texto se guarda por línea de SO (Tanda 2, VAG20).

### J. Import / API / mantenimiento masivo

**ATTRCAP42. ¿Puedo importar atributos y valores por CSV/Excel?** — NATIVO · baja · import estándar de `product.attribute`/`product.attribute.value` · Modelos normales; se importan con External ID.

**ATTRCAP43. ¿Puedo crear atributos por API (XML-RPC/JSON-RPC)?** — NATIVO · baja · ORM `create` · Sí; respeta los mismos constraints (multi/no_variant, etc.).

**ATTRCAP44. ¿Hay un asistente para actualizar precios extra masivamente?** — NATIVO · baja · wizard `update.product.attribute.value` · Recorre PTAV de un PAV y reescribe `price_extra` con `default_extra_price` (VAG9).

### K. Diferencias v18 → v19 (atributos)

**ATTRCAP45. ¿Cambió la definición de constraints en v19?** — SÍ · baja · `models.Constraint` · v18 usa `_sql_constraints` (lista); **v19 usa `_check_multi_checkbox_no_variant = models.Constraint(...)`**. Mismo CHECK, sintaxis nueva.

**ATTRCAP46. ¿Hay nuevos display_type en v19?** — SÍ · baja · `('image','Image')` (`product/models/product_attribute.py:44`, `class ProductAttribute` v19) · v19 añade tipo de visualización "Image" dedicado.

**ATTRCAP47. ¿Cambió alguna acción a readonly en v19?** — SÍ (menor) · baja · `@api.readonly` en `action_open_product_template_attribute_lines` (`:154` v19) · Marcada readonly (optimización RPC), sin cambio funcional.

**ATTRCAP48. ¿`create_variant`, `_create_variant_ids`, `_without_no_variant_attributes` siguen igual en v19?** — SÍ · baja · diff limpio · La lógica de generación no cambió v18→v19 (solo constraints/display_type/readonly).

---

## TANDA 2 — Valores de atributo (`product.attribute.value` = PAV, `product.template.attribute.value` = PTAV)

> Dos capas: **PAV** = valor global ("Rojo"); **PTAV** = valor materializado por template (con `price_extra` y exclusiones propias). La clave de la variante son las PTAV, no las PAV (ver `variants.md`).

### L. Precio extra por valor

**PAVCAP1. ¿Puedo cobrar un extra por un valor (ej. talla XXL +5€)?** — NATIVO · baja · PAV `default_extra_price` → PTAV `price_extra` (`product/models/product_attribute_value.py:35`, `class ProductAttributeValue`, `product/models/product_template_attribute_value.py:35`, `class ProductTemplateAttributeValue`) · El extra se suma al precio base de la variante (`product.product.price_extra`).

**PAVCAP2. ¿El extra es global o por producto?** — NATIVO (ambos) · baja · `default_extra_price` (global) vs `price_extra` (por template) · `default_extra_price` es la plantilla; el real aplicado vive en cada PTAV, por lo que **el mismo "Rojo" puede costar +5€ en una camiseta y +0€ en otra**.

**PAVCAP3. ¿Si cambio el extra global se actualizan los productos existentes?** — NO (no sincroniza) · media · VAG9 · `default_extra_price` solo se copia a `price_extra` al **crear** la PTAV. Cambiarlo después NO propaga (gotcha clásico).

**PAVCAP4. ¿Cómo propago un cambio de extra a todas las variantes existentes?** — NATIVO (wizard) · baja · `action_update_prices` → `update.product.attribute.value` mode `update_extra_price` (`product/models/product_attribute_value.py:166-178`, `ProductAttributeValue.action_update_prices`) · Botón "Update product extra prices" reescribe `price_extra` de todas las PTAV con el `default_extra_price`.

**PAVCAP5. ¿El extra puede ser negativo (descuento por valor)?** — NATIVO · baja · `price_extra = fields.Float(default=0.0)` · No hay constraint de signo; un negativo resta al precio.

**PAVCAP6. ¿El extra respeta la moneda del producto?** — NATIVO · baja · PTAV `currency_id` (related a `product_tmpl_id.currency_id`) · El `price_extra` se interpreta en la moneda del template.

**PAVCAP7. ¿El extra entra en la pricelist / se recalcula con reglas?** — CONFIG · media · `product.product.price_extra` + pricelist · El extra se aplica a la base ANTES de las reglas de pricelist (ver `pricelist_supplier_category.md`); `lst_price = list_price + price_extra` NO incluye pricelist.

**PAVCAP8. ¿Puedo ver si un valor tiene el precio "desincronizado"?** — NATIVO · baja · `default_extra_price_changed` (compute, `product/models/product_attribute_value.py:80-93`, `class ProductAttributeValue`) · Flag UI que compara `default_extra_price` con los `price_extra` de las PTAV (usa `sudo` para ver productos sin acceso).

### M. Texto libre / personalización (is_custom)

**PAVCAP9. ¿Puedo dejar que el cliente escriba un valor (grabado, dedicatoria)?** — NATIVO · baja · PAV `is_custom` ("Free text", `:36-38`) · Marca el valor como texto libre; el cliente teclea en el configurador.

**PAVCAP10. ¿Dónde se guarda el texto que escribe el cliente?** — NATIVO · baja · `product.attribute.custom.value` (`custom_value` Char) · Se guarda por **línea de SO** (`custom_product_template_attribute_value_id` → PTAV), no en `product.product` (VAG20).

**PAVCAP11. ¿Un valor is_custom crea una variante nueva por cada texto?** — NO · baja · VAG20 · Usa la variante "base"; el texto es un adjunto de la línea. Reporting de personalizaciones exige unir SO line + custom.value.

**PAVCAP12. ¿is_custom funciona en cualquier modo de variante?** — NATIVO · baja · `is_custom` independiente de `create_variant` · Sí, típicamente con `no_variant`/`dynamic`.

**PAVCAP13. ¿El nombre del custom value muestra "Atributo: texto"?** — NATIVO · baja · `product_attribute_custom_value.py::_compute_name` · Prefija con el `display_name` de la PTAV + el texto.

### N. Color, imagen, presentación del valor

**PAVCAP14. ¿Puedo asignar un color HTML a un valor?** — NATIVO · baja · PAV `html_color` (hex `#ff0000`, `:39-42`) · Usado con `display_type='color'`.

**PAVCAP15. ¿Puedo poner una imagen (foto de tela) a un valor?** — NATIVO · baja · PAV `image` (Image, max 70×70, `:45-50`) · La imagen tiene prioridad sobre `html_color` en el swatch (VAG18).

**PAVCAP16. ¿Hay un color de UI (kanban) distinto del swatch?** — NATIVO · baja · PAV `color` (Integer, default aleatorio 1-11, `:44`) · `color` = índice de color de la UI interna; distinto de `html_color` (swatch web).

**PAVCAP17. ¿El display_type se hereda del atributo al valor?** — NATIVO · baja · PAV `display_type = related('attribute_id.display_type')` · No se define por valor.

### O. Traducción, orden, unicidad (PAV)

**PAVCAP18. ¿El nombre del valor es traducible?** — NATIVO · baja · `name = Char(translate=True)` · Sí.

**PAVCAP19. ¿Puedo ordenar los valores dentro del atributo?** — NATIVO · baja · PAV `sequence` (index) · `_order='attribute_id, sequence, id'`; reordenar fuerza `flush_all`/`invalidate_all` (`:108-114`).

**PAVCAP20. ¿Puedo tener dos valores con el mismo nombre en un atributo?** — CONFIG (sin constraint en PAV) · baja · — · A nivel PAV no hay unicidad de nombre; la unicidad SQL está en **PTAV** `(attribute_line_id, product_attribute_value_id)` (mismo valor una vez por producto).

**PAVCAP21. ¿Puedo mover un valor de un atributo a otro?** — NO (si está usado) · baja · `write` override (`:97-106`) · `UserError` si `is_used_on_products`; el atributo del valor se bloquea una vez usado.

### P. Archivado y borrado (PAV)

**PAVCAP22. ¿Puedo borrar un valor usado en productos?** — NO · baja · `_unlink_except_used_on_product` / `check_is_used_on_products` (`:117-130`) · `UserError` listando productos.

**PAVCAP23. ¿Qué pasa si borro un valor usado solo en variantes archivadas?** — NATIVO (archiva) · media · `unlink` override (`:132-145`) · Si hay `product.product` inactivos que lo usan (y ninguno activo), el PAV se **archiva** en vez de borrarse (preserva historial).

**PAVCAP24. ¿Puedo archivar un valor?** — NATIVO · baja · PAV `active` (`:51`) · Sí; desaparece del configurador nuevo, las líneas antiguas siguen mostrándolo.

**PAVCAP25. ¿Borrar un atributo borra sus valores?** — NATIVO · baja · `attribute_id` ondelete='cascade' (`:25`) · Sí, cascade (pero el borrado del atributo está bloqueado si tiene productos, ATTRCAP29).

**PAVCAP26. ¿Duplicar un atributo duplica los enlaces a líneas de los valores?** — NO · baja · `pav_attribute_line_ids` copy=False (`:33`) · Los valores se copian (ATTRCAP30) pero no sus vínculos a PTAL existentes.

### Q. Aplicar un valor a muchos productos de golpe

**PAVCAP27. ¿Puedo añadir un valor nuevo a todos los productos que ya usan el atributo?** — NATIVO (wizard) · baja · `action_add_to_products` → `update.product.attribute.value` mode `add` (`:152-164`) · Botón "Add to all products".

**PAVCAP28. ¿Puedo saber en qué productos se usa un valor?** — NATIVO · baja · `is_used_on_products` (compute) + `pav_attribute_line_ids` · Solo cuenta templates activos.

### R. PTAV — valor materializado por producto

**PAVCAP29. ¿Por qué el campo se llama `ptav_active` y no `active`?** — NATIVO · media · PTAV `ptav_active` (`product/models/product_template_attribute_value.py:21-23`, `class ProductTemplateAttributeValue`) · Se llama `ptav_active` (NO `active`) a propósito para que `active_test` del ORM no filtre los archivados: las vistas de config muestran los valores archivados por defecto (VAG8). No existe campo `active` en el modelo → un dominio `('active','=',True)` NO da vacío: lanza error (campo inexistente). Filtrar activos con `('ptav_active','=',True)`. Verificado 2026-07-15.

**PAVCAP30. ¿Puedo cambiar el valor (PAV) de una PTAV existente?** — NO · baja · `write` override (`:96-105`) · `UserError`; hay que borrar/archivar y recrear (VAG12).

**PAVCAP31. ¿Puedo mover una PTAV a otro producto?** — NO · baja · `write` override (`:106-111`) · `UserError` al cambiar `product_tmpl_id`.

**PAVCAP32. ¿Puedo asignar variantes a una PTAV directamente?** — NO · baja · `create`/`write` override (`:83-95`) · `ptav_product_variant_ids` es readonly; escribir lanza `UserError`. Solo desde `product.product.product_template_attribute_value_ids` (VAG10).

**PAVCAP33. ¿El nombre mostrado de una PTAV incluye siempre el atributo?** — NATIVO (incondicional) · baja · `_compute_display_name` (`:151-158`) · Siempre `"Atributo: Value"`; a diferencia del PAV, **no respeta** `with_context(show_attribute=False)` (VAG11).

**PAVCAP34. ¿Cómo se compone el nombre de la variante?** — NATIVO · media · `_get_combination_name` (`:169-173`) · Concatena PTAV **excluyendo** las de líneas single-value y las de atributos `no_variant` (`_filter_single_value_lines`, `_without_no_variant_attributes`).

**PAVCAP35. ¿La unicidad de valor por producto está garantizada?** — NATIVO · baja · SQL `attribute_value_unique` (`:65-69`) · `unique(attribute_line_id, product_attribute_value_id)`.

**PAVCAP36. ¿Se valida que la PTAV pertenezca a los valores de su línea?** — NATIVO · baja · `_check_valid_values` (`@api.constrains`, `:71-81`) · Una PTAV activa cuyo PAV no esté en `attribute_line_id.value_ids` → `ValidationError`.

**PAVCAP37. ¿Borrar una PTAV usada en pedidos rompe el historial?** — NO (archiva) · media · `unlink` override (`:117-149`) · Intenta borrar dentro de un `savepoint`; si falla por FK, la **archiva** (`ptav_active=False`) en vez de romper.

**PAVCAP38. ¿Borrar una PTAV de una línea single-value la quita de las variantes?** — NATIVO · media · `unlink` (`:130-134`) · Si la línea tenía un solo valor, hace `Command.unlink` del PTAV en las variantes antes de borrar.

### S. Exclusiones a nivel valor

**PAVCAP39. ¿Puedo excluir combinaciones desde un valor (Rojo no en talla XL)?** — NATIVO · media · PTAV `exclude_for` (O2M `product.template.attribute.exclusion`, `:43-48`) · Se guardan en la PTAV; cualquier cambio dispara `_create_variant_ids` (`:113-114`).

**PAVCAP40. ¿Puedo excluir valores de productos opcionales/accesorios (cross-producto)?** — NATIVO · media · `exclude_for` (help: "optional and accessory products") · La exclusión soporta PTAV de productos opcionales, no solo del mismo template.

**PAVCAP41. ¿Las exclusiones son bidireccionales?** — NATIVO (validación) / config (almacenamiento) · media · VAG6 · Se guardan en una dirección pero `_is_combination_possible` valida ambas; recomendable definir en ambos extremos por claridad.

**PAVCAP42. ¿Una exclusión puede dejar el producto sin variantes?** — SÍ (bloquea) · media · VAG7 · Si todas las combinaciones quedan imposibles → `UserError` y se revierte.

### T. Diferencias v18 → v19 (valores)

**PAVCAP43. ¿Cambió el modelo PAV en v19?** — CASI NO · baja · diff · Solo `@api.readonly` en `action_add_to_products` y `action_update_prices` (`:152,167` v19). Campos y lógica idénticos.

**PAVCAP44. ¿Cambió el modelo PTAV en v19?** — MENOR · baja · diff · `_sql_constraints` (lista) → `_attribute_value_unique = models.Constraint(...)`; y `write(self, values)` renombrado a `write(self, vals)` con alias interno (`product/models/product_template_attribute_value.py:90-91`, `class ProductTemplateAttributeValue` v19). Sin cambio funcional.

**PAVCAP45. ¿`price_extra`, `is_custom`, exclusiones, `_get_combination_name` cambian en v19?** — NO · baja · diff limpio · Idénticos v18→v19.

---

## TANDA 3 — Generación de variantes (`product.product` / `product.template`)

> El motor es `product.template._create_variant_ids()` (`product/models/product_template.py:721`, `ProductTemplate._create_variant_ids` v18 / `:770` v19).
> Modelo variante: `product.product`, `_inherits = {'product.template': 'product_tmpl_id'}`.

### U. Cómo genera Odoo las variantes

**VARCAP1. ¿Cómo crea Odoo las variantes de un producto?** — NATIVO (automático) · media · `_create_variant_ids` (`product/models/product_template.py:721`, `ProductTemplate._create_variant_ids`) · Producto cartesiano de los valores activos de cada línea (`itertools.product`), filtrando combinaciones imposibles por exclusión. Se dispara al cambiar líneas/valores/exclusiones.

**VARCAP2. ¿Siempre existe al menos una variante por producto?** — NATIVO · baja · `_create_variant_ids` (comentario `:763-764`) · Sí: un template sin atributos crea 1 `product.product` (`not any([])` es True). Excepción legítima: modo `dynamic` puede dejar 0 (VAG4).

**VARCAP3. ¿Puedo forzar la regeneración manual de variantes?** — NATIVO · baja · `_create_variant_ids` (botón/acción) · Se puede invocar; también corre tras cambios en PTAL/PTAV/exclusión.

**VARCAP4. ¿Un atributo con un solo valor crea variantes extra?** — NO · baja · single-value injection (`product/models/product_template.py:741-753`, `ProductTemplate._create_variant_ids`) · No multiplica: el valor se **inyecta** en las variantes existentes sin generar combinaciones nuevas (VAG2).

**VARCAP5. ¿Cómo sé cuántas variantes tiene un template?** — NATIVO · baja · `product_variant_count` (compute, `product/models/product_template.py:150`, `class ProductTemplate`) · Campo computed.

### V. Modo `always` (Instantly)

**VARCAP6. ¿En modo "always" cuándo se crean las variantes?** — NATIVO · baja · `_create_variant_ids` (rama `not has_dynamic_attributes`, `:765`) · Al instante, todas las combinaciones válidas.

**VARCAP7. ¿Hay un límite de variantes generables?** — NATIVO (config) · media · `product.dynamic_variant_limit` default 1000 (`:779-784`) · Superarlo → `UserError` ("number of variants... above allowed limit"). Se comprueba `> int(variant_limit)` durante la creación (VAG5).

**VARCAP8. ¿Puedo subir el límite de 1000?** — CONFIG · baja · `ir.config_parameter` `product.dynamic_variant_limit` · Sí, pero la performance degrada por encima de ~10k variantes.

**VARCAP9. ¿Qué pasa si añado un valor a un atributo "always" ya en uso?** — NATIVO · media · `_create_variant_ids` · Se generan las nuevas combinaciones y se reactivan las archivadas que reaparezcan.

### W. Modo `dynamic` (Dynamically)

**VARCAP10. ¿Puedo NO pre-generar miles de variantes (configurador)?** — NATIVO · baja · `create_variant='dynamic'` · No se crean hasta que se piden.

**VARCAP11. ¿Cómo se crea una variante bajo demanda?** — NATIVO · media · `_get_variant_for_combination` / `_create_product_variant` (`product/models/product_template.py:1133`, `ProductTemplate._get_variant_for_combination` y `product/models/product_template.py:1149`, `ProductTemplate._create_product_variant`) · Buscar-o-crear al añadir la combinación en un SO/quotation.

**VARCAP12. ¿Basta con que UN atributo sea dynamic para diferir todo?** — NATIVO (efecto global) · media · `has_dynamic_attributes` (`:830`) · Sí: si el template tiene ≥1 atributo dynamic, `_create_variant_ids` NO pre-genera ninguna combinación (VAG13/gotcha ATTRCAP13).

**VARCAP13. ¿`product_variant_count==0` es un bug en dynamic?** — NO · baja · VAG4 · Es legítimo; reportes que asumen ≥1 variante rompen.

**VARCAP14. ¿Puedo detectar por campo si un template es dinámico? (v19)** — NATIVO (v19) · baja · `is_dynamically_created` (Boolean compute, `product/models/product_template.py:175`, `ProductTemplate` y `product/models/product_template.py:239`, `ProductTemplate._compute_product_variant_id` v19) · **Nuevo en v19**: campo booleano que expone `has_dynamic_attributes()`. En v18 hay que llamar al método.

### X. Modo `no_variant` (Never)

**VARCAP15. ¿Un atributo no_variant genera product.product?** — NO · baja · `_without_no_variant_attributes` (`:732`) · Se filtra fuera del cartesiano; no crea variantes.

**VARCAP16. ¿El extra price de un no_variant se aplica igual?** — NATIVO · media · `_get_no_variant_attributes_price_extra` (`product/models/product_product.py:777`, `ProductProduct._get_no_variant_attributes_price_extra`) · Sí: el `price_extra` de PTAV no_variant se pasa vía contexto `no_variant_attributes_price_extra` a la línea de venta, sin materializar variante.

### Y. Unicidad / identidad de la variante (`combination_indices`)

**VARCAP17. ¿Cómo identifica Odoo una combinación única?** — NATIVO · media · `combination_indices` (Char stored+index, `product/models/product_product.py:51`, `class ProductProduct`) · String de IDs de PTAV ordenados (`_ids2str`); `_compute_combination_indices` (`:255-258`).

**VARCAP18. ¿Hay unicidad a nivel BD de la combinación?** — NATIVO · baja · UNIQUE INDEX `product_product_combination_unique` (`product/models/product_product.py:201`, `ProductProduct.init`) · `(product_tmpl_id, combination_indices) WHERE active is true` — no puede haber dos variantes activas idénticas.

**VARCAP19. ¿Puedo buscar una variante por combinación con `('combination_indices','=',...)`?** — NO (frágil) · media · VAG16 · El string está ordenado por ID; comparar con orden distinto falla. Usar `_get_variant_for_combination(ptav_recordset)`.

**VARCAP20. ¿Escribir la combinación en la variante refresca caches?** — NATIVO · media · `product.product.write` (`:381-388`) · Cambiar `product_template_attribute_value_ids` o `active` hace `registry.clear_cache()` (afecta `_get_variant_id_for_combination`).

### Z. Relación template ↔ variante (`_inherits`)

**VARCAP21. ¿Los campos del template se ven en la variante?** — NATIVO · baja · `_inherits={'product.template':'product_tmpl_id'}` (`product/models/product_product.py:19`, `class ProductProduct`) · Delegación: leer un campo de template en la variante lo trae del template.

**VARCAP22. ¿Escribir un campo de template en una variante afecta a las hermanas?** — SÍ (cuidado) · media · PRG2 / `_inherits` · Escribir un campo delegado en una variante escribe en el **template compartido** → cambia todas las variantes.

**VARCAP23. ¿Qué campos son propios de la variante (no del template)?** — NATIVO · media · `product.product` fields · `standard_price` (company_dependent por variante), `default_code`, `barcode`, `image_variant_1920`, `product_template_attribute_value_ids`, `combination_indices`, `price_extra`, volumen/peso propios.

**VARCAP24. ¿El coste (`standard_price`) puede diferir por variante y por compañía?** — NATIVO · media · `standard_price` company_dependent en variante (PRG3) · Sí; el template muestra el del primer/único variante (`_compute_template_field_from_variant_field`, `:261`).

**VARCAP25. ¿`weight`/`volume` del template reflejan la variante?** — NATIVO (parcial) · baja · `_compute_template_field_from_variant_field` (`:317,324`) · El template muestra el valor del variante solo si hay 1 variante; con varias devuelve 0/base (PRG4).

**VARCAP26. ¿Puedo poner precio de venta (`list_price`) distinto por variante?** — NO (directo) / NATIVO (vía extra o pricelist) · media · `list_price` es del template · El precio base es del template; la diferencia por variante se hace con `price_extra` (PTAV) o reglas de pricelist a nivel variante.

**VARCAP27. ¿Puedo poner imagen distinta por variante?** — NATIVO · baja · `image_variant_1920` · Sí; si se borra la última variante con imagen y el template no tiene, la imagen sube al template (`unlink`, `product_product.py:~407`).

**VARCAP28. ¿Referencia interna/código de barras por variante?** — NATIVO · baja · `default_code`, `barcode` en `product.product` · Propios de la variante.

### AA. Archivado, borrado, reactivación

**VARCAP29. ¿Se borran o se archivan las variantes que dejan de ser válidas?** — NATIVO (archiva) · media · `_unlink_or_archive` (`product/models/product_product.py:426`, `ProductProduct._unlink_or_archive`) · Intenta borrar; si hay FK (SO/PO/stock) las archiva (`active=False`). No borra historial (VAG15).

**VARCAP30. ¿Una combinación archivada se reactiva si vuelve a ser válida?** — NATIVO · media · `_create_variant_ids` (`:775-776,797-799`) · Sí: si la combinación existe como variante inactiva, se reactiva en vez de crear nueva (preserva FK).

**VARCAP31. ¿Borrar la última variante borra el template?** — NATIVO (condicionado) · media · `product.product.unlink` (`:~415`) · Si es la última y el template NO es dynamic, se borra también el template. En dynamic se conserva (se recrea on-demand).

**VARCAP32. ¿Debo borrar manualmente variantes archivadas?** — NO · baja · VAG15 · No; SO/PO/stock antiguos las referencian.

**VARCAP33. ¿Qué pasa con los "combo items" al desaparecer una variante?** — NATIVO · baja · `_create_variant_ids` (`:807-812`) · Los `product.combo.item` que referencian la variante eliminada se `unlink`.

### BB. Precio de la variante

**VARCAP34. ¿Cómo se calcula el precio de una variante?** — NATIVO · baja · `lst_price = list_price + price_extra` (`product/models/product_product.py:278-290`, `class ProductProduct`) · `price_extra` = suma de `price_extra` de las PTAV de la combinación (`_compute_product_price_extra`, `:273-276`). NO incluye pricelist (usar `_get_product_price`).

**VARCAP35. ¿El `lst_price` depende de la UoM del contexto?** — NATIVO · media · `@api.depends_context('uom')` · Sí; convierte `list_price` a la UoM del contexto (ver `uom.md`).

### CC. Casos límite / performance

**VARCAP36. ¿Qué pasa si una exclusión elimina la última variante posible?** — SÍ (bloquea) · media · `_create_variant_ids` (`:804-806`) · `UserError`: "This configuration of product attributes, values, and exclusions would lead to no possible variant. Please archive or delete your product directly if intended." (VAG7).

**VARCAP37. ¿Miles de variantes degradan el sistema?** — SÍ · alta · `product.dynamic_variant_limit` · El cartesiano crece multiplicativamente; recomendación >1000 combinaciones = pasar a `dynamic`.

**VARCAP38. ¿`_create_variant_ids` es transaccional/seguro ante fallos?** — NATIVO · media · `_unlink_or_archive` con savepoint + `flush_all`/`invalidate_all` (`:814-819`) · Sí; usa savepoints y recarga caches por el active_test.

**VARCAP39. ¿Puedo crear variantes por API sin pasar por atributos?** — SÍ (desaconsejado) · media · `product.product.create` · Se puede crear un `product.product` con `product_template_attribute_value_ids`; debe respetar la unicidad de combinación y validez, si no rompe el UNIQUE INDEX.

### DD. Diferencias v18 → v19 (variantes)

**VARCAP40. ¿Cambió el motor `_create_variant_ids` en v19?** — NO · baja · diff (solo shift de líneas) · Firma y lógica idénticas v18→v19.

**VARCAP41. ¿Qué se añadió en v19 sobre variantes?** — SÍ (menor) · baja · `is_dynamically_created` (`product/models/product_template.py:175`, `class ProductTemplate` v19, computed NO almacenado) · Campo booleano nuevo; True si el template tiene alguna `attribute_line_ids` con `create_variant='dynamic'`. Lógica propia paralela a `has_dynamic_attributes()` pero NO lo invoca (usa `attribute_line_ids`, no `valid_product_template_attribute_line_ids`). Útil en vistas; NO usable en domains (sin store ni search). Verificado 2026-07-15. (VARCAP14)

**VARCAP42. ¿`combination_indices`, `_inherits`, unicidad, límite 1000 siguen igual en v19?** — MAYORMENTE (1 cambio de forma) · baja · diff · `combination_indices` (field+compute+`_ids2str`), `_inherits` y el límite 1000 (`dynamic_variant_limit`) son idénticos. La **unicidad cambió de mecanismo**: v18 crea el índice en `init()` con SQL crudo (`product_product_combination_unique`, `product/models/product_product.py:196-203`, `ProductProduct.init`); v19 lo declara con `models.UniqueIndex("(product_tmpl_id, combination_indices) WHERE active IS TRUE")` (`:121`) y elimina el `init()`. Misma semántica (columnas + WHERE active), pero el nombre del índice ya no es `product_product_combination_unique`. Verificado 2026-07-15.

---

## TANDA 4 — Unidades de Medida (`uom.uom`, `product.uom`) — eje consultor

> Verificado contra `odoo18/addons/uom` y `odoo19/addons/uom` + `product`/`sale`/`purchase`/`stock`. Fecha 2026-07-17.
> Muchas respuestas **divergen v18↔v19** por el refactor del modelo `uom.uom` (árbol `relative_uom_id`, `factor` absoluto, packaging→`product.uom`). Debugging cruzado en `gotchas.md` UOG1-UOG34.

### A. Doble UoM compra / stock / venta

**UOMCAP1. ¿Puedo comprar en cajas y stockar/vender en unidades?** — NATIVO (flujo distinto en v19) · baja · v18 `product.uom_po_id` / v19 `product.uom_ids` + `allowed_uom_ids` · v18: se fija una "purchase UoM" (`uom_po_id`) por defecto. **v19: `uom_po_id` ELIMINADO** — hay que añadir la caja a `product.uom_ids` y elegirla a mano en la línea de PO (`purchase.order.line.product_uom_id`, sin default automático). Conversión a stock automática al confirmar. Ver UOG25/UOG33.

**UOMCAP2. ¿Odoo convierte solo la cantidad de compra (cajas) a stock (unidades)?** — NATIVO · baja · `uom.uom._compute_quantity` · Sí; al confirmar la PO el `stock.move` guarda la cantidad en la UoM de stock del producto. Doble conversión (vendor→línea→stock) puede arrastrar epsilon de redondeo (UOG12).

**UOMCAP3. ¿El proveedor puede cotizar en una UoM distinta a la de compra?** — NATIVO · baja · `product.supplierinfo.product_uom_id` (v19; v18 `product_uom`) · Sí; el precio del vendor se convierte a la UoM de la línea con `_compute_price`. Ojo v19: campo renombrado a `product_uom_id` (UOG25).

### B. Conversión entre magnitudes distintas (el clásico "vale la pena desarrollar")

**UOMCAP4. ¿Precio por kg y facturar por litros (u otra magnitud)?** — **NO nativo → DESARROLLO** · alta · `_has_common_reference` / `_compute_quantity` · Solo se convierte **dentro del mismo árbol** (v18: misma categoría; v19: misma raíz). kg (Peso) y litro (Volumen) no comparten árbol → no hay conversión estándar (requiere densidad). **v19 lo empeora**: `_compute_quantity`/`_compute_price` NO validan nada y devuelven un número **erróneo sin error** (`raise_if_failure` muerto, UOG28/UOG29). Cualquier conversión peso↔volumen↔longitud = módulo a medida con factor de densidad.

**UOMCAP5. ¿Puedo montar una jerarquía multinivel "1 palet = 4 cajas = 48 uds"?** — NATIVO y MEJOR en v19 · media · v19 `relative_uom_id` + `relative_factor` (`uom/models/uom_uom.py:36-40`, `UomUom.relative_factor`, y `uom/models/uom_uom.py:41`, `UomUom.relative_uom_id`, v19) · v18: todas las UoM son planas respecto a 1 reference de la categoría. **v19: árbol real** (parent/child), `factor` absoluto computed recursivo → jerarquías anidadas nativas. Config media (definir el árbol bien la primera vez).

### C. Precisión y redondeo

**UOMCAP6. ¿Distinta precisión de redondeo por UoM (kg 3 decimales, uds 0)?** — v18 SÍ / **v19 NO (regresión)** · media · v19 `uom.rounding` computed global (`_compute_rounding`) · v18: `rounding` editable por UoM. **v19: `rounding` es global**, derivado de `decimal.precision('Product Unit')` — TODAS las UoM comparten precisión. Para precisión distinta por magnitud en v19 → no nativo (workaround: precisión específica en el concepto donde aplica, o desarrollo). Ver UOG31 y `v19_changes.md` §1.

**UOMCAP7. ¿Impedir vender/mover en fracciones (solo enteros)?** — v18 config (rounding=1.0 por UoM) / v19 global · media · `decimal.precision('Product Unit')` · v18: UoM con `rounding=1.0`. v19: al ser global, poner 0 decimales afecta a TODO; alternativa `uom._check_qty` para forzar múltiplos de packaging. No hay "solo esta UoM entera" nativo en v19.

### D. Packagings / códigos de barras por unidad

**UOMCAP8. ¿Vender un producto en varios formatos (pack, caja) con su propio código de barras?** — NATIVO en v19 · baja · `product.uom` (`product/models/product_uom.py`) + `product.template.uom_ids` · v19 unifica "packaging" y "UoM secundaria" en el modelo `product.uom` (link producto↔UoM con barcode **obligatorio y único**, cross-check GS1). Reemplaza `product.packaging` (eliminado). Ver UOG34.

**UOMCAP9. ¿Migran mis `product.packaging` de v18 al actualizar a v19?** — SÍ (con riesgo) · alta · script `product/migrations/19.0.*` · Se convierten a `product.uom`/UoM. **Riesgos**: packagings sin barcode necesitan uno (barcode required); flags `sales`/`purchase` se pierden; UoMs históricas no listadas en `uom_ids` desaparecen del selector. Ver `v19_changes.md` §2/§10.

### E. Cambios de UoM en producto vivo

**UOMCAP10. ¿Puedo cambiar la UoM de un producto que ya tiene movimientos?** — **NO / peligroso** · alta · v19 `product.product._update_uom` (`stock/models/product.py:770`, `ProductTemplate.compute_is_storable`) · **v19**: si el producto tiene moves en una UoM distinta a la del template → **UserError, bloqueado** ("archive and create a new one"). Si todos los moves están en la UoM del template → hace **swap de nombre 1:1** (10 "uds" pasan a 10 "kg" SIN reconvertir cantidades). Nunca esperar reconversión numérica. Ver UOG32.

**UOMCAP11. ¿Cambiar el factor de una UoM ya usada recalcula el histórico?** — NO (solo avisa) · media · `_onchange_critical_fields` · Un warning UI dice que los datos existentes NO se actualizan. Cambiar factor en producción deja inconsistencias silenciosas → crear UoM nueva en su lugar. Ver UOG3.

### F. Restricción del selector / gobierno

**UOMCAP12. ¿Puedo limitar qué UoMs se pueden elegir en la línea de venta/compra?** — NATIVO en v19 (automático) · baja · `allowed_uom_ids = product.uom_id | product.uom_ids` · v18: el dropdown mostraba toda la categoría. **v19: solo la UoM principal + las de `uom_ids`** del producto. Gobierno "de fábrica"; para ampliar hay que añadir la UoM a `uom_ids`. Ver UOG33.

**UOMCAP13. ¿Ocultar las UoM a usuarios que no las usan?** — NATIVO (config) · baja · grupo `uom.group_uom` · Desactivar el grupo oculta los selectores y opera todo en stock UoM; no borra datos. Ver UOG18.

### G. Servicios, POS y fiscal

**UOMCAP14. ¿Facturar servicios por horas con jornada distinta de 8h/día?** — CONFIG · media · `uom.product_uom_hour` / crear UoM propia · La UoM Day estándar = 8h y la usa timesheets/HR. Para 7,5h NO cambiar la Day estándar (afecta a todo) → crear UoM "Day 7.5h" propia. Ver UOG20.

**UOMCAP15. ¿Vender por peso variable en POS (balanza)?** — LIMITADO · media · `pos.order.line.product_uom_id` (readonly) · POS no permite cambiar UoM ni vender en packaging en la línea; hay que usar un producto con `uom_id=kg` directamente. Sin conversión de UoM en la UI de POS. Ver UOG19.

**UOMCAP16. ¿La UoM de la línea llega a la factura electrónica (SII/Facturae)?** — SÍ pero literal → DESARROLLO/CONFIG l10n_es · media · `account.move.line.product_uom_id` → XML · La UoM va como texto; AEAT puede rechazar UoM no estándar (claves IVA que exigen kg/m³/litros). Mapear a UoM estándar antes de enviar. Ver UOG15.
</content>
</invoke>
