# Orderlines y flujo de pedido (el núcleo de vender)

El objeto que tocas en casi cualquier desarrollo POS: **`pos.order`** y
**`pos.order.line`**. Aquí: añadir/mutar líneas por código, extender el cálculo de
línea, combos, notas/atributos, y la **rotura crítica v18→v19** (la API JS pasó a
camelCase y el cálculo de precio salió del modelo orderline).

> Verificado contra `point_of_sale/static/src/app/models/` de la fuente v18 y v19
> (2026-07). Rutas de modelo: v18 `app/models/`, v19 `app/models/` (mismo dir; el
> que se movió a `app/services/` es el store, no los modelos).

## Modelos (Python + JS)

| Capa | v18 | v19 |
|------|-----|-----|
| Python | `pos.order`, `pos.order.line` | idem |
| JS Order | `app/models/pos_order.js` | idem |
| JS Orderline | `app/models/pos_order_line.js` | idem |
| Store (añadir línea) | `app/store/pos_store.js` | `app/services/pos_store.js` |

En JS son modelos `related_models` (ver `architecture-and-store.md`): se extienden con
`patch(PosOrderline.prototype, {...})`, NO reescribiendo la clase.

## Añadir una línea por código

Método del store (misma firma en v18 y v19, verificado):
```js
// pos_store.js: async addLineToCurrentOrder(vals, opts = {}, configure = true)
await this.pos.addLineToCurrentOrder(
    { product_id: product, qty: 2 },   // vals
    {},                                // opts
    false                              // configure: false = NO abrir el configurador
);
```
Forma de `vals` (lo que arma `addLineToOrder`, `point_of_sale/static/src/app/store/pos_store.js:713`, `PosStore` v18):
- `product_id`: **objeto** producto o **id numérico** (el store lo resuelve a objeto).
- `qty`: por defecto 1.
- `price_unit`: si lo pasas, `price_type` se marca `"manual"` **y la línea NO se
  fusiona** con otra igual (`if ("price_unit" in vals) merge = false`). Sin él,
  `price_type` = `"original"` y sí puede fusionarse.
- `discount`: % de descuento.
- `tax_ids`: se arma con comandos tipo `["link", tax]` (formato x2many del cliente),
  por defecto `product.taxes_id`.
- `configure=true` abre el flujo de configurador/combos/lotes; pásalo `false` para
  añadir "en seco".

> Para el patrón "un botón que añade un producto", copia cómo lo hace
> `ProductScreen`/`Numpad` en el core, no reinventes la resolución de `vals`.

## Leer y mutar una línea

Métodos clave (nombres **v18**; para v19 aplica el rename de abajo):
`get_quantity()`, `set_quantity(q, keep_price)`, `get_discount()`,
`set_discount(d)`, `get_unit_price()`, `set_unit_price(p)`, `get_product()`,
`can_be_merged_with(other)`, `getDisplayData()` (lo que pinta el ticket de línea),
`getPriceString()`, `get_customer_note()`/`set_customer_note()`,
`get_price_extra()`/`set_price_extra()`.

## ⚠️ ROTURA CRÍTICA v18 → v19: camelCase + precio fuera de orderline

**1. Toda la API JS de order/orderline se renombró a camelCase en v19.** Un
`patch()` de v18 sobre `set_quantity` en v19 crea un método muerto y el real
(`setQuantity`) corre SIN parchear — fallo silencioso, sin error.

| v18 (snake_case) | v19 (camelCase) |
|------------------|-----------------|
| `set_quantity` | `setQuantity` |
| `set_quantity_by_lot` | `setQuantityByLot` |
| `get_quantity` / `get_quantity_str` | `getQuantity` / `getQuantityStr` |
| `set_discount` / `get_discount` | `setDiscount` / `getDiscount` |
| `get_discount_str` | `getDiscountStr` |
| `set_unit_price` | `setUnitPrice` |
| `get_product` | `getProduct` |
| `get_unit` | `getUnit` |
| `can_be_merged_with` | `canBeMergedWith` |
| `set_customer_note` / `get_customer_note` | `setCustomerNote` / `getCustomerNote` |
| `set_price_extra` / `get_price_extra` | `setPriceExtra`(*) / `getPriceExtra` |
| `set_full_product_name` / `get_full_product_name` | `setFullProductName` / `getFullProductName` |
| `get_valid_lots` / `has_valid_product_lot` | `getValidLots` / `hasValidProductLot` |
| `is_pos_groupable` | `isPosGroupable` |
| `get_total_cost` | `getTotalCost` |
| `set_product_lot` | (revisar; el flujo de lotes cambió) |

