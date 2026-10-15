Recetas VERIFICADAS (solo `confirmed`/`corrected`). Todos los anclajes de línea son Odoo 18 EE (`stock_barcode` / `mrp_workorder`). Los números de línea DRIFTAN entre versiones: úsalos como pista, verifica siempre contra fuente antes de emitir.

---

## A. stock_barcode — añadir un botón/acción a una línea (bajo riesgo)

**Dónde:** `components/line.xml`, sub-templates `stock_barcode.LineLowerButtons` (:57), `LineUpperButtons` (:22), `LineTitle` (:4), `LineQuantity` (:38), `LineOwner` (:98), `LineLot` (:105), `LineDestinationLocation` (:121).

**Patrón (declarativo, sin JS de re-registro):**
```xml
<t t-inherit="stock_barcode.LineLowerButtons" t-inherit-mode="extension">
  <xpath expr="//div[...]" position="inside">
    <button t-on-click="() => this.env.model.miMetodo(line)">Mi acción</button>
  </xpath>
</t>
```
El handler debe existir en el modelo (`patch(BarcodeModel.prototype,...)`) o en el componente parcheado.

**Riesgo:** el impacto depende de QUÉ sub-template extiendes — no todos los componentes t-llaman todos:
- `LineQuantity`, `LineDestinationLocation` → los TRES (Line/Grouped/Package).
- `LineTitle` → Line+Grouped (Package tiene cabecera propia).
- `LineLowerButtons` → Line+Grouped (Package lo oculta con `lowerButtons=false`).
- `LineOwner` → Line+Package (Grouped NO).
- `LineLot` → Line+Grouped.

**Alternativa sin xpath (sustituir grupo entero de botones):** en scope del `t-call` (`odoo_enterprise/18/stock_barcode/static/src/components/line.xml:18`, `stock_barcode.LineTitle`/`:53`, `t-call="{{lowerButtons || 'stock_barcode.LineLowerButtons'}}"`):
```xml
<t t-set="lowerButtons" t-value="'mi_modulo.MisBotones'"/>  <!-- o t-value="false" para ocultar -->
```
`odoo_enterprise/18/stock_barcode/static/src/components/package_line.xml:21`, `stock_barcode.PackageLineComponent` ya lo hace. **Coste/riesgo:** pierdes los botones estándar (+1/-1/fulfill/delete) salvo que los repliques. Coste migración: BAJO (seam declarativo).

---

## B. stock_barcode — añadir un paso/validación al procesado de escaneo (medio)

**Dónde:** `BarcodeModel.prototype._processBarcode` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_processBarcode`).
```js
import { patch } from "@web/core/utils/patch";
import BarcodeModel from "@stock_barcode/models/barcode_model";
patch(BarcodeModel.prototype, {
  async _processBarcode(barcode) {
    // ...pre-lógica...
    const r = await super._processBarcode(barcode);
    // ...post-lógica... (OJO: super puede haber hecho early-return)
    return r;
  }
});
```
**Riesgo:** `_processBarcode` tiene múltiples early-return (action `:1261`, check.error `:1271`, stopped `:1287`, noProductToast `:1356`, SN duplicado `:1398`); tu post-lógica puede no ejecutarse. Corre bajo `actionMutex`. Mutaciones que no pasen por `updateLine→_markLineAsDirty` NO se persisten, y sin `this.trigger('update')` la UI no refresca. Coste migración: MEDIO (parche a internals).

Para procesar tipos GS1 nuevos o location/package: `_processLocation` (`:1534`), `_processPackage` (`:1550`, abstracto), `_fetchRecordFromTheCache` (`:1071`), `_convertDataToFieldsParams` (`:778`, abstracto).

---

## C. stock_barcode — añadir un campo (LA receta más pedida) — 3 PUNTOS OBLIGATORIOS

### C.1 Picking (stock.move.line)
Patchea LOS TRES a la vez, o el campo se pierde/crashea:
1. `BarcodePickingModel._getFieldToWrite` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::_getFieldToWrite`) → whitelist del diff.
2. `_createCommandVals` (`:1194`) → vals de creación (el bucle `:1211-1213` aplica `_fieldToValue`, método del padre `:905`). **NO alterar** `picked:true` (`:1203`) ni `state:'assigned'` (`:1209`): rompe `split_uncompleted_moves`.
3. `_getMoveLineData` (`:1217`) → re-hidratación desde cache.
4. Backend: `_inherit stock.move.line` + override `_get_fields_stock_barcode` (`stock_barcode/models/stock_move_line.py:93`, `StockMoveLine._get_fields_stock_barcode`) devolviendo `super()+['mi_campo']`.

