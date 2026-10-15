# stock_barcode — Capa UI de componentes OWL (Odoo 19 Enterprise)

Referencia para extender la app cliente de barcode en v19. Rutas `file:line` verificadas contra `odoo_enterprise/19/stock_barcode/static/src/`.

## 1. Arquitectura general

La app de barcode es un **client action** OWL: `registry.category("actions").add("stock_barcode_client_action", MainComponent)` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:657`, `stock_barcode_client_action`). `MainComponent` orquesta todo y **crea el modelo de negocio** en `_getModel()` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js::_getModel`), que elige la clase con `_getBarcodeModel()` según `res_model` (`stock.picking`→`BarcodePickingModel`, `stock.quant`→`BarcodeQuantModel`; `odoo_enterprise/19/stock_barcode/static/src/components/main.js::_getBarcodeModel-305`). Lo expone vía `useSubEnv`:

```
useSubEnv({ model, dialog: this.dialog });   // odoo_enterprise/19/stock_barcode/static/src/components/main.js:103-106
```

A partir de ahí **todos** los componentes hijos acceden a:
- `env.model` — instancia de clase JS (`BarcodePickingModel` / `BarcodeQuantModel`) cuyo base **hereda `EventBus` de `@odoo/owl`** (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:12`, `BarcodeModel`, import en `:9`). **NO es `EventTarget`** (aunque su API `addEventListener`/`trigger` lo parezca). Es el único canal de datos y de acciones de negocio, y trae su propio `this.dialogService = useService("dialog")`.
- `env.dialog` — el `dialog` service.

**Punto crítico**: `env.model` **no es reactivo**. Los componentes de presentación leen su estado en *getters* y el re-render depende de eventos del `EventBus`. En `MainComponent`:
- En `setup()`: `useBus(this.env.model, "flash"|"playSound"|"blockUI"|"unblockUI"|"addBarcodesCountToProcess"|"updateBarcodesCountProcessed"|"clearBarcodesCountProcessed", …)` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:132-149`, `MainComponent.setup`).
- En `onWillStart()` (NO en `setup`): `this.env.model.addEventListener("process-action"|"refresh"|"update"|"history-back", …)` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-258`). El de `"update"` hace `if (!this.state.uiBlocked) this.render(true)` (`:253-257`).

Por eso, cualquier mutación del modelo que deba reflejarse en pantalla termina en `this.env.model.trigger("update")`.

## 2. Componentes de presentación (bindeo a env.model)

### 2.1 `LineComponent` — `odoo_enterprise/19/stock_barcode/static/src/components/line.js:4`, `LineComponent`
Base de toda línea. `static props = ["displayUOM","line","subline?","editLine"]` (`:5`). **Export default** (no registry). Template `stock_barcode.LineComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/line.xml:148`, `stock_barcode.LineComponent`).

Getters que **bindean a env.model**: `destinationLocationPath`, `displayDeleteButton`→`lineCanBeDeleted` (`:22`), `displayFulfillbutton`/`displayIncrementButton`→`getDisplayIncrementBtn`/`getDisplayIncrementBtnForSerial`/`getIncrementQuantity` (`:31-49`), `isSelected`→`lineIsSelected` + `lastScanned.packageId` (`:69-75`), `isTracked`→`lineIsTracked` (`:78`), `qtyDemand`/`qtyDone`→`getQtyDemand`/`getQtyDone` (`:99-105`), `componentClasses`→`lineIsFaulty` (`:126`).

**Acciones emitidas hacia el modelo** (aquí insertas comportamiento):
| Método componente | Llama a env.model | Notas |
|---|---|---|
| `select(ev)` (odoo_enterprise/19/stock_barcode/static/src/components/line.js::select) | `selectLine(line)` + `trigger("update")` (:178-179) | click en la línea |
| `delete()` (odoo_enterprise/19/stock_barcode/static/src/components/line.js::delete) | `deleteLine(line)` | |
| `addQuantity(q)` (odoo_enterprise/19/stock_barcode/static/src/components/line.js::addQuantity) | `updateLineQty(virtualId, q)` | resuelve subline por lote |
| `completePackage()` (odoo_enterprise/19/stock_barcode/static/src/components/line.js::completePackage) | `completePackage(virtual_id)` | |
| `toggleAsCounted(ev)` (odoo_enterprise/19/stock_barcode/static/src/components/line.js::toggleAsCounted) | `toggleAsCounted(line)` | inventario |
| `onClickImage()` (odoo_enterprise/19/stock_barcode/static/src/components/line.js::onClickImage) | `env.dialog.add(ProductImageDialog,{record: this.line.product_id})` (:187) | único diálogo abierto desde un componente |
| botón editar | `props.editLine(this.line)` → **`MainComponent.onOpenProductPage`** (bind en odoo_enterprise/19/stock_barcode/static/src/components/main.xml:87) | NO existe `_editLine` |

`imageSource` usa `image_128` (`odoo_enterprise/19/stock_barcode/static/src/components/line.js::setup`).

Template partido en **sub-templates** (todas en `line.xml`), targets naturales de `t-inherit`/xpath: `LineTitle` (:4), `LineUpperButtons` (:23), `LineQuantity` (:39), `LineLowerButtons` (:62), `LineOwner` (:109), `LineLot` (:116), `LineDestinationLocation` (:132).

**Hook de plantilla diseñado**: `LineTitle` hace `t-call="{{upperButtons || 'stock_barcode.LineUpperButtons'}}"` con `t-if="upperButtons !== false"` (`odoo_enterprise/19/stock_barcode/static/src/components/line.xml:19`, `stock_barcode.LineTitle`); `LineQuantity` igual con `lowerButtons` (`odoo_enterprise/19/stock_barcode/static/src/components/line.xml:58`, `stock_barcode.LineQuantity`). Si tu template padre hace `t-set` de esas variables puedes sustituir/ocultar los bloques de botones sin xpath — **pero solo funciona si tu propio componente instancia LineComponent con esas vars en scope**; el flujo estándar (`main.xml`) nunca las setea.

### 2.2 `GroupedLineComponent` — `odoo_enterprise/19/stock_barcode/static/src/components/grouped_line.js:3`, `GroupedLineComponent`
`extends LineComponent`, `static components = { LineComponent }` (`:4`), template `stock_barcode.GroupedLineComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/grouped_line.xml:4`, `stock_barcode.GroupedLineComponent`). Renderiza sublíneas con `<LineComponent t-props="sublineProps"/>` (`odoo_enterprise/19/stock_barcode/static/src/components/grouped_line.xml:26`, `stock_barcode.GroupedLineComponent`).
- `isSelected` → `line.virtual_ids.indexOf(model.selectedLineVirtualId)` (`:28`).
- `opened` → `model.groupKey(line) === model.unfoldLineKey` (`:32`).
- `linesToDisplay` filtra según `model.showReservedSns` (`:44`).
- `toggleSublines(ev)` → `model.toggleSublines(line)` (`:91`).

### 2.3 `PackageLineComponent` — `odoo_enterprise/19/stock_barcode/static/src/components/package_line.js:3`, `PackageLineComponent`
`extends LineComponent`, `static props = ["displayUOM","line","openPackage"]` (`:4`), template `stock_barcode.PackageLineComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/package_line.xml:4`, `stock_barcode.PackageLineComponent`).
- Acciones: `delete(ev)`→`model.deleteLines(line.lines)`+`trigger("update")` (`:57-61`); `select(ev)`→`model.selectPackageLine(line)`+`trigger("update")` (`:63-67`); `unpack()`→`model.unpack(line.lines)` (`:77`, async); `openPackage()`→`props.openPackage(packageIds)` → **`MainComponent.onOpenPackage`** (bind en odoo_enterprise/19/stock_barcode/static/src/components/main.xml:85) (`:69`).
- Lógica de paquete anidado en `hasSameSourceAndResultPackage()` (`:46`) y `packageDataAttribute`/`packageLabel`/`resultPackageLabel`.

**Recorrido en main.xml**: NO se itera `env.model.groupedLines`/`packageLines` directamente. Se itera `env.model.groupedLinesByLocation` (`odoo_enterprise/19/stock_barcode/static/src/components/main.xml:71`, `stock_barcode.MainComponent`) → `group.lines` (`odoo_enterprise/19/stock_barcode/static/src/components/main.xml:84`, `stock_barcode.MainComponent`), y por cada línea se despacha: `PackageLineComponent` si `line.isPackageLine`, `GroupedLineComponent` si `line.lines`, si no `LineComponent` (`odoo_enterprise/19/stock_barcode/static/src/components/main.xml:85-87`, `stock_barcode.MainComponent`). `displayUOM="false"` para paquetes, `groups.group_uom` para el resto.

### 2.4 Diálogos

Todos se lanzan **desde la capa MODELO** vía `this.dialogService.add(...)`, salvo `ProductImageDialog` (desde componente) y `CountScreenRFID` (declarativo).

**`ConfirmQuantDialog` — `odoo_enterprise/19/stock_barcode/static/src/components/confirm_quant_dialog.js:4`, `ConfirmQuantDialog` (NUEVO v19)**
`extends Component`, `static components = { Dialog }` (Dialog genérico de `@web/core/dialog/dialog`). Template `stock_barcode.ConfirmQuantDialog`. `static props = {close, onConfirm, onWaitReview}` (Function). Captura `inventoryReason` (`useState`, default `"Physical Inventory"`). Botones: **Apply Now** → `onConfirm({inventory_name})` + `close`, **Wait a Review** → `onWaitReview()` + `close`, **Cancel** → `props.close`.
Lanzado desde `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js:41`, `BarcodeQuantModel.apply` dentro de `apply(options)`, pasando **solo** `{onConfirm: confirm, onWaitReview: waitReview}` (`close` lo inyecta el dialog service). `apply()` tiene atajos `shouldConfirm`/`shouldWaitReview` que ejecutan `_apply()`/`waitReview()` **sin** mostrar el diálogo (`:36-39`). `waitReview` = `save()` + `trigger("history-back")` + `trigger("update")`. En v18 `apply()` iba directo a `_apply()` sin este diálogo (verificado en v18 `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js:24-26`, `BarcodeQuantModel.apply`) → **cambio de flujo v19**.

**`ApplyQuantDialog` — `odoo_enterprise/19/stock_barcode/static/src/components/apply_quant_dialog.js:4`, `ApplyQuantDialog`** (existía ya en v18)
`extends Component` + `Dialog`. `static props = {onApply, onApplyAll, close}`. Lanzado desde `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::checkBeforeApply` en `checkBeforeApply()` cuando hay números de serie hermanos sin contar.

**`BackorderDialog` — `odoo_enterprise/19/stock_barcode/static/src/components/backorder_dialog.js:4`, `BackorderDialog`**
`extends Component` + `Dialog`. `static props = {displayUoM, uncompletedLines, onApply, close}`. Lanzado desde `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_validate`.

**`ExtraProductDialog` — `odoo_enterprise/19/stock_barcode/static/src/components/extra_product_dialog.js:7`, `ExtraProductDialog`**
`extends ConfirmationDialog` (web). Template `t-inherit="web.ConfirmationDialog" t-inherit-mode="primary"` con `xpath //p position="replace"` (`odoo_enterprise/19/stock_barcode/static/src/components/extra_product_dialog.xml:5`, `template`). Prop extra `products: Map`. Renderiza checkboxes por producto no reservado. `setup()` resuelve `displayUOM` con `user.hasGroup("uom.group_uom")` (`:24-26`). Lanzado desde `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_addInConfirmationQueue` con `{products: this.extraProducts, cancel: this.extraProducts.clear, confirm: this._confirmExtraProductDialog.bind(this), close: this.extraProducts.clear}`.

