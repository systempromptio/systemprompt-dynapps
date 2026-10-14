
# BarcodeQuantModel — Ajustes de inventario en stock_barcode v19 (Enterprise)

> Verificado contra fuente v19 el 2026-07-09. Todas las citas `file:line` releídas contra
> `odoo_enterprise/19/stock_barcode/...` (799 LOC el modelo). Los diffs v18 se verificaron
> contra `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js` (751 LOC).

Ruta principal: `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js` (799 LOC). Extiende `BarcodeModel` (`@stock_barcode/models/barcode_model`). Es el modelo cliente OWL que gobierna la app barcode cuando `resModel === "stock.quant"` (Ajuste de Inventario / Physical Inventory). Contraparte del `BarcodePickingModel` (transferencias).

## 1. Arquitectura y ciclo de vida

- `constructor` (:7-13): `super(...arguments)`, fija `lineModel = this.resModel` (stock.quant), `validateMessage = _t("The inventory count has been updated")`, `validateMethod = "action_validate"` y `deleteLineMethod = this.validateMethod`. "Validar" y "borrar línea" comparten método servidor. **Confirmado.**
- `setData(data)` (:213-221): lee `user_id` (:214), `show_quantity_count` (:215), fuerza `this.countEntireLocation = false` (:216), llama `super.setData(...arguments)` (:217), y guarda `companyIds` (de `data.data.records["res.company"]`, :218-219) y `lineFormViewId = data.data.line_view_id` (:220). **Cambio v19 CONFIRMADO**: en v18 `countEntireLocation` venía del server (`data.data.count_entire_location`, v18:191); en v19 se fuerza a `false` y solo se activa por UI en runtime.
- `_createLinesState()` (:739-764): construye las líneas a partir de `this.cache.dbIdCache["stock.quant"]` (:742). Descarta quants de otro usuario (`quant.user_id && quant.user_id !== this.userId`) o con `inventory_date` futura (`quant.inventory_date > today`) en el filtro de :744. Reusa `virtual_id` previos (:750-753) y resuelve los m2o (product/uom/location/lot/package/owner) contra la cache (:754-760). **Confirmado.**
- Datos iniciales por RPC server-side: `stock.quant._get_stock_barcode_data()` (`odoo_enterprise/19/stock_barcode/models/stock_quant.py:76`, `StockQuant._get_stock_barcode_data`) que serializa quants/locations/etc. y añade `data['line_view_id'] = self.env.ref('stock_barcode.stock_quant_barcode').id` (`stock_barcode/models/stock_quant.py:107`, `StockQuant._get_stock_barcode_data`). **Confirmado.**

## 2. Flujo de "aplicar inventario" (núcleo del subsistema)

Cadena: `_validate()` / comando barcode → `apply(options)` → `checkBeforeApply()` → (diálogo) → `_apply(context)` → RPC `stock.quant.action_validate`.

- `_validate()` (:15-17): **v19 CONFIRMADO** llama `this.apply({ shouldConfirm: true })` → aplica **directo, sin diálogo**. (v18:16 llamaba `apply()` a secas → v18:24 `apply()` sin opciones.)
- `waitReview()` (:19-21) **NUEVO v19 CONFIRMADO**: `return this.apply({ shouldWaitReview: true })`. No existe en v18.
- `apply(options = {})` (:27-47) **reescrito v19 CONFIRMADO** (v18:24 `apply()` sin argumentos que solo hacía `if (checkBeforeApply()) return this._apply()`, v18:24-28). Si `checkBeforeApply()` pasa (:28):
  - Desestructura `{ shouldConfirm = false, shouldWaitReview = false }` (:29).
  - `confirm = (context) => this._apply(context)` (:30); `waitReview = () => { this.save(); this.trigger("history-back"); this.trigger("update"); }` (:31-35).
  - `shouldConfirm` → `return confirm()` (:37) — llamado **sin argumento**, luego `_apply(undefined)` → `context = {}` (sin `inventory_name`).
  - `shouldWaitReview` → `return waitReview()` (:39). **No aplica**; guarda + vuelve atrás + refresca. Deja los quants contados para revisión posterior.
  - por defecto → `this.dialogService.add(ConfirmQuantDialog, { onConfirm: confirm, onWaitReview: waitReview })` (:41-44). No hay `return` en esta rama.