### C.2 Quant (ajuste inventario)
1. `BarcodeQuantModel._getFieldToWrite` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::_getFieldToWrite-436`).
2. `_createCommandVals` (`:307-325`).
3. Backend `stock.quant._get_fields_stock_barcode` (`stock_barcode/models/stock_quant.py:126-142`, `StockQuant._get_fields_stock_barcode`).

**Crash real si faltan puntos** (`_getSaveLineCommand` base `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_getSaveLineCommand-924`): para una línea con `id` existente, si el campo es objeto compara `fieldValue.id !== initialValue.id`; si el campo NO viaja en la carga (no está en `_get_fields_stock_barcode`), `initialLine[field]` es `undefined` → `initialValue.id` lanza `TypeError`. Y sin estar en carga, el diff marca cambios fantasma. Deben COINCIDIR: whitelist diff + create + carga backend. Coste migración: MEDIO.

---

## D. stock_barcode — capturar foto / firma (confirmado, distinto por caso)

- **Firma (picking):** ya existe el flujo. `_validate` (`:960`) → si `shouldOpenSignatureModal` retorna en `:1010` y DIFIERE a `uploadSignature → super._validate()` (`:936-937`). Un override de `_validate` DEBE preservar esa ruta, no solo la llamada directa (`:1012`). Campo `signature` ya en `_get_fields_stock_barcode` del picking.
- **Foto de producto (visualización):** `ProductImageDialog` (`odoo_enterprise/18/stock_barcode/static/src/components/product_image_dialog.js:6`, `ProductImageDialog`), abierto vía `env.dialog.add` desde `odoo_enterprise/18/stock_barcode/static/src/components/line.js::onClickImage` (`onClickImage`). Es el ÚNICO diálogo abierto desde un componente con `env.dialog.add`.
- **Captura de imagen en form embebido:** widget `image_preview` (`odoo_enterprise/18/stock_barcode/static/src/widgets/image_preview.js:35`, `image_preview`), referenciado en `odoo_enterprise/18/stock_barcode/views/stock_move_line_views.xml:39`, `stock_move_line_product_selector` y `odoo_enterprise/18/stock_barcode/views/stock_inventory_views.xml:30`, `stock_quant_barcode`. Opera sobre el RelationalModel estándar (`record.update`), NO sobre BarcodeModel.

---

## E. stock_barcode — nuevo comando de escaneo (custom "OMICOD") (bajo)

```js
patch(BarcodeModel.prototype, {
  _getCommands() {
    const c = super._getCommands();       // base: OCDMENU (:861), OBTVALI (:863, solo !isDone)
    c['OMICOD'] = this.miFn.bind(this);
    return c;
  }
});
```
En el quant el base añade `OBTAPPLY` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::_getCommands-285`). Se resuelve como `result.action` en `_parseBarcode` (`:1017`, resolución ~`:1022`) y cortocircuita `_processBarcode` (`:1260-1262`). **Riesgo:** el string no debe colisionar con nomenclatura/GS1; `Object.assign` sobreescribe silenciosamente. Si tu fn muta líneas, termina en `this.trigger('update')`. Coste: BAJO.

---

## F. stock_barcode — registrar un modelo nuevo en la cache de barcode (medio)

Para que `getRecordByBarcode` reconozca un modelo nuevo hacen falta DOS listas hardcodeadas + atributo:
1. **Cliente:** `patch` del constructor de `LazyBarcodeCache` para añadir clave a `barcodeFieldByModel` (`odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js::constructor-20`).
2. **Servidor:** override de `_get_barcode_field_by_model` (`odoo_enterprise/18/stock_barcode/controllers/stock_barcode.py:494`, `StockBarcodeController._get_barcode_field_by_model`, return `:504` filtra por `hasattr(_barcode_field)`) para incluir el modelo.
3. Definir `_barcode_field` en el `_inherit` + `_get_fields_stock_barcode` + **enviar el modelo en el payload** (`_get_stock_barcode_data`).

