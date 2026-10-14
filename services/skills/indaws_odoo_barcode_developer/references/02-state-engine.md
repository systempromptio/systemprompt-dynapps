## `BarcodeModel` — motor de estado base del Barcode app (Odoo 18 Enterprise)

Fichero único: `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js` (1911 líneas de código; 1912 con la última en blanco).
Es la **clase abstracta** de la que heredan `BarcodePickingModel` (`stock.picking`) y `BarcodeQuantModel` (`stock.quant`). Contiene toda la lógica genérica de estado, escaneo, agrupación, persistencia y eventos; las subclases rellenan los "huecos" (`throw new Error('Not Implemented')`) y ajustan getters de configuración.

### 1. Arquitectura y rol
- `export default class BarcodeModel extends EventBus` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:14`, `BarcodeModel`). Extiende `EventBus` de `@odoo/owl` → **el modelo NO es reactivo**; emite eventos y el componente OWL (`components/main.js`) re-renderiza a mano al recibirlos. No hay `useState` sobre el modelo.
- Se instancia en `components/main.js`: el método `_getModel()` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel`) construye `services` en `:280` y devuelve `new BarcodePickingModel(...)` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel`) si `resModel === 'stock.picking'`, `new BarcodeQuantModel(...)` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel`) si `'stock.quant'`, y si no → `throw new Error('No JS model define')` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel`). El modelo se inyecta en `useSubEnv({ model, dialog })` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:95-98`, `MainComponent.setup`), accesible como `env.model`.
- El constructor recibe `(resModel, resId, services)` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::constructor`). **`services` NO es uniforme**: en `odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel` es `{ rpc, orm, notification, action }`; `dialog` se añade a `services` **solo para `stock.picking`** (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel`), no para `stock.quant`.

**Gotcha crítico (matizado)**: el constructor (`:15`) asigna `this.dialogService = useService('dialog')` (`:17`). `useService` es un hook OWL que **solo funciona dentro de `setup()` de un componente**; funciona aquí porque `_getModel()` se invoca desde el `setup()` de `Main` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:93`, `MainComponent.model`). El modelo **obtiene dialog vía hook, NO desde `services`** — el `services.dialog` que Main pasa para picking es redundante. `orm`, `notification` y `action` sí se leen del objeto `services` (`:18-20`). **`rpc` NO se lee de `services`**: se usa el `rpc` importado a nivel de módulo (`:8`) llamándolo directamente (p.ej. `save()` `:487`); `this.rpc` nunca se asigna.

### 2. Ciclo de vida (setup → load → setData → _createState)
El estado NO se construye en el constructor. Secuencia real (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart onWillStart`):
1. `rpc("/stock_barcode/get_barcode_data", {model, res_id})` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart`) — controller Python devuelve `{data:{records, nomenclature_id, config, precision...}, groups, ...}`.
2. `this.env.model.setData(barcodeData)` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart`) y registro de listeners con `addEventListener` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart-238`).
3. `setData(data)` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::setData`):
   - `this.cache = new LazyBarcodeCache(data.data.records)` (`:39`) — cache local de todos los records; **fuente de verdad de datos ORM en cliente**.
   - Construye `nomenclature.rules` y `this.parser = new BarcodeParser({nomenclature})` (`:40-45`).
   - `BarcodeObject.setEnv(this.cache, this.parser)` (`:46`).
   - `this.actionMutex = new Mutex()` (`:49`).
   - `this.config`, `this.groups`, `this.groupingLinesEnabled = this.groups.group_production_lot` (`:50-52`).
   - `this.packageTypes` (`:54-60`) si `group_tracking_lot`.
   - `this._createState()` (`:62`).
   - `this.name = this._getName()` (`:67`), `this.commands = this._getCommands()` (`:69`).

`_createState()` (`:1864`) — corazón del estado:
- `this.record = this._getModelRecord()` (`:1865`).
- `const lines = this._createLinesState()` (**abstracto**, `:969`).
- Ordena con `_sortingMethod` (`:1868`) y asigna `line.sortIndex` incremental (`:1870`).
- **Preservación de estado entre refrescos** (`:1873-1889`): si ya existe `currentState`, copia a las nuevas líneas los flags `wasUpdated` (`:1879-1881`) y `lastScannedDestination` (`:1882-1884`) matcheando por `virtual_id`. Todo lo demás se pierde en un refresh.
- `this.initialState = {lines}` (`:1891`); `this.currentState = JSON.parse(JSON.stringify(this.initialState))` (`:1892`) — **deep copy vía JSON**. `initialState` = baseline para el diff de guardado; `currentState` = estado vivo editable.
- `this.groupLines()` (`:1893`).

