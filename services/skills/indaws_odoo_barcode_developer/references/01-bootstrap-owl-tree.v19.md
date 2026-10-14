## stock_barcode v19 — Arranque y árbol OWL

Fuente: `odoo_enterprise/19/stock_barcode`. Comparado contra v18 (`odoo_enterprise/18/stock_barcode`). El módulo base `barcodes` (CE) vive en `odoo19/addons/barcodes`. Todas las citas file:line reverificadas contra la fuente v19.

### 1. Registro de client actions (grep verificado)

Dos client actions OWL, ambas registradas por nombre en el registry `actions`:

- **`stock_barcode_main_menu`** → `MainMenu`
  `odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js:166`, `stock_barcode_main_menu` → `registry.category("actions").add("stock_barcode_main_menu", MainMenu);`
- **`stock_barcode_client_action`** → `MainComponent`
  `odoo_enterprise/19/stock_barcode/static/src/components/main.js:657`, `stock_barcode_client_action` → `registry.category("actions").add("stock_barcode_client_action", MainComponent);`

Registros server-side (`ir.actions.client`):
- `odoo_enterprise/19/stock_barcode/views/stock_barcode_views.xml:6` `stock_barcode_action_main_menu` (tag `stock_barcode_main_menu`, `path=barcode`, `target=fullscreen`).
- `odoo_enterprise/19/stock_barcode/data/data.xml:2` `stock_barcode_picking_client_action` — `res_model=stock.picking`, `target=fullscreen`, tag `stock_barcode_client_action`.
- `odoo_enterprise/19/stock_barcode/data/data.xml:9` `stock_barcode_inventory_client_action` — `res_model=stock.quant`, `target=fullscreen`, tag `stock_barcode_client_action`.

> ⚠️ Corrección vs draft: estos records estáticos **NO** llevan `context`/`active_id`. El `active_id` que luego lee `MainComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:100`, `MainComponent.setup`) lo inyecta el servidor al resolver el escaneo en `/stock_barcode/scan_from_main_menu` (devuelve la action ya contextualizada).

Menú de entrada: `odoo_enterprise/19/stock_barcode/views/stock_barcode_views.xml:13`, `stock_barcode_menu` `menuitem id="stock_barcode_menu"` → `action=stock_barcode_action_main_menu`, `web_icon="stock_barcode,static/description/icon.png"`, `sequence=155`, `groups="stock.group_stock_user"`.

### 2. Flujo main_menu → client action

1. El menú abre `MainMenu` (landing page barcode). `MainMenu.setup()` (`odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js::setup`):
   - Services: `action`, `dialog`, `pwa`, `home_menu`, `notification`, `barcode`.
   - `useBus(this.barcodeService.bus, "barcode_scanned", ...)` → `_onBarcodeScanned` (`odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js::setup`).
   - `onWillStart` hace `rpc("/stock_barcode/get_main_menu_data")` (`odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js::setup`): groups locations/package/tracking, quant_count, play_sound, precarga audio success.
2. Al escanear (`_onBarcodeScanned`, `odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js::_onBarcodeScanned`): `rpc("/stock_barcode/scan_from_main_menu", {barcode})` (`:108`). Si `res.action` → `playSound("success")` + `this.actionService.doAction(res.action)`. Si no hay action → `notificationService.add(res.warning, {type:"danger"})`.
3. `MainComponent` (`stock_barcode_client_action`) recibe la action y arranca la app real.

### 3. MainComponent: montaje del modelo de estado

`MainComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:74`, `MainComponent`):
- `static props = { ...standardActionServiceProps }` (import de `@web/webclient/actions/action_service`, `odoo_enterprise/19/stock_barcode/static/src/components/main.js:31`).
- `static template = "stock_barcode.MainComponent"`.
- En `setup()` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:93`, `MainComponent.setup`):
  - Services: `orm`, `notification`, `dialog`, `action`; `actionMutex = new Mutex()`.
  - `this.resModel = this.props.action.res_model` (`stock.picking` o `stock.quant`).
  - `this.resId = this.props.action.context.active_id || false`.
  - `const model = this._getModel()`; `model.newScrapProduct = this.newScrapProduct.bind(this)`; `useSubEnv({ model, dialog: this.dialog })` → **el modelo se expone por env**, todos los subcomponentes leen `env.model`.
  - `state = useState({...})` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:109`, `MainComponent.setup`): `view` (default `"barcodeLines"`), `cameraScannedEnabled`, `displayNote`, `displayCountRFID`, `uiBlocked`, `barcodesProcessed`, `barcodesToProcess`, `readyToToggleCamera`.
  - Buses OWL: `barcodeService.bus`→`onBarcodeScanned`, `mobileService.bus` (`mobile_reader_scanned`)→`onMobileReaderScanned`, y varios `useBus(this.env.model, ...)` (flash, playSound, blockUI/unblockUI, contadores) + `useBus(bus, "refresh", ...)` sobre el `EventBus` de módulo (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:39`, `bus`).
  - Lifecycle: `onWillStart(() => this.onWillStart())` (rpc `/stock_barcode/get_barcode_data` en `odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart`, carga sonidos, `setupCameraScanner`, `env.model.setData(barcodeData)` y `addEventListener` process-action/refresh/update/history-back en `:251-258`), `onPatched`→scroll a línea, **dos** `onWillUnmount` (clearTimeout buffering en `:153` + `env.model._onExit()` en `:161`).

> ⚠️ Valores reales de `state.view` (main.xml): `barcodeLines`, `scrapProductPage`, `productPage`, `packagePage`, `infoFormView`, `actionsView`. El comentario en `odoo_enterprise/19/stock_barcode/static/src/components/main.js:111`, `MainComponent.setup` que menciona `'printMenu'`/`'editFormView'` está **obsoleto**: no existen como ramas de vista (el menú de impresión vive dentro de `actionsView`, `odoo_enterprise/19/stock_barcode/static/src/components/main.xml:127-140`, `stock_barcode.MainComponent`).

`_getModel()` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js::_getModel`) construye el modelo con `services = { rpc, orm, notification, action }` y `_getBarcodeModel()` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js::_getBarcodeModel`) selecciona la clase: `stock.picking`→`BarcodePickingModel`, `stock.quant`→`BarcodeQuantModel`, otro → `throw new Error("No JS model define")` (`:304`).

Modelo: `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:12`, `BarcodeModel` `export default class BarcodeModel extends EventBus`. Constructor `:13 constructor(resModel, resId, services)` — hace `this.dialogService = useService("dialog")` (`:15`, hook válido porque se instancia durante el `setup` de MainComponent) y guarda `orm/notificationService/action`. El estado real (cache, parser, config, groups) se puebla en `setData(data)` (`:35`).

### 4. Árbol de componentes OWL (`components/main.xml`)

Raíz `stock_barcode.MainComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/main.xml:10`, `stock_barcode.MainComponent`), hijos declarados en `static components` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:77`): `BarcodeInput, BarcodeVideoScanner, Chatter, CountScreenRFID, GroupedLineComponent, ImportBlockUI, LineComponent, PackageLineComponent, View`.

Composición condicionada por `state.view`:
- Overlays: `CountScreenRFID` (t-if `state.displayCountRFID`, `odoo_enterprise/19/stock_barcode/static/src/components/main.xml:11`, `stock_barcode.MainComponent`), `ImportBlockUI` (t-if `state.uiBlocked`, `odoo_enterprise/19/stock_barcode/static/src/components/main.xml:12`).
- Header con `BarcodeVideoScanner t-props="barcodeVideoScannerProps"` (t-if `state.cameraScannedEnabled`, `odoo_enterprise/19/stock_barcode/static/src/components/main.xml:46`, `stock_barcode.MainComponent`).
- `barcodeLines`: foreach `env.model.groupedLinesByLocation` → por línea `PackageLineComponent` / `GroupedLineComponent` / `LineComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/main.xml:85-87`, `stock_barcode.MainComponent`).
- `scrapProductPage`/`productPage`: `<View>` embebido (scrap form / line edit form, `odoo_enterprise/19/stock_barcode/static/src/components/main.xml:93,96`, `stock_barcode.MainComponent`).
- `packagePage`: `<View type="'kanban'">` de `stock.quant` (`odoo_enterprise/19/stock_barcode/static/src/components/main.xml:99`, `stock_barcode.MainComponent`).
- `infoFormView`: `<View type="'form'">` del `resModel` + `<Chatter threadModel="resModel" threadId="resId"/>` (`odoo_enterprise/19/stock_barcode/static/src/components/main.xml:106-113`, `stock_barcode.MainComponent`).
- `actionsView`: `<BarcodeInput onSubmit.bind="onBarcodeSubmitted"/>` (input manual, `odoo_enterprise/19/stock_barcode/static/src/components/main.xml:117`, `stock_barcode.MainComponent`) + botones de impresión y operaciones.

Otros registros del propio main.js: `StockBarcodeUnlinkButton` (view_widget `stock_barcode_unlink_button`, `odoo_enterprise/19/stock_barcode/static/src/components/main.js:53`, `stock_barcode_unlink_button` (el `registry.category("view_widgets").add`), template `stock_barcode.UnlinkButton`), `ImportBlockUI` (clase exportada en `odoo_enterprise/19/stock_barcode/static/src/components/main.js:60`, `ImportBlockUI`, template `stock_barcode.BlockUI`).

