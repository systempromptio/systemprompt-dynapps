# Módulos `website_sale_*` (extras)

Rutas (v18): `$ODOO_SRC/odoo18/addons/website_sale_{wishlist,comparison,loyalty,stock}/`.

## `website_sale_stock` — disponibilidad y bloqueo de qty

**El stock en web NO reserva**: sólo muestra disponibilidad y limita/avisa al añadir.

Campos en `product.template` (`website_sale_stock/models/product_template.py:13-17`, `class ProductTemplate` — el módulo es `website_sale_stock`):
- `allow_out_of_stock_order` (default `True`) — si `False`, no deja comprar más que el stock.
- `available_threshold` (default 5.0), `show_availability`, `out_of_stock_message`.

Inyecta datos en la página de producto vía **`_get_additionnal_combination_info`**
(`website_sale/models/product_template.py:25-56`, `ProductTemplate._check_company_auto`): añade `free_qty` (= `website._get_product_available_qty(product)`),
`cart_qty` (`product._get_cart_qty(website)`), `available_threshold`, `show_availability`,
`out_of_stock_message`, `allow_out_of_stock_order`. Es el patrón a copiar para inyectar
datos propios.

Bloqueo de cantidad en el carrito — `_verify_updated_quantity` (`website_sale/models/sale_order.py:32-49`, `class SaleOrder`):
si el producto es `is_storable and not allow_out_of_stock_order`, calcula
`_get_cart_and_free_qty(product, line)` (`:67-83`) y si `available_qty < total_cart_qty`
recorta y setea `shop_warning` vía `_set_shop_warning_stock(...)`. La revisión global del
carrito (`:99-102`) marca warnings al entrar al checkout.

Para extender el stock web: sobreescribir `_verify_updated_quantity` (super + lógica) o
`_get_additionnal_combination_info`. Controllers extra en `controllers/`
(`website_sale.py`, `variant.py`, `reorder.py`).

## `website_sale_loyalty` — promociones y cupones

Hooks en `sale.order` (`models/sale_order.py`):
- `_cart_update(...)` override (`:174-185`): tras `super()._cart_update(...)` llama
  `_update_programs_and_rewards()` y `_auto_apply_rewards()`. Si se pone qty 0 a una línea
  de recompensa de descuento, fuerza el borrado. (En v19 el hook colgará del nuevo
  `_cart_add`/`_cart_update_line_quantity` — verificar.)
- `_allow_nominative_programs()` (`:195`): bloquea programas nominativos para el usuario
  público (`request.website.is_public_user()`).
- `_get_free_shipping_lines()` / `_get_non_delivery_lines()` override para excluir las
  líneas de envío gratis del cálculo.

Controllers (`controllers/main.py`) — todos heredan re-decorando `@route()`:
- `pricelist(promo, reward_id=None, **post)` (`:14-37`): aplica código promocional con
  `order._try_apply_code(promo)`; si `not_found` delega en `super().pricelist`. Guarda
  errores en `request.session['error_promo_code']`.
- `cart(**post)` (`:47`) y `shop_payment(**post)` (`:39`): re-aplican programas
  (`_update_programs_and_rewards` + `_auto_apply_rewards`) antes de delegar en `super()`.
- `/coupon/<string:code>` (`:58`) `activate_coupon` — guarda `pending_coupon_code` en
  sesión y `order._try_pending_coupon()`.
- `/shop/claimreward` (`:80`) `claim_reward(reward_id, code=None, **post)`.

`controllers/delivery.py` y `payment.py` ajustan recompensas de envío y validación.

## `website_sale_wishlist` — lista de deseos

Modelo `product.wishlist` (`website_sale_wishlist/models/product_wishlist.py:8`, `ProductWishlist`): `partner_id`, `product_id`,
`website_id`, con constraint único `(product_id, partner_id)`. `current()` (`:25`) devuelve
los wishes del partner actual (o de sesión si público). `_add_to_wishlist(pricelist_id,
currency_id, website_id, price, product_id, partner_id)` (`:43`) crea el registro.
Al loguear, los wishes de sesión se reasignan al partner (`:58-65`).

Controller (`controllers/main.py`):
- `/shop/wishlist/add` (json) `add_to_wishlist(product_id, **kw)` (`:10-37`): para usuario
  público usa `Wishlist.sudo()` y `partner_id=False`, guardando el id en
  `request.session['wishlist_ids']`. Precio vía `product._get_combination_info_variant()['price']`.
- `/shop/wishlist` (http) `get_wishlist(count=False, **kw)` (`:39`).
- `/shop/wishlist/remove/<int:wish_id>` (json) (`:55`).

Para añadir un wish al carrito se reutiliza `/shop/cart/update_json` (ver cart-checkout.md).

## `website_sale_comparison` — comparador de productos

Vive en cookie (no en `sale.order`). Controller (`controllers/main.py`):
- `/shop/compare` (http) `product_compare(**post)` (`:10`).
- `/shop/get_product_data` (json) `get_product_data(product_ids, cookies=None)` (`:25`):
  devuelve los datos de los productos a comparar; el JS mantiene los ids en cookie.

Modelos: `product.attribute.category` (agrupa atributos para la tabla comparativa),
`product.product._prepare_categories_for_display()` (`website_sale/models/product_product.py:11`, `ProductProduct`),
extensiones de `product.attribute` y `product.template.attribute.line`.

## Notas de dependencia

Declarar en `__manifest__.py` el módulo extra concreto que se extiende
(`website_sale_stock`, `website_sale_loyalty`, …), no sólo `website_sale`. El override de
`_cart_update`/`_verify_updated_quantity` debe llamar siempre a `super()` para no romper la
cadena (stock + loyalty se encadenan sobre el mismo método).