**Gotcha**: el deep-copy por `JSON.parse(JSON.stringify())` significa que `initialState` y `currentState` son objetos **distintos y planos**; funciones o referencias circulares se perderían. Todo lo que metas en una línea debe ser JSON-serializable.

`refreshCache(records)` (`:475`): `this.cache.setCache(records)` + `this._createState()` → rehace TODO el estado. Se llama tras cada `save()` con `route`.

### 3. Modelo de datos: `currentState.lines`, virtual_id y "dirty tracking"
- `get pageLines()` → `this.currentState.lines` (`:362`).
- Cada línea tiene `virtual_id` (identidad estable en cliente, generado por `get _uniqueVirtualId` contador `:1846`), opcionalmente `id` (id real ORM), `sortIndex`, y campos de negocio (objetos anidados con `.id`).
- `selectedLineVirtualId` (`:64`). `get selectedLine()` (`:404`) busca por `dummy_id || virtual_id` (`:406`).
- `scannedLinesVirtualId` (array, `:47`) — historial de virtual_ids tocados; lo usan `get lastScannedLine()` (`:330`) y `get previousScannedLines()` (`:370`).
- `linesToSave` (`:63`) — array de `virtual_id` pendientes de persistir.
- `_markLineAsDirty(line)` (`:992`): push a `scannedLinesVirtualId` + añade a `linesToSave` (sin duplicar). **Único mecanismo que AÑADE una línea a "a guardar".** (`linesToSave` también se vacía en `save()` `:490` y se filtra en `deleteLine` `:839`.) Si mutas una línea sin pasar por `updateLine` (que llama `_markLineAsDirty` en `:566`), el cambio no se persiste.

### 4. Flujo completo de un escaneo (dispatch)
Entrada pública: `processBarcode(barcode, options)` (`:652`).
1. `splitBarcode(barcode)` (`:637`) — descompone múltiples barcodes (URIs `urn:...` o `config.barcode_separator_regex`).
2. **Dedup de tanda**: si `barcodes.length>1 && barcode===this._currentBarcode` → return (`:658`). `_currentBarcode` se limpia en `postProcessBarcode` (`:724`).
3. Filtra URIs ya cacheadas (`uriInCache` `:627` / `this.uriCache` Set `:33`) (`:669-676`).
4. Si tanda múltiple y no RFID → `trigger("addBarcodesCountToProcess", n)` (`:679`).
5. Parseo previo con `BarcodeObject.forBarcode(bc)` + `await barcodeObject.setRecords()` (`:684-685`), luego `_getMissingRecords()` (def `:734`) que rellena la cache (`cache.getMissingRecords`, `:736`).
6. Segunda pasada `setRecords()` (`:695`); descarta barcodes cuyo producto no existe (`:696-703`).
7. **Procesado serializado**: `this.actionMutex.exec(async () => { for(...) await this._processBarcode(bo.rawValue) })` (`:708`).
8. `postProcessBarcode()` (`:721`): `trigger("clearBarcodesCountProcessed")`, `notificationCache.clear()`, `delete this._currentBarcode`.

