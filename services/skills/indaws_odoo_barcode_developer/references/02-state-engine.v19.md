# BarcodeModel — Motor de estado base del scanner (stock_barcode, Odoo 19 EE)

**Fichero:** `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js` (2094 LOC según `wc -l`).
**Subclases:** `barcode_picking_model.js` (stock.picking), `barcode_quant_model.js` (stock.quant).
**Componente dueño del ciclo de vida:** `odoo_enterprise/19/stock_barcode/static/src/components/main.js` (659 LOC).

## 0. Realidad cruda: ¿reescrito respecto a v18?
**NO es un revamp.** v18 = 1911 LOC (verificado), v19 = 2094 LOC (+183, ~9,6%). La arquitectura es **la misma que v18**: misma clase `extends EventBus` (:12), mismo pipeline `setData` → `processBarcode` → `_processBarcode` → `_findLine` → `_createState`, mismos métodos abstractos. Los cambios funcionales están dominados por los **renombrados de modelos backend de v19** (`stock.quant.package`→`stock.package`, `product.packaging`→`product.uom`) más lógica nueva de UoM/packaging y paquetes anidados.

> CORRECCIÓN sobre el borrador: la afirmación "~85% del diff es Prettier" es una precisión **fabricada y no verificable** sin un diff línea a línea contra v18; se retira. Lo que SÍ está verificado: (a) el fichero v19 usa comillas dobles y no lleva banner `/** @odoo-module **/` (línea 1 es un `import`), coherente con ESM nativo v19; (b) la identidad arquitectónica (clase, cadena de métodos) es real; (c) hay cambios funcionales reales, no meramente cosméticos (renames + bloques UoM + paquetes anidados + `setConfig`/`deleteLines`). Quien migre un patch de v18 se rompe por los renames y por firmas cambiadas.

## 1. Naturaleza: EventBus, NO reactivo
```js
export default class BarcodeModel extends EventBus {  // línea 12
```
Confirmado: hereda `EventBus` de `@odoo/owl` (:9, :12). **No usa `useState`/`reactive`.** El estado (`currentState.lines`, `selectedLineVirtualId`, `lastScanned`, `scanHistory`…) son objetos JS planos. El refresco de UI es **manual**: el modelo emite `this.trigger("update")` y `main.js` lo traduce en `this.render(true)`:
```js
// odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-257  (dentro de onWillStart)
this.env.model.addEventListener("update", () => {
    if (!this.state.uiBlocked) { this.render(true); }
});
```
**Gotcha central:** mutar estado sin `trigger("update")` deja la UI congelada.

## 2. Ciclo de vida (vive en el COMPONENTE, no en el modelo)
El modelo **no tiene** `setup()` ni `load()`. La secuencia real la orquesta `Main` (main.js):
1. `Main._getModel()` (**odoo_enterprise/19/stock_barcode/static/src/components/main.js::_getModel**) → `new BarcodeModel(resModel, resId, services)`; el switch `_getBarcodeModel()` (**odoo_enterprise/19/stock_barcode/static/src/components/main.js::_getBarcodeModel**, cuerpo 299-305) sólo mapea `stock.picking`→`BarcodePickingModel` y `stock.quant`→`BarcodeQuantModel`, si no `throw new Error("No JS model define")` (odoo_enterprise/19/stock_barcode/static/src/components/main.js::_getBarcodeModel). Se instancia en `setup()` (odoo_enterprise/19/stock_barcode/static/src/components/main.js:101) y se inyecta con `useSubEnv`.
2. Carga inicial en **`onWillStart()`** (odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-259) — **NO existe ningún método `getBarcodeData()`**: hace `rpc("/stock_barcode/get_barcode_data", {model, res_id})` (odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart) y luego `this.env.model.setData(barcodeData)` (odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart).
3. Cableado de eventos, todo dentro de `onWillStart`: `addEventListener` para `process-action` (:251), `refresh` (:252), `update` (:253-257) y `history-back` (**:258**). Además `useBus` en `setup()` para `flash` (:132), `playSound` (:133), `blockUI`/`unblockUI` (:134-135), `addBarcodesCountToProcess` (:136), `updateBarcodesCountProcessed` (:139), `clearBarcodesCountProcessed` (:144) y `refresh` sobre el bus global (:149).