**`ProductImageDialog` — `odoo_enterprise/19/stock_barcode/static/src/components/product_image_dialog.js:4`, `ProductImageDialog`**
`extends Component` + `Dialog`. `static props = {record, close}`. Muestra `/web/image/product.product/<id>/image_1024` (`:13`). Único diálogo abierto **desde un componente** (`LineComponent.onClickImage`, `odoo_enterprise/19/stock_barcode/static/src/components/line.js::onClickImage`).

**`CountScreenRFID` — `odoo_enterprise/19/stock_barcode/static/src/components/count_screen_rfid.js:4`, `CountScreenRFID`**
`extends Component`. `static props = {close, receivedRFIDs[], totalRFIDs[]}`. Pantalla en vivo del conteo RFID: `duration`, `readRate` (state), `totalRead`/`uniqueTags` (getters). Usa `setInterval`/`setTimeout` propios. **Aviso**: su `onWillUnmount` (`:46-49`) limpia `updateReadsRateInterval` y `activeScanningTimeout`, **pero NO `timeInterval`** (el `setInterval` de duración de `:22`) → ese timer queda sin limpiar (fuga en el código de Odoo). Renderizado **declarativamente** en `odoo_enterprise/19/stock_barcode/static/src/components/main.xml:11`, `stock_barcode.MainComponent` (`t-if="state.displayCountRFID"`, `close.bind="closeRFIDCount"`), no vía dialogService.

