---
name: indaws-odoo-pos-developer
version: 0.1.22
effort: high
model: opus
description: >-
  Desarrolla y extiende el Point of Sale de Odoo (v17-v19): app OWL independiente en
  /pos/ui. Carga de datos (pos.load.mixin, _load_pos_data*), modelos JS, PosStore/usePos,
  screens (pos_screens v18 / pos_pages v19), terminales de pago (PaymentInterface),
  recibos, restaurant, self-order. NOT el web client backend (→ odoo-owl-component-builder),
  NOT la tienda web (→ odoo-ecommerce-developer).
  Triggers: "punto de venta", "POS Odoo", "pantalla POS", "boton POS", "terminal de pago
  POS", "recibo POS", "cargar datos al POS", "PosStore", "pos_screens", "pos_pages",
  "extender POS", "self order".
allowed-tools: [Read, Write, Edit, Bash, Glob, Grep]
---

# Odoo POS Developer

> Convenciones de codigo: ver `references/shared/coding-standards.md`.
> Fuente Odoo a verificar: resolver via `references/shared/odoo-source.md`
> (NO asumir `$ODOO_SRC`).
> Citas `fichero:linea`: el **simbolo manda sobre el numero**. Los numeros se
> anclaron a la fuente v18/v19 de un snapshot y pueden desplazarse unas lineas al
> actualizar el fuente (parches). Si el numero no cae exacto, busca el simbolo
> citado (funcion/metodo) en ese fichero. Lineas reverificadas contra fuente 2026-07.

Desarrolla y extiende el **Point of Sale** de Odoo (v17-v19). El POS es una **app
OWL independiente** montada en `/pos/ui` (NO es el web client backend ni la tienda
web). Hay tres superficies de extension: **Python** (que datos viajan al navegador),
**modelos JS** (`related_models`) y **componentes/screens OWL**. Casi todo se hace
con `patch()` + `registry`, NO reescribiendo piezas.

## Workflow

1. **Confirmar version (v17/v18/v19).** Es DECISIVO: v19 reorganiza rutas de import
   y cambia firmas Python de carga de datos y el registry de screens. Antes de tocar
   nada, consulta `references/version-diffs.md` (cheat-sheet de paths v18<->v19 +
   los 4 patrones canonicos con variante por version).
2. **Localizar la superficie.** Decide si el cambio es de datos (Python), de modelo
   JS, o de UI (screen/popup/boton/recibo). Para arquitectura, bundles, PosStore y
   modelos JS carga `references/architecture-and-store.md`. **Antes de escribir**,
   busca en `references/module-catalog.md` qué módulo oficial ya hace algo parecido
   y cópialo como ejemplar (heredar > inventar): 106 módulos POS v18 (CE+EE) mapeados
   por objetivo.
3. **Si toca datos -> `references/data-loading.md`.** Eje critico: anadir un campo
   o un modelo nuevo al POS pasa por `pos.load.mixin` y `_load_pos_data*`. Las
   FIRMAS cambian en v19. Un modelo nuevo se registra en DOS sitios (Python + JS).
   Si tocas la VENTA (lineas de pedido, precio, cantidad, descuento, combos) ->
   `references/orderlines-and-order-flow.md` (ojo rename camelCase v19).
4. **Si toca UI/pagos/recibos -> `references/screens-payments-receipts.md`.** Screens
   (`pos_screens` v18 / `pos_pages` v19), popups via servicio `dialog`, terminales
   de pago (`PaymentInterface` + `register_payment_method`), recibos (`OrderReceipt`
   por `t-inherit`), y a alto nivel restaurant y self-order.
5. **Extender, no reescribir.** `patch(Clase.prototype, {...})` para JS y `t-inherit`
   para plantillas. Casi siempre extiendes un modulo existente (sin scaffolding). Si NO
   existe: [delega] el esqueleto en `odoo-module-creator` (`depends: ['point_of_sale']`,
   glob en `_assets_pos` via `assets` — gotcha 1; contrato base-dominio en
   `references/shared/base-domain-delegation.md`). Tests -> `odoo-test-generator`.

## GOTCHAS (las 15 completas, con anclas file:line, en `references/gotchas.md` — léelas antes de dar por buena una extensión)

1. **Assets por glob, no fichero a fichero** — tu módulo declara `'assets':
   {'point_of_sale._assets_pos': ['tu_modulo/static/src/**/*']}`; el glob del core no
   incluye terceros → sin la clave tu JS no carga (verificado runtime v19).
2. **Modelo JS nuevo = DOS sitios** (Python `_load_pos_data_models` + JS `pythonModel`
   en `pos_available_models`) o no existe, en silencio.
3. **`_load_pos_data_fields` FILTRA** — un campo Python no viaja al JS si no parcheas ese método.
4. **AccessError se traga en silencio** en la carga → lista vacía inexplicable (revisa ACL/rules del grupo POS).
5. **Firmas de carga v19 != v18** (`_load_pos_data_search_read/_fields/_domain/_models`): override portado sin ajustar firma se ignora o peta.
6. **Registry de screens v19** (`pos_pages` + router) != v18 (`pos_screens` + `showScreen`).
7. **PosStore es servicio**: `patch(PosStore.prototype)` y, si añades dep, también `PosStore.serviceDependencies`.
8. **`usePos()` = `useState(useService("pos"))`**: `useService("pos")` a pelo pierde reactividad.
9. **tech_name del pago debe CASAR** entre `register_payment_method` y `_get_payment_terminal_selection`.
10. **PaymentInterface cambia de path en v19** (`app/utils/payment/...`).
11. **OrderReceipt por `t-inherit`**, nunca reescribir.
12. **Offline = IndexedDB** (`databaseTable`/`dynamicModels`) o se pierde al recargar sin red.
13. **self-order es OTRA app OWL** (bundle/router propios): parchear el POS clásico no le afecta.
14. **v19 mueve/absorbe módulos enteros** (`hw_*`→iot, `pos_preparation_display`→`pos_enterprise`…): override a módulo inexistente falla al instalar.
15. **v19 renombró la API JS de order/orderline a camelCase** + los getters de precio salieron del modelo: `patch()` snake_case = método muerto silencioso.

