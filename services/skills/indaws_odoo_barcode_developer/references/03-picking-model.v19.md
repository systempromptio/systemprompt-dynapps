# BarcodePickingModel (Odoo 19 Enterprise) — Referencia de subsistema

`stock_barcode/static/src/models/barcode_picking_model.js` (2526 LOC, +30% vs v18: 1950 LOC — verificado con `wc -l` en ambas versiones). Es el modelo cliente OWL que especializa `BarcodeModel` (`@stock_barcode/models/barcode_model`) para `stock.picking`. Gestiona el estado JS de las líneas de picking, la reserva/reconciliación, backorders, lotes/series, paquetes (incluidos **anidados / pack-in-pack, gran novedad v19**), put-in-pack, split de líneas y la persistencia vía el controller `/stock_barcode/save_barcode_data`.

## 1. Arquitectura y ciclo de vida

- **Herencia**: `BarcodePickingModel extends BarcodeModel` (:12). `this.lineModel = "stock.move.line"` (:15). `resModel` **NO está hardcoded**: es parámetro del constructor `super(resModel, resId, services)` (:13-14); lo único fijado a `"stock.picking"` es `this.backorderModel` (:27).
- **Carga**: `setData` (:33) → calcula `_useReservation` (`initialState.lines.some(l => !l.picked)`, :35), `useTrackingNumber` (:37), view ids (:41-44), `precision` (:45). `setConfig` (:48, **nuevo v19**) fuerza `restrict_scan_dest_location = "no"` si `!useScanDestinationLocation` (:50-51).
- **Estado de líneas**: `_createLinesState` (:1532) recorre `picking.move_line_ids` (:1535) y llama `_getMoveLineData` (:1448) por cada id, que resuelve todos los relacionales desde `this.cache` y **fabrica el campo virtual `reserved_uom_qty`** (ver sección 3).
- **Persistencia**: `save()` (:1037) → `super.save()` → `_getSaveCommand` (:1666) monta `{route:"/stock_barcode/save_barcode_data", params:{model, res_id, write_field:"move_line_ids", write_vals: comandos}}`. Los comandos los produce `_createCommandVals` (:1423) para las líneas sucias (vía `_getSaveLineCommand`, definido en `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::_getSaveLineCommand`); los campos comparados para detectar dirty los define `_getFieldToWrite` (:1612).
- **Salida**: `beforeQuit` (:1887, **nuevo v19**) → `_onExit` (:1892) manda a `stock.move.post_barcode_process` (:1910) los agregados por move (`quantity_done`, `reserved_uom_qty`) calculados sobre `initialState.lines` (:1897).
- **Validación**: `_validate` (:1129) → `button_validate` (vía `super._validate`, :1196), con gates de destino, empaquetado, backorder y firma. `_closeValidate` (:1352) detecta `state=="done"` (:1354), busca backorders (`backordersDomain` :434) y notifica.

## 2. Modelos y campos (cambios v19)

- **RENAME v19 (breaking)**: `stock.quant.package` → `stock.package` en todo el fichero. Verificado: v18 usa `'stock.quant.package'` 4 veces / `'stock.package'` 0; v19 usa `'stock.package'` 10 veces / `'stock.quant.package'` 0. Ocurrencias v19: `_getMoveLineData` :1471, :1474, :1477, :1515, :1524; `updateLine` :378, :395; `_processPackageType` :2259. Código v18 portado que use `getRecord('stock.quant.package',...)` devuelve `undefined`.
- `stock.lot` en `_getMoveLineData:1467`; pervive el alias `stock.production.lot` en `_addInConfirmationQueue:109` (inconsistencia heredada, verificada).
- **`packaging_uom_id`** nuevo en `_getMoveLineData:1463` (0 usos en v18).
- Campos de línea clave: `qty_done` (lo escaneado), `reserved_uom_qty` (reserva, **solo JS**), `quantity` (servidor), `picked`, `lot_id`/`lot_name`, `package_id`, `result_package_id`, **`outermost_result_package_id`** (contenedor externo, nuevo v19: 0 usos en v18), `is_entire_pack` (nuevo v19: 0 usos en v18), `move_id`.

## 3. Reserva y reconciliación (núcleo delicado)

