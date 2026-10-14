## Subsistema: Arranque y árbol OWL de la app Barcode (Odoo 18 EE `stock_barcode`)

Todas las rutas son absolutas bajo `odoo_enterprise/18/stock_barcode/`. Verificado contra fuente 2026-07-09.

### 1. Registro de assets (manifest)
`__manifest__.py` inyecta TODO el frontend con globs en `web.assets_backend`:
```
# web.assets_backend  (__manifest__.py:36)
'stock_barcode/static/src/**/*.js'                          # :37
'stock_barcode/static/src/**/*.scss'                        # :38
'stock_barcode/static/src/**/*.xml'                         # :39
('remove', 'stock_barcode/static/src/**/*.dark.scss')       # :42
# web.assets_web_dark (:44-45): solo los *.dark.scss
# web.qunit_suite_tests (:47-48): static/tests/units/**/*
# web.assets_tests (:50-51): static/tests/tours/**/*
```
- `web.assets_backend` (`stock_barcode/__manifest__.py:36-42`) → todo `static/src`. No hay lista explícita de ficheros: **añadir un fichero JS/XML/SCSS bajo `static/src` lo incluye automáticamente** (no hace falta tocar el manifest para extender por asset propio del módulo; sí para módulos que hereden, que declaran sus propios globs).
- Tours en `web.assets_tests` (`:50-51`); tests unit JS en `web.qunit_suite_tests` (`:47-48`).
- `depends`: `['stock', 'web_tour', 'web_mobile']`. `auto_install: True`, `application: True`, licencia `OEEL-1`.

### 2. Dos client actions distintas + una act_window kanban
Hay **dos** componentes OWL registrados como client actions y **tres** puntos de entrada de navegación:

| tag (registry `actions`) | Componente OWL | Registrado en |
|---|---|---|
| `stock_barcode_main_menu` | `MainMenu` | `odoo_enterprise/18/stock_barcode/static/src/main_menu/main_menu.js:108`, `stock_barcode_main_menu` |
| `stock_barcode_client_action` | `MainComponent` | `odoo_enterprise/18/stock_barcode/static/src/components/main.js:620`, `stock_barcode_client_action` |

Registros de las `ir.actions.client` (XML):
- **Menú principal**: `odoo_enterprise/18/stock_barcode/views/stock_barcode_views.xml:6-11`, `stock_barcode_action_main_menu` → `id=stock_barcode_action_main_menu`, `tag=stock_barcode_main_menu`, `path=barcode`, `target=fullscreen`. El `menuitem` de la app está en `:13-18` (`stock_barcode_menu`, icono `stock_barcode,static/description/icon.png`, `groups="stock.group_stock_user"`).
- **Cliente de picking**: `odoo_enterprise/18/stock_barcode/data/data.xml:2-7`, `stock_barcode_picking_client_action` → `id=stock_barcode_picking_client_action`, `tag=stock_barcode_client_action`, `res_model=stock.picking`, `target=fullscreen`.
- **Cliente de inventario**: `odoo_enterprise/18/stock_barcode/data/data.xml:9-14`, `stock_barcode_inventory_client_action` → `id=stock_barcode_inventory_client_action`, `tag=stock_barcode_client_action`, `res_model=stock.quant`, `target=fullscreen`.

Ambas client actions de operación comparten el MISMO tag/componente (`MainComponent`); lo que las diferencia es `res_model` en el action, que `MainComponent` lee para elegir el modelo JS de estado.

La act_window kanban de operaciones es `stock_picking_type_action_kanban` (`stock_barcode/views/stock_picking_type_views.xml:36`, `stock_picking_type_action_kanban`, `view_mode=kanban`, vista `stock_picking_type_kanban` en `:5-34`).