**Riesgo:** patchear solo el cliente NO puebla `dbBarcodeCache[model]`; `setCache` solo crea la clave si el modelo llega en algún payload (`:47-49`). Si nunca llega, `getRecordByBarcode` con modelo lanza en `:197` (rama sin onlyInCache/fetchLater). Cliente y servidor son listas INDEPENDIENTES: si divergen, el matching falla en silencio. Coste: MEDIO.

**Para GS1 con padding:** además `gs1LengthsByModel` (`:21-26`: product.product/product.packaging=14, stock.location=13, stock.quant.package=18) — patch del constructor o `_setBarcodeInCacheForGS1` (`:383-404`). Solo actúa si hay UNA `barcode.nomenclature` en el blob (`:28`) Y `is_gs1_nomenclature` Y el modelo está en `gs1LengthsByModel` (`:82`). Colisiones de padded solo emiten `console.log` (`:398-402`).

---

## G. stock_barcode — nuevo "type" de barcode (matizado / server-side obligatorio)

`BarcodeObject.setRecords` (`odoo_enterprise/18/stock_barcode/static/src/barcode_object.js:41-100`, `BarcodeObject.setRecords`) SOLO despacha `type==='product'`→`fetchProduct` y `type==='lot'`→`fetchTrackingNumber` (`:52-56`); **ignora en silencio** package/location/owner/otros. El `type` lo produce `barcode.rule.type`, un Selection **FIJO en el core `barcodes`**.
- Type estándar ya soportado por la nomenclatura → basta `patch(BarcodeObject.prototype, {setRecords})` + añadir la `barcode.rule` (dato).
- Type nuevo arbitrario → hay que EXTENDER el Selection de `barcode.rule.type` server-side, además del patch. Coste: ALTO.

Enganche de parsing GS1: `patch(BarcodeParser.prototype, {...})` en `@barcodes/js/barcode_parser.js` (`parseBarcodeNomenclature:186`, `match_pattern:111`), replicando lo que hace `barcodes_gs1_nomenclature/static/src/js/barcode_parser.js:24`, `patch`. **Gotcha:** el parser es singleton estático `__parser` (`odoo_enterprise/18/stock_barcode/static/src/barcode_object.js::parser-71`, seteado en `setEnv :106`); el patch debe cargar ANTES de `setEnv`. Errores de `parse_barcode` se tragan en el constructor de `BarcodeObject` (`:15-22`) → fallback silencioso.

---

## H. stock_barcode — manejar producto/lote no reservado (confirmado)

- **Producto desconocido al escanear:** `patch` de `noProductToast` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::noProductToast`) → decide si maneja o `return super.noProductToast(barcodeData)`. Patrón real: `stock_barcode_barcodelookup/static/src/models/barcode_model.js:17`, `noProductToast`, con `super` en `:62` y `openProductForm` en `:65`.
- **Producto no reservado en picking:** `createNewLine` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::createNewLine`) intercepta con `ExtraProductDialog`; defaults en `_getNewLineDefaultValues` (`:1325`, `reserved_uom_qty:0`, hereda `move_id` si mismo producto `:1327-1330`, serial+packaging pone `reserved_uom_qty:1` `:1338-1340`). Flag `barcode_allow_extra_product`. **Riesgo:** líneas creadas aquí nacen `reserved_uom_qty=0`; forzar reserva sin move con reserva real desincroniza.

---

## I. stock_barcode — diálogo custom (confirmado)

```js
// Desde un componente:
this.env.dialog.add(MiDialog, {...});
// Desde el modelo (NO tiene env — pasar callbacks .bind(this)):
this.dialogService.add(MiDialog, { onApply: () => super._validate(), ... });
```
El modelo obtiene `dialogService` vía `useService('dialog')` en el constructor (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::constructor`). Ejemplos reales: `ExtraProductDialog` (`:94`), `BackorderDialog` (`:998`), `ApplyQuantDialog` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::checkBeforeApply`). Para heredar layout de confirmación: `t-inherit="web.ConfirmationDialog" t-inherit-mode="primary"` (como `odoo_enterprise/18/stock_barcode/static/src/components/extra_product_dialog.xml:4`, `template`). **Riesgo:** el diálogo debe recibir y llamar `close()`. Coste: MEDIO.

