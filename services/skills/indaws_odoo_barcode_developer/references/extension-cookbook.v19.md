# Cookbook de extensión — stock_barcode + Shop Floor v19 (recetas VERIFICADAS)

Todas las recetas confirmadas contra fuente v19. Al final, las recetas v18 que **YA NO aplican**.

---

## A. stock_barcode — arranque y componente principal

### A1. Cambio puntual en el componente principal
- **Dónde**: `MainComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:657`, `stock_barcode_client_action`) o `MainMenu` (`odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js:166`, `stock_barcode_main_menu`).
- **Cómo**: `patch(MainComponent.prototype, {...})`.
- **Para sustituir el componente entero**: `registry.category('actions').add(tag, NewComp, {force:true})`. **SIN `{force:true}` lanza `DuplicatedKeyError`** (`web/static/src/core/registry.js:102-107`, `Registry.validateSchema`) — NO es last-write-wins. Preferir `patch()`.
- **Ficheros**: tu módulo, assets_backend.
- **Riesgo**: con `{force:true}` importa el orden de carga de assets. Los `addEventListener` de ciclo de vida están en `onWillStart` (`:251-258`), no en `setup`.

### A2. Registrar un modelo de negocio propio (nuevo res_model escaneable)
- **Dónde**: `MainComponent._getBarcodeModel()` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js::_getBarcodeModel`) / `_getModel()` (`:307`).
- **Cómo**: `patch(MainComponent.prototype, { _getBarcodeModel() {...} })` devolviendo tu `BarcodeModel` según `this.resModel`.
- **Requisitos ineludibles**: registrar tu `ir.actions.client`, y que el controller `/stock_barcode/get_barcode_data` responda para ese modelo (si no, `setData` revienta).
- **Riesgo**: olvidar un método abstracto = error en runtime, no en carga.

### A3. Subclasear BarcodeModel completo
- **Dónde**: `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:12`, `BarcodeModel` (extends EventBus).
- **Cómo**: subclase que implementa el contrato abstracto completo (ver overview) + declarar `this.precision` (cf. `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::setData`) y `_clearScanData()` (`:857`). Cablear en `_getBarcodeModel`.
- **Esqueleto**:
```js
import { BarcodeModel } from "@stock_barcode/models/barcode_model";
export class MyBarcodeModel extends BarcodeModel {
    setData(data) { super.setData(data); /* ... */ }
    getQtyDone(line) { /* ... */ }
    _getSaveCommand() { /* x2many commands */ }
    // ...resto del contrato abstracto
}
```
- **Riesgo**: `precision`/`_clearScanData` no declarados petan en `_parseFloat` y `displayBarcodeLines`.

### A4. Hooks del pipeline de escaneo (patch de prototype)
- **Dónde / firmas verificadas**: `_parseBarcode(:1086)`, `_fetchRecordFromTheCache(:1141)`, `_processGs1Data(:1228)`, `_processBarcode(:1300)`, `_findLine(:1838)`, `_checkBarcode(:805)`, `_shouldSearchForAnotherLine(:2010)`, `_shouldSearchForAnotherLot(:2001)`.
- **Cómo**: `patch(BarcodeModel.prototype, {...})` con `super()`.
- **Contrato `_checkBarcode`**: la base devuelve `true` (boolean); el **override** devuelve `{error, message, title}`, consumido en `_processBarcode:1348-1351`.
- **Riesgo**: `_processBarcode` es un monolito con orden estricto; un `super()` mal colocado rompe flash/playSound/update. `_findLine` tiene prioridad delicada con `packaging_uom` (nuevo v19).

### A5. Agrupación de líneas
- **Dónde**: `groupKey(:209)`, `lineCannotBeGrouped(:213)`, `_groupLines(:242)`, `_groupSublines(:1002)`, `getLineLocation(:270)`.
- **Cómo**: override de `groupKey` para cambiar criterio; `getLineLocation` (default `location_id`) gobierna `groupedLinesByLocation`.
- **Riesgo**: `_groupSublines` **cambió de firma en v19** (ya no recibe `ids`/`qty` como args; reenvía `...arguments` a super y recomputa con conversión UoM) → overrides v18 se rompen. `subline[parentKey]=groupedLine` crea referencia circular que `JSON.stringify` NO tolera.

### A6. Persistir un campo nuevo de stock.move.line (picking)
- **Dónde (patrón canónico)**: `_getFieldToWrite` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_getFieldToWrite`) **Y** `_createCommandVals` (`:1423`). Hay que tocar **ambos**.
- **Cómo**: override de cada uno llamando super + `push`/`Object.assign`.
- **Esqueleto**:
```js
patch(BarcodePickingModel.prototype, {
    _getFieldToWrite() { const f = super._getFieldToWrite(); f.push("x_my_field"); return f; },
    _createCommandVals(line) { const v = super._createCommandVals(line); v.x_my_field = line.x_my_field; return v; },
});
```
- **Riesgo**: en uno solo → o no persiste o no se detecta el cambio. Relacionales pasan por `_fieldToValue` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::_fieldToValue`, objeto→id). Si el campo custom no está en `initialState.lines` y la línea ya tiene `id`, el diff accede a `initialLine[field]` undefined → `TypeError` si es objeto.

### A7. Reserva clientside / split de líneas (picking) — ALTO RIESGO
- **Dónde**: `_getMoveLineData` bloque `reserved_uom_qty` (`:1483-1511`); `splitLine(:786)` / `shouldSplitLine(:770)`.
- **Cómo**: override para condicionar cuándo se parte o qué hereda la nueva línea. `super.splitLine` reparte `reserved_uom_qty` (`:800-801`) y borra `lot_id/lot_name` de la newLine a propósito (`:803-804`).
- **Call sites reales de `this.splitLine()`**: `changeDestinationLocation:821`, `_assignEmptyPackage:1214`, `_putInPack:2336`. **`_processLocationSource` NO llama splitLine** — usa `shouldSplitLine` como gate (`:1807`) y replica manualmente con `_createNewLine` (`:1816/:1825`).
- **Riesgo**: `reserved_uom_qty` gobierna splitLine, backorders, `_lineIsComplete`, highlightValidate. **La fórmula de conversión UoM se invirtió en v19** (`:1492-1494: qty*prevFactor/smlFactor`, v18 era `/prevFactor*smlFactor`): NO portar la de v18. Un split mal hecho descuadra reservas → backorders erróneos.

### A8. Put-in-pack / pack-in-pack anidado (picking) — NUEVO v19
- **Dónde**: `_processPackage(:1941)`, `_putInPack(:2314)`, `_putPackInPack(:2383)`, `getPackageToPackIds(:2358)`, `_getCandidateLinesToPack(:2291)`.
- **Cómo**: override `_putInPack` (RPC `action_put_in_pack` sobre `stock.picking:2339`) o `_putPackInPack` (sobre `stock.package:2398`).
- **Riesgo**: cruza `outermost_result_package_id`/`parent_package_id`/`package_dest_id`. `_putInPack` hace `save()` ANTES de la RPC (`:2338`): si el override omite el save, el servidor empaqueta estado obsoleto.

### A9. Producto extra no reservado (picking) — NUEVO v19
- **Dónde**: `createNewLine(:63)`, `askBeforeNewLinesCreation(:55)`, `_addInConfirmationQueue(:92)`, `_confirmExtraProductDialog(:146)`.
- **Cómo**: override `askBeforeNewLinesCreation` para cambiar cuándo se confirma.
- **Riesgo**: `_addInConfirmationQueue`/`_confirmExtraProductDialog` **sustituyen a `selectExtraProduct`** (eliminado). `needSourceConfirmation` (guard en `:65`) es frágil: devolver algo distinto de `false`/`super` rompe el guard. `_confirmExtraProductDialog` crea 1 línea y hace `updateLine` para el resto (`:154-158`): no asumas 1 línea por producto.

### A10. Gate de validación / salida (picking)
- **Dónde**: `_validate(:1129)` + `_needsScanDestinationBeforeValidation(:1122)`; `beforeQuit(:1887)` / `_onExit(:1892)`.
- **Cómo**: override `_validate` con checks previos y `super._validate()` al final (`:1196`); override `beforeQuit` (llamar super) para persistencia extra al salir.
- **Riesgo**: `_onExit` usa `initialState.lines` (`:1897`), no `currentState` → campos solo-currentState no se envían. La firma corta el flujo (`return` sin validar `:1194`) y valida en callback (`:1107-1109`): duplicar la llamada valida dos veces. `beforeQuit`/`_onExit` son nuevos en v19.

---

## B. stock_barcode — inventario (BarcodeQuantModel)

### B1. Validación previa al apply
- **Dónde**: `checkBeforeApply()` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::checkBeforeApply-90`).
- **Cómo**: patch; devolver `false` + notificación para bloquear, o super y encadenar.
- **Riesgo**: corre ANTES de `ConfirmQuantDialog`. Devolver `true` saltándose super pierde el chequeo de SN/lotes no contados (`ApplyQuantDialog`, `:76-88`).