- `checkBeforeApply()` (:54-90):
  - si `this.applyOn === 0` → notifica warning y `return false` (:55-59). (`applyOn` = nº de `pageLines` con `inventory_quantity_set`, :114-116.)
  - detecta SN/lotes contados con hermanos NO contados en la misma ubicación (`groupedLines`). **Cambio v19 CONFIRMADO**: filtra por `["lot","serial"].includes(gl.product_id.tracking)` (:65) — en v18 solo `gl.product_id.tracking === "serial"` (v18:43). Si hay hermanos sin contar, abre `ApplyQuantDialog` con `onApply: this._apply.bind(this)` / `onApplyAll` (marca los faltantes vía `toggleAsCounted` y aplica) y `return false` (:76-88).
  - si todo OK → `return true` (:89).
- `_apply(context = {})` (:96-112) **cambios v19 CONFIRMADOS**:
  - `await this.save()` (:97), luego `const quantIds = this.pageLines.map((quant) => quant.id)` (:98) — **envía TODOS los ids de la página**, no solo los contados (v18:76 filtraba `pageLines.filter(line => line.inventory_quantity_set)`). El filtrado real lo hace el server.
  - RPC `await this.orm.call("stock.quant", "action_validate", [quantIds], {context})` (:99) — **v19 pasa `{context}`** (v18:78 no pasaba context).
  - `notifyAndGoAhead(res)` (:100-107): si `res && res.special` → `return this.trigger("refresh")`; si no, notifica `validateMessage` y `trigger("history-back")`. Si `action && action.res_model` → `this.action.doAction(action, { onClose: notifyAndGoAhead })` (:108-109); si no, `notifyAndGoAhead()` sin arg (:111).

Server-side (`odoo_enterprise/19/stock_barcode/models/stock_quant.py:62-70`, `StockQuant.action_validate`), **CONFIRMADO literal**:
```python
def action_validate(self):
    quants = self.with_context(inventory_mode=True).filtered(lambda q: q.inventory_quantity_set)
    quants._compute_inventory_diff_quantity()
    res = quants.action_apply_inventory()
    if res:
        return res
    return True
```
El `inventory_name` del context se propaga como referencia del movimiento de stock: `odoo19/addons/stock/models/stock_quant.py:1289-1290` dentro de `_get_inventory_move_values` (def en :1253):
```python
if self.env.context.get('inventory_name'):
    res['inventory_name'] = self.env.context.get('inventory_name')
```
**Confirmado.**

## 3. ConfirmQuantDialog (NUEVO v19)

`components/confirm_quant_dialog.js` (clase :4-32, `}` en :33) + `.xml` (23 líneas). Template `stock_barcode.ConfirmQuantDialog`, título `title.translate="Confirm Inventory Adjustment"` (traducido vía QWeb).
- Estado `this.inventoryReason = useState({ value: "Physical Inventory" })` (:14) — **literal SIN `_t`, NO traducido**. (Nota: la misma cadena aparece en el modelo como `_getName()` → `_t("Physical Inventory")` en `:766-768`, ahí SÍ traducida; solo la copia del diálogo queda cruda.)
- `static props = { close, onConfirm, onWaitReview }` (:7-11), todos `Function`.
- `onConfirm()` (:17-22): `this.props.onConfirm({ inventory_name: this.inventoryReason.value })` + `this.props.close()`. El valor viaja como `context.inventory_name` hasta `action_validate` → referencia del asiento.
- `onWaitReview()` (:24-27): `this.props.onWaitReview()` + `this.props.close()`.
- `onInventoryReasonChange(ev)` (:29-31): `this.inventoryReason.value = ev.target.value` — **manejador del input** (el borrador lo omitía). Cableado en el `.xml` vía `t-on-change="onInventoryReasonChange"` (:12).
- Footer XML (3 botones): **Apply Now** (`o_confirm`, `onConfirm`, :17), **Wait a Review** (`o_confirm`, `onWaitReview`, :18), **Cancel** (`o_cancel`, `props.close`, :19).

