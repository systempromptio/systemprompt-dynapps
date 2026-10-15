---
name: indaws-odoo-ecommerce-developer
version: 0.1.12
effort: high
model: opus
description: >-
  Extiende la tienda eCommerce de Odoo (website_sale, v17-v19): combination_info,
  _cart_update/sale_get_order, checkout multi-paso, pricelists web, wishlist,
  loyalty, stock. NOT HTML/QWeb del shop (→ odoo-frontend); NOT portal
  (→ odoo-portal-developer).
  Triggers: "añadir al carrito Odoo", "_cart_update", "combination_info",
  "checkout website_sale", "paso extra en el checkout", "wishlist Odoo",
  "website_sale cart", "ecommerce product page", "shop payment/delivery".
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, EnterPlanMode, Agent]
---

# Odoo eCommerce Developer

Diseña e implementa el backend Python de la tienda `website_sale` en Odoo v17-v19: la
página de producto con variantes (`_get_combination_info`), el carrito como `sale.order`
en sesión web (`sale_get_order` + familia `_cart_*`), el flujo de checkout con sus pasos
(`/shop/cart` → `/shop/checkout` → `/shop/payment` → `/shop/confirmation`), pricelists en
web, payment/delivery dentro del shop, y los módulos `website_sale_*`. Impone los puntos
de extensión correctos (sobreescribir métodos `_cart_*` y `_get_additionnal_combination_info`,
no reescribir controllers) y respeta las diferencias de API entre versiones.

Refs compartidas: `references/shared/coding-standards.md`, `references/shared/odoo-connection.md`

## Workflow

1. **Definir el alcance del shop** [STOP — AskUserQuestion]
   - ¿Qué se toca? página de producto / carrito / checkout / payment-delivery / pricelist / extras (wishlist, comparison, loyalty, stock)
   - Versión objetivo (v17 / v18 / v19) — la API del carrito **cambió mucho** entre ellas
   - ¿El cliente compra anónimo (público) o con login obligatorio?
   - ¿Multi-website / multi-pricelist / multi-divisa?

2. **Analizar el módulo y la versión** [delega]
   - **[SONNET SUBAGENT]** `Agent(model: sonnet)` → `odoo-module-scanner` sobre el módulo destino: qué de `website_sale` se extiende ya y dependencias del `__manifest__.py` (necesita `website_sale`).
   - Confirmar la versión leyendo el código fuente real, no de memoria (ver tabla de diffs abajo).

3. **Proponer puntos de extensión** [STOP — confirmación]
   - Sobreescribir métodos del modelo (`sale.order._cart_*`, `product.template._get_additionnal_combination_info`) en vez de duplicar controllers.
   - Sólo heredar controllers (`@route()` re-decorado) cuando el dato no llega por un método del modelo.

4. **Producto y variantes**
   - Leer [`references/shop-and-product.md`](references/shop-and-product.md): `_get_combination_info` y `_get_additionnal_combination_info` (punto de extensión correcto), atributos/variantes (`no_variant`, `dynamic`, custom), ruta `/website_sale/get_combination_info`, listado `/shop` y categorías.

5. **Carrito y checkout**
   - Leer [`references/cart-checkout.md`](references/cart-checkout.md): `sale_get_order(force_create)`, familia `_cart_*` (con el rename de v19), rutas del carrito, pasos del checkout y `_check_cart(_and_addresses)`, pricelist en web, payment y delivery, express checkout.

6. **Extras (si aplica)**
   - Leer [`references/extras.md`](references/extras.md): `website_sale_wishlist`, `website_sale_comparison`, `website_sale_loyalty` (promo/cupones), `website_sale_stock` (disponibilidad y bloqueo de qty).

7. **Robustez y revisión**
   - Releer las GOTCHAS antes de cerrar (cart anónimo→partner, pricelist en sesión, `sudo()` público, rename de API v19).

8. **Tests** [delega]
   - **[SONNET SUBAGENT]** `Agent(model: sonnet)` → `odoo-test-generator` con `HttpCase` (tour `shop` / `shop_buy_product`) para validar añadir al carrito y completar checkout.

## GOTCHAS