## NEVER

- NUNCA uses esta skill para el web client backend (-> `odoo-owl-component-builder`)
  ni para la tienda web/eCommerce (-> `odoo-ecommerce-developer`).
- NUNCA listes los assets POS fichero a fichero: anade UN glob de tu `static/src/**/*`
  al bundle `point_of_sale._assets_pos` en tu manifest (sin la clave `assets` tu JS no
  carga; el glob del core NO incluye modulos de terceros).
- NUNCA registres un modelo JS en un solo sitio: Python + JS o no existe.
- NUNCA portes overrides de carga de datos de v18 a v19 sin ajustar las firmas.
- NUNCA reescribas screens/recibos estandar: `patch()` + `t-inherit`.
- NUNCA codigo, identificadores ni plantillas (`t-name`, tech_name) en otro idioma
  que no sea ingles; solo los strings de cara al usuario se traducen.

## ALWAYS

- SIEMPRE confirma la version antes de elegir paths y firmas.
- SIEMPRE que anadas un campo al POS, parchea `_load_pos_data_fields` del modelo.
- SIEMPRE extiende `PosStore` con `patch(PosStore.prototype)` y, si anades dep de
  servicio, tambien `PosStore.serviceDependencies`.
- SIEMPRE haz que el tech_name JS del pago case con el selector Python.
- SIEMPRE usa `usePos()` en componentes para conservar reactividad.
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-pos-developer · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

- **odoo-module-creator** (BASE, solo si el modulo no existe) — scaffold del esqueleto;
  `depends: ['point_of_sale']` + glob en `_assets_pos` via `assets`. Contrato:
  `references/shared/base-domain-delegation.md`.
- **odoo-owl-component-builder** — patrones OWL genericos (hooks, registry, patch,
  plantillas) que el POS comparte con el web client.
- **odoo-version-api-reference** — firmas exactas de API JS/Python por version.
- **odoo-test-generator** — tests JS (Hoot), tours POS y HttpCase.

## Consumed by

- **odoo-module-creator** — cuando un modulo nuevo extiende el POS (`depends` incluye
  `point_of_sale`), enruta aqui la parte de dominio POS.

## REFERENCES

| Fichero | Contenido | Cargar cuando |
|---------|-----------|---------------|
| [references/architecture-and-store.md](references/architecture-and-store.md) | App OWL (`/pos/ui`, `main.js`/`startPosApp`, `Chrome`, `window.posmodel`), bundle `_assets_pos`, PosStore como servicio `"pos"` + `usePos`, modelos JS `related_models` + registries | Paso 2, arquitectura y store |
| [references/data-loading.md](references/data-loading.md) | `pos.load.mixin` + `_load_pos_data*`, anadir un CAMPO y un MODELO al POS, delta-loading / `pos_last_server_date`, `_filtered_access`, FIRMAS v18 vs v19 lado a lado | Paso 3, cualquier cambio de datos |
| [references/orderlines-and-order-flow.md](references/orderlines-and-order-flow.md) | `pos.order`/`pos.order.line` (Python+JS), `addLineToCurrentOrder(vals, opts, configure)` y forma de `vals`, mutar/leer linea, combos, atributos/notas, ROTURA v19 (API JS a camelCase + getters de precio fuera del modelo orderline), modelos JS nuevos v19 | Paso 3, cuando tocas la venta (lineas, precio, qty, descuento, combos) |
| [references/screens-payments-receipts.md](references/screens-payments-receipts.md) | Screens (`pos_screens` v18 / `pos_pages` v19 + router), popups via `dialog`/`makeAwaitable`/`ask`, ControlButtons, `PaymentInterface` + `register_payment_method` + Python, `OrderReceipt`, restaurant y self-order a alto nivel | Paso 4, UI / pagos / recibos |
| [references/version-diffs.md](references/version-diffs.md) | Cheat-sheet de import paths v18<->v19 + roturas conocidas + los 4 patrones canonicos (anadir campo / anadir modelo / anadir screen / terminal de pago) con variante v18 y v19 | Paso 1 y cualquier duda de version |
| [references/module-catalog.md](references/module-catalog.md) | Catalogo de los ~106 modulos POS oficiales v18 (63 CE + 43 EE) como EJEMPLARES de extension: mapa "quiero hacer X -> estudia modulo Y", tablas por funcion (promo, ventas, HR, restaurant, self-order, preparacion, pagos, hardware, l10n fiscal) con edicion/depends/superficies, y lectura de la cadena de dependencias/bridges | Paso 2, para decidir de que modulo oficial copiar el patron |
| [references/gotchas.md](references/gotchas.md) | Las 15 gotchas completas con anclas `file:line` v18/v19 y el porqué de cada fallo silencioso | Antes de dar por buena una extensión |
