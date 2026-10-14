# Delta v18 → v19 para el que extiende — stock_barcode + Shop Floor

Solo lo verificado contra fuente. Cada fila: qué ancla/receta v18 rompe y su equivalente v19.

## Modelo mental (corregir antes de nada)
- **`BarcodeModel` NO es "clase plana" ni `EventTarget`** → es `extends EventBus` (`@odoo/owl`, `stock_barcode/static/src/models/barcode_model.js:9`; la clase, en `stock_barcode/static/src/models/barcode_model.js:12`, `class BarcodeModel`, v19). Expone `trigger()`/`addEventListener()`. Sigue sin ser reactivo; repintado guardado por `!state.uiBlocked` (`stock_barcode/static/src/components/main.js:253-257`, `onWillStart`, v19).
- **NO existe `Main.getBarcodeData()`** → la carga inicial vive en `onWillStart()` (`stock_barcode/static/src/components/main.js:209-259`, `onWillStart`; el RPC en `stock_barcode/static/src/components/main.js:210`). Patchear ahí, no un método inexistente.
- **`addEventListener` de ciclo de vida** están en `onWillStart` (`:251-258`), NO en `setup()`. En `setup()` van los `useBus` (`:132-149`).
- **`state.view` no tiene `printMenu` ni `editFormView`** → valores reales: barcodeLines/scrapProductPage/productPage/packagePage/infoFormView/actionsView. El menú de impresión vive dentro de `actionsView`.
- **Registry de actions NO es last-write-wins** → `add()` sin `{force:true}` lanza `DuplicatedKeyError`. Usar `patch()` o `{force:true}`.

## Fichero movido / eliminado
| v18 | v19 | Acción del extensor |
|---|---|---|
| `stock_barcode/static/src/components/manual_barcode.js` (templates `stock_barcode.BarcodeInput`/`ManualBarcodeScanner`) | Movido a `barcodes/...`, templates `barcodes.BarcodeInput`/`barcodes.ManualBarcodeScanner` | Parchear en módulo `barcodes` (afecta a TODAS las apps que lo usan) |
| `mrp_quality_check_confirmation_dialog.*` (Shop Floor) | Eliminado | `MrpRegisterProductionDialog` ahora `extends ConfirmationDialog`; reengancharse en `registerProduction`/`validate`/`MrpMenuDialog` |
| `mrp_record_line/mrp_worksheet.js/.xml` | Eliminado | Sin equivalente directo |
| carpeta JS `mrp_employee_list/` + `MrpEmployeeList`/`popupAddEmployee` | Eliminada (acción server `action_open_employee_list` SIGUE en `mrp_workorder/views/mrp_workorder_views.xml:441`, `view_mrp_production_workorder_form_view_filter_my_work_orders`, v19) | Usar `setConnectedEmployees` → `hr.employee.set_employees_connected` |

## Métodos eliminados / renombrados
| v18 | v19 | Nota |
|---|---|---|
| `selectExtraProduct` | `createNewLine`/`_addInConfirmationQueue`/`_confirmExtraProductDialog` | Producto extra no reservado |
| `getDisplayIncrementPackagingBtn` | eliminado | — |
| `isValidForBarcodeLookup` | eliminado | — |
| `_groupSublines(ids, qty, ...)` (args nombrados) | `_groupSublines()` sin args nombrados (reenvía `...arguments` a super + recomputa UoM) | Overrides con firma v18 rompen |
| GS1 `check_digit` | `get_barcode_check_digit` (`barcodes_gs1_nomenclature/static/src/js/barcode_parser.js:88`, `decimal`) | — |
| Shop Floor `emp.workorder.length` | `emp.workcenters.length` (`MGMSort`) | — |
| `stock_move.js` v18: `isLongPressable`/`onAnimationEnd`/`toggleQuantityDone` | eliminados | — |

