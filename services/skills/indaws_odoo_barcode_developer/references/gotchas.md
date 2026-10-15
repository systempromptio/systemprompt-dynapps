# Gotchas transversales — apps de Barcode (Odoo 18 EE)

Fallos que **rompen en silencio** una extensión (sin traceback, o con diff/UI fantasma). Verificado contra fuente v18 @ 2026-07.

1. **El modelo JS NO es reactivo.** `BarcodeModel extends EventBus` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:14`, `BarcodeModel`). Toda mutación de estado debe terminar en `this.trigger('update')` (u otro evento suscrito) o la UI no refresca, sin ningún error. `MainComponent` escucha `update` → `this.render(true)` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart`).

2. **Añadir un campo custom exige coincidir en TRES puntos** o crashea / da diffs fantasma: `_getFieldToWrite` (diff), `_createCommandVals` (create) y backend `_get_fields_stock_barcode` (carga). Si el campo no viaja en la carga y es objeto, `_getSaveLineCommand` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_getSaveLineCommand-924`) hace `initialValue.id` sobre `undefined` → TypeError.

3. **Dos listas hardcodeadas independientes cliente/servidor** de modelos-barcode: `barcodeFieldByModel` (`odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js::constructor-20`) y `_get_barcode_field_by_model` (`stock_barcode/controllers/stock_barcode.py:494`, `StockBarcodeController._get_barcode_field_by_model`). Si divergen, el matching falla en silencio. Patchear solo el cliente NO puebla `dbBarcodeCache[model]`.

4. **Serialización con `JSON.parse(JSON.stringify(record))`** (`odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js:114`, `LazyBarcodeCache.getRecord`): fechas → string, `undefined`/funciones se pierden. Many2one llega como id crudo (`load=False`): hay que enviar también el modelo relacionado o `cache.getRecord` rompe.

5. **`reserved_uom_qty` es un campo virtual cliente-only.** `splitLine` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::splitLine`) es el punto MÁS frágil: cualquier reparto que cree/destruya reserva desincroniza con la reserva real → backorders fantasma, líneas nunca completas.

6. **El objeto `services` NO es uniforme.** `odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel` arma `{rpc, orm, notification, action}`; `dialog` se añade SOLO para `stock.picking` (`:282`), NO para `stock.quant`. El modelo usa el `rpc` importado a nivel de módulo (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:8`), `this.rpc` nunca se asigna; `dialog` viene de `useService('dialog')` en el constructor (`:17`).

7. **`_onExit` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_onExit`) se llama SIN `await`** en `onWillUnmount` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:143-145`, `MainComponent.onWillUnmount`): no fiable para persistir. Usa `save()`/`beforeQuit` (con `await`) para guardar de forma síncrona.

8. **Un `patch` a `LineComponent.prototype` se PROPAGA** a `GroupedLineComponent` y `PackageLineComponent` (extends). `PackageLineComponent.props` NO incluye `editLine` ni `subline` (solo `displayUOM`, `line`, `openPackage`). No asumir props comunes entre las tres líneas.

9. **`barcode.rule.type` es un Selection FIJO** en el core `barcodes`. `setRecords` (`odoo_enterprise/18/stock_barcode/static/src/barcode_object.js:52-56`, `BarcodeObject.setRecords`) SOLO despacha type `product` y `lot`, ignora el resto en silencio. Para un type nuevo arbitrario hay que extender la Selection server-side, no basta con añadir una `barcode.rule` (dato).

10. **`QualityCheck.pass()` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_record_line/quality_check.js::pass`) NO tiene rama `measure`**: cae al default `clicked()`→dialog, y `do_measure` se procesa en el dialog (`validate()` `:71`). El backend de `quality.check` está repartido entre `mrp_workorder`, `quality_mrp_workorder`, `quality_control(_worksheet)` y `quality_mrp_workorder_worksheet`: un `test_type` nuevo puede requerir tocar varios módulos.

11. **Los estados `state.view` REALES** son `barcodeLines`/`actionsView`/`infoFormView`/`productPage`/`packagePage`/`scrapProductPage`. `printMenu` y `editFormView` NO existen (solo un comentario obsoleto en `odoo_enterprise/18/stock_barcode/static/src/components/main.js:103`, `MainComponent.setup`). El listado itera `env.model.groupedLinesByLocation` (agrupado por ubicación), NO `this.lines`.

12. **`get_specific_barcode_data_batch` (`stock_barcode/controllers/stock_barcode.py:238`, `class StockBarcodeController`) es RUTA MUERTA en v18**: ningún caller JS. No usarla como endpoint de flujos batch.

13. **`const bus` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:30`, `bus`) NO se exporta**: solo accesible vía el view_widget `stock_barcode_unlink_button`. No confundir con `env.model` (EventBus del modelo) ni `barcodeService.bus`.

14. **Los números de línea son pistas, no contrato.** Driftan 2-4 líneas incluso dentro de la misma v18 y totalmente entre majors. Casi todos los puntos de extensión útiles son patches a internals JS = impuesto de migración recurrente. Preferir seams declarativos (`t-inherit` QWeb, override de `_get_fields_stock_barcode`/`_get_groups_data`/`_get_barcode_config` en Python).

15. **`mrp_display.processValidationStack` es batch async con animación en tablet**: awaits largos bloquean la UI. Context `skip_redirection`/`skip_backorder`/`mo_ids_to_backorder` mal gestionado → backorders incorrectos. Romper `recordCacheIds` hace parpadear/reordenar las tarjetas.

16. **El `BarcodeParser` es singleton estático `__parser`** (`odoo_enterprise/18/stock_barcode/static/src/barcode_object.js::parser-71`, seteado en `setEnv` `:106`): un patch al parser debe cargar ANTES de `setEnv`. Errores de `parse_barcode` se tragan en el constructor de `BarcodeObject` (`:15-22`) → fallback silencioso sin aviso.