Papel: introduce en v19 la decisión explícita "aplicar ya vs dejar para revisión" con un motivo de inventario editable, en el punto de confirmación del comando barcode.

## 4. set_quant_stock_move_line.js (NUEVO v19) — flujo de PICKING, no inventario

`widgets/set_quant_stock_move_line.js` (84 LOC). Define `StockBarcodeQuantOne2ManyField extends X2ManyField` y registra el objeto `stockBarcodeQuantOne2ManyField` como campo `stock_barcode_quant_one2many` (:79-84). Usado en `odoo_enterprise/19/stock_barcode/views/stock_move_line_views.xml:138`, `stock_move_line_product_selector` (`<field name="product_stock_quant_ids" widget="stock_barcode_quant_one2many" mode="kanban" ...>`) sobre `product_stock_quant_ids` (One2many computado en `odoo_enterprise/19/stock_barcode/models/stock_move_line.py:17`, `class StockMoveLine` con `_compute_product_stock_quant_ids` en `:52`).

**Importante**: pertenece al **flujo de picking (stock.move.line)**, NO al modelo de inventario. Su papel es hacer **seleccionables** las tarjetas de quant de stock para cambiar la **fuente** de la línea (location/lot/package/owner) tocando un quant.
- `setup()`/`onWillStart` (:13-26): `isQuantSelectable` solo si `picking_code ∈ {internal, outgoing, mrp_operation}` (:15-16) **y** el usuario tiene alguno de los grupos `stock.group_production_lot` / `stock.group_stock_multi_locations` / `stock.group_tracking_lot` / `stock.group_tracking_owner` (:19-22). (El borrador citaba erróneamente `onWillStart :14-23`; el rango real es :13-26 con los `hasGroup` en :19-22.)
- `openRecord(record)` (:36-54): si `isQuantSelectable` y cambia el quant, `record.update({location_id, lot_id, package_id, owner_id})` (:38-43,51); si el paquete resultante coincidía con el fuente y cambia, limpia `result_package_id`/`outermost_result_package_id` (:44-49).
- `_findMatchingQuant` (:59-70) resalta el quant que coincide con los datos actuales de la move line; `_highlightSelectedQuant` (:72-76).

**Cambio v19**: en v18 `product_stock_quant_ids` se mostraba como campo informativo de solo lectura (sin widget); v19 lo convierte en selector interactivo mediante este widget + el campo nuevo `outermost_result_package_id` (compute/inverse) en `stock_barcode/models/stock_move_line.py:27`, `StockMoveLine.outermost_result_package_id` (`stock_barcode/models/stock_move_line.py:30-33`, `StockMoveLine._compute_outermost_result_package_id`, y `stock_barcode/models/stock_move_line.py:79-95`, `StockMoveLine._inverse_outermost_result_package_id`), v19. **Confirmado** (la comparación exacta con v18:107-119 no se releyó, se marca como plausible).

## 5. Kanban (`static/src/kanban/`)

Vista `stock_barcode_list_kanban` (registrada en `odoo_enterprise/19/stock_barcode/static/src/kanban/stock_barcode_kanban_view.js:10`, `stock_barcode_list_kanban`), `Controller` + `Renderer`. Es la **pantalla de entrada** (lista de operaciones/inventarios) previa al client action barcode:
- `StockBarcodeKanbanController` (`stock_barcode_kanban_controller.js`) extiende `KanbanController`: escucha `barcode_scanned` (bus del `barcode` service, `useBus` :9-11); `openRecord` abre `stock_barcode.stock_barcode_picking_client_action` (:17-21); `createRecord` llama `stock.picking.action_open_new_picking` (:23-31). `_onBarcodeScannedHandler` llama `filter_on_barcode` para filtrar la lista (:42-53).
- `StockBarcodeKanbanRenderer` (`stock_barcode_kanban_renderer.js`) extiende `KanbanRenderer`: template `stock_barcode.KanbanRenderer` (:9), botón de escáner manual (`ManualBarcodeScanner`, `openManualBarcodeDialog` :19-27), y `transferTip` según grupos tracking/package/uom (:37-83). `displayTransferProtip` solo para `stock.picking` (:15).