## Renombres de modelo (core Odoo; stock_barcode solo `_inherit`)
- `product.packaging` → `product.uom` (campos: `packaging.qty`/`category_id` → `uom.uom` con `uom_id`/`factor`). Portar la lógica v18 da cantidad mal calculada sin excepción.
- `stock.quant.package` → `stock.package`. (Inconsistencia de alias residual: `stock.production.lot` en `stock_barcode/static/src/models/barcode_picking_model.js:109`, `_addInConfirmationQueue`, v19, vs `stock.lot` en `stock_barcode/static/src/models/barcode_picking_model.js:1471`, `_getMoveLineData`, v19.)

## Cambios funcionales que rompen overrides
| Tema | v18 | v19 | Impacto |
|---|---|---|---|
| Conversión UoM de `reserved_uom_qty` | `(reserved/prevFactor)*smlFactor` (v18:1247) | `(reserved*prevFactor)/smlFactor` (v19:1492-1494) | **Fórmula invertida** — NO portar |
| `_getFieldToWrite`/`_createCommandVals` | sin `is_entire_pack`/`outermost_result_package_id` | ambos añadidos (`:1614/:1620` y `:1426/:1431`) | Persistencia de campos nuevos requiere tocar ambos |
| Paquetes anidados | `outermost_result_package_id`/`parent_package_id`/`package_dest_id` = 0 usos | 32 usos; cadena `_putInPack`/`_putPackInPack`/`getPackageToPackIds`/`_getCandidateLinesToPack` (nueva) | Recetas de packaging v18 no cubren anidamiento |
| Salir sin validar | — | `beforeQuit`/`_onExit` → `stock.move.post_barcode_process` (usa `initialState.lines`) | Nuevo hook de persistencia |
| Inventario `apply()` | llamaba directo `_apply()` sin diálogo (v18:24-26) | `apply()` con `ConfirmQuantDialog` (nuevo) + atajos shouldConfirm/shouldWaitReview | Flujo de diálogo cambiado |
| `checkBeforeApply` | serial-only (v18:43) | `['lot','serial']` (`:65`) + `ApplyQuantDialog` | — |

## Ficheros/puntos NUEVOS en v19 (no existían en v18)
- `stock_barcode/static/src/js/stock_barcode_sml_form.js` (form controller `stock_barcode_sml_form`, vista `o_barcode_line_form` en productPage).
- `widgets/set_quant_stock_move_line.js` (widget `stock_barcode_quant_one2many`, `stock_barcode/views/stock_move_line_views.xml:138`, `stock_quant_barcode_kanban`).
- `components/confirm_quant_dialog.js` (`ConfirmQuantDialog`).
- Métodos picking: `setConfig`, `getDisplayCompletePackageBtn`/`completePackage`, `lineCanBePackedIntoPackage`, `_getPackageLines`, `lastScannedPackages`, `_packUnscannedLines`, `unpack`.
- Campos: `outermost_result_package_id`, `is_entire_pack`, `packaging_uom_id`.
- Rutas RPC `get_epc`/`get_epc_sgtin` (sin caller JS — dudoso como punto de extensión).
- Bundle `web.assets_unit_tests` en el manifest (ausente en v18).

## Rutas HTTP
- 7 rutas `type='json'` → `type='jsonrpc'`; 2 nuevas jsonrpc; `print_inventory_commands` sigue `http`.
- `get_specific_barcode_data` ahora usa `odoo.fields.Domain` (objetos), no listas.
- `get_specific_barcode_data_batch` borrada (ya muerta en v18).

## NO son cambios (mitos refutados — no rehacer overrides por esto)
- `printButtons` "Print Packages" bajo `group_tracking_lot`: idéntico byte-a-byte en v18 y v19. No es diff.
- La rama de fetch de packaging la gobierna `group_uom` (`:1184/:1275`), NO `group_stock_packaging` (que solo sobrevive en `previousScannedLines:380`).
- `package_type_id` ya existía en v18 (8 usos); no es campo nuevo.
- `ApplyQuantDialog` y `BackorderDialog` ya existían en v18 (no son nuevos v19).
- La acción server `action_open_employee_list` (Shop Floor) NO fue eliminada.