`reserved_uom_qty` **no es un campo del servidor**. Se crea en `_getMoveLineData`:
- Primera carga (:1510): `reserved_uom_qty = quantity`.
- Recarga (`reloadingMoveLines`, bloque :1483-1511): si `quantity && !qty_done` → la reserva cambió, `reserved_uom_qty = quantity` (:1486-1488); si distinta UoM → conversión `(prev.reserved_uom_qty * prev.factor) / sml.factor` (:1492-1494) — **dirección INVERTIDA respecto a v18**, que hacía `(prev.reserved_uom_qty / prev.factor) * sml.factor` (v18:1247, verificado); si línea creada en barcode (sin prevLine) → `qty_done = quantity`, `reserved_uom_qty = 0` (:1503-1504).

Getters: `getQtyDone` (:200) = `qty_done`; `getQtyDemand` (:204) = `reserved_uom_qty||0`; `lineIsReserved` (:364) = `!picked && quantity`.

Completitud: `_lineIsComplete` (:1712) = `reserved_uom_qty && qty_done >= reserved_uom_qty` (+ chequeo de sublíneas tracked/agrupadas); `_lineIsNotComplete` (:1736) simétrico; `lineIsFaulty` (:944) = `_useReservation && qty_done > reserved_uom_qty` (con condición `showReservedSns || !lineIsTracked`).

**Split** (`splitLine` :786, idéntico a v18): crea `newLine` con `_createNewLine({copyOf, fieldsParams})` (:797), reparte `newLine.reserved_uom_qty = line.reserved_uom_qty - line.qty_done` (:800) y `line.reserved_uom_qty = line.qty_done` (:801), y borra `lot_id/lot_name` de la nueva (:803-804). `shouldSplitLine` (:770) solo si hay reserva y `qty_done < reserved_uom_qty`. **Call sites reales de `this.splitLine()`**: `changeDestinationLocation` (:821), `_assignEmptyPackage` (:1214), `_putInPack` (:2336). **Ojo**: `_processLocationSource` NO llama `this.splitLine`; usa `shouldSplitLine` (:1807) como gate y replica el split manualmente con `_createNewLine` (:1816, :1825).

`changeDestinationLocation` (:815) además **roba reserva** de sublíneas hermanas incompletas (:833-848) para lotes agrupados: punto típico de descuadre.

`_groupSublines` (:1686, **reescrito v19**): la firma ya no declara los parámetros de v18, pero sigue reenviando `...arguments` a `super._groupSublines` (:1687); luego itera `groupedLine.lines` y **recalcula** `reserved_uom_qty`/`qty_done` con conversión UoM `(qty * subline.factor) / groupedFactor` (:1691-1697).

## 4. Backorders

`_validate` (:1129), con `create_backorder=="ask"` (:1142): recorre líneas (usando parent line, :1148), suma `qty_done` de move lines hermanas **sin reserva** del mismo `move_id` (:1157-1165), y si sigue `< reserved_uom_qty` marca `uncompletedLines` (:1166-1168). Si hay parcial y `showBackOrderDialog`, muestra `BackorderDialog` con `onApply: () => super._validate()` (:1179-1183). `validateContext` (:20-23) lleva `display_detailed_backorder`+`skip_backorder`; si `return_id`, añade `picking_ids_not_to_backorder: this.resId` (:1189).

## 5. Lotes / series

`useTrackingNumber` (setData:37) = `use_create_lots||use_existing_lots`. `showReservedSns` (:729), `canCreateNewLot` (:725), `useExistingLots` (:1077). `_updateLineQty` (:2483) impide `qty>1` en productos serial cuando hay gestión de lotes (:2485-2489). `_updateLotName` (:2496), `_canOverrideTrackingNumber` (:2500) permite reescribir el número si misma ubicación, sin paquete/owner y `qty_done==0` (:2503-2506). Deduplicación de lotes/series escaneados en cola extra: `_addInConfirmationQueue` (:92, lógica de dedup :97-122).

## 6. Paquetes y put-in-pack (LA reescritura v19)

Paquetes anidados pasan de inexistentes a centrales: los tres campos genuinamente nuevos (`outermost_result_package_id`, `parent_package_id`, `package_dest_id`) tienen **0 ocurrencias en v18 → 32 en v19**. Incluyendo `package_type_id` (que **ya existía en v18** con 8 usos, no es nuevo): **8 → 59**. (El borrador decía "7→55", incorrecto en ambos extremos.) Tres niveles: `package_id` (origen) / `result_package_id` (destino inmediato) / `outermost_result_package_id` (contenedor externo). `package_dest_id` y `parent_package_id` modelan el árbol.