### B2. Context/post-proceso del apply
- **Dónde**: `_apply(context)` (`:96-112`).
- **Cómo**: override para inyectar claves de context antes de la RPC `action_validate` (`:99`), o post-procesar en `notifyAndGoAhead` (`:100-111`).
- **Riesgo**: envía TODOS los `pageLines` ids (`:98`); el filtrado por `inventory_quantity_set` es server-side (`stock_barcode/models/stock_quant.py:56-57`, `StockQuant.action_validate`). `context={}` por defecto → sin `inventory_name` el movimiento sale sin referencia.

### B3. Flujo de diálogo del apply
- **Dónde**: `apply(options)` (`:27-47`).
- **Riesgo**: asimetría real — `_validate(:16)` usa `shouldConfirm:true` y NO muestra diálogo ni fija `inventory_name`; el comando `OBTAPPLY(:315)` llama `apply()` sin opciones y SÍ muestra diálogo.

### B4. ConfirmQuantDialog (NUEVO v19)
- **Dónde**: `odoo_enterprise/19/stock_barcode/static/src/components/confirm_quant_dialog.js:4-32`, `ConfirmQuantDialog` (+ `.xml`).
- **Cómo**: patch del componente o su template `stock_barcode.ConfirmQuantDialog`. `onConfirm(:17-22)` construye `{inventory_name}`. Handler del input: `onInventoryReasonChange(ev)` (`:29-31`), cableado en xml `t-on-change` (`:12`).
- **Riesgo**: `inventoryReason` arranca con literal `'Physical Inventory'` **SIN `_t`** (`:14`) → referencia del asiento de stock (`stock/models/stock_quant.py:1289-1290`, `StockQuant._get_inventory_move_values`). Traducir si el cliente lo requiere. (El `_getName()` del modelo `:766` sí va traducido.)