### 5. CAMBIO CLAVE v18→v19: dónde vive el escaneo manual

En **v18** existía `stock_barcode/static/src/components/manual_barcode.js` + `.xml` con las clases `BarcodeInput` (template `stock_barcode.BarcodeInput`) y `ManualBarcodeScanner` (template `stock_barcode.ManualBarcodeScanner`). main.js las importaba con `import { BarcodeInput } from "./manual_barcode"` (`v18 odoo_enterprise/19/stock_barcode/static/src/components/main.js:23`) y main_menu/kanban con `../components/manual_barcode`.

En **v19 ese fichero fue ELIMINADO de stock_barcode** (verificado: no existe `manual_barcode.*` en v19 stock_barcode). Ambas clases se promovieron al módulo base `barcodes`:
`odoo19/addons/barcodes/static/src/components/manual_barcode.js` (+ `.xml`), con templates **renombrados** a `barcodes.BarcodeInput` y `barcodes.ManualBarcodeScanner` (verificado también que en v18 el módulo `barcodes` NO tenía este fichero → movimiento real, no duplicado).

- `BarcodeInput` (`barcodes/static/src/components/manual_barcode.js:6`, `BarcodeInput`): template `barcodes.BarcodeInput`, props `{onSubmit, placeholder?}`, `defaultProps.placeholder = _t("Enter a barcode...")` (`:12-14`), autofocus `onMounted` (`:21`), submit con Enter (`:32`).
- `ManualBarcodeScanner` (`:40`): `extends BarcodeDialog` de `@web/core/barcode/barcode_dialog`, template `barcodes.ManualBarcodeScanner`, compone `BarcodeInput`, props `[...BarcodeDialog.props, "placeholder?"]` (`:46`).

Nuevos imports en v19 (todos vía alias de módulo, no rutas relativas):
- `odoo_enterprise/19/stock_barcode/static/src/components/main.js:32` `import { BarcodeInput } from "@barcodes/components/manual_barcode";`
- `odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js:7` `import { ManualBarcodeScanner } from "@barcodes/components/manual_barcode";`
- `odoo_enterprise/19/stock_barcode/static/src/kanban/stock_barcode_kanban_renderer.js:2` `import { ManualBarcodeScanner } from "@barcodes/components/manual_barcode";`

`ManualBarcodeScanner` se abre como diálogo (`dialogService.add`) desde `MainMenu.openManualBarcodeDialog` (`odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js::openManualBarcodeDialog`) y `StockBarcodeKanbanRenderer.openManualBarcodeDialog`, ambos con `{facingMode:"environment", onResult, onError}`.

### 6. Otros cambios v18→v19 del arranque/estructura

- **Nueva carpeta `static/src/js/`**: solo `js/stock_barcode_sml_form.js`. Define `StockBarcodeSmlFormController` (extiende `FormController`, `:10`) y registra la vista `stock_barcode_sml_form` en el registry `views` (`:81`). Override de `beforeExecuteActionButton` (`:37`) que, **solo cuando `clickParams.special === "save"`**, comprueba stock disponible en la ubicación (`searchRead` sobre `product.product` qty_available) y pide confirmación si el producto storable no está allí. Es el `js_class` del form `o_barcode_line_form` (`odoo_enterprise/19/stock_barcode/views/stock_move_line_views.xml:9`, `stock_move_line_product_selector`), i.e. la vista embebida del `<View>` en `state.view='productPage'`. **No existía en v18** (no había carpeta `static/src/js/`).
- **Nuevo widget `widgets/set_quant_stock_move_line.js`**: `StockBarcodeQuantOne2ManyField` (extiende `X2ManyField`, `:6`), registrado como field `stock_barcode_quant_one2many` (`:84`). Usa `product_stock_quant_ids` (`:11`) y `picking_code` (`:16`). **No existía en v18** (widgets v18 solo digipad/image_preview/set_reserved_qty_button).
- **Sin banner `/** @odoo-module **/`** en main.js v19 (ESM nativo, línea 1 es `import`); v18 aún lo llevaba (verificado).
- **manifest `web.assets_unit_tests`** (`stock_barcode/__manifest__.py:47-49`): nuevo bundle `stock_barcode/static/tests/units/*.test.js` (tests Hoot). No existía en el manifest v18. El bundle backend (`:37-39`) usa 3 globs `static/src/**/*.js|scss|xml` + `('remove', '…/*.dark.scss')` (`:42`), y los dark van a `web.assets_web_dark` (`:44-46`) → **cualquier fichero nuevo bajo src/ se empaqueta solo**.
- `depends` v19: `['stock', 'web_tour', 'web_mobile']` (`stock_barcode/__manifest__.py:12`).