- `_moveEntirePackage` (:1770) = `record.picking_type_entire_packs`. Cuando true, `pageLines` (:985, **override nuevo v19**) filtra las líneas entire-pack (`package_id && result_package_id && is_entire_pack`, :988-990) y las presenta como `packageLines`.
- `packageLines` (:956) → `_getPackageLines` (:1627, **extraído y reescrito v19**): filtra `is_entire_pack && package_id && result_package_id && package_id.complete_name===result_package_id.complete_name` (:1628-1634), agrupa por par `(upperSourcePackageId, upperDestPackageId)` (considerando `parent_package_id`/`outermost_result_package_id`, :1638-1646), y marca `reservedPackage` (:1655).
- `completePackage` (:213, **nuevo**): togglea `qty_done = reserved_uom_qty||qty_done||packedQuantity` de todas las líneas del paquete (:219). `getDisplayCompletePackageBtn` (:168).
- `lineCanBePackedIntoPackage` (:312, **nuevo**): reglas de si una línea puede entrar en un paquete (ubicación, contenido, tipo distinto para anidar).
- Put-in-pack: `_putInPack` (:2314) hace `splitLine` de las incompletas (:2332-2337), `save()` (:2338), y `action_put_in_pack` sobre `stock.picking` (:2339). Si `selectedLine` ya empaquetada → `_putPackInPack` (:2383, **nuevo**) sobre `stock.package` (:2398). `getPackageToPackIds` (:2358, **nuevo**) elige qué paquetes anidar. `_packUnscannedLines` (:2175) y `_getCandidateLinesToPack` (:2291) cubren el caso de reabrir operación sin escaneo. `_processPackageType` (:2195) reescrito: put/pack-in-pack o `orm.write('stock.package', {package_type_id})` (:2259).
- `_processPackage` (:1941) reescrito: usa `_isPackageInPackage` (definido en `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::_isPackageInPackage`, verificado) para detectar paquete escaneado padre/igual (:1980), rama pack-in-pack vía `_putPackInPack` cuando la línea ya tiene `result_package_id` (:2063), trae quants con `stock.quant.get_stock_barcode_data_records` (:2018), soporta `outermost_result_package_id` para quants dentro de contenedor (:2157-2159) y memoriza `packedQuantity` (:2163) para entire-pack.
- `unpack` (:2411, **nuevo**): quita `outermost_result_package_id` o `result_package_id` y guarda (:2413-2419).
- Comandos barcode: `OBTPACK`→`_putInPack` (:1558), `OBTRETU`→`_returnProducts` (:1555), `OBTSCRA`→`_scrap` (:1554), `OCDCANC`→`_cancel` (:1559) (`_getCommands` :1550).

## 7. Persistencia de campos nuevos

`_createCommandVals` (:1423) y `_getFieldToWrite` (:1612) — **v19 añadió `is_entire_pack` y `outermost_result_package_id`** a ambos (verificado: `_createCommandVals` :1426/:1431, `_getFieldToWrite` :1614/:1620; v18 no los tenía en ninguna de las dos listas). Para persistir un campo custom por línea hay que añadirlo en LOS DOS: uno decide el `write_vals` enviado, otro qué cambio se detecta como dirty. Valores relacionales pasan por `_fieldToValue` (:1443, definido en `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::_fieldToValue`).

## 8. Otros

- Firma delivery: `displaySignatureButton` (:696), `shouldOpenSignatureModal` (:1115), `openSignatureDialog` (:1093), `uploadSignature` (:1081) escribe `signature` (base64) en el picking (:1085); `_validate` la fuerza antes de validar en salidas (:1192-1195).
- Devolución: `displayReturnButton` (:899), `_returnProducts` (:2423) → `action_create_return_picking` (:2424).
- Destino obligatorio: `_needsScanDestinationBeforeValidation` (:1122) usa `lines_need_destination_location` + `wasUpdated`/`lastScannedDestination` (:1123-1124).
- `_setUser` (:2441) reasigna `user_id` del picking al usuario activo (:2442-2444) en save/validate/edit/`_updateLineQty`.
- `printButtons` (:1004): añade 'Print Packages' (`action_print_packges`) si `group_tracking_lot` (:1022-1028). **NO es un cambio v19**: v18 (:880-886) tiene el mismo bloque condicional idéntico.

## 9. Puntos donde tocar rompe la reconciliación (resumen)

