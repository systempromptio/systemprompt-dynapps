# Arquitectura v19 — Apps de Barcode de Odoo Enterprise

Dos apps OWL independientes, ambas registradas como `ir.actions.client`:

1. **stock_barcode** — escaneo de transferencias (`stock.picking`) y ajustes de inventario (`stock.quant`).
2. **Shop Floor / mrp_display** (módulo `mrp_workorder`) — panel de fabricación por centro de trabajo.

No comparten árbol de componentes ni modelo de estado; comparten filosofía (client action OWL + carga inicial por RPC + estado no reactivo dirigido por eventos) y el módulo base `barcodes` para el parser GS1 y el input manual.

## 1. stock_barcode — modelo mental

Un **client action único** (`stock_barcode_client_action` → `MainComponent`, `odoo_enterprise/19/stock_barcode/static/src/components/main.js:657`, `stock_barcode_client_action`) que, según el `res_model` de la acción, instancia **un modelo de negocio JS distinto** (NO un componente): `stock.picking` → `BarcodePickingModel`, `stock.quant` → `BarcodeQuantModel`. Cualquier otro `res_model` lanza `throw new Error('No JS model define')` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js::_getBarcodeModel`, dentro de `_getBarcodeModel` en `:298`).

Clave arquitectónica (corrige el modelo mental v18): **`BarcodeModel extends EventBus`** (`@odoo/owl`, `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:9/:12`). NO es una "clase plana" ni hereda `EventTarget`. NO es reactivo OWL: mutar estado no repinta; hay que emitir `this.trigger('update')`, y aun así el repintado está **guardado** por `if (!this.state.uiBlocked)` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-257`). El estado se serializa con `JSON.stringify` (`_createState`, `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::_createState/:2075`), así que **referencias circulares o valores no-JSON en el estado revientan el snapshot**.

`MainComponent` es un cascarón: crea el modelo en `setup()` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:101`, `MainComponent.model`), lo expone vía `useSubEnv({model, dialog})` (`:103-106`) para que todo hijo acceda a `env.model`, y cablea eventos. Los `useBus(env.model, ...)` van en `setup()` (`:132-149`); los `addEventListener('update'|'refresh'|'process-action'|'history-back')` van en **`onWillStart()`** (`:251-258`), NO en setup. La **carga inicial** también vive en `onWillStart` (`:227-259`): `rpc('/stock_barcode/get_barcode_data', {model, res_id})` (`:228`) → `env.model.setData(barcodeData)` (`:249`). **No existe ningún método `Main.getBarcodeData()`** (error frecuente del análisis previo).

### Árbol de render (stock.picking)

```
MainComponent (odoo_enterprise/19/stock_barcode/static/src/components/main.xml:10, js_class stock_barcode_client_action)
├─ BarcodeInput / BarcodeVideoScanner        (input manual + cámara; del módulo barcodes)
├─ CountScreenRFID    (odoo_enterprise/19/stock_barcode/static/src/components/main.xml:11, t-if state.displayCountRFID)
└─ [state.view]  ← valores REALES en main.xml:
   ├─ barcodeLines      (:28/:48/:65)  ← itera env.model.groupedLinesByLocation (:71) → group.lines (:84)
   │     ├─ PackageLineComponent   (si line.isPackageLine; openPackage.bind=onOpenPackage)
   │     ├─ GroupedLineComponent   (si line.lines; extiende LineComponent)
   │     └─ LineComponent          (resto; editLine.bind=onOpenProductPage)
   ├─ scrapProductPage  (:92)
   ├─ productPage       (:95)  ← <View> con js_class stock_barcode_sml_form (form o_barcode_line_form)
   ├─ packagePage       (:98)
   ├─ infoFormView      (:105)
   └─ actionsView       (:115)  ← el menú de impresión vive DENTRO de aquí (:127-140)
