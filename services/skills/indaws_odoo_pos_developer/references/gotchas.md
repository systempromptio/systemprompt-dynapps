# POS — GOTCHAS (los que rompen en silencio)

> Citas `fichero:linea`: el **simbolo manda sobre el numero** (anclados a snapshot v18/v19,
> pueden desplazarse al parchear la fuente). Lineas reverificadas contra fuente 2026-07.

1. **Declara tu glob en el bundle, NO fichero a fichero** — tu modulo pos_* DEBE
   anadir su glob al bundle en SU manifest: `'assets': {'point_of_sale._assets_pos':
   ['mi_modulo/static/src/**/*']}` (patron de `pos_discount`/`pos_hr`, verificado v19).
   El glob de `point_of_sale/__manifest__.py:129` solo incluye los ficheros DEL PROPIO
   `point_of_sale`, NO los de terceros: si no declaras el glob, tu JS NO carga.
   Lo que NO debes hacer es listar los ficheros uno a uno (rompe orden y duplica).
   [Verificado en runtime v19: sin la clave `assets` el JS no entra en el bundle;
   con el glob, si.]
2. **Modelo JS nuevo = DOS sitios** — Python: anadirlo a `_load_pos_data_models`
   (`point_of_sale/models/pos_session.py:138`, `PosSession._load_pos_data_models` v18). JS: `static pythonModel = "x.y"`
   (`point_of_sale/static/src/app/models/product_product.js:16`, `ProductProduct`) + `registry.category("pos_available_models")
   .add(ProductProduct.pythonModel, ProductProduct)` (`point_of_sale/static/src/app/models/product_product.js:301`).
   Si falta cualquiera, el modelo no existe en el cliente y falla en silencio.
3. **`_load_pos_data_fields` FILTRA** — solo viajan al navegador los campos que
   devuelve (`point_of_sale/models/pos_session.py:150`, `PosSession._load_pos_data_fields` v18 / `:151` v19). Anadir un campo Python no basta;
   hay que parchear el metodo del modelo. Por eso "el campo existe pero es undefined
   en JS".
4. **AccessError se traga en silencio** — en la carga, si el usuario POS no tiene
   acceso a un modelo, `point_of_sale/models/pos_session.py:182`, `PosSession.load_data` (v18) captura `AccessError` y devuelve
   `{'data': [], ...}` SIN error visible. Sintoma: lista vacia inexplicable. Revisa
   ir.model.access / record rules del grupo POS.
5. **Firmas v19 != v18** — v19 renombra/cambia argumentos:
   `_load_pos_data_search_read(data, config)` sustituye a `_load_pos_data(data)`,
   `_load_pos_data_fields(config)` (objeto, no `config_id`),
   `_load_pos_data_domain(data, config)`, `_load_pos_data_models(config)`
   (`point_of_sale/models/pos_session.py:139-159`, `PosSession._load_pos_data_models` v19). Un override v18 portado a v19 sin ajustar
   la firma se ignora o peta. Ver `references/data-loading.md`.
6. **screens: registry cambia en v19** — v18 `registry.category("pos_screens")
   .add("X", Class)` (`point_of_sale/static/src/app/screens/product_screen/product_screen.js:541`, `ProductScreen`) + `pos.showScreen(name)`.
   v19 `registry.category("pos_pages").add("X", {component, ...})`
   (`point_of_sale/static/src/app/screens/product_screen/product_screen.js:450`, `ProductScreen.getProductsByCategory`) + router nuevo (`point_of_sale/static/src/app/services/pos_router_service.js:51`, `PosRouter.page`).
   Mismo registry para ambos rompe.
7. **PosStore es un servicio** — extiende con `patch(PosStore.prototype, {...})`.
   Para anadir una DEPENDENCIA de servicio nueva hay que parchear tambien
   `PosStore.serviceDependencies` (definida en `point_of_sale/static/src/app/store/pos_store.js:57`, `PosStore` v18, usada
   por `posService.dependencies` `:2397`); si no, `useService("...")` da undefined en el setup.
8. **`usePos()` envuelve en useState** — `usePos()` es `useState(useService("pos"))`
   (`point_of_sale/static/src/app/store/pos_hook.js:8`, `usePos` v18). Llamar `useService("pos")` a pelo pierde reactividad
   en la plantilla.
9. **tech_name del pago debe CASAR** — el primer arg de
   `register_payment_method("tech_name", Clase)`
   (`point_of_sale/static/src/app/store/pos_store.js:2395`, `register_payment_method` v18 / `point_of_sale/static/src/app/services/pos_store.js:3112`, `register_payment_method`, v19) DEBE ser
   identico al valor que devuelve `_get_payment_terminal_selection`
   (`point_of_sale/models/pos_payment_method.py:11`, `PosPaymentMethod._get_payment_terminal_selection`). Si no casan, el terminal no se instancia y no hay
   error claro.
10. **Path del PaymentInterface se mueve en v19** — v18
    `@point_of_sale/app/payment/payment_interface`; v19
    `@point_of_sale/app/utils/payment/payment_interface`. Import viejo en v19 = modulo
    no encontrado.
11. **OrderReceipt se personaliza por herencia** — `OrderReceipt` es un Component
    (`point_of_sale/static/src/app/screens/receipt_screen/receipt/order_receipt.js:8`, `OrderReceipt`, template
    `point_of_sale.OrderReceipt`). NO lo reescribas: `t-inherit` de la plantilla.
12. **Offline = IndexedDB** — los modelos con `databaseTable`/`dynamicModels`
    (`point_of_sale/static/src/app/models/data_service_options.js:4,37`, `DataServiceOptions`) se persisten en IndexedDB para trabajar
    sin red. Si tu modelo nuevo debe sobrevivir offline, declaralo ahi; si no, se
    pierde al recargar sin conexion.
13. **self-order es OTRA app OWL** — `pos_self_order` tiene su propio bundle, router
    y `static/src/app`. Parchear el POS clasico NO afecta al self-order (y viceversa).
14. **v19 mueve/absorbe modulos enteros** — no es solo cambio de imports. `hw_*` (CE)
    desaparece -> `iot_base`/`iot_drivers`; `pos_preparation_display` (EE) se ABSORBE
    en `pos_enterprise` (ya no hay modulo base que heredar); `pos_viva_wallet` ->
    `pos_viva_com`; Epson/Six/PayTM retirados de CE (Epson va por IoT). Un override
    v18 contra un modulo que en v19 ya no existe falla al instalar. Detalle completo
    en `references/version-diffs.md` seccion "Cambios de MODULOS v18 -> v19".
15. **v19 renombro la API JS de order/orderline a camelCase** — `set_quantity` ->
    `setQuantity`, `can_be_merged_with` -> `canBeMergedWith`, etc. Un `patch()` de
    v18 sobre el nombre snake_case en v19 crea un metodo MUERTO y el real corre sin
    parchear: fallo silencioso. Ademas los getters de PRECIO (`get_all_prices`,
    `get_display_price`...) SALIERON del modelo orderline en v19: viven en
    `app/models/accounting/pos_order_line_accounting.js` como **getters** (`displayPrice`,
    `priceIncl`, `priceExcl`…), y `get_all_prices` **no tiene equivalente por nombre** — 0 ficheros
    en todo `point_of_sale/static/src` de v19 (verificado 2026-08-07). Tabla de mapeo v18→v19 en
    `references/orderlines-and-order-flow.md`.
