## `BarcodePickingModel` — conocimiento extremo (stock_barcode v18 EE)

**Fichero:** `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js` (~1950 LOC, clase cierra en :1950). Padre: `BarcodeModel` (`@stock_barcode/models/barcode_model`). Backend acoplado: `controllers/stock_barcode.py` (ruta `/stock_barcode/save_barcode_data` :81-87), `models/stock_move.py` (`post_barcode_process` :54, `split_uncompleted_moves` :16, `_truncate_overreserved_moves` :43), `odoo_enterprise/18/stock_barcode/models/stock_quant.py:96`, `StockQuant.get_stock_barcode_data_records` (`get_stock_barcode_data_records`).

Especializacion de `BarcodeModel` para `stock.picking`. `lineModel='stock.move.line'` (:20), `resModel='stock.picking'`. Las lineas de trabajo son `stock.move.line`; los `stock.move` solo se tocan en backend via `post_barcode_process`.

### 1. reserved_uom_qty: el campo virtual nucleo de la reconciliacion
Traduccion server→cliente en `_getMoveLineData` (:1217-1273):
- `line.qty_done` = cantidad escaneada. `getQtyDone`(:190) devuelve solo `line.qty_done`.
- `line.reserved_uom_qty` = reserva original. **VIRTUAL cliente-only** (comentario :1259-1264: "keeps in memory the initial reservation clientside only, lost once the user closes"). NUNCA se escribe (no esta en `_getFieldToWrite` :1344). `getQtyDemand`(:194)=`reserved_uom_qty||0`. Toda la logica completo/backorder/split cuelga de aqui.
- `line.quantity` = campo servidor `stock.move.line.quantity`, usado para detectar cambios de reserva.

Recalculo al recargar (`reloadingMoveLines = currentState!==undefined`, :891-893):
- prevLine + `quantity && !qty_done` (if :1240): reserva cambio → `reserved_uom_qty=quantity` (:1242).
- UoM distinta: conversion por factor (:1244-1248). Misma UoM: conserva prevLine (:1251).
- linea creada en la app (sin prevLine): `qty_done=quantity`; `reserved_uom_qty=0` (:1256-1257).
- Primer load: `reserved_uom_qty=quantity` (:1263).

`_useReservation`(:43)= hay al menos una linea no picked = transferencia planificada vs inmediata. Bifurca `canBeValidate`(:566), `askBeforeNewLinesCreation`(:56), validaciones de sublocacion.

### 2. Persistencia (save → controller → move lines)
`save()`(:895) llama `_setUser` si `linesToSave.length>0` y delega en `super.save`, que arma `_getSaveCommand`(:1357): route `/stock_barcode/save_barcode_data`, `write_field='move_line_ids'`, `write_vals` x2many. `_getSaveLineCommand` (padre :909): update `[1,id,changedValues]` solo campos de `_getFieldToWrite` (:925); create `[0,0,_createCommandVals]` (:933). `_createCommandVals`(:1194) fija `quantity:qty_done` (:1207), **`picked:true` (:1203), `state:'assigned'` (:1209)**, `dummy_id:virtual_id` (:1196; recuperado tras reload en :1219). Controller `save_barcode_data`(:82) hace `write(move_line_ids)` (:86) y devuelve `_get_stock_barcode_data` fresco (:87) que re-hidrata.
**`_getFieldToWrite`(:1344) = whitelist:** `location_id`, `location_dest_id`, `lot_id`, `lot_name`, `package_id`, `owner_id`, `qty_done`, `result_package_id`. `reserved_uom_qty`/`picked`/`state` NO estan.

### 3. Reconciliacion de reservas en servidor: `_onExit` → `post_barcode_process`
`_onExit`(:1552-1571) agrega por move desde `initialState.lines`: `quantitiesByMove[moveId]={quantity_done:sum(qty_done), reserved_uom_qty:sum(reserved_uom_qty)}` (:1557-1569) y llama `orm.call("stock.move","post_barcode_process",[moveIds, quantitiesByMove])` (:1570). Backend `post_barcode_process`(stock_move.py:54): 
1. `split_uncompleted_moves`(:16): primero **resetea** los moves `picked && quantity==0` (unlink lines, `quantity=product_uom_qty`, `picked=False`, :19-24); luego para los moves picked no done/cancel hace `_create_backorder` (:25) → move picked con qty hecha + move NO picked con resto; `_action_assign` sobre los nuevos (:27); limpia move_lines `quantity==0 && picked` (:33-36); `_merge_moves` (:40). Motor real de backorders.
2. `_truncate_overreserved_moves`(:43): si `move.quantity > max(quantity_done, reserved_uom_qty)` (float_compare :51) reduce con contexto `unreserve_unpicked_only=True` (:52). Clave del dict = `str(move.id)` (:48); agrega por `_getLineMoveId`(:1548)=`line.move_id`.
`_onExit` corta si state done/cancel o `moveIds` vacio (:1553).