### 7. Gotchas de arranque

- El modelo de estado **NO es un componente OWL ni es reactivo OWL**, pero **tampoco es una clase plana**: `BarcodeModel extends EventBus` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:12`, `BarcodeModel`) — de ahí `trigger()`/`addEventListener()`. Los re-render se disparan por `env.model.addEventListener('update', ...)` → `this.render(true)` **guardado por `if (!this.state.uiBlocked)`** (`odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-257`) y por buses. Si extiendes el modelo y mutas estado sin emitir `'update'`/bus, la UI no repinta; y con la UI bloqueada, ni siquiera el `'update'` repinta.
- `dialogService = useService("dialog")` se llama dentro del **constructor del modelo** (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::constructor`), no en un `setup` de componente. Funciona porque se ejecuta durante el `setup` de `MainComponent`; si instancias el modelo fuera de ese contexto (tests, código propio) el hook peta.
- `this.resId` sale de `props.action.context.active_id` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:100`, `MainComponent.setup`): si abres la client action sin `active_id` arrancas en modo "sin registro" (resId=false) y el flujo depende de escanear para crear.
- El escaneo manual ya **NO** se parchea en stock_barcode: para tocar `ManualBarcodeScanner`/`BarcodeInput` hay que patchear el módulo base `barcodes` (afecta a TODAS las apps barcode, no solo stock).
- **Sustituir las client actions NO es "last-write-wins"**: `registry.add(key, value)` lanza `DuplicatedKeyError` si la key ya existe (`odoo19/addons/web/static/src/core/registry.js:104-106`, `Registry.add`). Para reemplazar el componente hay que pasar `{force:true}` (y entonces sí depende del orden de assets); en la práctica, usa `patch()`.


## Correcciones aplicadas en verificación (v19)

- EXTENSION POINT #1 (registry actions) — REFUTADO 'last-write-wins via re-add'. Verificado en odoo19/addons/web/static/src/core/registry.js:102-107: `add(key,value,{force})` lanza `DuplicatedKeyError` si la key ya existe y `force` no es true. Re-registrar la misma tag sin `{force:true}` PETA, no gana el último. Lo correcto: `patch(MainComponent.prototype,{...})` o, si de verdad quieres sustituir el componente, `add(tag, NewComp, {force:true})` (y entonces sí importa el orden de carga de assets).
- SECCIÓN 3 (state.view) — CORREGIDO. El draft lista 'printMenu' como valor de state.view. En main.xml NO hay rama t-if/t-elif para 'printMenu' (ni para 'editFormView'). Los valores reales son: barcodeLines (odoo_enterprise/19/stock_barcode/static/src/components/main.xml:28/48/65), scrapProductPage (:92), productPage (:95), packagePage (:98), infoFormView (:105), actionsView (:115). El comentario del código en odoo_enterprise/19/stock_barcode/static/src/components/main.js:111 ('Could be also printMenu or editFormView') está OBSOLETO; el menú de impresión vive DENTRO de actionsView (odoo_enterprise/19/stock_barcode/static/src/components/main.xml:127-140), no es un state.view.
- GOTCHA / SECCIÓN 3 — CORREGIDO. El modelo NO es 'una clase plana'. `BarcodeModel extends EventBus` (odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:12). Por eso expone `trigger()`/`addEventListener()`. Sigue siendo cierto que no es un componente OWL ni es reactivo OWL, pero la afirmación 'clase plana' es imprecisa.
- GOTCHA (repintado) — MATIZADO. El `addEventListener('update') -> this.render(true)` está GUARDADO por `if (!this.state.uiBlocked)` (odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-257), no es incondicional. Con la UI bloqueada, un 'update' NO repinta.
- SECCIÓN 1 — CLARIFICADO. Los records estáticos `ir.actions.client` (odoo_enterprise/19/stock_barcode/data/data.xml:2 y :9) NO llevan `context`/`active_id`. El `context.active_id` que lee `MainComponent` (odoo_enterprise/19/stock_barcode/static/src/components/main.js:100) lo inyecta el servidor al resolver el escaneo en `/stock_barcode/scan_from_main_menu`. El draft lo atribuía al record estático.
- BarcodeInput (barcodes/manual_barcode.js:6) — AÑADIDO: tiene `static defaultProps = { placeholder: _t('Enter a barcode...') }` (líneas 12-14), además de props {onSubmit, placeholder?}. Dato omitido en el draft, no incorrecto.