---

## J. stock_barcode — widget de campo en form embebido (confirmado)

Registrar en `registry.category('fields'|'view_widgets').add(...)` con `{component, extractProps}` y referenciar por herencia en los form de stock.move.line/stock.quant/stock.scrap. Ejemplos: `digipad` (`odoo_enterprise/18/stock_barcode/static/src/widgets/digipad.js:182`, `digipad`, referenciado en `odoo_enterprise/18/stock_barcode/views/stock_move_line_views.xml:52`, `stock_move_line_product_selector`), `set_reserved_qty_button` (`:42`), `image_preview` (`:35`). **Estos widgets viven en el form embebido (vista productPage), NO en el client action OWL**: operan sobre RelationalModel estándar con `record.update`. `extractProps` mapea attrs `field_to_edit`/`fulfilled_at` a camelCase (`odoo_enterprise/18/stock_barcode/static/src/widgets/digipad.js:175-180`, `digipad`). Coste: MEDIO.

---

## K. stock_barcode — imprimir etiqueta (corregido)

Los `action_print_*` de stock_barcode son SOLO tres (`stock_barcode/models/stock_picking.py:76`, `StockPicking.action_print_barcode`; `:79`, `StockPicking.action_print_delivery_slip`; `:82`, `StockPicking.action_print_packges`): `action_print_barcode`, `action_print_delivery_slip`, `action_print_packges`. `do_print_picking` NO pertenece a stock_barcode (es core `odoo18/addons/stock/models/stock_picking.py:1158`, `Picking.do_print_picking`; se llama desde JS `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::printButtons,1296` pero se hereda del core). **Para añadir un botón de impresión nuevo:** override del `action_print_*` en Python NO basta — el botón se define en JS (getter `printButtons`), hay que `patch` del getter. Coste: MEDIO.

---

## L. stock_barcode — modelo JS nuevo / reemplazar la pantalla entera (alto)

**Reemplazo total:** `registry.category('actions').add('stock_barcode_client_action', TuComponente, {force:true})` (el registry lanza si el key existe sin `{force:true}`).

**Subclasear MainComponent** (`export default`, `odoo_enterprise/18/stock_barcode/static/src/components/main.js:66`, `MainComponent`):
```js
import MainComponent from "@stock_barcode/components/main";
class Sub extends MainComponent { setup(){ super.setup(); } }
Sub.components = { ...MainComponent.components, Nuevo };  // static NO se merge-hereda
registry.category('actions').add('stock_barcode_client_action', Sub, {force:true});
```
Overrides típicos: `_getModel()` (`:279`), `lineFormViewProps` (`:245`), `scrapViewProps` (`:573`), `_exit()` (`:418`).

**Modelo JS custom** (patrón real `stock_barcode_picking_batch/static/src/models/barcode_picking_batch_model.js:8`, `BarcodePickingBatchModel`): `class MiModel extends BarcodePickingModel` (implementar abstractos §M) + `patch(MainComponent.prototype, {_getModel})`. **Gotcha services:** el objeto se arma en `odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel` como `{rpc, orm, notification, action}` — SIN `dialog`; `dialog` se añade SOLO para stock.picking (`:282`), NO para quant (`:285`). Aun así funciona porque el constructor hace `useService('dialog')`. Replica el ensamblado exacto, NO "arregles" dialog. El constructor del quant es `constructor(params)` con `super(...arguments)` (nombre engañoso; el base es `constructor(resModel, resId, services)`, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::constructor`) → si declaras constructor propio, reenvía `...arguments`. `rpc` viene en services pero el modelo usa el `rpc` importado a nivel de módulo (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:8`), `this.rpc` nunca se asigna. Coste: ALTO (herencia frágil ante updates).

---

## M. stock_barcode — abstractos a implementar en un modelo custom

Lanzan `throw new Error('Not Implemented')` en la base y DEBEN implementarse en picking/quant o subclase:
`getQtyDone` (`:74`), `getQtyDemand` (`:78`), `lineIsFaulty` (`:342`), `updateLineQty` (`:576`), `_updateLineQty` (`:1851`), `apply` (`:119`), getter `barcodeInfo` (`:123`), `_convertDataToFieldsParams` (`:778`), `_getFieldToWrite` (`:901`). `_createCommandVals` NO existe en la base (proveerlo en subclase). **Refutado del draft:** `getDisplayIncrementBtn` (`:82`→true), `lineCanBeDeleted` (`:338`→`!getQtyDemand`), `lineCanBeEdited` (`:1576`→true) SÍ tienen implementación base — parchear la base surte efecto sobre ambas subclases.