## 3. Widgets (registry-based — extensibles por registry)

Todos registran un **descriptor objeto** (patrón v17+), no la clase.

| Widget | Registry / key | Clase | Fichero |
|---|---|---|---|
| `digipad` | `view_widgets` / "digipad" | `Digipad` | odoo_enterprise/19/stock_barcode/static/src/widgets/digipad.js:184-191 |
| `image_preview` | `fields` / "image_preview" | `ImagePreviewField` (ext. `ImageField`) | odoo_enterprise/19/stock_barcode/static/src/widgets/image_preview.js:29-33 |
| `set_reserved_qty_button` | `fields` / "set_reserved_qty_button" | `SetReservedQuantityButton` | odoo_enterprise/19/stock_barcode/static/src/widgets/set_reserved_qty_button.js:32-42 |
| `stock_barcode_quant_one2many` | `fields` / "stock_barcode_quant_one2many" | `StockBarcodeQuantOne2ManyField` (ext. `X2ManyField`) | odoo_enterprise/19/stock_barcode/static/src/widgets/set_quant_stock_move_line.js:79-84 (**NUEVO v19**) |
| `stock_barcode_unlink_button` | `view_widgets` / "stock_barcode_unlink_button" | `StockBarcodeUnlinkButton` | odoo_enterprise/19/stock_barcode/static/src/components/main.js:53-55 |