### 4. Split y robo de reserva (fragil)
- `shouldSplitLine`(:616): solo si `0<qty_done<reserved_uom_qty` (:617; sobre parentLine si grupo, :620). Gate de todo.
- `splitLine`(:632-653): `_createNewLine({copyOf})`; `newLine.reserved_uom_qty = old − qty_done` (:646); `old.reserved_uom_qty=qty_done` (:647); borra lote de la nueva (:649-650). **Llamado por**: `changeDestinationLocation` (:667), `_assignEmptyPackage` (:1029), `_putInPack` (:1818). (`_processLocationSource` NO llama `splitLine`: hace su propio reparto con `shouldSplitLine`+`_createNewLine`, :1475-1505.)
- `changeDestinationLocation`(:661-696): si sin lote hace `splitLine` (:666-668); para lote agrupado sin reserva roba reserva de hermana incompleta: `stolenReservation=min(remaining,qty_done)` (:683). La suma del grupo debe conservarse.
- `_processLocationSource`(:1460-1507): planificadas validan `_isSublocation`(:1515) del origen (:1462); dividen reserva restante a linea nueva si otra ubicacion (:1475-1505).
- `_assignEmptyPackage`(:1019): divide sublines parciales antes de asignar `result_package_id`, intercambia `sortIndex` (:1030).

### 5. Lotes / SN
`useTrackingNumber`(:45)=`use_create_lots||use_existing_lots`. `_incrementTrackedLine`(:1384)=`!(use_create_lots||use_existing_lots)`. `_updateLineQty`(:1893): conversion UoM (rechaza categorias incompatibles :1899), serial con lotes bloquea `qty>1`/SN (:1912-1916, return silencioso), suma `qty_done` y `_setUser` (:1918-1919). `_canOverrideTrackingNumber`(:1927): permite sobreescribir lote si misma ubicacion, sin package/owner, `qty_done===0`. `showReservedSns`(:583) depende de `picking_type.show_reserved_sns`; off si `canCreateNewLot && !useExistingLots`. `lineIsTracked` (padre :346)+`_getParentLine` agrupan lineas tracked por lote.

### 6. Paquetes / put-in-pack
`_moveEntirePackage()`(:1441)=`picking_type_entire_packs`. `packageLines`(:800) agrupa lines con `package_id && result_package_id` en pseudo-lineas `{lines, isPackageLine:true}` (:819-822); `pageLines`(:847) las excluye (:851-853). `_putInPack`(:1803): valida `group_tracking_lot` (:1805), DIVIDE lineas incompletas (`splitLine` :1818), `save()` (:1820), `action_put_in_pack` (:1821). `_processPackage`(:1593-1771): put-in-pack forzado, match `packageLines`, asigna paquete vacio, o crea linea por quant (RPC `get_stock_barcode_data_records` :1649, respeta `barcode_allow_extra_product` :1658). `_processPackageType`(:1773): put-in-pack o `write stock.quant.package.package_type_id` (:1791). `_lineNeedsToBePacked`(:1436)=`lines_need_to_be_packed && qty_done && !result_package_id` (bloquea `_validate` :964-967).

### 7. Backorders / validacion
`_validate`(:960-1013): destino escaneado (`_needsScanDestinationBeforeValidation` :949), empaquetado (:964), `_setUser` (:968), si `create_backorder==='ask'` calcula `uncompletedLines` (agrega hermanas mismo `move_id` sin reserva :984-989), `BackorderDialog.onApply` llama `super._validate` (:1001), firma si `shouldOpenSignatureModal` (:1008). `validateContext`(:25): `{display_detailed_backorder:true, skip_backorder:true}`; +`picking_ids_not_to_backorder` si `return_id` (:1005-1007). `_closeValidate`(:1131): relee state (:1132); si done busca backorders (`backordersDomain [['backorder_id','=',resId]]` :331) y notifica con botones. `canBeValidate`(:566): inmediatas siempre; planificadas exigen `barcode_validation_full` o linea con qty.