### 3. Flujo main_menu → client action
`MainMenu` (`main_menu/main_menu.js`) es el hub de arranque:
- `setup()` (`:18-44`): usa servicios `action`, `dialog`, `pwa`, `home_menu`, `notification`, `barcode`. En `onWillStart` (`:29-43`) hace `rpc("/stock_barcode/get_main_menu_data")` → grupos (locations/package/tracking), `quant_count`, `play_sound`. `static components = {}` (`:15`).
- Botón **Operations** (`odoo_enterprise/18/stock_barcode/static/src/main_menu/main_menu.xml:44-47`, `stock_barcode.MainMenu`): `actionService.doAction('stock_barcode.stock_picking_type_action_kanban')` (act_window kanban en `stock_barcode/views/stock_picking_type_views.xml:36`, `stock_picking_type_action_kanban`, `stock_picking_type_action_kanban`).
- Botón **Inventory count** (`odoo_enterprise/18/stock_barcode/static/src/main_menu/main_menu.xml:48-52`, `stock_barcode.MainMenu`): `actionService.doAction('stock_barcode.stock_barcode_inventory_client_action')` → `MainComponent` con `res_model=stock.quant`.
- Escaneo desde el menú (`odoo_enterprise/18/stock_barcode/static/src/main_menu/main_menu.js::_onBarcodeScanned-105`): `rpc('/stock_barcode/scan_from_main_menu', {barcode})`; si devuelve `action`, `doAction(res.action)` (típicamente abre `stock_barcode_picking_client_action` sobre un picking concreto).
- Al salir de una operación, `MainComponent._exit()` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::_exit-435`) vuelve al menú con `doAction("stock_barcode.stock_barcode_action_main_menu")` si `breadcrumbs.length === 1`.

Backend del arranque de datos: controlador `controllers/stock_barcode.py` — `/stock_barcode/get_main_menu_data` (`:117`) y `/stock_barcode/get_barcode_data` (`:89`), que delega en `request.env[model]._get_stock_barcode_data()` (implementado en `odoo_enterprise/18/stock_barcode/models/stock_picking.py:85`, `StockPicking._get_stock_barcode_data` y `odoo_enterprise/18/stock_barcode/models/stock_quant.py:75`, `StockQuant._get_stock_barcode_data`) + `_get_groups_data()` (`:475`).

### 4. `MainComponent`: instanciación del modelo de estado
`static/src/components/main.js`:
- `static props = { ...standardActionServiceProps }` (`:67`) → recibe `action`, `actionId`, `updateActionState`, etc. del action service.
- `static template = "stock_barcode.MainComponent"` (`:68`).
- `setup()` (`:85-146`):
  - Servicios: `orm`, `notification`, `dialog`, `action` (`:86-89`), más `barcode` (`:115`) y `mobile` (`:119`).
  - `this.resModel = this.props.action.res_model` (`:91`); `this.resId = this.props.action.context.active_id || false` (`:92`).
  - `const model = this._getModel()` (`:93`) — **fábrica del modelo de estado** (`:279-289`):
    - `stock.picking` → `new BarcodePickingModel(resModel, resId, services)` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js:17`, `BarcodePickingModel`, extends `BarcodeModel`).
    - `stock.quant` → `new BarcodeQuantModel(...)` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js:7`, `BarcodeQuantModel`, extends `BarcodeModel`).
    - Otro → `throw new Error('No JS model define')` (`:287`).
    - `services = { rpc, orm, notification, action }` (+`dialog` solo para picking).
  - `useSubEnv({ model, dialog })` (`:95-98`) → **el modelo se expone como `env.model`**, patrón central: casi todos los subcomponentes y la plantilla acceden al estado vía `env.model`, no vía props.
  - `this.state = useState({...})` (`:101-110`): máquina de vistas `state.view` inicializada a `"barcodeLines"`. Los valores REALES que renderiza `main.xml` son `barcodeLines`, `actionsView`, `infoFormView`, `productPage`, `packagePage`, `scrapProductPage`. (`printMenu`/`editFormView` aparecen solo en un comentario obsoleto en `:103` y no se usan en ningún sitio.) El resto de `state`: `cameraScannedEnabled`, `displayNote`, `displayCountRFID`, `uiBlocked`, `barcodesProcessed`, `barcodesToProcess`, `readyToToggleCamera`.
  - `BarcodeModel extends EventBus` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:14`, `BarcodeModel`): el modelo NO es OWL reactivo; comunica cambios por eventos. `MainComponent` se suscribe con `useBus(this.env.model, ...)` a `flash`, `playSound`, `blockUI`, `unblockUI`, `addBarcodesCountToProcess`, `updateBarcodesCountProcessed`, `clearBarcodesCountProcessed` (`:124-130`) y con `addEventListener` a `process-action`, `refresh`, `update`, `history-back` (dentro de `onWillStart`, `:231-238`). El re-render se dispara manualmente vía listener `update` → `this.render(true)` (`:233-237`).
  - `onWillStart(() => this.onWillStart())` (`:133`): `onWillStart()` (`:209-239`) hace `rpc("/stock_barcode/get_barcode_data", {model, res_id})`, arma `this.config` (sonidos, delay), llama `this.env.model.setData(barcodeData)` (`:229`, carga `LazyBarcodeCache`, parser GS1, config, grupos) y registra los listeners del modelo.
  - `useBus(this.barcodeService.bus, "barcode_scanned", ...)` (`:116-118`) → `onBarcodeScanned` → `env.model.processBarcode` bajo `Mutex` (`actionMutex`, `:90`).