### B5. Filtro de líneas de quant
- **Dónde**: `_createLinesState()` (`:739-764`), itera `cache.dbIdCache['stock.quant']` (`:742`), filtra por `user_id` e `inventory_date <= hoy` (`:744`).
- **Riesgo**: cambiar el filtro puede hacer aparecer líneas que el server luego no valida.

### B6. Defaults de línea nueva de quant
- **Dónde**: `_getNewLineDefaultValues()` (`:447-462`, con `inventory_quantity:0`/`inventory_date=hoy` en `:450-451`) y `_getNewLineDefaultContext()` (`:326-337`, con `default_inventory_quantity:1`/`hide_qty_to_count` en `:330/:335`). **Son dos métodos distintos** (no conflar).
- **Riesgo**: la lógica de `inventory_quantity_set` (`:455-460`, comentario "to avoid tragedy") evita marcar como contado un quant con múltiples SN/lotes; alterarla provoca ajustes por error.

### B7. Comandos de barcode del quant model
- **Dónde**: `_getCommands()` (`:313-318`).
- **Cómo**: `Object.assign` sobre `super._getCommands()` (patrón OBTAPPLY/OBTWREV); si no, pierdes los heredados. Handlers van bind al modelo.

### B8. Widget de selección de quant origen (picking) — NUEVO v19
- **Dónde**: `StockBarcodeQuantOne2ManyField` (`widgets/set_quant_stock_move_line.js`), registrado en `category('fields')` como `'stock_barcode_quant_one2many'` (`:79-84`), `openRecord(:36-54)`.
- **Cómo**: patch del widget. Usado en `odoo_enterprise/19/stock_barcode/views/stock_move_line_views.xml:138`, `stock_move_line_product_selector` sobre `product_stock_quant_ids`.
- **Riesgo**: es del flujo de PICKING (`stock.move.line`), NO del quant/inventario. `isQuantSelectable` depende de `picking_code in ['internal','outgoing','mrp_operation']` (`:15-16`) y 4 grupos (gating `onWillStart :13-26`, hasGroup `:19-22`). Limpia `result_package_id`/`outermost_result_package_id` al cambiar de paquete (`:44-49`).

---

## C. stock_barcode — capa UI OWL