Order (v19, ejemplos): `getOrderlines()`, `getSelectedOrderline()`,
`selectOrderline()`, `removeOrderline()`, `getLastOrderline()`,
`addPaymentline()`/`removePaymentline()`. Confirma el nombre exacto en
`app/models/pos_order.js` de la v19 antes de parchear.

**2. El cálculo de precio SALIÓ del modelo orderline en v19.** En v18 la línea
tiene `get_all_prices()`, `get_display_price()`, `get_unit_display_price()`,
`get_price_with_tax()`, `get_price_without_tax()`, `get_base_price()`,
`get_taxed_lst_unit_price()`, `get_tax_details()`… En v19 **esos getters ya NO
están en `pos_order_line.js`**: la lógica de impuestos/precio se movió a
`app/models/accounting/`, que en v19 contiene exactamente tres ficheros:
`pos_order_accounting.js`, `pos_order_line_accounting.js`, `product_template_accounting.js`
(verificado 2026-08-07; **no** hay `account_tax.js` ahí — ese fichero existía en `app/models/` de
v18 y desapareció).

Y no es un renombrado camelCase: **son GETTERS con otros nombres**, en
`PosOrderlineAccounting extends Base` (`app/models/accounting/pos_order_line_accounting.js`):

| v18 (método) | v19 (getter en la capa accounting) |
|---|---|
| `get_display_price()` | `get displayPrice()` (`:43`) |
| `get_unit_display_price()` | `get displayPriceUnit()` (`:67`), `displayPriceUnitIncl` (`:72`), `displayPriceUnitExcl` (`:75`) |
| `get_price_with_tax()` | `get priceIncl()` (`:84`) |
| `get_price_without_tax()` | `get priceExcl()` (`:87`) |
| `get_all_prices()` | **no existe con ese nombre**: `grep -rl get_all_prices` sobre `point_of_sale/static/src` de v19 da **0 ficheros**. Los precios se leen por getter individual |

Los cálculos delegan en `accountTaxHelpers` de `@account/helpers/account_tax`, no en el POS.

**3. v19 amplió mucho los modelos JS cargados.** Nuevos en `app/models/` v19:
`product_template.js`, `product_pricelist.js`, `product_template_attribute_line/value.js`,
`uom_uom.js`, `res_currency.js`, `decimal_precision.js`, `account_fiscal_position.js`,
`account_cash_rounding.js`, `pos_preset.js`. Datos que en v18 venían derivados o
aplanados en v19 son modelos JS de primera clase; revisa si tu lógica los necesita.

## Extender el cálculo/pintado de una línea

```js
import { patch } from "@web/core/utils/patch";
import { PosOrderline } from "@point_of_sale/app/models/pos_order_line";

patch(PosOrderline.prototype, {
    // v18: getDisplayData()  |  v19: mismo nombre (getDisplayData sobrevive)
    getDisplayData() {
        return { ...super.getDisplayData(), miCampo: this.mi_campo };
    },
});
```
`getDisplayData()` es el punto canónico para inyectar datos de línea en ticket y
UI (existe en v18 y v19). El precio en ticket sale de `getPriceString()` (v18);
en v19 confírmalo (parte del refactor de precio).

## Combos

Una línea de combo tiene `combo_parent_id` y `combo_line_ids`. Útiles:
`getAllLinesInCombo()` (v18 y v19), `isPartOfCombo()`. Al cambiar la cantidad del
padre, v19 reparte a los hijos (`comboLine.setQuantity(...)`,
`point_of_sale/static/src/app/models/pos_order_line.js:306`, `PosOrderline.set_quantity_by_lot` v19). Para combos custom, estudia `pos_restaurant` y el core
`ProductScreen`, no montes el reparto a mano.

## Atributos de producto y notas

- Atributos configurables: v19 los modela explícito
  (`product_template_attribute_value.js`, `product_attribute_custom_value.js`).
  El valor elegido viaja en la línea; para leerlo, mira cómo el core lo resuelve en
  `getDisplayData`.
- Nota de cliente por línea: `set_customer_note`/`get_customer_note` (v18) →
  `setCustomerNote`/`getCustomerNote` (v19).
- Precio extra por variante: `price_extra` (`set_price_extra`/`get_price_extra`).

## Guardar / sincronizar

La línea persiste con el pedido vía el `data` service (ver GOTCHA 12 offline). No
llames a ORM directamente desde la línea; el flujo de guardado del pedido
(`pos.order`) es quien sincroniza. Mutar una línea y esperar persistencia inmediata
sin pasar por ese flujo es un error común.