> CORRECCIÓN: el borrador situaba `history-back` en odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart; está en **:258**. Y llamaba al load "Main.getBarcodeData()"; ese método **no existe**, el load es `onWillStart()`.

### Constructor (:13-33)
Guarda servicios: `this.dialogService` (:15, `useService("dialog")`), `this.orm` (:16), `this.notificationService` (:17, **no `this.notification` — ese es un método, :483**), `this.action` (:18). Inicializa `scanHistory=[]` (:25), `lastScanned={packageId,product,sourceLocation}` (:27), `useTrackingNumber=true` (:30), `uriCache=new Set()` (:31), `notificationCache=new Set()` (:32).

### setData(data) (:35-72) — el "load" real del modelo
- `this.cache = new LazyBarcodeCache(data.data.records)` (:37).
- Construye `barcode.nomenclature` + `rules`, instancia `BarcodeParser`, `BarcodeObject.setEnv(cache, parser)` (:38-47).
- `this.actionMutex = new Mutex()` (:50).
- **v19:** `this.setConfig(data.data.config)` (:52) — en v18 era asignación directa.
- `this.groupingLinesEnabled = this.groups.group_production_lot` (:53).
- Rellena `this.packageTypes` (barcode→id) desde `stock.package.type` si `group_tracking_lot` (:55-62).
- `this._createState()` (:64), `linesToSave=[]` (:65), `selectedLineVirtualId=false` (:66), `name=this._getName()` (:69), `commands=this._getCommands()` (:71).

### setConfig(config) (:74-76) — NUEVO en v19
```js
setConfig(config) { this.config = { ...this.config, ...config }; }
```
Merge en vez de reemplazo. Se llama en `setData` (:52) y **tras cada `save()`** (`this.setConfig(res.config)`, :507).

## 3. _createState() — snapshot inmutable (:2047, cuerpo hasta :2077)
```js
this.initialState = { lines };                                // :2074
this.currentState = JSON.parse(JSON.stringify(this.initialState)); // :2075 deep copy
this.groupLines();                                            // :2076
```
- `_createLinesState()` (abstracto, :1038) genera las líneas crudas; se ordenan con `_sortingMethod` (:2051) y se les asigna `sortIndex` (:2052-2054).
- Si ya existía `currentState`, preserva `wasUpdated`/`lastScannedDestination` por `virtual_id` (:2056-2072).
- **`initialState`** es la referencia contra la que `_getSaveLineCommand` calcula el diff.
- **Gotcha:** el deep copy vía JSON borra funciones/undefined/ciclos. Por eso `parentLine` se asigna **después** en `groupLines`, nunca en el estado base.

## 4. Manejo de un escaneo end-to-end
### processBarcode(barcode, options) (:685-752) — entrypoint público
1. `splitBarcode` (:670): separa múltiples URIs (regex `urn:`) o por `config.barcode_separator_regex`.
2. Anti-doble-escaneo: si `barcodes.length>1 && barcode===this._currentBarcode` → return (:691-698).
3. Filtra URIs ya en `uriCache` (:701-709).
4. Si multi-barcode y no RFID → `trigger("addBarcodesCountToProcess", n)` (:711-713).
5. Parsea cada uno a `BarcodeObject.forBarcode(bc)` + `await barcodeObject.setRecords()` (:715-720).
6. `await this._getMissingRecords()` (:722) — resuelve records faltantes de un tirón.
7. Segunda pasada `setRecords()` para los que tenían `hasMissingRecords`; descarta URN con producto faltante (:725-742).
8. **`this.actionMutex.exec(async () => { for (bo of validBarcodes) { await this._processBarcode(bo.rawValue); trigger("updateBarcodesCountProcessed"); } })`** (:744-750).
9. `postProcessBarcode()` (:757): `trigger("clearBarcodesCountProcessed")`, `notificationCache.clear()`, `delete this._currentBarcode`.