---

## N. stock_barcode — override server-side (contrato Python, bajo/medio)

- Más campos/datos al front: `_get_stock_barcode_data` (picking `:85` / quant `:75`), `_get_stock_barcode_specific_data` (product.product `:39`, product.packaging `:11`, stock.quant `:147`, stock.lot `:17`).
- Nuevo flag de config por tipo de operación: campo en `stock.picking.type` + `_get_barcode_config` (`stock_barcode/models/stock_picking_type.py:86`, `StockPickingType._get_barcode_config`); leer en JS `this.config.<flag>`.
- Grupos que condicionan ramas del modelo: `_get_groups_data` (`stock_barcode/controllers/stock_barcode.py:475`, `StockBarcodeController._get_groups_data`); leer `this.groups.<x>`.
- Post-proceso de moves en validación: `post_barcode_process` / `split_uncompleted_moves` (`stock_barcode/models/stock_move.py:16`, `StockMove.split_uncompleted_moves`) / `_truncate_overreserved_moves` (`:43`) — override en `_inherit stock.move`. **Riesgo alto:** `split_uncompleted_moves` primero resetea moves `picked && quantity==0` (`:19-24`) y LUEGO `_create_backorder`(`:25`)→`_action_assign`(`:27`)→limpieza(`:33-36`)→`_merge_moves`(`:40`).
- Inventario: `barcode_write` (`stock_barcode/models/stock_quant.py:17`, `class StockQuant`, `@api.model`, `inventory_mode=True`, crea `stock.lot` on-the-fly desde `lot_name`; formato posicional `[[1,id,{}],[0,0,{}]]`, mapea `dummy_id`→quant) y `action_validate` (`:56`).
- Nuevos targets del menú principal: `_try_open_*` (`stock_barcode/controllers/stock_barcode.py:316-431`, `StockBarcodeController._try_open_lot`) encadenados en `main_menu` (`:15`); cubrir GS1/no-GS1, respetar orden y `has_group`.

**Riesgo transversal server-side:** el front debe leer la clave nueva (patch JS) o se ignora; Many2one llega como id crudo (`load=False`) → hay que enviar también el modelo relacionado; los valores deben ser JSON-safe (`getRecord` hace `JSON.parse(JSON.stringify(record))`, `odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js:114`, `LazyBarcodeCache.getRecord` → fechas a string, undefined/funciones se pierden).

---

## O. stock_barcode — reparto de reserva / split (PUNTO MÁS FRÁGIL — alto)

`splitLine` (`:632`), `shouldSplitLine` (`:616`), `changeDestinationLocation` (`:661`), `_putInPack` (`:1803`), `_assignEmptyPackage` (`:1019`). `splitLine` SOLO se llama desde `changeDestinationLocation` (`:667`), `_assignEmptyPackage` (`:1029`) y `_putInPack` (`:1818`); `_processLocationSource` (`:1460`) hace su propio reparto sin `splitLine`. **Riesgo:** cualquier reparto que cree/destruya reserva desincroniza `reserved_uom_qty` (virtual cliente-only) de la reserva real → backorders fantasma, líneas nunca completas, over-reserva no corregida. Mantén la suma de `reserved_uom_qty` del grupo constante. Coste: ALTO — probar serial/lote/multi-location.

---

## P. mrp_display — añadir campo al Shop Floor (bajo)

`patch(MrpDisplayAction.prototype)` sobre el getter `fieldsStructure` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_action.js::fieldsStructure-122`): `super` + push del campo. Related anidado además en `MrpDisplay._makeModelParams` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display.js::_makeModelParams-522`). Campos html/pesados → `fieldsManuallyFetched` (`:125-131`) para no inflar cada read del batch. Campo ausente llega `undefined` sin error. Coste: BAJO.

---

## Q. mrp_display — nuevo test_type de control de calidad (alto, multi-módulo)