**`_processBarcode(barcode)` (`:1223`) — el dispatcher real** (línea a línea):
1. `filters` (`:1228-1237`): `stock.lot` por producto de la línea seleccionada + `filters.all.company_id`.
2. `barcodeData = await this._parseBarcode(barcode, filters)` (`:1239`).
3. `_shouldSearchForAnotherLot` (`:1240`, def `:1822`) — reintento sin filtros.
4. `scanHistory.unshift(barcodeData)` (`:1253`).
5. `if(match) trigger('flash')` (`:1255-1257`).
6. **Dispatch por tipo (orden fijo de menos a más ambiguo)**:
   - `barcodeData.action` → `return await barcodeData.action()` (`:1260-1262`). Cortocircuita todo.
   - `barcodeData.packaging` → `_retrievePackagingData` (`:1264`, def `:1598`).
   - `_checkBarcode(barcodeData)` (`:1269`, hook base devuelve `true` `:766`). Si `.error` → notifica danger y return (`:1270-1272`).
   - `barcodeData.product` → memoriza `lastScanned.product` (`:1274-1276`).
   - `barcodeData.lot && !product` → `_retrieveTrackingNumberInfo` (`:1278`, def `:1613`).
   - `await this._processLocation(barcodeData)` (`:1282`, def `:1534`) → si hay `location` llama `_processLocationSource` (`:1542`) que setea `this.location` y pone `barcodeData.stopped=true`.
   - `await this._processPackage(barcodeData)` (`:1283`, **abstracto** `:1550`).
   - Si `barcodeData.stopped` → return (`:1284-1288`).
   - `barcodeData.weight` → `quantity = weight.value` (`:1290-1292`).
   - **Inferencia de producto** (`:1294-1353`): toma producto de `selectedLine`/`lastScannedLine`; producto anterior tracked sin match → asume nuevo lote (`barcodeData.lotName = barcode`). Caso especial GS1 (`:1314-1345`).
   - Sin producto → `return this.noProductToast(barcodeData)` (`:1355-1356`, def `:1523`).
   - **Selección de línea**: `_findLine(barcodeData)` (`:1366`, def `:1699`) salvo `currentLine` válida.
   - **Cálculo de cantidad** (`:1371-1383`): default 1 para untracked / con SN; bloquea qty>1 en serial.
   - **Prefill owner/package** vía quants (`:1385-1453`) con `cache.getQuants` (`:1430`).
   - **Update vs Create** (`:1456-1507`): línea encontrada → `updateLine`; si excede reserva (`_shouldCreateLineOnExceed` `:847`) crea nueva con `_createNewLine({copyOf, fieldsParams})` (`:1487`). Sin línea → `createNewLine({fieldsParams})`; si `createSingleLinesForPackaging` (`:514`) crea N líneas serial.
   - `_selectLine(currentLine)` (`:1511`, def `:1617`).
   - Cachea URI (`:1514-1519`).
   - **`this.trigger('update')`** (`:1520`) → dispara re-render (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart` → `render(true)` `:235`).

`_convertDataToFieldsParams(barcodeData)` (**abstracto** `:778`) traduce el `barcodeData` genérico a los nombres de campo del modelo concreto. Obligatorio extenderlo al añadir un tipo de dato nuevo.

### 5. Sistema de eventos / notificaciones
- El modelo hereda `EventBus`; emite con `this.trigger(name, payload)`. El componente escucha con `useBus(env.model, name, ...)` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:124-131`, `MainComponent.setup`) y `addEventListener` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart-238`).
- **Eventos realmente emitidos por el modelo base** (verificado por grep de `trigger(`): `'update'` (re-render), `'flash'`, `'playSound'` (payload `"success"|"error"|null`), `'refresh'` (`:843`), `'history-back'` (`:774`), `'addBarcodesCountToProcess'`, `'updateBarcodesCountProcessed'`, `'clearBarcodesCountProcessed'`.
- **NO emitidos por el modelo base** (corrección del borrador): `'process-action'` lo emite la subclase `BarcodePickingModel` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js:1828`, `BarcodePickingModel._putInPack`), no la base. `'blockUI'`/`'unblockUI'` **NO se disparan en ningún punto de `stock_barcode/static/src`**; Main solo los escucha (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:126-127`, `MainComponent.setup`) pero nadie los emite en el módulo.
- **Notificaciones**: `notification(message, options={})` (`:464`). Deduplica vía `this.notificationCache` (Set, limpiado en `postProcessBarcode`). Si `options.type==="danger"` → `trigger("playSound","error")` (`:469-471`). Delega en `notificationService.add`. (Nota: el docstring de Odoo dice "warning" pero el código comprueba `"danger"`.)

**Gotcha**: `notificationCache` deduplica por texto exacto de mensaje durante una tanda. Dos avisos distintos con el mismo string se muestran una sola vez.

### 6. Comandos de barcode
`_getCommands()` (`:860`) devuelve `{barcode_string: fn}`. Base: `OCDMENU` → `_goToMainMenu` (`:861`), `OBTVALI` → validar (`:863`, solo si `!isDone`). En `_parseBarcode` (`:1022-1026`), si `this.commands[barcode]` existe se devuelve como `result.action` y cortocircuita `_processBarcode` (`:1260`). Para añadir un comando: override `_getCommands()` con `const c = super._getCommands(); c['OMICOD'] = this.miAccion.bind(this); return c;`.

### 7. Parseo: `_parseBarcode` y `_fetchRecordFromTheCache`
- `_parseBarcode(barcode, filters)` (`:1017`): primero comandos; luego `this.parser.parse_barcode`. GS1 devuelve lista → `_processGs1Data` por elemento (def `:1154`, loop `:1039-1045`); nomenclatura normal devuelve objeto con `type` weight/price/product. Finalmente `_fetchRecordFromTheCache` (`:1067`).
- `_fetchRecordFromTheCache(barcode, filters, data)` (`:1071`): `cache.getRecordByBarcode` y rellena `result` según `groups` (location si `group_stock_multi_locations` `:1082`, package/packageType si `group_tracking_lot` `:1090`, product siempre `:1104`, packaging si `group_stock_packaging` `:1109`, lot si `useExistingLots` `:1116`). Al final, match parcial de package types por prefijo (`:1124`).

### 8. Agrupación de líneas
- `get groupedLines()` (`:217`) → `groupLines()` (`:222`). Si `!groupingLinesEnabled` solo ordena (`_sortLine`, `:223-226`). Si activo, agrupa por `groupKey(line)` (`:203`, default `${product_id.id}_${location_id.id}`).
- `lineCannotBeGrouped(line)` (`:207`): no agrupa untracked ni ya agrupadas (`line.lines`).
- `_groupSublines(...)` (`:943`): crea **línea sintética** `groupedLine` (`Object.assign({}, referenceLine, {ids, lines, opened:false, virtual_ids})`, `:950-955`) y pone `subline.parentLine = groupedLine` (`:957`). `referenceLine` = la de menor `id` (`:947-949`).
- `_getParentLine(line)` (`:897`) → `line.parentLine`.
- `get groupedLinesByLocation()` (`:279`) agrupa `groupedLines`+`packageLines` por `getLineLocation(line)` (`:274`) y ordena alfabéticamente (`:297-300`).

**Gotcha**: `groupedLine` es efímera, se recrea en cada `groupLines()`. `parentLine` se borra al inicio del loop (`:232-235`) y se reasigna. No guardes estado en ella.

### 9. Persistencia (save → diff → RPC → refresh)
- `save()` (`:484`): `const {route, params} = this._getSaveCommand()`; si `route`, `rpc(route, params)`, luego `refreshCache(res.records)`; vacía `linesToSave` (`:490`).
- `_getSaveCommand()` (**abstracto** `:939`) — la subclase construye `{route, params}` usando `_getSaveLineCommand()`.
- `_getSaveLineCommand()` (`:909`) — **el diff engine**: para cada `virtualId` en `linesToSave`, busca la línea; si tiene `id` compara contra su `initialLine` en `initialState` los campos de `_getFieldToWrite()` (**abstracto** `:901`): escalares por `!==`, objetos por `.id` (`:921-924`); genera `[1, id, vals]` (`:930`). Sin `id` → `[0, 0, this._createCommandVals(line)]` (`:933`; `_createCommandVals` NO existe en la base, lo provee la subclase). `_fieldToValue` (`:905`) aplana objeto→`.id`.
- Puntos que llaman `save()`: `beforeQuit` (`:480`), `_validate` (`:600`), `deleteLine` con id (`:841`), `_goToMainMenu` (`:963`), `print` (`:1139`).

**Gotcha**: usa comandos x2many legacy `(0,0,..)/(1,id,..)` (no `Command.*`). Es JS que arma el payload para el controller Python. Al portar a v19 vigilar.

### 10. Validación y RPCs al servidor
- `validate()` (`:595`) → `actionMutex.exec(_validate)`. `_validate` (`:599`): `save()`, luego `orm.call(resModel, this.validateMethod, [this.recordIds], {context})` (`:603`); `context.barcode_trigger=true` (`:602`). `validateMethod='button_validate'` en picking (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::constructor`). Maneja la action con `doAction` (`onClose:_closeValidate`, `:610/:617`).
- Otras llamadas al servidor:
  - `orm.search(this.lineModel, [['id','=',lineId]])` en `displayBarcodeLines` (`:425`).
  - `orm.call(this.lineModel, this.deleteLineMethod, [line.id])` en `deleteLine` (`:842`; `deleteLineMethod='unlink'` en `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::constructor`, `lineModel='stock.move.line'` `:20`).
  - `orm.call(this.resModel, method, [[this.resId]])` en `print` (`:1145`).
  - `rpc("/stock_barcode/get_barcode_data", ...)` (`getActionRefresh` def `:94`, ruta en `:96` + carga inicial en `odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart`).
  - `rpc(route, params)` en `save` (route del `_getSaveCommand` de la subclase, típicamente `/stock_barcode/save_barcode_data`).