### _processBarcode(barcode) (:1300-1655) — dispatch de UN barcode
Monolito de ~355 líneas con orden ESTRICTO:
1. Construye `filters` (por `stock.lot` según línea seleccionada + `all.company_id`) (:1303-1314).
2. `barcodeData = await this._parseBarcode(barcode, filters)`; reintento sin filtros si `_shouldSearchForAnotherLot` (:1316-1324).
3. `scanHistory.unshift(barcodeData)` (:1330).
4. Si `barcodeData.match` → `trigger("flash")` (:1332-1335).
5. **Si `barcodeData.action`** → ejecuta y **return** (:1338-1341).
6. Si `packaging` → `_retrievePackagingData` (:1343-1345).
7. `_checkBarcode` (:805; **la base devuelve `true` (boolean)**; el override devuelve `{error,message,title}` → notificación danger y stop, consumido en :1348-1351).
8. Recuerda `lastScanned.product` (:1353-1356).
9. `_processLocation` (:1670) y `_processPackage` (:1686, abstracto → throw). Si `barcodeData.stopped` → return (:1364-1368).
10. Si no hay producto, intenta heredarlo de `selectedLine`/`lastScannedLine`; si el producto previo es tracked y no hubo match, asume nuevo lote (`lotName=barcode`) (:1376-1405).
11. Caso GS1 sin producto: re-`_fetchRecordFromTheCache` sin parser (:1407-1425).
12. Sin producto → `noProductToast` (:1426-1428).
13. **v19 — bloques UoM (:1533-1563):** `expressedInPackagingUom` (:1537) / `expressedInDifferentUom` (:1553) fuerzan crear línea nueva si la UoM escaneada difiere de la de la línea en productos tracked-by-lot.
14. **`_findLine`** o reuse de `currentLine` (:1438-1440).
15. Update vs create (:1566-1614): si hay línea → chequea `reserved_uom_qty` (excedente crea línea extra); si `newLotName && !_canOverrideTrackingNumber` → `createNewLine({copyOf})` (:1589-1593), si no → `updateLine`. Sin línea → `createNewLine` (con loop `createSingleLinesForPackaging`, :1621-1634).
16. `_selectLine(currentLine)` (:1645), añade URI a `uriCache` (:1652), `trigger("update")` (:1654).

### _parseBarcode (:1086) / _fetchRecordFromTheCache (:1141) / _processGs1Data (:1228)
- `_parseBarcode`: comandos → parser (GS1 devuelve lista → `_processGs1Data` por dato) → weight/price/product-alias → fallback `_fetchRecordFromTheCache`.
- `_fetchRecordFromTheCache`: mapea `recordByData` a `location` (:1152-1158, si `group_stock_multi_locations`)/`package`(**stock.package**, :1162)/`packageType`/`product`/`packaging`(**product.uom**, bajo `group_uom` :1184)/`lot`; heurística de `packageTypes` por prefijo (:1199-1212).
- Devuelve un `barcodeData` con `{barcode, match, [action|location|product|lot|package|packaging|packageType|quantity|weight|uom|owner]}`.

## 5. Sistema de eventos (EventBus)
El modelo emite con `this.trigger(name, payload)`; los consumidores usan `addEventListener(name, ev => ev.detail)` o `useBus`.

| Evento | Payload | Consumidor (main.js) | Efecto |
|---|---|---|---|
| `update` | — | :253-257 addEventListener | `render(true)` si `!uiBlocked` |
| `refresh` | detail | :252 addEventListener (+ :149 useBus global) | `_onRefreshState` |
| `history-back` | — | **:258** addEventListener | `_exit()` |
| `process-action` | — | :251 addEventListener | `_onDoAction` |
| `flash` | — | :132 useBus | animación pantalla |
| `playSound` | "error"/"success"/undefined | :133 useBus | audio (undefined→sonido "notify") |
| `blockUI`/`unblockUI` | — | :134-135 useBus | bloqueo UI |
| `addBarcodesCountToProcess` | n | :136 useBus | contador multi-scan |
| `updateBarcodesCountProcessed` | — | :139 useBus | contador |
| `clearBarcodesCountProcessed` | — | :144 useBus | reset contador |

> Nota: `playSound` sin detail reproduce el sonido "notify" (odoo_enterprise/19/stock_barcode/static/src/components/main.js::playSound), no "success".

