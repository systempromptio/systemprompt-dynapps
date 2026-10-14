# Shop listing, product page & variants (`website_sale`)

Rutas de origen (v18 salvo nota): `$ODOO_SRC/odoo18/addons/website_sale/`. Diffs marcados v17/v19.

## Listado del shop y categorías

`WebsiteSale.shop(page, category, search, min_price, max_price, ppg, **post)` —
`website_sale/controllers/main.py:228-465`, `WebsiteSale.shop` (v18). Sirve `/shop` y `/shop/page/<int>`,
`/shop/category/<model("product.public.category"):category>`. Productos publicados se
filtran por `website_published` + dominio de website; el render usa la plantilla
`website_sale.products`. El `sitemap` de productos es la función `sitemap_products`
(`main.py` cabecera). Para añadir un filtro/orden, sobreescribir `shop` re-decorando
`@route(...)` o, mejor, los hooks `_get_search_domain` / `_shop_get_query_url_kwargs`.

## Página de producto

`WebsiteSale.product(product, category, search, **kwargs)` — `website_sale/controllers/main.py:465-497`, `WebsiteSale.product` (v18),
ruta `/shop/<model("product.template"):product>` (`auth='public'`, `website=True`,
`readonly=True`). Pasa `main_object=product` al qcontext (habilita editor/SEO). El precio
y la disponibilidad mostrados al cargar y al cambiar variante salen de `_get_combination_info`.

## `_get_combination_info` — el corazón de la variante en web

`product.template._get_combination_info(...)` — `website_sale/models/product_template.py:342-460`, `ProductTemplate._get_combination_info` (v18).

Firma por versión (VERIFICAR siempre):
- **v17** (`website_sale/models/product_template.py:338`, `ProductTemplate._is_add_to_cart_possible`) y **v18** (`:342`):
  `_get_combination_info(combination=False, product_id=False, add_qty=1.0, parent_combination=False, only_template=False)`
- **v19** (`website_sale/models/product_template.py:465`, `ProductTemplate._get_additionnal_combination_info`): añade parámetro **`uom_id`** (=False).

Devuelve un dict con, entre otros: `product_id` (variante que casa la combinación, o
`False` si no existe), `product_template_id`, `display_name`, `price`, `price_extra`,
`list_price`, `has_discounted_price`, `is_combination_possible`, `parent_exclusions`,
`display_image`. Resuelve la variante:
- Si hay `product_id` y la `combination` no está representada en él → `_get_variant_for_combination(combination)` (`:405-408`).
- Si no hay `product_id` ni `combination` ni `only_template` → `_get_first_possible_combination(parent_combination)` (`:398-399`).

El precio se calcula bajo `_get_product_price_context(combination)` (`:421-422`), por lo que
respeta pricelist y `add_qty`.

## Punto de extensión correcto: `_get_additionnal_combination_info`

Para AÑADIR claves al dict (stock, badges, datos custom) **no** sobreescribas
`_get_combination_info`; sobreescribe `_get_additionnal_combination_info`, que es lo que
hace el core. `website_sale/models/product_template.py:462`, `ProductTemplate._get_additionnal_combination_info` (v18):

```python
def _get_additionnal_combination_info(self, product_or_template, quantity, date, website):
    res = super()._get_additionnal_combination_info(product_or_template, quantity, date, website)
    res['my_flag'] = product_or_template.x_my_flag
    return res
```

- v18: `(self, product_or_template, quantity, date, website)` — `:462`.
- v19: `(self, product_or_template, quantity, uom, date, website)` — `website_sale/models/product_template.py:580`, `ProductTemplate._apply_taxes_to_price` (añade `uom`).

`website_sale_stock` lo usa para inyectar `free_qty`, `cart_qty`, `available_threshold`,
`show_availability`, `out_of_stock_message`, `allow_out_of_stock_order`
(`website_sale_stock/models/product_template.py:25-56`, `ProductTemplate._get_additionnal_combination_info`).

## Ruta AJAX de la variante

`/website_sale/get_combination_info` — `website_sale_renting/controllers/variant.py:10-18`, `WebsiteSaleRentingVariantController` (v18, json, POST,
`auth='public'`, `website=True`). Recibe `product_template_id`, `product_id`,
`combination`, `add_qty`, `parent_combination`, `**kw` y llama
`product_template._get_combination_info(...)`. Es lo que dispara el JS al cambiar un
atributo en la página de producto. `/sale/create_product_variant` (`website_sale/controllers/variant.py:49`, `WebsiteSaleVariantController.create_product_variant`) crea
la variante dinámica on-the-fly.

## Tipos de atributo (de `product`)

- **`always` (Instantly)**: crea variantes reales (`product.product`) para cada combinación.
- **`dynamic`**: la variante se crea bajo demanda (`create_product_variant`).
- **`no_variant`**: no crea variante; el valor viaja como
  `no_variant_attribute_value_ids` y se guarda en la línea de carrito
  (`product_no_variant_attribute_value_ids`). `_cart_find_product_line` SÍ lo compara.
- **custom value** (`is_custom`): texto libre del cliente; viaja como
  `product_custom_attribute_values` (JSON) y SIEMPRE crea línea nueva (no se casa en
  `_cart_find_product_line`, que lo fuerza a `False`). Ver `website_sale/models/sale_order.py:434`, `SaleOrder._cart_find_product_line` (v18).

## Diffs de versión resumidos

| Aspecto | v17 | v18 | v19 |
|---|---|---|---|
| `_get_combination_info` firma | sin `uom_id` | sin `uom_id` | `+ uom_id` |
| `_get_additionnal_combination_info` | `(p_or_t, qty, date, website)` | igual | `+ uom` |
| Controllers | `main.py` monolítico + `variant.py`,`delivery.py` | split: `payment.py`,`sale.py`,`product_configurator.py`,`reorder.py`,`combo_configurator.py` | `+ cart.py`, `product_feed.py` |