### C1. Personalizar la línea (LineComponent)
- **Dónde**: `odoo_enterprise/19/stock_barcode/static/src/components/line.js:4`, `LineComponent` (export default) + sub-templates en `line.xml` (`LineTitle:4`, `LineUpperButtons:23`, `LineQuantity:39`, `LineLowerButtons:62`, `LineOwner:109`, `LineLot:116`, `LineDestinationLocation:132`, `LineComponent:148`).
- **Cómo**: `patch(LineComponent.prototype, {...})` (import del default export) y/o `t-inherit` de la sub-template concreta (p.ej. `stock_barcode.LineLowerButtons`). Nuevos getters bindando a `env.model.<método>`; handlers → `env.model.<accion>()` + `env.model.trigger('update')`.
- **Riesgo**: NO está en registry → solo patch de prototype (frágil). `GroupedLineComponent`/`PackageLineComponent` **extienden** `LineComponent`: el patch base se propaga a los tres. El botón editar llama `props.editLine` (=`MainComponent.onOpenProductPage`), no un método propio. `imageSource` usa `image_128` (`:10`).

### C2. Hook de plantilla upperButtons/lowerButtons
- **Dónde**: `odoo_enterprise/19/stock_barcode/static/src/components/line.xml:19`, `stock_barcode.LineTitle` y `:58` (`t-call="{{upperButtons || 'stock_barcode.LineUpperButtons'}}"`).
- **Cómo**: en una plantilla padre que **tú** instancies con `LineComponent`, `t-set` `upperButtons`/`lowerButtons` con el nombre de tu sub-template (o `false` para ocultar).
- **Riesgo**: el flujo estándar NUNCA setea estas vars; solo sirve si TU componente instancia `LineComponent` con ellas en scope. Evita xpath sobre el core.

### C3. Field widgets vía registry
- **Dónde**: `digipad`→`view_widgets`(`odoo_enterprise/19/stock_barcode/static/src/widgets/digipad.js:191`, `digipad`), `image_preview`→`fields`(`odoo_enterprise/19/stock_barcode/static/src/widgets/image_preview.js:33`, `image_preview`, descriptor `imageClickEnlarge`), `set_reserved_qty_button`→`fields`(`:42`), `stock_barcode_quant_one2many`→`fields`(`:84`), `stock_barcode_unlink_button`→`view_widgets`(`odoo_enterprise/19/stock_barcode/static/src/components/main.js:53`, `stock_barcode_unlink_button`).
- **Cómo**: re-registrar con la misma key (gana por orden de carga del bundle) o `patch` de la clase. Descriptor v19: objeto `{component, extractProps({attrs}, dynamicInfo)}`.
- **Riesgo**: `set_reserved_qty_button.extractProps` es CONDICIONAL (`{}` si no hay `attrs.field_to_set`). `image_preview` NO define extractProps (hereda de `ImageField`). `digipad._fetchPackagingButtons` hace ORM directo a `uom.uom`/`product.product` (modelo UoM v19 `uom_ids`) → NO retro-portable a v18.

### C4. MainComponent y sub-env
- **Dónde**: `odoo_enterprise/19/stock_barcode/static/src/components/main.js:657`, `stock_barcode_client_action`; static components `:77-87`; `useSubEnv({model, dialog})` `:103-106`; `t-inherit` de `stock_barcode.MainComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/main.xml:10`, `stock_barcode.MainComponent`).
- **Cómo**: `patch(MainComponent.prototype)` + ampliar `static components` + `t-inherit`.
- **Riesgo**: para lógica de negocio patchea la **clase de modelo**, no el componente. `env.model` hereda EventBus (no store reactivo): sin `trigger('update')` no repinta.

### C5. Diálogos lanzados desde el modelo
- **Dónde/lanzamiento**: `ConfirmQuantDialog` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js:41`, `BarcodeQuantModel.apply`), `ApplyQuantDialog` (`:77`), `BackorderDialog` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_validate`), `ExtraProductDialog` (`:127`). `ProductImageDialog` desde `LineComponent.onClickImage` (`odoo_enterprise/19/stock_barcode/static/src/components/line.js::onClickImage`, `image_1024`). `CountScreenRFID` declarativo en `odoo_enterprise/19/stock_barcode/static/src/components/main.xml:11`, `stock_barcode.MainComponent`.
- **Cómo**: para los del modelo, patchea el método del modelo o la clase/template del diálogo.
- **Riesgo**: `ExtraProductDialog` hereda `ConfirmationDialog` y su template hace `xpath //p position='replace'` (`odoo_enterprise/19/stock_barcode/static/src/components/extra_product_dialog.xml:5`, `template`) → frágil. Sus callbacks reales: `{products, cancel: extraProducts.clear, confirm: _confirmExtraProductDialog.bind, close: extraProducts.clear}`. `CountScreenRFID.onWillUnmount` (`:46-49`) NO limpia `timeInterval` (`:22`) → fuga real; al extender, limpia timers.