1. Cualquier manipulación directa de `reserved_uom_qty`/`qty_done` fuera de `splitLine`/`changeDestinationLocation`/`_getMoveLineData` descuadra backorders y completitud.
2. `_createCommandVals`/`_getFieldToWrite` desalineados → campos que no persisten o cambios no detectados.
3. Omitir el `save()` previo en put-in-pack (`_putInPack` :2338 / `_putPackInPack` :2397) → el servidor empaqueta estado obsoleto.
4. Portar la fórmula de conversión UoM de v18 (dirección invertida en v19) → reservas mal calculadas en UoM distinta.
5. Ignorar `outermost_result_package_id`/`parent_package_id` al agrupar paquetes → `packageLines`/`completePackage` inconsistentes.


## Correcciones aplicadas en verificación (v19)

- REFUTED — `printButtons` 'Print Packages' NOT a v19 change: the draft's changedVsV18 claimed printButtons:1004 conditionally adds 'Print Packages' if group_tracking_lot 'que v18 no tenía condicionado igual'. FALSE. v18 (lines 880-886) already pushes the exact same button inside an identical `if (this.groups.group_tracking_lot)` block. The v19 code (1022-1028) is byte-for-byte equivalent. This is NOT a diff; removed from changed list.
- CORRECTED — nested-package occurrence counts are wrong. Draft said 'de 7 (v18) a 55 (v19)'. Verified with grep on both files: the three genuinely-new nested fields (outermost_result_package_id, parent_package_id, package_dest_id) = 0 occurrences in v18 → 32 in v19. Including package_type_id (which ALREADY existed in v18 with 8 uses, so it is NOT anecdotic/new): 8 (v18) → 59 (v19). Corrected both endpoints and clarified package_type_id pre-existed.
- CORRECTED — `_processLocationSource` is NOT a `this.splitLine()` call site. Draft §3 listed splitLine as invoked from '_processLocationSource (:1807)'. Line 1807 is a `shouldSplitLine()` GATE; the method then replicates the split manually via `_createNewLine` (lines 1816 and 1825) and does NOT call `this.splitLine`. Real `this.splitLine()` call sites: changeDestinationLocation:821, _assignEmptyPackage:1214, _putInPack:2336. Corrected.
- CORRECTED — `resModel = 'stock.picking'` is not hardcoded in the constructor. `resModel` is a constructor parameter passed to `super(resModel, ...)` (line 13-14). Only `backorderModel = 'stock.picking'` is hardcoded (line 27). Wording adjusted.
- CLARIFIED — `_groupSublines` (:1686) 'ya no recibe args' is imprecise: the method declares no named parameters but still forwards `...arguments` to `super._groupSublines` (line 1687). Reworded to 'signature dropped the named params; still forwards arguments to super and then recomputes with UoM conversion'.
- CONFIRMED (v18 cross-check) — UoM conversion direction inversion is real: v18 line 1247 `(prevLine.reserved_uom_qty / prevLine.factor) * smlData.factor`; v19 lines 1492-1494 `(prevLine.reserved_uom_qty * prevLine.factor) / smlData.factor`. Genuine breaking change.
- CONFIRMED (v18 cross-check) — model rename real: v18 uses 'stock.quant.package' 4 times, 'stock.package' 0; v19 uses 'stock.package' 10 times, 'stock.quant.package' 0. Alias inconsistency also confirmed: 'stock.production.lot' at v19 line 109 vs 'stock.lot' at line 1467.
- CONFIRMED (v18 cross-check) — all methods the draft flagged as 'new in v19' have 0 occurrences in v18: setConfig, _addInConfirmationQueue, _confirmExtraProductDialog, getDisplayCompletePackageBtn, completePackage, lineCanBePackedIntoPackage, _getPackageLines, lastScannedPackages, _packUnscannedLines, _getCandidateLinesToPack, getPackageToPackIds, _putPackInPack, unpack, beforeQuit. Fields outermost_result_package_id, is_entire_pack, packaging_uom_id: 0 in v18.
- CONFIRMED (v18 cross-check) — removed methods real: selectExtraProduct (2 uses v18→0 v19), getDisplayIncrementPackagingBtn (1→0), isValidForBarcodeLookup (1→0).
- CONFIRMED (v18 cross-check) — v19 added `is_entire_pack` and `outermost_result_package_id` to BOTH _createCommandVals (v19 lines 1426, 1431) and _getFieldToWrite (v19 lines 1614, 1620); v18 versions of both lists lack them.
- CONFIRMED — _isPackageInPackage is defined in odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::_isPackageInPackage (verified by grep).
- CONFIRMED — file is 2526 LOC (v18: 1950 LOC, +30% is accurate).