### 5. Árbol de componentes OWL de alto nivel
`MainComponent.static components` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:69-79`, `MainComponent`): `BarcodeInput`, `BarcodeVideoScanner`, `Chatter`, `CountScreenRFID`, `GroupedLineComponent`, `ImportBlockUI`, `LineComponent`, `PackageLineComponent`, `View`.

```
actions registry
├─ "stock_barcode_main_menu" → MainMenu            (odoo_enterprise/18/stock_barcode/static/src/main_menu/main_menu.js:108)
│     └─ ManualBarcodeScanner (dialog service, components/manual_barcode.js) — NO es static component
└─ "stock_barcode_client_action" → MainComponent   (components/main.js)
      env.model = BarcodePickingModel | BarcodeQuantModel   (useSubEnv, :95)
      ├─ CountScreenRFID            (t-if state.displayCountRFID, xml:11)
      ├─ ImportBlockUI             (t-if state.uiBlocked, xml:12)
      ├─ header .o_barcode_header  (título/exit, botón cámara, botón cog → changeView('actionsView'); xml:13-64)
      │     └─ BarcodeVideoScanner (t-if state.cameraScannedEnabled, xml:46)
      ├─ [view==='barcodeLines'] .o_barcode_lines (xml:65-87)
      │     └─ t-foreach env.model.groupedLinesByLocation (xml:67) → por cada group.lines:
      │           ├─ PackageLineComponent (line.isPackageLine)
      │           ├─ GroupedLineComponent (line.lines) → LineComponent (sublíneas)
      │           └─ LineComponent        (resto)
      ├─ [view==='scrapProductPage'] <View t-props="scrapViewProps"/> (xml:88-90)
      ├─ [view==='productPage']      <View t-props="lineFormViewProps"/> (xml:91-93)
      ├─ [view==='packagePage']      <View type="'kanban'" resModel="'stock.quant'".../> (xml:94-100)
      ├─ [view==='infoFormView']     <View type="'form'".../> + <Chatter/> (xml:101-110)
      └─ [view==='actionsView']      <BarcodeInput/> + botones Print/Operations (xml:111-153)
      (footers de acción condicionales al final: xml:154-184)