---

## D. Cache cliente y parsing barcode/GS1

### D1. Soportar un modelo nuevo escaneable
- **Dónde**: `LazyBarcodeCache.barcodeFieldByModel` y `gs1LengthsByModel` (`odoo_enterprise/19/stock_barcode/static/src/lazy_barcode_cache.js::constructor-24`).
- **Cómo**: patch/override de la clase para añadir la entrada (y longitud GS1 si aplica). En servidor: declarar `_barcode_field` e implementar `_get_fields_stock_barcode` (opcional `_get_stock_barcode_specific_data`).
- **Riesgo**: objetos literales estáticos en constructor → un patch mal hecho pisa a todos los modelos. **Colisión GS1: `_setBarcodeInCacheForGS1` conserva el PRIMER record cacheado y solo avisa por consola** (el texto del log "The last one will be used" es engañoso). Ambigüedad silenciosa.

### D2. Nuevos tipos de BarcodeObject
- **Dónde**: `barcode_object.js` — `setRecords/fetchProduct/fetchTrackingNumber` (`:39-56,72-79,81-102`); `setEnv(cache,parser)` (`:59-69`, llamado en `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::setData`).
- **Riesgo**: `setRecords` solo reacciona a type `'product'`(`:50`) y `'lot'`(`:52`); otros se ignoran. **Singleton estático de clase**: `__parser`/`__cache` compartidos; dos vistas de barcode vivas se pisan. `mappingRawBarcodeToObject` (Map estático `:106`) NUNCA se limpia → fuga + datos obsoletos. `forBarcode(:113-118)` devuelve instancia cacheada (un raw ya visto no se re-parsea).

### D3. Extender el parser GS1
- **Dónde**: el parser real vive en `barcodes/js/barcode_parser.js`; GS1 lo **MONKEY-PATCHEA** con `patch(BarcodeParser)` (`:8`) y `patch(BarcodeParser.prototype)` (`:22`) — **NO es subclase**.
- **Cómo**: añadir `barcode.rule` (nomenclatura) o parchear el prototype. **NO subclasar** (una subclase no se usaría; `BarcodeModel` instancia la clase importada de `@barcodes`).
- **Riesgo**: `parse_barcode` va en try/catch en `BarcodeObject` (`:12-20`): si tu patch lanza, se traga el error y cae al flujo clásico. Método de dígito de control: `get_barcode_check_digit` (`:88`), NO `check_digit`.

### D4. Adjuntar records relacionados al escanear (servidor)
- **Dónde**: `_get_stock_barcode_specific_data()` en el modelo; el controller lo llama por `hasattr` (`odoo_enterprise/19/stock_barcode/controllers/stock_barcode.py:226`, `StockBarcodeController._get_records_fields_stock_barcode`, def `_get_records_fields_stock_barcode` en `:223`). Ejemplo `stock_barcode/models/product_uom.py:16-19`, `ProductUom._get_stock_barcode_specific_data`.
- **Riesgo**: `hasattr`-gated (nombre exacto o fallo silencioso). Si el método peta, rompe el fetch. Las claves deben coincidir con `barcodeFieldByModel` o el cache no indexa por barcode.

---

## E. Shop Floor (mrp_display)

### E1. Añadir campos al modelo de datos
- **Dónde**: `MrpDisplay._makeModelParams` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display.js::_makeModelParams-508`) + `fieldsStructure` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_action.js::fieldsStructure`).
- **Cómo**: `patch(MrpDisplay.prototype)` extendiendo `params.config.activeFields` tras el super, con relaciones anidadas.
- **Riesgo**: los dos puntos están **desacoplados**; añadir a `fieldsStructure` sin declarar el activeField anidado → no carga (silencioso). Orden de asignación importa (related sobre related).

### E2. Nuevo tipo de quality check
- **Dónde**: `QualityCheck.clicked(:116)`/`doActionAndNext(:147)`/`icon(:61)`/`showQty(:98)`.
- **Cómo**: patch/subclase; añadir case a `clicked()`/`icon()` para el `test_type` nuevo + render en template `mrp_workorder.QualityCheck` (`t-inherit`).
- **Riesgo**: **`StockMove extends QualityCheck`**: override sin default/super afecta también a líneas de consumo.