### 3.1 `Digipad` — odoo_enterprise/19/stock_barcode/static/src/widgets/digipad.js:9
Teclado +/-, fulfill, packaging. `static props = {...standardWidgetProps, fieldToEdit, fulfilledAt?}`. `extractProps: ({attrs}) => ({fieldToEdit: attrs.field_to_edit, fulfilledAt: attrs.fulfilled_at})` (`:186`).
Escribe con `this.props.record.update(this.changes)` donde `changes = {[fieldToEdit]: Number(value)}` (`_increment`, `:109`; getter `changes` `:46`).
**Cambio v19**: `_fetchPackagingButtons()` (`:117`) consulta `uom.uom` filtrando por `product.product.uom_ids` (`:128`) y `factor<=quantityToFulfill`, leyendo `['name','factor','package_type_id']` (`:137-142`), con `limit:2`. En v18 consultaba `product.packaging` (verificado en v18 `odoo_enterprise/19/stock_barcode/static/src/widgets/digipad.js::_fetchPackagingButtons`). Refleja la fusión packaging→UoM de v19.
Usado en `odoo_enterprise/19/stock_barcode/views/stock_move_line_views.xml:54`, `stock_move_line_product_selector`, `odoo_enterprise/19/stock_barcode/views/stock_inventory_views.xml:44`, `stock_quant_barcode`, `odoo_enterprise/19/stock_barcode/views/stock_scrap_views.xml:63`, `scrap_product_selector`.

### 3.2 `StockBarcodeQuantOne2ManyField` — odoo_enterprise/19/stock_barcode/static/src/widgets/set_quant_stock_move_line.js:6 (NUEVO v19)
`extends X2ManyField`. Permite elegir un `stock.quant` existente y propagar `{location_id, lot_id, package_id, owner_id}` a la move line vía `this.props.record.update(vals)` (`openRecord`, `:36-51`). Auto-empareja el quant actual (`_findMatchingQuant`, `:59`), resalta seleccionado mutando `quant.selected` (`_highlightSelectedQuant`, `:72`), y resetea `result_package_id`/`outermost_result_package_id` cuando el paquete cambia (`:47-48`). Gating en `onWillStart`: `isQuantSelectable` requiere `picking_code ∈ {internal,outgoing,mrp_operation}` **y** alguno de `stock.group_production_lot`/`group_stock_multi_locations`/`group_tracking_lot`/`group_tracking_owner` (`:15-23`).
Usado en `odoo_enterprise/19/stock_barcode/views/stock_move_line_views.xml:138`, `stock_move_line_product_selector` (`widget="stock_barcode_quant_one2many" mode="kanban"` sobre `product_stock_quant_ids`; en v18 no existía el widget).