### 8. Completitud de linea
`_lineIsComplete`(:1394): `reserved_uom_qty && qty_done>=reserved_uom_qty`; package sin reserva con qty es complete (:1396); grupo falla si subline tracked con qty sin lote o subline con reserva sin cubrir (:1399-1409). `_lineIsNotComplete`(:1414): inverso con matiz parentLine. `_lineCannotBeTaken`(:1388): packed sin reserva o completa. `lineIsReserved`(:289)=`!picked && quantity`. `lineIsFaulty`(:788)=`_useReservation && qty_done>reserved_uom_qty && (showReservedSns || !lineIsTracked)`.

### 9. Restricciones de escaneo (config del picking_type, poblado en `setData` :40)
`restrict_scan_source_location`, `restrict_scan_dest_location{no,optional,mandatory}`, `restrict_scan_product`, `restrict_scan_tracking_number`, `restrict_put_in_pack{no,optional,mandatory}`, `barcode_allow_extra_product`, `lines_need_to_be_packed`, `barcode_validation_full`, `barcode_validation_after_dest_location`, `create_backorder`. Gobiernan `lineCanBeSelected`(:224), `lineCanBeEdited`(:256), `lineCanBeTakenFromTheCurrentLocation`(:269, mecanismo `needSourceConfirmation` :276-285 + `createNewLine` :63-72), `_checkBarcode`(:1068), `_mustScanProductFirst`(:1120).

### 10. Como extender
Clase ES `export default` (:17), prototipo OWL: `patch(BarcodePickingModel.prototype, {...})` con import de `@stock_barcode/models/barcode_picking_model` y `@web/core/utils/patch`. Bundle `web.assets_backend`. Persistir campo nuevo = tocar SIEMPRE 3 puntos: `_getFieldToWrite` (update), `_createCommandVals` (create), `_getMoveLineData` (re-hidratacion). Omitir uno = perdida silenciosa entre reloads.

---
**RPC salientes verificados:** `stock.move.post_barcode_process` (`_onExit` :1570) · `stock.quant.get_stock_barcode_data_records` (`_processPackage` :1649) · `stock.picking.action_put_in_pack` (`_putInPack` :1821) · `stock.picking.action_cancel` (`_cancel` :1055) · `stock.picking.action_create_return_picking` (`_returnProducts` :1835) · `stock.quant.package.write package_type_id` (`_processPackageType` :1791) · `stock.picking.write user_id/signature` (`_setUser` :1857 / `uploadSignature` :914) · `stock.picking.read state` (:1132) + `searchRead` backorders (:1135) en `_closeValidate`.


## Correcciones aplicadas en verificación

- Seccion 4: el borrador lista _processLocationSource como usuario de splitLine. FALSO: splitLine (:632) solo se llama desde changeDestinationLocation (:667), _assignEmptyPackage (:1029) y _putInPack (:1818). _processLocationSource (:1460) hace su propio reparto de reserva con shouldSplitLine + _createNewLine directamente (:1475-1505), sin llamar splitLine.
- Seccion 3: la descripcion de split_uncompleted_moves omite el primer paso real. Antes del backorder, resetea los moves 'picked && quantity==0' (moves_to_reset, stock_move.py:19-24): unlink move_line_ids, quantity=product_uom_qty, picked=False. Solo despues (self - moves_to_reset) filtrado por picked && state not in done/cancel va a _create_backorder (:25).
- Seccion 3: el orden real en split_uncompleted_moves es _create_backorder (:25) → _action_assign (:27) → limpieza de move_lines qty0/picked (:33-36) → _merge_moves (:40). El borrador enumera '_merge_moves + _action_assign' invirtiendo/agrupando el orden.
- Seccion 1: el comentario que documenta reserved_uom_qty como virtual cliente-only abarca :1259-1264 (el borrador dice :1260-1264); y la asignacion reserved_uom_qty=quantity al detectar cambio de reserva esta en :1242 (el if de guarda esta en :1240, que es lo que cita el borrador).
- Header: el fichero tiene ~1950 lineas de contenido (la clase cierra con '}' en :1950); 'de verdad' no son 1951 LOC de codigo, es un desfase trivial de conteo del total de lineas.
- Seccion 10 / extension point 5: matiz — _validate NO siempre llama super._validate() en su cuerpo. Si shouldOpenSignatureModal es true, retorna en :1010 y la validacion se difiere a uploadSignature→super._validate() (:936-937). Un override debe preservar esa ruta, no solo la llamada directa.
- Extension point de _createCommandVals: _fieldToValue no se pasa por parametro; es un metodo de padre (:905) aplicado en bucle sobre todos los valores dentro de _createCommandVals (:1211-1213). Redaccion del 'how' aclarada.
- No es un error del borrador pero se confirma: createNewLine del PADRE esta en :782 (el override hijo en :61); _createNewLine padre en :795. El borrador cita correctamente _createNewLine padre :795.