Este kanban es compartido con el flujo de picking; el inventario entra por su propio client action pero comparte controller/renderer. **Confirmado.**

## 6. Otros métodos relevantes del modelo

- **Escaneo de paquetes** `_processPackage(barcodeData)` (:507-638): cambia `package_type_id` de un paquete existente (`orm.write("stock.package", ...)` :524); crea paquete (`stock.package.action_create_from_barcode` :543-547 y :555-559); trae quants contenidos (`stock.quant.get_stock_barcode_data_records` :569 sobre `recPackage.contained_quant_ids`); crea/incrementa líneas por cada quant. Detecta paquete ya escaneado (:601-604).
- **Contar ubicación entera** `_processLocation(barcodeData)` (:640-645) + `loadQuantsForLocation(barcodeData)` (:647-680): si `countEntireLocation`, al escanear ubicación trae todos sus quants (`stock.location.get_counted_quant_data_records` :648) y crea líneas con `inventory_quantity_set = false` (:673, a contar).
- **Creación de líneas** `_createNewLine(params)` (:359-427): rechaza no-storables (`if (!product.is_storable)` :363 → notifica y `return false`, hasta :372); si existen quants para el producto/ubicación crea una línea por quant y las mete en `initialState` para evitar falsos dirty (:397-421); si no, línea vacía (:424).
- **Defaults de línea** `_getNewLineDefaultValues(fieldsParams)` (:447-462): `inventory_date = hoy` (:450), `inventory_quantity: 0` (:451), `quantity`, `user_id`, y lógica anti-error de `inventory_quantity_set` (:455-460, comentario "to avoid tragedy"). **Distinto** de `_getNewLineDefaultContext()` (:326-337), que fija el **context** del formulario: `default_inventory_quantity: 1` (:330), `default_company_id`/`default_location_id`/`default_user_id`, `inventory_mode: true`, `barcode_view: true`, `display_default_code: false`, `hide_qty_to_count: !this.showQuantityCount` (:335). (El borrador conflaba ambos; aquí separados.)
- **Persistencia** `_getSaveCommand()` (:478-492) vía route `/stock_barcode/save_barcode_data` con `res_id: false`, `write_field: false` (cada línea = quant independiente). `_getFieldToWrite()` (:464-476) y `_createCommandVals(line)` (:339-357) definen los campos serializados.
- **Comandos barcode** `_getCommands()` (:313-318): `Object.assign(super._getCommands(), { OBTAPPLY: this.apply.bind(this), OBTWREV: this.waitReview.bind(this) })` (**OBTWREV nuevo v19**).
- **UI/lógica de línea**: `getQtyDone` (:227-229, usa `inventory_quantity` solo si set), `getQtyDemand` (:231-233, `quantity` solo si `showQuantityCount`), `getActionRefresh` (:235-242, fuerza `res_id` = ids de todas las líneas + newId), `highlightValidateButton` (:244-246), `IsNotSet` (:248-250), `lineCanBeDeleted` (:252-254), `lineIsFaulty` (:256-261), `lineIsTracked` (:263-273, trata SN sin número con qty≤1 como no-tracked), `toggleAsCounted` (:294-299), `updateLineQty` (:301-307), `_updateLineQty` (:682-708, conversión UoM y clamp de SN a 0/1 :704-706), `zeroQtyClass` (:788-790).

## 7. Diferencias clave con el modelo de PICKING

- El inventario opera sobre `stock.quant` y persiste con `res_id: false` (cada línea es un quant autónomo); el picking opera sobre `stock.move.line` ligadas a un `stock.picking`.
- El inventario NO usa UoM de packaging (`_shouldBeExpressedInPackagingUom()` = `false`, :796-798).
- La selección de "quant fuente" (widget `set_quant_stock_move_line`) es exclusiva del picking; el inventario no la usa.
- `action_validate` (inventario) aplica ajuste; el picking valida con `button_validate`/backorders.

## 8. Gotchas críticos (ver lista estructurada)