### 11. Contrato de herencia — métodos abstractos (throw Not Implemented)
Verificado uno a uno. Toda subclase DEBE implementar: `getQtyDone` (`:74`), `getQtyDemand` (`:78`), `apply` (`:119`), `get barcodeInfo` (`:123`), `get printButtons` (`:396`), `updateLineQty` (`:576`), `_convertDataToFieldsParams` (`:778`), `_getNewLineDefaultContext` (`:893`), `_getFieldToWrite` (`:901`), `_getSaveCommand` (`:939`), `_createLinesState` (`:969`), `_lineIsNotComplete` (`:983`), `_processPackage` (`:1550`), `lineIsFaulty` (`:342`), `_updateLineQty` (`:1851`), `_updateLotName` (`:1855`), `_getCompanyId` (`:1904`).
Y define propiedades en su constructor/`setData`: `lineModel`, `validateMethod`, `deleteLineMethod`, `precision`, `validateMessage`, `lineFormViewId` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::constructor-24,49,53`). Ojo: `_createCommandVals` (usado en el diff `:933`) también es responsabilidad de la subclase.

### 12. Patrón de extensión REAL (patch de prototipo)
Los módulos dependientes NO subclasean; hacen **monkey-patch del prototipo** con `patch()` de `@web/core/utils/patch` y `super`. Ejemplo real verificado: `stock_barcode_barcodelookup/static/src/models/barcode_model.js` → `patch(BarcodeModel.prototype, {...})` (`:9`), con `async noProductToast(barcodeData){...}` (`:17`) que decide y termina en `return super.noProductToast(barcodeData)` (`:62`) más `openProductForm` (`:65`).
Otros patch verificados: `stock_barcode_mrp`, `stock_barcode_quality_control`, `stock_barcode_product_expiry` patchean `BarcodePickingModel.prototype` (en su `models/barcode_picking_model.js`); `stock_barcode_mrp_subcontracting` patchea **AMBOS** `BarcodeModel.prototype` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:6`) y `BarcodePickingModel.prototype`.
La subclase concreta por herencia también existe: `odoo_enterprise/18/stock_barcode_picking_batch/static/src/models/barcode_picking_batch_model.js:8`, `BarcodePickingBatchModel` → `class BarcodePickingBatchModel extends BarcodePickingModel`.


