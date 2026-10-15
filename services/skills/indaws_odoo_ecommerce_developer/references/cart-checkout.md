# Cart, checkout, pricelist, payment & delivery (`website_sale`)

Rutas de origen (v18 salvo nota): `$ODOO_SRC/odoo18/addons/website_sale/`.

## Obtener el carrito

`website.sale_get_order(force_create=False)` — `website_sale/models/website.py:381`, `Website.sale_get_order` (v18). **En v19 este método
no existe** (verificado 2026-08-07: 0 ocurrencias de `def sale_get_order` en `addons/`): el
equivalente es `website._get_and_cache_current_cart()` (`website_sale/models/website.py:780`, `Website._get_and_cache_current_cart`), con la clave de
sesión tras `CART_SESSION_CACHE_KEY` (`:24`).
- Lee `request.session['sale_order_id']`; si no, `partner.last_website_so_id`.
- Sin `force_create=True` puede devolver recordset vacío (`:425-430`).
- Crea con `_prepare_sale_order_values(partner_sudo)` (`:459-469`): `partner_id`,
  `website_id`, pricelist, etc.
- **Reasigna partner al loguear**: si `partner_sudo.id` no coincide con
  `order.partner_id`, llama `_update_address(partner_sudo.id, ['partner_id'])` (`:454-455`).
- `sale_reset()` (`:496`) limpia `sale_order_id` de la sesión (tras pagar o invalidar).

El carrito SIEMPRE se opera en `sudo()` (usuario público sin permisos).

## Familia `_cart_*` (modelo `sale.order`)

### v17 / v18
`models/sale_order.py` (v18 `:309`, v17 `:176`):
- `_cart_update(product_id, line_id=None, add_qty=0, set_qty=0, **kwargs)` — orquesta:
  busca línea (`_cart_find_product_line`), verifica qty (`_verify_updated_quantity`),
  crea/actualiza/borra (`_cart_update_order_line`). Devuelve
  `{line_id, quantity, option_ids, warning}`. v18 `:309-392`.
- `_cart_update_order_line(product_id, quantity, order_line, **kwargs)` — crea/actualiza/
  borra la `sale.order.line`. v18 `:236-252`.
- `_cart_find_product_line(product_id, line_id=None, linked_line_id=False, no_variant_attribute_value_ids=None, **kwargs)` —
  v18 `:394-454`. Casa por `product_id` + `linked_line_id` + `combo_item_id=False`;
  `product_custom_attribute_value_ids` forzado a `False`; los `no_variant` SÍ se comparan
  (`:443-452`). Combo → siempre línea nueva.
- `_verify_updated_quantity(order_line, product_id, new_qty, **kwargs)` — v18 `:457`. Hook
  para limitar la qty (lo extiende `website_sale_stock`). Devuelve `(quantity, warning)`.

### v19 (RENOMBRADO — VERIFICAR)
`models/sale_order.py` (v19):
- `_cart_add(product_id, quantity=1.0, *, uom_id=None, **kwargs)` — `:338`.
- `_cart_update_line_quantity(line_id, quantity, **kwargs)` — `:438`.
- `_cart_update_order_line(order_line, quantity, **kwargs)` — `:495` (firma DISTINTA: ahora
  recibe `order_line` primero, sin `product_id`).
- `_verify_updated_quantity(order_line, product_id, new_qty, uom_id, **kwargs)` — `:492`
  (añade `uom_id`).
- `_cart_find_product_line(...)` — `:386`.

Al extender el carrito, sobreescribir el método de la versión correcta y llamar `super()`.

## Rutas del carrito

### v17 / v18 (`controllers/main.py`)
- `/shop/cart` (http) — `cart(...)` muestra la página. v18 `:757`, v17 `:804`.
- `/shop/cart/update` (http, POST) — `cart_update(...)` añadir sin opciones, redirige a
  `/shop/cart` o (si `express`) a `/shop/checkout?express=1`. v17 `:851-883`, v18 `:803`.
- `/shop/cart/update_json` (json, POST) — `cart_update_json(product_id, line_id, add_qty, set_qty, display, product_custom_attribute_values, no_variant_attribute_value_ids, **kwargs)`.
  Llama `order._cart_update(...)` y devuelve `notification_info`, `cart_quantity`,
  `cart_lines`/`total` renderizados, `minor_amount`/`amount` (express). v18 `:838-941`.
- `/shop/cart/quantity` (json) `:948`, `/shop/cart/clear` (json) `:954`.

### v19 (`controllers/cart.py`)
- `/shop/cart` (http) — `cart(id, access_token, revive_method, **post)` `:20`.
- `/shop/cart/add` (json) — `add_to_cart(...)` `:75-83` (sustituye `cart_update`/`update_json`).
- `update_cart(line_id, quantity, product_id=None, **kwargs)` `:284-292`.
- `/shop/cart/quantity` `:423`, `/shop/cart/clear` `:435`.

## Flujo de checkout y pasos