### 3.3 `SetReservedQuantityButton` — odoo_enterprise/19/stock_barcode/static/src/widgets/set_reserved_qty_button.js:6
Botón que copia una cantidad de un campo a otro: `record.update({[fieldToSet]: record.data[name]})` (`:24-29`). `extractProps({attrs})` **condicional**: `{fieldToSet: attrs.field_to_set}` si existe, si no `{}` (`:34-39`).

### 3.4 `ImagePreviewField` / `FullScreenImage` — widgets/image_preview.js
`ImagePreviewField extends ImageField` (`:14`); abre `FullScreenImage` en dialog service (`openImageFullScreen`, `:22`). Descriptor `imageClickEnlarge` (`:29`), key `"image_preview"` (`:33`). No define `extractProps` (hereda de ImageField).

## 4. manual_barcode: qué cambió (v18 → v19)

En **v18** existía `stock_barcode/static/src/components/manual_barcode.js` (verificado presente en v18). **En v19 ese fichero NO existe.** `MainComponent` importa `BarcodeInput` desde `@barcodes/components/manual_barcode` (módulo base `barcodes`) — `odoo_enterprise/19/stock_barcode/static/src/components/main.js:32`. Las clases `BarcodeInput` y `ManualBarcodeScanner` viven ahora en `odoo19/addons/barcodes/static/src/components/manual_barcode.js` (templates `barcodes.BarcodeInput` / `barcodes.ManualBarcodeScanner`, verificados en el .xml base). El kanban renderer (`odoo_enterprise/19/stock_barcode/static/src/kanban/stock_barcode_kanban_renderer.js:2`) y `odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js:7` también importan `ManualBarcodeScanner` de `@barcodes`.
→ **Cualquier import de `@stock_barcode/components/manual_barcode` de código v18 rompe en v19.**

## 5. Dónde insertar UI propia — resumen práctico

1. **Botón/dato por línea** → `patch(LineComponent.prototype, {...})` (importar default export) + `t-inherit` de la sub-template concreta (`stock_barcode.LineLowerButtons`, etc.). Handler → `env.model.<accion>()` + `env.model.trigger("update")`.
2. **Sustituir bloque de botones sin xpath** → usar el hook `upperButtons`/`lowerButtons` (`odoo_enterprise/19/stock_barcode/static/src/components/line.xml:19`, `stock_barcode.LineTitle` (y `:58`, `stock_barcode.LineQuantity`)) **desde tu propio componente** que instancie LineComponent (no aplica al flujo estándar).
3. **Nuevo widget de campo/vista** → registrar descriptor en `fields`/`view_widgets` con la misma key (override) o key nueva; usar `extractProps({attrs})`.
4. **Personalizar diálogos de negocio** (ConfirmQuant, ApplyQuant, ExtraProduct, Backorder) → patchear el **método del modelo** que los abre (`barcode_quant_model.apply`/`checkBeforeApply`, `barcode_picking_model`), no los componentes.
5. **Nueva vista/sección o tipo de línea** → `patch(MainComponent.prototype)` + ampliar `static.components` + `t-inherit` de `stock_barcode.MainComponent`; para negocio, patchear la clase de modelo (`BarcodePickingModel`/`BarcodeQuantModel`).

## 6. Gotchas

