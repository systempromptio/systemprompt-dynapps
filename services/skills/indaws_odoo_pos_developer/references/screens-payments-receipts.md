# Screens, popups, pagos, recibos, restaurant y self-order

> Citas verificadas contra v18/v19. Resolver fuente via
> `references/shared/odoo-source.md`.

## Screens

Una "screen" es el componente principal a pantalla completa (ProductScreen,
PaymentScreen, ReceiptScreen, TicketScreen...). El **mecanismo de registro cambia en
v19**.

### v18: registry `pos_screens` + showScreen

```js
import { registry } from "@web/core/registry";
registry.category("pos_screens").add("MiScreen", MiScreen);
// cf. screens/product_screen/product_screen.js:541
```

Se navega con `pos.showScreen("MiScreen", props)` (`point_of_sale/static/src/app/store/pos_store.js:1615`, `PosStore.showScreen`,
`:1614` hace `registry.category("pos_screens").get(name)`).

### v19: registry `pos_pages` + router

```js
registry.category("pos_pages").add("MiScreen", {
    component: MiScreen,
    // ...params de ruta
});
// cf. screens/product_screen/product_screen.js:450
```

Hay un router dedicado: `point_of_sale/static/src/app/services/pos_router_service.js:51`, `PosRouter.page` resuelve
`registry.category("pos_pages").get(this.state.current)`, y
`point_of_sale/static/src/app/services/pos_store.js:206`, `PosStore.navigate` lo usa para navegar. NO mezcles `pos_screens` con
`pos_pages`: usar el registry equivocado para la version hace que la screen no exista.

### Extender una screen estandar

`patch()` para la logica + `t-inherit` para la plantilla. NO reescribas la screen.

```js
import { patch } from "@web/core/utils/patch";
import { ProductScreen } from "@point_of_sale/app/screens/product_screen/product_screen";
patch(ProductScreen.prototype, { miMetodo() { /* ... */ } });
```

```xml
<t t-name="mi_modulo.ProductScreenExt" t-inherit="point_of_sale.ProductScreen"
   t-inherit-mode="extension">
    <xpath expr="..." position="inside">...</xpath>
</t>
```

## Popups / dialogs

Los popups NO usan registry. Se abren via el servicio `dialog` con helpers de
`store/make_awaitable_dialog.js`:

- `makeAwaitable(dialog, Component, props, options)` (`:3`) — abre un popup propio y
  espera su resultado (promesa).
- `ask(dialog, props, options, comp = ConfirmationDialog)` (`:38`) — confirmacion
  rapida si/no.

```js
import { makeAwaitable } from "@point_of_sale/app/store/make_awaitable_dialog";
const result = await makeAwaitable(this.dialog, MiPopup, { title: _t("...") });
```

En v19 los popups viven en `app/components/popups/`. Inyecta `this.dialog =
useService("dialog")` en el setup.

## ControlButtons

Los botones de la barra de acciones (`ControlButtons`,
`point_of_sale/static/src/app/screens/product_screen/control_buttons/control_buttons.js:11`, `ControlButtons`) son un Component, NO
un registry. Para anadir un boton: `patch(ControlButtons.prototype, {...})` +
`t-inherit` de su plantilla insertando tu boton. (En modulos como `pos_restaurant`
hay control_buttons propios como ejemplo.)

## Terminales de pago (PaymentInterface)

Integrar un datafono pasa por TRES piezas que deben encajar.

### 1. JS — implementar PaymentInterface

Path **cambia en v19**:
- v18: `@point_of_sale/app/payment/payment_interface`
  (`static/src/app/payment/payment_interface.js`)
- v19: `@point_of_sale/app/utils/payment/payment_interface`
  (`static/src/app/utils/payment/payment_interface.js`)

`export class PaymentInterface` (`:19` en ambas). Metodos a implementar:
`send_payment_request(uuid)` (`point_of_sale/static/src/app/payment/payment_interface.js:66`, `PaymentInterface.send_payment_request` v18),
`send_payment_cancel`, `send_payment_reversal`, `close`.

```js
import { PaymentInterface } from "@point_of_sale/app/payment/payment_interface"; // v18
export class PaymentMiTerminal extends PaymentInterface {
    async send_payment_request(uuid) { /* hablar con el datafono */ return true; }
    async send_payment_cancel(order, uuid) { /* ... */ }
}
```

### 2. JS — registrar

```js
import { register_payment_method } from "@point_of_sale/app/store/pos_store"; // v18
register_payment_method("mi_terminal", PaymentMiTerminal);
// def: store/pos_store.js:2391 (v18) / services/pos_store.js:3100 (v19)
```

### 3. Python — exponer el tech_name y los campos

```python
class PosPaymentMethod(models.Model):
    _inherit = "pos.payment.method"

    def _get_payment_terminal_selection(self):     # pos_payment_method.py:11
        return super()._get_payment_terminal_selection() + [("mi_terminal", "Mi Terminal")]
```

El primer arg de `register_payment_method(...)` DEBE ser identico al value del
selector (`"mi_terminal"`). Si no casan, el terminal no se instancia y no hay error
claro. Los campos extra de configuracion del terminal (API key, endpoint...) viajan
al cliente via `_load_pos_data_fields` de `pos.payment.method` (ver `data-loading.md`).

## Recibos (OrderReceipt)

`OrderReceipt` es un Component
(`point_of_sale/static/src/app/screens/receipt_screen/receipt/order_receipt.js:7`, `OrderReceipt`,
`static template = "point_of_sale.OrderReceipt"` `:8`). Personalizar SIEMPRE por
herencia de plantilla, nunca reescribiendo:

```xml
<t t-name="mi_modulo.OrderReceiptExt" t-inherit="point_of_sale.OrderReceipt"
   t-inherit-mode="extension">
    <xpath expr="//div[hasclass('pos-receipt')]" position="inside">
        <div t-esc="props.data.mi_dato"/>
    </xpath>
</t>
```

## Restaurant (alto nivel)

`pos_restaurant` anade floors/tables, split de cuenta y propinas. Carpetas de
referencia (v18): `floor_screen`, `split_bill_screen`, `tip_screen`,
`tip_receipt`, `control_buttons`, `bill_screen`, `models`. Es buen patron canonico
de como un modulo `pos_*` anade modelos (`restaurant.floor`, `restaurant.table`),
screens nuevas y botones. Extiende el POS clasico via los mecanismos de arriba.

## Self-order (app SEPARADA)

`pos_self_order` es **otra app OWL** con su propio bundle, router y
`static/src/app` (estructura `app/backend/overrides`). Parchear el POS clasico NO
afecta al self-order y viceversa. Si el cambio debe verse en ambos, hay que tocar los
dos. No asumas que un `patch(PosStore.prototype)` llega al self-order.