```
- `GroupedLineComponent` y `PackageLineComponent` **heredan de `LineComponent`** (`odoo_enterprise/18/stock_barcode/static/src/components/grouped_line.js:5`, `GroupedLineComponent`, `odoo_enterprise/18/stock_barcode/static/src/components/package_line.js:5`, `class PackageLineComponent`); `GroupedLineComponent.static components = { LineComponent }` (`odoo_enterprise/18/stock_barcode/static/src/components/grouped_line.js:6`, `GroupedLineComponent`) para renderizar sublíneas.
- Los formularios embebidos (info de picking, form de línea, scrap) y el kanban de paquete se montan con el componente estándar `<View>` de `@web/views/view` con `display: { controlPanel: false }`, vía props calculados (`lineFormViewProps` `:245-257`, `scrapViewProps` `:573-585`) o inline en la plantilla.
- `Chatter` (`@mail/chatter/web_portal/chatter`) solo se renderiza en la rama `infoFormView` (`xml:109`). `BarcodeInput` (`./manual_barcode`) solo en la rama `actionsView` (`xml:113`).
- Otros diálogos registrados/instanciados: `apply_quant_dialog`, `backorder_dialog`, `extra_product_dialog`, `product_image_dialog` (en `static/src/components/`), abiertos vía `dialog` service, no en el árbol estático.

### 6. Modelos JS de estado (jerarquía)
- `BarcodeModel` (base, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:14`, `BarcodeModel`) `extends EventBus`. Métodos clave de arranque: `setData`, cache `LazyBarcodeCache`, parser `BarcodeParser`/GS1, `processBarcode`, `save`, `validate`, `getActionRefresh`, `displayBarcodeLines`, getters `groupedLines`/`packageLines`/`barcodeInfo`.
- `BarcodePickingModel extends BarcodeModel` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js:17`, `BarcodePickingModel`).
- `BarcodeQuantModel extends BarcodeModel` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js:7`, `BarcodeQuantModel`).

### 7. Widget auxiliar y kanban
- `StockBarcodeUnlinkButton` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:32-46`, `StockBarcodeUnlinkButton`) registrado en `view_widgets` como `stock_barcode_unlink_button` (`:44`); dispara un `EventBus` local `bus` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:30`, `bus`) con evento `refresh` que `MainComponent` escucha (`:131`).
- Vista kanban propia para el listado de operaciones: `static/src/kanban/` (`stock_barcode_kanban_view.js`, `_controller.js`, `_renderer.js/.xml`), usada por la act_window `stock_picking_type_action_kanban`.


## Correcciones aplicadas en verificación

- barcode_model.js: la clase base NO está en la línea 13 sino en la 14 (`export default class BarcodeModel extends EventBus {` en odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:14). El draft citaba `:13` en la sección 4, sección 6, en la gotcha del EventBus y en el extension point del modelo.
- __manifest__.py: los números de línea de assets del draft están mal. `web.assets_backend` está en `:36` (no `:44-50`); los tres globs (**/*.js, **/*.scss, **/*.xml) en `:37-39` (no `:45-47`); el `('remove', ...*.dark.scss)` en `:42`; `web.assets_web_dark` en `:44-45`; `web.qunit_suite_tests` (units) en `:47-48`; `web.assets_tests` (tours) en `:50-51`.
- state.view: los valores 'printMenu' y 'editFormView' NO son estados reales de la máquina de vistas. Solo sobreviven en un comentario obsoleto (odoo_enterprise/18/stock_barcode/static/src/components/main.js:103 `// Could be also 'printMenu' or 'editFormView'.`); grep en static/src no encuentra ningún uso real. Los estados realmente renderizados por main.xml son: barcodeLines, actionsView, infoFormView, productPage, packagePage y scrapProductPage.
- Árbol OWL (sección 5): Chatter NO es un hermano de alto nivel; se monta SOLO dentro de la rama `view==='infoFormView'` (odoo_enterprise/18/stock_barcode/static/src/components/main.xml:109). BarcodeInput NO es hermano de alto nivel; se monta SOLO dentro de la rama `view==='actionsView'` (odoo_enterprise/18/stock_barcode/static/src/components/main.xml:113). El diagrama del draft omitía la rama `actionsView` (destino del botón cog, que contiene BarcodeInput + botones Print/Operations) y trataba la rama `packagePage` (un `<View type='kanban'>` sobre stock.quant, odoo_enterprise/18/stock_barcode/static/src/components/main.xml:94-100) como no representada.
- Sección 5: la lista principal de líneas no itera un `groupedLines` plano sino `env.model.groupedLinesByLocation` (odoo_enterprise/18/stock_barcode/static/src/components/main.xml:67), agrupando por ubicación; dentro de cada grupo el t-foreach sobre `group.lines` decide PackageLineComponent / GroupedLineComponent / LineComponent (odoo_enterprise/18/stock_barcode/static/src/components/main.xml:80-84). El getter `this.lines` (odoo_enterprise/18/stock_barcode/static/src/components/main.js::lines) sí devuelve `env.model.groupedLines`, pero no es lo que consume la plantilla del listado.