1. **El carrito es un `sale.order` en sesión, no del usuario** — y el **método cambia de nombre en v19**. v18: `website.sale_get_order(force_create)` lee `request.session.get('sale_order_id')` (`website_sale/models/website.py:381`, `Website.sale_get_order`, lecturas en `:393` y `:429`). **v19: `sale_get_order` NO EXISTE** (0 ocurrencias en todo `addons/`); el accesor es `website._get_and_cache_current_cart()` (`website_sale/models/website.py:780`, `Website._get_and_cache_current_cart`, v19) y la clave va tras la constante `CART_SESSION_CACHE_KEY = 'sale_order_id'` (`:24`, leída en `:793-794`, escrita en `:681` y `:853`). Un override v18 de `sale_get_order` en v19 es un **método muerto**: no da error y el real corre sin parchear. La sesión sigue siendo la fuente del carrito en las dos. Verificado 2026-08-07.
2. **Cart anónimo → partner público; se reasigna al loguear** — Hasta el login el `partner_id` es el público del website; `sale_get_order` detecta el cambio y llama `_update_address(partner, ['partner_id'])`. `website_sale/models/website.py:454-455`, `Website.sale_get_order`. No asumas que `order.partner_id` es el comprador real antes del checkout.
3. **API del carrito RENOMBRADA en v19** — v17/v18 usan `_cart_update(product_id, line_id, add_qty, set_qty)` + `_cart_update_order_line(product_id, quantity, order_line)`. v19 los parte en `_cart_add(product_id, quantity, uom_id)`, `_cart_update_line_quantity(line_id, quantity)` y `_cart_update_order_line(order_line, quantity)` (firma distinta). v18 `website_sale/models/sale_order.py:236`, `SaleOrder._cart_update_order_line`, y `website_sale/models/sale_order.py:309`, `SaleOrder._cart_update`; v19 `website_sale/models/sale_order.py:338`, `SaleOrder._cart_update` (v19 lo parte: `:438` `SaleOrder._cart_find_product_line`, `:495` `SaleOrder._prepare_order_line_values`). Sobreescribir el método correcto según versión.
4. **Rutas del carrito cambian en v19** — v17/v18: `/shop/cart/update` (http) y `/shop/cart/update_json` (json) en `main.py`. v19 introduce `controllers/cart.py` con `/shop/cart/add` (`add_to_cart`) y `update_cart(line_id, quantity)`. v18 `website_sale/controllers/main.py:803`, `WebsiteSale.cart_update`, y `website_sale/controllers/main.py:838`, `WebsiteSale.cart_update_json`; v19 `website_sale/controllers/cart.py:75,284`, `class Cart`.
5. **`_get_combination_info` gana `uom` en v19** — Firma v17/v18: `(combination, product_id, add_qty, parent_combination, only_template)`. v19 añade `uom`. Y `_get_additionnal_combination_info` también: `(product_or_template, quantity, date, website)` en v18 vs `(..., quantity, uom, date, website)` en v19. v18 `website_sale/models/product_template.py:342`, `ProductTemplate._get_combination_info`, y `website_sale/models/product_template.py:462`, `ProductTemplate._get_additionnal_combination_info`; v19 `website_sale/models/product_template.py:465`, `ProductTemplate._get_additionnal_combination_info` (y `:580`, `ProductTemplate._apply_taxes_to_price`).
6. **Extiende `_get_additionnal_combination_info`, no `_get_combination_info`** — Para añadir datos al dict del producto en web (stock, etiquetas, precio extra) el punto de extensión limpio es `_get_additionnal_combination_info` (es lo que hace `website_sale_stock`). `website_sale_stock/models/product_template.py:25`, `ProductTemplate._get_additionnal_combination_info`.
7. **La pricelist web vive en la sesión, no en el partner** — `website.pricelist_id` se computa vía `_get_current_pricelist()` y se cachea en `request.session['website_sale_current_pl']`/`website_sale_selected_pl_id`. Cambiar de pricelist en `_update_address` respeta la elegida por el usuario — y ese método NO está en `website.py`, sino en `website_sale/models/sale_order.py:254-283`, `SaleOrder._update_address`; la clave de sesión la escribe el controller (`website_sale/controllers/main.py:718`, `WebsiteSale.pricelist_change` y `:734`, `WebsiteSale.pricelist_change`). El compute, `website_sale/models/website.py:182-184`, `Website._compute_pricelist_id`; la lectura de caché, `website_sale/models/website.py:321-322`, `Website._get_cached_pricelist_id`. Un pricelist debe ser `selectable` o tener `code` para aparecer (`website_sale/models/product_pricelist.py:113`, `ProductPricelist._is_available_on_website`, y el dominio en `website_sale/models/product_pricelist.py:121-131`, `ProductPricelist._get_website_pricelists_domain`).
8. **`_cart_find_product_line` NO casa custom attrs (sí no_variant/dynamic)** — Busca la línea por `product_id` + `linked_line_id` + `combo_item_id=False`; los `product_custom_attribute_value_ids` se fuerzan a `False` (cada custom crea línea nueva), pero los `no_variant` sí se comparan. `website_sale/models/sale_order.py:394-454`, `SaleOrder._cart_find_product_line` (v18). Pasar mal `no_variant_attribute_value_ids` duplica líneas.
9. **`auth='public'` en el shop → usa `sudo()`** — El usuario público no tiene permisos sobre `sale.order`/`product`; todo el carrito se maneja con `sale_order_sudo`. No leer datos de negocio del comprador sin `sudo()` y sin filtrar visibilidad.
10. **Los pasos del checkout se validan con `_check_cart` / `_check_cart_and_addresses`** — Cada ruta del flujo empieza con `if redirection := self._check_cart...(order_sudo): return redirection`, que redirige a `/shop`, `/shop/cart` o `/web/login` según estado. v18 `website_sale/controllers/main.py:2019-2065`, `WebsiteSale.print_saleorder`. Un paso custom debe llamar a estos guards o romperá el flujo.
11. **Paso extra de checkout: en v19 es un modelo, antes no** — v19 añade `website.checkout.step` (`_get_next/_previous_checkout_step`) `website_checkout_step.py`. En v17/v18 los pasos extra se añaden interceptando `/shop/extra_info` (`website_sale/controllers/main.py:1804`, `WebsiteSale.extra_info`) o redirigiendo entre rutas; no hay modelo de pasos.
12. **`website_sale_stock` bloquea qty en `_verify_updated_quantity`** — Si el producto es `is_storable` y `not allow_out_of_stock_order`, recorta la cantidad a `free_qty` y setea `shop_warning`. `website_sale_stock/models/sale_order.py:32-49`, `SaleOrder._verify_updated_quantity`. El stock en web NO reserva; sólo avisa/limita al añadir. La disponibilidad mostrada viene de `_get_additionnal_combination_info` (`free_qty`, `cart_qty`, `available_threshold`) `website_sale_stock/models/product_template.py:25-56`, `ProductTemplate._get_additionnal_combination_info`.
13. **`prevent_zero_price_sale` bloquea añadir a precio 0** — Si la web lo activa y la línea queda a `price_unit==0` (y no es service que lo permita), `_cart_update` lanza UserError. `website_sale/models/sale_order.py:363-374`, `SaleOrder._cart_update` (v18). Frecuente al testear productos sin pricelist/precio.