## 6. Agrupación de líneas
- `groupedLines` getter (:223) → `groupLines()` + `_sortLine(this._groupedLines)` (el sort se aplica aquí, :227).
- `groupLines()` (:230) → `_groupedLines=[...pageLines]`; si `groupingLinesEnabled` → `_groupLines(lines, "parentLine", groupKey.bind)` (:233-237).
- **`_groupLines(lines, parentKeyString, groupKeyMethod, conditionalGrouping=true)` (:242)** — genérico: agrupa por `groupKeyMethod`, salta `lineCannotBeGrouped` (tracking==="none" o ya agrupada). Sublines de tamaño 1 se dejan planas (:260-262).
- **`_groupSublines(sublines, parentKey="parentLine")` (:1002)** — firma cambiada en v19: calcula internamente `ids`/`virtual_ids`/`totalQtyDemand`/`totalQtyDone` y los pega en la grouped line (:1017-1024). La `referenceLine` es la de menor `id` (:1008-1010). Cada subline recibe `subline[parentKey]=groupedLine` (:1025-1027) → referencia circular (por eso fuera del estado JSON-serializado).
- `groupKey(line)` (:209) default `` `${product_id.id}_${location_id.id}` `` — punto de override.
- `getLineLocation(line)` (:270) default `location_id`; `groupedLinesByLocation` (:275) agrupa por él y ordena alfabéticamente por `display_name`.

## 7. Persistencia (save + RPC)
### save() (:503-511)
```js
const { route, params } = this._getSaveCommand();   // abstracto (:998)
if (route) {
    const res = await rpc(route, params);
    this.setConfig(res.config);                      // v19 (:507)
    await this.refreshCache(res.records);            // reconstruye estado (:508)
}
this.linesToSave = [];                               // :510
```
- `_getSaveLineCommand()` (:962): para cada `virtualId` en `linesToSave`, si `line.id` → diff campo-a-campo (`_getFieldToWrite`, :954, abstracto) contra `initialState` → command `[1,id,changed]` (:988); si nueva → `[0,0,_createCommandVals(line)]` (:992). `_fieldToValue` (:958) reduce objetos a `.id`.
- `beforeQuit()` (:499) = `save()`. `_goToMainMenu`/`print`/`deleteLine(s)`/`_validate` hacen `save()` previo.
- `_markLineAsDirty(line)` (:1061): push a `scannedLinesVirtualId` (con duplicados) y a `linesToSave` (único).

### _validate() (:635-653) dentro de actionMutex (envuelto por validate() :631)
`save()` → `orm.call(resModel, validateMethod, [recordIds], {context:{barcode_trigger:true}})` (:639-641) → `doAction(action)` o `_closeValidate`.

## 8. Patrón de override de subclases
Dos vías:
1. **Subclase** de `BarcodeModel`. Debe implementar el contrato abstracto (todos hacen `throw new Error("Not Implemented")`): `getQtyDone` (:85), `getQtyDemand` (:89), `apply` (:129), `get barcodeInfo` (:133), `lineIsFaulty` (:338), `updateLineQty` (:609), `_convertDataToFieldsParams` (:817), `_getFieldToWrite` (:954), `_getSaveCommand` (:998), `_createLinesState` (:1038), `_lineIsNotComplete` (:1052), `_updateLineQty` (:2034), `_updateLotName` (:2038), `_getCompanyId` (:2087), `_getNewLineDefaultContext` (:946), `get printButtons` (:395), `_processPackage` (:1686). **Además** debe definir `this.precision` (usada en :79, definida en `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::setData`) y `_clearScanData()` (usada en :445, definida en `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_clearScanData`), que la base usa pero no declara. Y patchear `main.js:_getBarcodeModel` (:298-305) para cablear el resModel.
2. **`patch(BarcodeModel.prototype, {...})`** de `@web/core/utils/patch` para interceptar hooks del pipeline (`_parseBarcode`, `_fetchRecordFromTheCache`, `_findLine`, `_checkBarcode`, `groupKey`, `_shouldSearchForAnotherLine`, `_shouldBeExpressedInPackagingUom`…).