- `env.model` **hereda `EventBus` de `@odoo/owl`** (NO `EventTarget`; `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:12`, `BarcodeModel`). No es reactivo: getters + `trigger("update")`. Sin `trigger("update")` no hay re-render. Ver `odoo_enterprise/19/stock_barcode/static/src/components/line.js::select` (select), `odoo_enterprise/19/stock_barcode/static/src/components/package_line.js::delete`.
- Los `addEventListener` del ciclo de vida están en `onWillStart` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-258`), no en `setup`; los `useBus` sí en `setup` (`odoo_enterprise/19/stock_barcode/static/src/components/main.js:132-149`, `MainComponent.setup`). El de `"update"` va guardado por `state.uiBlocked` y hace `render(true)`.
- `GroupedLineComponent`/`PackageLineComponent` heredan de `LineComponent`: un patch al prototype base afecta a los tres.
- Componentes de línea son export default, no registry → solo patch de prototype (frágil, no swap por registry).
- El callback `editLine` es `MainComponent.onOpenProductPage` y `openPackage` es `onOpenPackage` (binds en `odoo_enterprise/19/stock_barcode/static/src/components/main.xml:85-87`, `stock_barcode.MainComponent`); no hay método `_editLine`.
- `ExtraProductDialog.xml` hace `xpath //p position="replace"` sobre `web.ConfirmationDialog` → frágil ante cambios del markup base.
- `ConfirmQuantDialog` usa `Dialog` genérico (NO `ConfirmationDialog`); `static props {close,onConfirm,onWaitReview}`, pero en el lanzamiento solo se pasan `onConfirm`/`onWaitReview` (`close` lo inyecta el service); `onConfirm` recibe `{inventory_name}`.
- `digipad` asume modelo UoM de v19 (`uom_ids`/`uom.uom.factor`); no retro-portable a v18 sin reescribir `_fetchPackagingButtons`.
- `CountScreenRFID` NO limpia su `timeInterval` de duración en `onWillUnmount` (solo `updateReadsRateInterval` y `activeScanningTimeout`) → fuga; al extender, limpia los timers explícitamente.
- Diálogos de negocio (ConfirmQuant, ApplyQuant, ExtraProduct, Backorder) se lanzan desde la capa modelo (`this.dialogService.add`), no desde componentes; excepciones: `ProductImageDialog` (LineComponent) y `CountScreenRFID` (declarativo en `odoo_enterprise/19/stock_barcode/static/src/components/main.xml:11`, `stock_barcode.MainComponent`).
- Los widgets registran el **objeto descriptor** `{component, extractProps}`, no la clase. `set_reserved_qty_button.extractProps` es condicional; `image_preview` no define `extractProps` (hereda de ImageField).


## Correcciones aplicadas en verificación (v19)