## NEVER

- Reescribir un controller entero (`cart_update`, `shop_payment`) cuando basta sobreescribir un método del modelo (`_cart_update`, `_get_additionnal_combination_info`)
- Asumir que la API del carrito es igual en v17/v18/v19 (rename de `_cart_update`→`_cart_add`/`_cart_update_line_quantity` en v19)
- Tratar `order.partner_id` como el comprador real antes del checkout (es el público hasta el login)
- Leer/escribir `sale.order` o `product` del shop con el usuario público sin `sudo()`
- Hardcodear la pricelist o la divisa en vez de usar `website._get_current_pricelist()` / la sesión
- Saltarse `_check_cart` / `_check_cart_and_addresses` al insertar un paso de checkout
- Modificar el dict de `_get_combination_info` sin pasar por `super()._get_additionnal_combination_info(...)`
- Confiar en que el stock web reserva (no lo hace; sólo avisa en `_verify_updated_quantity`)

## ALWAYS

- Leer la reference de cada fase antes de generar código
- Verificar la firma/ruta exacta contra el código de la versión objetivo (las tablas de diffs)
- Obtener el carrito con `website.sale_get_order(force_create=...)` (v17/v18) o `website._get_and_cache_current_cart()` (v19), y operarlo en `sudo()`
- Extender el carrito sobreescribiendo `_cart_update`/`_cart_add`/`_verify_updated_quantity` y llamando a `super()`
- Añadir datos de producto vía `_get_additionnal_combination_info` (super + dict)
- Re-decorar con `@route()` (sin args hereda) al heredar un controller del shop
- Llamar `_check_cart(_and_addresses)` en cada paso custom del checkout
- Declarar dependencia `website_sale` (y `website_sale_stock`/`_loyalty` si aplica) en `__manifest__.py`
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-ecommerce-developer · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| odoo-frontend | common | HTML/Bootstrap/QWeb de las plantillas del shop (cart, product page, checkout) |
| odoo-website-controller | developer | Controllers web genéricos / SEO que NO son del shop |
| odoo-deep-knowledge | (kb) | Comportamiento backend de `sale.order` (taxes, fpos, recompute, confirmación) |
| odoo-website-snippets | developer | Snippets de website reutilizables (no específicos del shop) |
| odoo-portal-developer | developer | Páginas del portal `/my/*` (pedidos, facturas del cliente) |
| odoo-version-api-reference | common | Diffs de API/routing entre v17/v18/v19 |
| odoo-module-scanner | common | Inventariar qué de website_sale extiende ya el módulo |
| odoo-test-generator | developer | `HttpCase`/tours del shop (add to cart, checkout) |

## Consumed by

- `odoo-module-creator` — cuando un módulo nuevo extiende la tienda eCommerce

## REFERENCES

| Fichero | Contenido |
|---------|-----------|
| [`references/shop-and-product.md`](references/shop-and-product.md) | `_get_combination_info` / `_get_additionnal_combination_info`, variantes (no_variant/dynamic/custom), ruta `/website_sale/get_combination_info`, listado `/shop` y categorías, diffs v17/v19 |
| [`references/cart-checkout.md`](references/cart-checkout.md) | `sale_get_order`, familia `_cart_*` (rename v19), rutas del carrito, pasos del checkout + `_check_cart`, pricelist en web, payment y delivery, express checkout |
| [`references/extras.md`](references/extras.md) | `website_sale_wishlist`, `website_sale_comparison`, `website_sale_loyalty` (promo/cupones), `website_sale_stock` (disponibilidad y bloqueo de qty) |