Dispatch repartido en TRES capas — olvidar una deja el check a medias:
1. JS: `patch(QualityCheck.prototype)` en `pass()` switch (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_record_line/quality_check.js::pass`) e `icon` getter (`:86`). `pass()` maneja `instructions`/`passfail`→`_pass`, `register_production`, `print_label`, `picture`; TODO lo demás (incl. `measure`) cae a `this.clicked()`→diálogo.
2. JS: `patch(MrpQualityCheckConfirmationDialog.prototype.validate)` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/dialog/mrp_quality_check_confirmation_dialog.js::validate`; `measure`→`do_measure` en `:71`). Ojo: el patch al dialog padre afecta a `MrpRegisterProductionDialog` (herencia).
3. Backend: extender `quality.check` con el test_type y su `action_*`. **Métodos repartidos entre módulos quality_*:** `action_print`/`action_next`/`action_continue`/`action_generate_serial_number_and_pass` en `mrp_workorder/models/quality.py`; `action_pass_and_next`/`action_fail_and_next` en `quality_mrp_workorder:82/87`; `do_measure` en `quality_control` (override en `quality_mrp_workorder:44`); `action_worksheet_check` en `quality_control_worksheet:88` + `quality_mrp_workorder_worksheet:11`; `action_open_quality_check_wizard` en `quality_control(_worksheet)`. Coste: ALTO.

---

## R. mrp_display — hook en la validación en lote (medio)

`patch(MrpDisplay.prototype.processValidationStack)` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display.js::processValidationStack`) super+lógica, o `validate()` de `MrpDisplayRecord` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_record.js::validate`). Servidor: override `mrp.production.button_mark_done` (`odoo18/addons/mrp/models/mrp_production.py:2117`, `MrpProduction.button_mark_done`) / `mrp.workorder.do_finish` / `pre_record_production`. **Riesgo:** batch async con animación — awaits largos bloquean la tablet; context `skip_redirection`/`skip_backorder`/`mo_ids_to_backorder` mal → backorders incorrectos. Coste: MEDIO.

---

## S. mrp_display — escaneo de barcode en Shop Floor (medio)

`patch(MrpDisplay.prototype._onBarcodeScanned)` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display.js::_onBarcodeScanned`): resolver el propio primero y delegar a `super`. Sub-handlers: `_onProductionBarcodeScanned` (`:225`), `_onWorkorderBarcodeScanned` (`:237`). Orden de resolución: `MO.name` → `wo.barcode` → empleado (`get_employee_barcode`, `mrp_workorder/models/mrp_workcenter.py:36`, `MrpWorkcenter.get_employee_barcode`, llamado en `:217`). Prefijos OBT/OCD (lotes) se ignoran con retorno vacío (`:204`). Coste: MEDIO.

---

## T. mrp_display — acción en el menú contextual (medio)

`patch(MrpMenuDialog.prototype)` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/dialog/mrp_menu_dialog.js:11`, `MrpMenuDialog`) + `t-inherit` del template `mrp_workorder.MrpDisplayMenuDialog`. `callAction(method)` (`:32`) hace `doAction`+reload con context `{from_shop_floor:true}` y **cierra el diálogo** (`:47`) — acciones que deban permanecer abiertas necesitan otro patrón. `MrpMenuDialog` SOLO lanza `MrpWorkcenterDialog` y `MrpQualityCheckSelectDialog` (`MrpLogNoteDialog` lo abre `MrpDisplayRecord.editLogNote`; `MrpWorksheetDialog` es aparte). `proposeChangeForCheck` (`:161`) → `action_add_step`/`action_propose_change`; `callAddComponentAction` (`:50`) → `action_add_component`. Coste: MEDIO.

---

## U. mrp_display — empleado conectado / PIN (medio)

Envolver/reemplazar `useConnectedEmployee` (`mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js:10`, `useConnectedEmployee`) en `MrpDisplay.setup` (patch). Devuelve `{connected, all, admin}` reactivo. Backend `hr.employee`: `login`(:28)/`logout`(:35)/`pin_validation`(:21)/`get_all_employees`(:155)/`remove_session_owner`(:51)/`stop_all_workorder_from_employee`(:116); `start_employee`(:45)/`stop_employee`(:49). PIN se valida en servidor. Coste: MEDIO.