## Correcciones aplicadas en verificación

- FALSO en §1: la forma de `services` NO es uniforme `{rpc, orm, notification, action, dialog}`. En odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel el objeto base es `{ rpc, orm, notification, action }`; `dialog` se AÑADE solo para `stock.picking` (odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel), no para `stock.quant` (odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel-285). Además el constructor NO lee `dialog` de `services`: lo obtiene con `useService('dialog')` (:17), de modo que `services.dialog` es redundante. Y `rpc` viene en `services` pero el modelo NO lo usa desde ahí: usa el `rpc` importado a nivel de módulo (odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:8) llamándolo directamente (p.ej. save() :487). `this.rpc` nunca se asigna.
- FALSO/INEXACTO en §5 (eventos emitidos por el modelo BASE): `process-action` NO lo emite BarcodeModel — lo emite la SUBCLASE BarcodePickingModel (odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js:1828). `blockUI`/`unblockUI` NO se emiten en NINGÚN punto de stock_barcode/static/src; Main solo los ESCUCHA vía useBus (odoo_enterprise/18/stock_barcode/static/src/components/main.js:126-127), pero nadie los dispara en el módulo. Los eventos realmente emitidos por el modelo base son: `update` (:511,:1520,:1538), `flash` (:1256,:1309), `playSound` (:470,:616,:867,:1479,:1505,:1537), `refresh` (:843), `history-back` (:774), `addBarcodesCountToProcess` (:679), `updateBarcodesCountProcessed` (:701,:712), `clearBarcodesCountProcessed` (:722).
- Preciso en §1: las instanciaciones concretas NO están en la línea 279 (esa es la firma de `_getModel()`). `new BarcodePickingModel(...)` está en odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel y `new BarcodeQuantModel(...)` en odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel; el `throw new Error('No JS model define')` en :287. El objeto `services` se construye en :280.
- Menor en §5/§4: el listener de `update` dispara `this.render(true)` en odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart (dentro del handler registrado en :233 con addEventListener), no en :233 directamente.
- Menor en §2/rpcCalls: `getActionRefresh` se define en odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::getActionRefresh (la cadena de ruta '/stock_barcode/get_barcode_data' está en :96). El draft citaba :96 como la definición.
- Incompleto en §12: `stock_barcode_mrp_subcontracting` NO patchea solo `BarcodePickingModel.prototype`; patchea AMBOS: `BarcodeModel.prototype` (odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:6) y `BarcodePickingModel.prototype` (models/barcode_picking_model.js). mrp / quality_control / product_expiry sí patchean `BarcodePickingModel.prototype` en su respectivo barcode_picking_model.js (confirmado).