Resumen: (1) el botón Validar (`_validate`) aplica directo sin diálogo ni `inventory_name`; solo el comando barcode `OBTAPPLY` (`apply()` sin opciones) abre `ConfirmQuantDialog`. (2) "Wait a Review" (`waitReview`/`OBTWREV`) NO aplica. (3) `_apply` manda TODOS los ids de `pageLines` (filtra el server). (4) `countEntireLocation` forzado `false` en `setData`. (5) `inventoryReason='Physical Inventory'` sin traducir se convierte en referencia del movimiento. (6) no confundir el widget `set_quant` (picking) con el modelo de inventario.


## Correcciones aplicadas en verificación (v19)

- ConfirmQuantDialog: el borrador omitía el método onInventoryReasonChange(ev) (odoo_enterprise/19/stock_barcode/static/src/components/confirm_quant_dialog.js::onInventoryReasonChange-31), handler del input cableado en el .xml vía t-on-change='onInventoryReasonChange' (:12). La clase abarca :4-32 (cierre en :33). Todo lo demás del diálogo (props, onConfirm/onWaitReview, título traducido, 3 botones) CONFIRMADO.
- set_quant_stock_move_line.js: cita corregida. El borrador situaba onWillStart en ':14-23'; el rango real es :13-26, con los 4 hasGroup en :19-22. Registro del campo en :79-84 (no solo :84). openRecord :36-54 y limpieza de result_package_id/outermost_result_package_id :44-49 CONFIRMADO.
- Sección 6: el borrador conflaba _getNewLineDefaultValues (:447-462) y _getNewLineDefaultContext (:326-337). inventory_quantity:0 e inventory_date=hoy están en Values (:450-451); default_inventory_quantity:1 y hide_qty_to_count están en Context (:330,:335). Separados en el doc final.
- Matiz añadido: la cadena 'Physical Inventory' también es _getName() del modelo (:766-768) pero ahí SÍ va envuelta en _t (traducida); solo la copia del ConfirmQuantDialog (:14) es literal crudo sin _t. El borrador solo señalaba la del diálogo (correcto), pero no notaba la duplicidad.
- Server-side todo CONFIRMADO releyendo fuente: action_validate stock_quant.py:62-70 (filtered inventory_quantity_set :63, _compute_inventory_diff_quantity :64, action_apply_inventory :65); line_view_id ref en _get_stock_barcode_data stock_quant.py:107; propagación inventory_name en odoo19/addons/stock/models/stock_quant.py:1289-1290 dentro de _get_inventory_move_values (def :1253).
- stock_move_line.py CONFIRMADO: product_stock_quant_ids One2many compute en :17, _compute_product_stock_quant_ids en :52, outermost_result_package_id (compute :30 + inverse :79) en :27. Vista odoo_enterprise/19/stock_barcode/views/stock_move_line_views.xml:138 con widget='stock_barcode_quant_one2many'.
- Diffs v18 verificados contra odoo_enterprise/18/stock_barcode/.../barcode_quant_model.js (751 LOC): _validate v18:16, apply() sin opciones v18:24 (solo 'if(checkBeforeApply()) return this._apply()'), _apply v18:74 con filter inventory_quantity_set v18:76 y RPC sin context v18:78, checkBeforeApply serial-only v18:43, countEntireLocation desde server v18:191, isValidForBarcodeLookup existía v18:92 y NO está en v19. Todos los cambiosVsV18 del borrador CONFIRMADOS.
- RPC line numbers del borrador CONFIRMADOS: action_create_from_barcode :543-547/:555-559, get_stock_barcode_data_records :569, stock.package write :524, get_counted_quant_data_records :648, save_barcode_data route _getSaveCommand :478-492.
- Kanban CONFIRMADO: view stock_barcode_list_kanban registrada en odoo_enterprise/19/stock_barcode/static/src/kanban/stock_barcode_kanban_view.js:10; controller openRecord->picking_client_action (:17-21), createRecord->action_open_new_picking (:23-31), _onBarcodeScannedHandler->filter_on_barcode (:42-53); renderer template stock_barcode.KanbanRenderer (:9), displayTransferProtip solo stock.picking (:15).