### E3. Comportamiento de consumo
- **Dónde**: `StockMove.clicked(:171)`/`addMoveLine(:122)`/`editUntrackedMove(:164)` (`mrp_record_line/stock_move.js`).
- **Riesgo**: `addMoveLine(:122)` y el import de `MrpSelectQuantDialog(:7)` están marcados `// TODO remove in master` → deuda que cambiará en el próximo major. (El TODO vive en `stock_move.js`, NO en `mrp_select_quant_dialog.js`.)

### E4. Cierre / validación de MO
- **Dónde**: `MrpDisplayRecord.validate(:312)`/`registerProduction(:71)`/`subRecordProps(:213)`.
- **Riesgo**: `validate()` maneja backorders, borrado in-place de `model.root.records` (`:368-373`) y `underValidation`; override incompleto deja UI inconsistente. RPC de cierre es un único `orm.call` en `:351`.

### E5. Menú contextual del panel
- **Dónde**: `MrpMenuDialog` (`dialog/mrp_menu_dialog.js:9`) + template `mrp_workorder.MrpDisplayMenuDialog`.
- **Cómo**: patch para métodos de acción; `t-inherit` para botones. Métodos: `action_add_component`(call `:62`), `action_add_byproduct`(`:69/70`), `action_add_step`(`:202`), `action_propose_change`(`:206`).
- **Riesgo**: `callAction` usa context `{from_shop_floor}` (`:41`) — variante distinta a `from_shopfloor` de calidad (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/quality_check.js:151`, `QualityCheck.result`). Confundirlas desactiva el modo Shop Floor en server.

### E6. Empleados conectados
- **Dónde**: hook `useConnectedEmployee` (`mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js:8`, `useConnectedEmployee`) + `MrpEmployeeDialog`. `setConnectedEmployees(:195)` → RPC `hr.employee.set_employees_connected(:200)`.
- **Riesgo**: el flujo v18 (`mrp_employee_list`/`popupAddEmployee`/`MrpEmployeeList`) ya no existe. `MGMSort(:84-116)` usa `emp.workcenters.length`.

### E7. Escaneo custom en Shop Floor
- **Dónde**: `MrpDisplay._onBarcodeScanned(:186)`/`_onProductBarcodeScanned(:213)`.
- **Riesgo**: ignora prefijos OBT/OCD y diálogos abiertos; respeta ese guard o tu handler dispara con modales abiertos.

---

## RECETAS v18 QUE YA NO APLICAN (retiradas)

1. **Patch a `stock_barcode/static/src/components/manual_barcode.js`** → el fichero NO existe en v19 (`BarcodeInput`/`ManualBarcodeScanner` migraron al módulo base `barcodes`, con templates renombrados `barcodes.BarcodeInput`/`barcodes.ManualBarcodeScanner`). Parchear ahora en `barcodes` afecta a TODAS las apps que lo usan.
2. **Patch/t-inherit a `mrp_quality_check_confirmation_dialog.*`** (Shop Floor) → **eliminado en v19**. `MrpRegisterProductionDialog` pasó de `MrpQualityCheckConfirmationDialog` (v18) a `ConfirmationDialog` (v19). Reengancharse en `MrpDisplayRecord.registerProduction`/`validate` o `MrpMenuDialog`.
3. **Patch a `selectExtraProduct`** (picking) → método eliminado; usar `createNewLine`/`_addInConfirmationQueue`/`_confirmExtraProductDialog`.
4. **Override de `_groupSublines(ids, qty, ...)` con la firma v18** → v19 no recibe esos args (reenvía `...arguments` y recomputa). Rompe.
5. **Portar la fórmula de conversión UoM de `reserved_uom_qty` de v18** → invertida en v19.
6. **Registrar la action tag con `add()` esperando last-write-wins** → peta con `DuplicatedKeyError`; usar `patch()` o `{force:true}`.
7. **`registry.add` re-registro del parser GS1 asumiendo subclase / método `check_digit`** → es monkey-patch; método `get_barcode_check_digit`.
8. **Overrides del flujo JS `mrp_employee_list`/`MrpEmployeeList`/`popupAddEmployee`** (Shop Floor) → carpeta eliminada; usar `setConnectedEmployees`/`set_employees_connected`. (La acción server `action_open_employee_list` SÍ sigue en `odoo_enterprise/19/mrp_workorder/views/mrp_workorder_views.xml:441`, `action_open_employee_list`.)
9. **Patch a `mrp_worksheet.js/.xml` y a `mrp_record_line/mrp_worksheet`** (Shop Floor) → eliminados en v19.