## 9. Cambios v18→v19 (verificados en el lado v19)
- **`stock.quant.package`→`stock.package`** en las lecturas de cache: `updateLine` (:581, :585), `_fetchRecordFromTheCache` (:1162), `_processGs1Data` (:1251), prefill de quants (:1519). Un patch v18 que lea `"stock.quant.package"` recibe `undefined` sin excepción.
- **NUEVO en `updateLine`:** resolución de paquete anidado — `package_id.parent_package_id` se resuelve a record si es number (:583-586).
- **`product.packaging`→`product.uom`**: `getGs1Filters` (:784), `_fetchRecordFromTheCache` (rama `group_uom`, :1185), `_processGs1Data` (rama product, :1276). **CORRECCIÓN:** estas ramas de packaging se gobiernan por `this.groups.group_uom` (:1184, :1275), **no** por `group_stock_packaging`. `group_stock_packaging` sobrevive en v19 pero **sólo** en `previousScannedLines` (:380). El borrador afirmaba que "el check `group_stock_packaging` sigue existiendo pero el record es product.uom" — es engañoso: el check de la rama packaging es `group_uom`.
- **`_retrievePackagingData` reescrito (:1734-1747):** usa `packaging.uom_id` (:1737) + nuevo hook `_shouldBeExpressedInPackagingUom()` (:900, default true) y conversión por factor UoM. La notificación "Wrong Unit of Measure" **no existe en v19** (grep confirmado); su presencia en v18 no se ha verificado directamente pero la ausencia en v19 sí.
- **NUEVO `_parseFloat(value)` (:78-81)** con `formatFloat`+`this.precision`. Usado en `getIncrementQuantity` (:112-115) y en `exceedingQuantity` (:1578). `this.precision` NO se define en la base (dependencia abstracta implícita).
- **NUEVO `setConfig` (:74-76)** merge; llamado en setData (:52) y tras save (:507).
- **`_groupLines`/`_groupSublines` refactor:** `_groupLines` genérico (:242); `_groupSublines(sublines, parentKey)` (:1002) ya no recibe ids/qty como args, los calcula y añade `totalQtyDemand`/`totalQtyDone`. `_sortLine` aplicado en getter `groupedLines` (:227).
- **NUEVO batch `deleteLines(lines)` (:875-894).**
- **NUEVOS:** getter `getDisplayCompletePackageBtn(line)` (:93, default false) y abstracto `completePackage(line)` (:451).
- **`displayBarcodeLines(lineId)` (:425-449):** selección con guardas `lineCanBeSelected` + `virtual_ids` (:437-440) y rama que invoca `_clearScanData()` cuando `config.restrict_scan_source_location` y la ubicación no coincide (:441-446).
- **`_processBarcode` nueva lógica UoM (:1533-1563):** `expressedInPackagingUom`/`expressedInDifferentUom`; al exceder limpia `packaging_uom_id`/`packaging_uom_qty` (:1610-1613); nuevo lot no sobrescribible → `createNewLine({copyOf})` (:1589-1593).
- **`createSingleLinesForPackaging` loop (:1621-1634):** convierte por factores UoM e itera sobre `quantity`.
- **`_findLine` con matching por `packaging_uom_id` (:1860-1869 y :1911-1913).**
- **`_getMissingRecordsParams` (:763):** añade `skip_preprocess_gs1: true` al context (:765).
- **`updateLine` (:556-559):** `product_uom_id` honra `args.uom?.id` antes de `product_id.uom_id`.