```

**Valores de `state.view`** (verificado contra `main.xml`): `barcodeLines`, `scrapProductPage`, `productPage`, `packagePage`, `infoFormView`, `actionsView`. **`printMenu` y `editFormView` NO son valores de `state.view`** — el comentario `odoo_enterprise/19/stock_barcode/static/src/components/main.js:111`, `MainComponent.setup` que los menciona está obsoleto.

`LineComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/line.js:4`, `LineComponent`, **export default**, no está en registry) es la unidad de línea; `GroupedLineComponent` y `PackageLineComponent` la **extienden**, así que un `patch()` a su prototype se propaga a los tres. Sus getters bindan a `env.model.<método>`; sus acciones llaman `env.model.<accion>()` y luego `env.model.trigger('update')`.

### Modelo de estado (barcode_model.js)

Contrato abstracto que toda subclase implementa (todos `throw 'Not Implemented'` en la base): `getQtyDone`, `getQtyDemand`, `apply`, `get barcodeInfo`, `lineIsFaulty`, `updateLineQty`, `_convertDataToFieldsParams`, `_getFieldToWrite`, `_getSaveCommand`, `_createLinesState`, `_lineIsNotComplete`, `_updateLineQty`, `_updateLotName`, `_getCompanyId`, `_getNewLineDefaultContext`, `get printButtons`, `_processPackage`. Además la subclase declara `this.precision` y `_clearScanData()`. Pipeline de escaneo central: `processBarcode` (`:685`) → `_processBarcode` (`:1300-1655`, monolito de ~355 líneas con orden estricto: action → packaging → checkBarcode → location → package → product → qty → find/update/create).

## 2. Shop Floor (mrp_display) — modelo mental

Client action `mrp_display` → `MrpDisplayAction` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_action.js:196`, `mrp_display`). El componente raíz `MrpDisplay` (`mrp_display.js`) construye a mano la estructura de campos anidados en `_makeModelParams` (`:446-508`), desacoplada de `fieldsStructure` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_action.js::fieldsStructure`): añadir un campo a `fieldsStructure` sin declararlo como `activeField` anidado NO lo carga (fallo silencioso).

Jerarquía de líneas de registro: **`StockMove extends QualityCheck`** (`mrp_record_line/`). `MrpWorkorder` también en la cadena. Un override de `QualityCheck.clicked()`/`icon()` sin `super()` afecta también a las líneas de consumo (`StockMove`).

Registro de MO: `MrpDisplayRecord.validate()` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_record.js::validate`) gestiona backorders, borrado in-place de `model.root.records` (`:368-373`) y el flag `underValidation`; la RPC real de cierre (`do_finish`/`button_mark_done`, nombres asignados en `:334`/`:345`) es un **único** `orm.call` en `:351`.

Escaneo: `MrpDisplay._onBarcodeScanned` (`:186`) resuelve en orden MO (`:196`) → WO (`:200`) → empleado (`:206`) → producto (`:210/213`), ignorando prefijos OBT/OCD y con guard de diálogos abiertos.

Empleados conectados: hook `useConnectedEmployee` (`mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js:8`, `useConnectedEmployee`) + `MrpEmployeeDialog`. Punto de entrada v19: `setConnectedEmployees` (`:195`) → RPC `hr.employee.set_employees_connected` (`:200`).

## 3. Flujo de un escaneo end-to-end (stock_barcode)

1. Servidor resuelve la acción cliente y, al escanear desde el menú principal, `/stock_barcode/scan_from_main_menu` **inyecta `context.active_id`** (leído en `odoo_enterprise/19/stock_barcode/static/src/components/main.js:100`, `MainComponent.setup`). Los records estáticos `ir.actions.client` (`odoo_enterprise/19/stock_barcode/data/data.xml:2`, `stock_barcode_picking_client_action` (y `:9`)) NO llevan context.
2. `MainComponent.onWillStart` → `rpc('/stock_barcode/get_barcode_data')` → `env.model.setData(...)`.
3. Usuario escanea → `BarcodeInput`/cámara → `env.model.processBarcode(barcode)` → `_processBarcode` (pipeline).
4. El parser (`BarcodeParser`, módulo `barcodes`, con GS1 monkey-patcheado) produce un `BarcodeObject`; el cache cliente (`LazyBarcodeCache`) resuelve records por barcode, pidiendo al servidor `/stock_barcode/get_specific_barcode_data` los que falten.
5. El modelo hace match de línea (`_findLine`, `:1838`), actualiza/crea línea, dispara `flash`/`playSound`/`update`.
6. `update` → `MainComponent.render(true)` si `!uiBlocked`.
7. Guardar: `save()` → `_getSaveCommand()` (override por modelo) → `rpc('/stock_barcode/save_barcode_data')` con x2many commands.
8. Validar: `_validate()` → `orm.call(resModel, validateMethod, [recordIds])`. Al salir sin validar: `beforeQuit`/`_onExit` → `stock.move.post_barcode_process` (solo picking, nuevo en v19).

## Qué cambia respecto a v18 (resumen de arquitectura)

- **`manual_barcode` migró del propio stock_barcode al módulo base `barcodes`** (`odoo19/addons/barcodes/`) y renombró sus templates. Parchearlo ahora afecta a TODAS las apps que usan `barcodes`.
- **`BarcodeModel extends EventBus`** (confirmado; el "clase plana" era impreciso).
- **Paquetes anidados** (put-in-pack / pack-in-pack) nuevos en `BarcodePickingModel`: `outermost_result_package_id`, `parent_package_id`, `package_dest_id`.
- **`reserved_uom_qty`** (reserva clientside, sin campo servidor) con **fórmula de conversión UoM invertida** respecto a v18.
- **Renombres de modelo core**: `product.packaging` → `product.uom`; `stock.quant.package` → `stock.package` (Odoo core, stock_barcode solo hace `_inherit`).
- **Rutas HTTP**: 7 migradas de `type='json'` a `type='jsonrpc'`, 2 nuevas (`get_epc`, `get_epc_sgtin`), 1 sigue `http`.
- **Shop Floor**: eliminada la carpeta JS `mrp_employee_list/` y `mrp_quality_check_confirmation_dialog.*`; `set_employees_connected` nuevo; pero la acción server `action_open_employee_list` SIGUE existiendo.
- **stock_barcode/static/src/js/** (form controller `stock_barcode_sml_form`) y **widget `stock_barcode_quant_one2many`** son nuevos en v19.