Secuencia: `/shop/cart` → `/shop/checkout` (`website_sale/controllers/main.py:1109`, `WebsiteSale.shop_address`) → `/shop/extra_info`
(opcional, `:1804`) → `/shop/payment` (`:1907`) → `/shop/payment/validate` (`:1935`) →
`/shop/confirmation` (`:1981`). Direcciones: `/shop/address` (`:1022`),
`/shop/update_address` (`:1752`), `/shop/confirm_order` (`:1787`).

**Guards obligatorios** — cada ruta empieza con:
```python
if redirection := self._check_cart(order_sudo):           # main.py:2033
    return redirection
# o, donde hace falta dirección válida:
if redirection := self._check_cart_and_addresses(order_sudo):  # main.py:2019
    return redirection
```
`_check_cart` redirige a `/shop` si no hay carrito draft, a `/shop/cart` si está vacío, o
a `/web/login` si público y `account_on_checkout == 'mandatory'` (`:2033-2058`).
`_check_addresses` redirige a `/shop/address` si `_is_anonymous_cart()` (`:2060+`).

**Paso extra**:
- v17/v18: interceptar `/shop/extra_info` (`extra_info`, `website_sale/controllers/main.py:1804`, `class WebsiteSale`) o insertar una
  ruta propia entre dos pasos y llamar a los guards. No hay modelo de pasos.
- v19: existe el modelo `website.checkout.step` (`models/website_checkout_step.py`,
  `_get_next_checkout_step`/`_get_previous_checkout_step`) — añadir un paso es un registro
  de datos, no código.

## Pricelist en web

- `website.pricelist_id` computado por `_get_current_pricelist()` (`website_sale/models/website.py:182-184`, `Website._compute_pricelist_id`; el resolutor, `website_sale/models/website.py:324-366`, `Website._get_current_pricelist`),
  cacheado en `request.session['website_sale_current_pl']` y, si el usuario eligió una,
  `website_sale_selected_pl_id` (`website_sale/models/sale_order.py:269-290`, `SaleOrder._update_address`).
- `website.pricelist_ids` (seleccionables) por `_get_website_pricelists_domain(website)`
  (`website_sale/models/product_pricelist.py:121-130`, `ProductPricelist._get_website_pricelists_domain`): un pricelist aparece si `selectable=True` o tiene
  `code`. `_is_available_on_website` `:98-113`.
- Cambiar pricelist: `/shop/change_pricelist/<pricelist>` (`website_sale/controllers/main.py:684`, `WebsiteSale.pricelist_change`) y `/shop/pricelist`
  (`:724`); el modelo usa `_cart_update_pricelist(pricelist_id)` (`website_sale/models/sale_order.py:302`, `SaleOrder._cart_update_pricelist`),
  que hace `_recompute_prices()`.

## Payment en el shop

- `/shop/payment` (`website_sale/controllers/main.py:1907`, `WebsiteSale._get_shop_payment_errors`) prepara valores con `_get_shop_payment_values`
  (`:1857`) — providers, tokens, métodos de entrega.
- `/shop/payment/transaction/<int:order_id>` (json) — `website_sale/controllers/payment.py:24-25`, `PaymentPortal.shop_payment_transaction`,
  `shop_payment_transaction(order_id, access_token, **kwargs)` crea la transacción;
  `_validate_transaction_for_order` (`website_sale/controllers/payment.py:17`, `PaymentPortal._validate_transaction_for_order`).
- `/shop/payment/validate` (`website_sale/controllers/main.py:1935`, `WebsiteSale.shop_payment`) — al volver del provider; llama
  `order._check_cart_is_ready_to_be_paid()` (`:1967`) y confirma.

## Delivery en el shop

`controllers/delivery.py` (v18):
- `/shop/delivery_methods` (json) — `shop_delivery_methods()` `:14`.
- `/shop/set_delivery_method` (json) — `shop_set_delivery_method(dm_id, **kwargs)` `:34`.
- `/shop/get_delivery_rate` (json, POST) — `shop_get_delivery_rate(dm_id)` `:90`.
- Pickup points: `/website_sale/get_pickup_locations` `:133`, `/website_sale/set_pickup_location` `:123`.

El modelo recalcula tarifa en `_cart_update` si hay `carrier_id`
(`website_sale/models/sale_order.py:375-383`, `SaleOrder._cart_update`) y la quita si `only_services`.

## Express checkout

- `_get_express_shop_payment_values(order)` — `website_sale/controllers/main.py:1831`, `WebsiteSale._get_express_shop_payment_values` (v18) / `website_sale/controllers/cart.py:257`, `Cart._get_express_shop_payment_values` (v19).
- `cart_update_json` devuelve `minor_amount`/`amount` para los botones express (Apple/Google
  Pay) — `website_sale/controllers/main.py:920-923`, `WebsiteSale.cart_update_json`.
- `/shop/express_checkout/shipping_address` (json) — `website_sale/controllers/main.py:1877-1881`, `WebsiteSale.express_checkout_shipping_address_compute_taxes`
  (`express_checkout_shipping_address_compute_taxes`).
- Delivery express: `website_sale/controllers/delivery.py:147-148`, `class Delivery` (`express_checkout_process_delivery_address`),
  `_get_delivery_methods_express_checkout` `:233`.