## 10. Gotchas críticos
1. **No reactivo.** Extiende `EventBus`; el estado son objetos planos. Refresco sólo vía `trigger("update")` → `odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-257` `render(true)` si `!uiBlocked`. Mutar sin `trigger("update")` = UI congelada.
2. **`_createState()` (:2075)** hace `JSON.parse(JSON.stringify(...))`: elimina funciones/undefined/ciclos. Por eso `parentLine` se asigna después, en `groupLines`.
3. **`this.precision` (:79) y `_clearScanData()` (:445) se USAN en la base pero se DEFINEN sólo en subclases** (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::setData` y `:857`). Dependencias abstractas no declaradas.
4. **Renombrado v19** `stock.quant.package`→`stock.package` y `product.packaging`→`product.uom`: patches v18 con nombres viejos → `getRecord` devuelve `undefined` sin excepción.
5. **`processBarcode` (:685) ≠ `_processBarcode` (:1300).** El primero es el entrypoint (split/uriCache/parse/missing + encola dentro de `actionMutex`); el segundo es el dispatch de un barcode.
6. **Todo serializado en `this.actionMutex`** (`@web/core/utils/concurrency`): un await largo bloquea la cola de escaneos.
7. **`notificationCache` y `uriCache` son Sets** que deduplican dentro del lote; `notificationCache.clear()` en `postProcessBarcode` (:759). Segunda notificación idéntica en el mismo lote se traga.
8. **`scannedLinesVirtualId` tiene duplicados por diseño** (push en cada `_markLineAsDirty`, :1062); `lastScannedLine` lee el último, `previousScannedLines` deduplica.
9. **`_findLine` (:1838)** es una máquina de prioridades muy sutil (línea seleccionada, ubicación actual, UoM perfecta, incompleta vs completa, `packaging_uom` nuevo en v19). Punto más frágil para overridear.
10. **`save()` (:503)** vacía `linesToSave`, muta config vía `setConfig(res.config)` y reconstruye el estado con `refreshCache→_createState`: referencias a líneas viejas quedan obsoletas; re-buscar por `virtual_id`.
11. **El lifecycle NO vive en el modelo:** no hay `setup()` ni `load()`. `Main.onWillStart()` (odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-259) hace el fetch RPC, `_getModel` (odoo_enterprise/19/stock_barcode/static/src/components/main.js::_getModel) instancia y `setData` carga. El modelo es un store pasivo dirigido por el componente.


## Correcciones aplicadas en verificación (v19)

- LOC: el fichero tiene 2094 líneas (wc -l), no 2095.
- '~85% del diff es Prettier' es una precisión fabricada y no verificable sin diff contra v18; retirada. Lo verificable: v18=1911 LOC, v19=2094 (+183, ~9,6%); identidad arquitectónica real (extends EventBus, mismo pipeline); cambios funcionales reales (renames + UoM/packaging + paquetes anidados + setConfig/deleteLines).
- main.js:_getModel() está en la línea 307, no 305.
- main.js:_getBarcodeModel() está en la línea 298 (switch 299-305, throw 'No JS model define' en :304).
- El listener addEventListener('history-back') está en odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart, no :254; el bloque de addEventListener va de :251 a :258.
- NO existe ningún método 'Main.getBarcodeData()'. La carga inicial (RPC + setData + wiring de eventos) vive en onWillStart() (odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-259); el rpc('/stock_barcode/get_barcode_data') está en la línea 228.
- El servicio de notificación se guarda como this.notificationService (constructor :17); this.notification es un MÉTODO (:483) que lo envuelve con deduplicación.
- _checkBarcode base devuelve `true` (boolean, :806); el contrato {error,message,title} es del override, consumido en _processBarcode:1348-1351.
- changedVsV18: la rama de fetch de packaging se gobierna por this.groups.group_uom (:1184, :1275, :1286), NO por group_stock_packaging. group_stock_packaging sobrevive en v19 sólo en previousScannedLines (:380). La frase del borrador sobre 'el check group_stock_packaging sigue existiendo pero el record es product.uom' es engañosa.
- playSound sin payload reproduce el sonido 'notify' (odoo_enterprise/19/stock_barcode/static/src/components/main.js::playSound), no 'success'.
- CONFIRMADAS (sin cambio) las citas clave: extends EventBus :12; setConfig :74-76 (llamado :52 y :507); _createState :2047 con JSON deep copy :2075; processBarcode :685-752; _processBarcode :1300-1655; _parseBarcode :1086; _fetchRecordFromTheCache :1141; _processGs1Data :1228; stock.package en :581/:585/:1162/:1251/:1519; product.uom en :784/:1185/:1276; _retrievePackagingData :1734-1747 con packaging.uom_id + _shouldBeExpressedInPackagingUom (:900); ausencia de 'Wrong Unit of Measure' en v19 (grep); _parseFloat :78-81; deleteLines :875-894; getDisplayCompletePackageBtn :93 / completePackage :451; _groupLines :242 / _groupSublines :1002; displayBarcodeLines :425-449; bloques UoM :1533-1563 y limpieza packaging_uom :1610-1613; createNewLine copyOf :1589-1593; _findLine :1838 con matching packaging_uom :1860-1869 y :1911-1913; skip_preprocess_gs1 :765; updateLine args.uom :556-559; save :503-511; _getSaveLineCommand :962; _validate :635-653; getActionRefresh :105-110 consumido en odoo_enterprise/19/stock_barcode/static/src/components/main.js::_onRefreshState; precision en odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::setData y _clearScanData en odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_clearScanData.