- CRÍTICO — env.model NO hereda EventTarget. Hereda OWL EventBus: `export default class BarcodeModel extends EventBus` (odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:12), con `import { EventBus } from "@odoo/owl"` (odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js:9). EventBus expone addEventListener/trigger, de ahí la confusión, pero NO es EventTarget. El borrador lo afirmaba mal en sección 1, sección 6 (gotchas), extension points y array gotchas. Corregido en todas las apariciones.
- Los addEventListener('update'|'process-action'|'refresh'|'history-back') están dentro de `onWillStart()` (odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-258), NO en `setup()`. Lo que SÍ está en setup() son los `useBus(this.env.model, ...)` (odoo_enterprise/19/stock_barcode/static/src/components/main.js:132-149). El borrador atribuía los addEventListener a setup().
- El handler de 'update' no es `this.render()` a secas: es `this.env.model.addEventListener('update', () => { if (!this.state.uiBlocked) { this.render(true); } })` (odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart-257). Va guardado por uiBlocked y usa render(true).
- El callback `editLine` NO es un método `_editLine` de MainComponent (no existe). Es `MainComponent.onOpenProductPage`, pasado como `editLine.bind="onOpenProductPage"` en odoo_enterprise/19/stock_barcode/static/src/components/main.xml:86 (GroupedLineComponent) y odoo_enterprise/19/stock_barcode/static/src/components/main.xml:87 (LineComponent). El botón editar de la plantilla llama `this.props.editLine(this.line)` (odoo_enterprise/19/stock_barcode/static/src/components/line.xml:31).
- El callback `openPackage` de PackageLineComponent es `MainComponent.onOpenPackage` (odoo_enterprise/19/stock_barcode/static/src/components/main.js::onOpenPackage), pasado como `openPackage.bind="onOpenPackage"` en odoo_enterprise/19/stock_barcode/static/src/components/main.xml:85.
- main.xml NO itera sobre `env.model.groupedLines` / `env.model.packageLines`. Itera `env.model.groupedLinesByLocation` (odoo_enterprise/19/stock_barcode/static/src/components/main.xml:71) y luego `group.lines` (odoo_enterprise/19/stock_barcode/static/src/components/main.xml:84), despachando por tipo: PackageLineComponent si `line.isPackageLine`, GroupedLineComponent si `line.lines`, si no LineComponent (odoo_enterprise/19/stock_barcode/static/src/components/main.xml:85-87). Los getters `lines`(→groupedLines, odoo_enterprise/19/stock_barcode/static/src/components/main.js::lines) y `packageLines`(odoo_enterprise/19/stock_barcode/static/src/components/main.js::packageLines) existen pero no son la fuente de iteración de la plantilla.
- OMISIÓN: existen dos diálogos más en components/ no mencionados. `ApplyQuantDialog` (odoo_enterprise/19/stock_barcode/static/src/components/apply_quant_dialog.js:4, extends Component + Dialog, props {onApply,onApplyAll,close}) lanzado desde odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::checkBeforeApply en checkBeforeApply cuando hay SN sin contar; NO es nuevo de v19 (ya existía en v18). `BackorderDialog` (odoo_enterprise/19/stock_barcode/static/src/components/backorder_dialog.js:4, extends Component + Dialog, props {displayUoM,uncompletedLines,onApply,close}) lanzado desde odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_validate.
- ConfirmQuantDialog: en el lanzamiento (odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js:41-44) solo se pasan `{onConfirm, onWaitReview}`; `close` lo inyecta automáticamente el dialog service (aunque esté en static props). Además apply(options) tiene atajos shouldConfirm/shouldWaitReview que ejecutan _apply()/waitReview() SIN mostrar el diálogo (odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js:36-39). waitReview = save() + trigger('history-back') + trigger('update').
- ExtraProductDialog: los callbacks reales en el lanzamiento (odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_addInConfirmationQueue-132) son `{products: this.extraProducts, cancel: this.extraProducts.clear, confirm: this._confirmExtraProductDialog.bind(this), close: this.extraProducts.clear}`. El borrador omitía el prop `products` y que cancel/close son `extraProducts.clear`.
- CountScreenRFID: onWillUnmount (odoo_enterprise/19/stock_barcode/static/src/components/count_screen_rfid.js::setup-49) limpia SOLO `updateReadsRateInterval` y `activeScanningTimeout`. NO limpia `timeInterval` (el setInterval de duración, línea 22) → ese timer QUEDA SIN LIMPIAR (fuga real en el código de Odoo). El borrador decía 'los limpia en onWillUnmount', lo cual es parcialmente falso.
- set_reserved_qty_button.extractProps es CONDICIONAL: devuelve `{fieldToSet: attrs.field_to_set}` solo si `attrs.field_to_set` existe, si no `{}` (odoo_enterprise/19/stock_barcode/static/src/widgets/set_reserved_qty_button.js:34-39). El borrador lo pintaba como incondicional.
- image_preview: la variable descriptor se llama `imageClickEnlarge` (no 'imagePreview'), key 'image_preview', registro en odoo_enterprise/19/stock_barcode/static/src/widgets/image_preview.js:33. ImagePreviewField extends ImageField (línea 14) y no define extractProps (correcto). También exporta la clase auxiliar FullScreenImage (línea 6).
- VERIFICADO contra fuente v18 (no solo afirmado): (a) v18 digipad._fetchPackagingButtons consultaba 'product.packaging' (v18 odoo_enterprise/19/stock_barcode/static/src/widgets/digipad.js::_fetchPackagingButtons) → confirma la migración a uom.uom en v19; (b) v18 SÍ tenía stock_barcode/static/src/components/manual_barcode.js → confirma su eliminación en v19; (c) v18 NO tenía confirm_quant_dialog.js → confirma nuevo en v19; (d) v18 NO tenía widgets/set_quant_stock_move_line.js → confirma nuevo en v19; (e) v18 apply() llamaba directo a _apply() sin ConfirmQuantDialog (v18 `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js:24-26`, `BarcodeQuantModel.apply`) → confirma el cambio de flujo. Todos confirmados.
- LineComponent.imageSource usa image_128 (odoo_enterprise/19/stock_barcode/static/src/components/line.js::setup); ProductImageDialog usa image_1024 (odoo_enterprise/19/stock_barcode/static/src/components/product_image_dialog.js::setup). El borrador solo citaba image_1024 (correcto para el diálogo).
