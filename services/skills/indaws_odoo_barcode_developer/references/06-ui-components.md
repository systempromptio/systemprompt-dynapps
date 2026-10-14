# stock_barcode — Capa UI (componentes OWL) · Odoo 18 Enterprise

Fuente: `odoo_enterprise/18/stock_barcode/static/src/components/` y `/static/src/widgets/`.
Bundle: `static/src/**` se carga en `web.assets_backend` mediante **tres globs separados** — `**/*.js` (`stock_barcode/__manifest__.py:37`), `**/*.scss` (`:38`), `**/*.xml` (`:39`) — más un `('remove', '**/*.dark.scss')` (`:42`). No hay orden fino: patches y `t-inherit` conviven en el mismo bundle.

## 1. Arquitectura del subsistema

La UI es una **capa de presentación "tonta"**: los componentes NO tienen estado de negocio propio. Todo el estado vive en un **modelo JS que es un `EventBus`** (`BarcodeModel extends EventBus`, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:14`, `BarcodeModel`), instanciado por `MainComponent` según el `res_model` (`stock.picking` → `BarcodePickingModel`, `stock.quant` → `BarcodeQuantModel`; `odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel-289`).

El modelo se inyecta a **todo el árbol** vía `useSubEnv({ model, dialog })` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:95-98`, `MainComponent.setup`). Por eso cada componente accede a él con `this.env.model` y abre diálogos con `this.env.dialog`.

### Flujo de renderizado (crítico entenderlo para extender)
1. Componentes leen datos con **getters que delegan al modelo** (`this.env.model.getQtyDone(line)`, etc.) y con **objetos `line` planos** pasados por props. NO usan `useState` sobre los datos de línea.
2. Interacción de usuario → el componente llama a un **método del modelo** (`selectLine`, `updateLineQty`, `deleteLine`…).
3. El modelo muta su estado y hace `this.trigger('update')` (`barcode_model.js`, p.ej. `:511` en `toggleSublines`, `:1520`, `:1538`).
4. `MainComponent` escucha ese evento (`this.env.model.addEventListener("update", () => { if (!this.state.uiBlocked) this.render(true); })`, `odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart-237`) y **re-renderiza el árbol completo**.

Consecuencia para extender: **si mutas estado del modelo desde un método parcheado y no disparas `trigger('update')`, la UI no refresca.** Y si `state.uiBlocked` está activo, el render se salta (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart`).

### Buses del modelo escuchados por MainComponent (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:124-130`, `MainComponent.setup`)
`flash`, `playSound`, `blockUI`, `unblockUI`, `addBarcodesCountToProcess`, `updateBarcodesCountProcessed`, `clearBarcodesCountProcessed`. Más eventos vía `addEventListener` en `onWillStart`: `process-action`, `refresh`, `update`, `history-back` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart-238`). Un `bus` global (EventBus a nivel módulo, `odoo_enterprise/18/stock_barcode/static/src/components/main.js:30`, `bus`) transporta `refresh` desde `StockBarcodeUnlinkButton` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onClick`; escuchado con `useBus(bus, "refresh", …)` en `:131`).

## 2. Componentes de línea

### `LineComponent` (`odoo_enterprise/18/stock_barcode/static/src/components/line.js:6`, `LineComponent`, template `stock_barcode.LineComponent` en `odoo_enterprise/18/stock_barcode/static/src/components/line.xml:137`, `stock_barcode.LineComponent`)
Clase base de todas las líneas.
- **Props** (validación estricta OWL): `static props = ["displayUOM", "line", "subline?", "editLine"]` (`odoo_enterprise/18/stock_barcode/static/src/components/line.js:7`, `LineComponent`). Pasar props extra sin ampliar esta lista **rompe** en runtime.
- **`line` (getter)** = `this.props.line` (`odoo_enterprise/18/stock_barcode/static/src/components/line.js::line`), objeto plano del modelo (no reactivo).
- **`editLine`** = callback inyectado desde `MainComponent` (`editLine.bind="onOpenProductPage"`, `odoo_enterprise/18/stock_barcode/static/src/components/main.xml:82-83`, `stock_barcode.MainComponent` → `odoo_enterprise/18/stock_barcode/static/src/components/main.js::onOpenProductPage`), abre el form view de la línea (vista `productPage`).
- **`setup`** solo calcula `this.imageSource` a partir de `product_id.has_image` (`odoo_enterprise/18/stock_barcode/static/src/components/line.js::setup-14`).
- **Getters que delegan al modelo** (contrato de datos real): `displayDeleteButton`→`lineCanBeDeleted`, `displayDestinationLocation`, `displayFulfillbutton`/`displayIncrementButton`→`getDisplayIncrementBtn`/`getDisplayIncrementBtnForSerial`, `incrementQty`→`getIncrementQuantity`, `displayResultPackage`, `isSelected`→`lineIsSelected`+`lastScanned.packageId`, `isTracked`→`lineIsTracked`, `qtyDemand`→`getQtyDemand`, `qtyDone`→`getQtyDone` (`odoo_enterprise/18/stock_barcode/static/src/components/line.js::displayDeleteButton-98`). Solo controlan *presentación*; la lógica real está en el modelo.
- **Getters de estilo**: `isComplete` (`:49`), `componentClasses` (`:100`, aplica `o_line_completed`/`o_line_not_completed` (excluyentes), `o_faulty`, `o_selected o_highlight`).
- **Métodos (eventos → modelo)**:
  - `select(ev)` → `env.model.selectLine(line)` + `trigger('update')` (`:129-133`).
  - `addQuantity(qty)` → resuelve el `virtual_id` (incl. subline por lote) y llama `env.model.updateLineQty(virtualId, qty)` (`:118-127`). `updateLineQty` es **abstracto en la base** (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:576`, `BarcodeModel.updateLineQty` lanza "Not Implemented"), implementado por cada modelo.
  - `toggleAsCounted(ev)` → `env.model.toggleAsCounted(line)` (`:135`).
  - `onClickImage()` → `this.env.dialog.add(ProductImageDialog, { record: this.line.product_id })` (`:139-141`).
- **Botones directos en template** (llaman al modelo inline): `-1`/`+1`/fulfill via `addQuantity` (`odoo_enterprise/18/stock_barcode/static/src/components/line.xml:73-88`, `stock_barcode.LineLowerButtons`), borrar via `env.model.deleteLine(line)` (`odoo_enterprise/18/stock_barcode/static/src/components/line.xml:89-94`), editar via `props.editLine(line)` (`odoo_enterprise/18/stock_barcode/static/src/components/line.xml:29-34`, `stock_barcode.LineUpperButtons`).

#### Sub-templates reutilizables (seam de extensión de layout, `line.xml`)
El template se descompone en `t-call` a sub-plantillas: `LineTitle` (`:4`), `LineUpperButtons` (`:22`), `LineQuantity` (`:38`), `LineLowerButtons` (`:57`), `LineOwner` (`:98`), `LineLot` (`:105`), `LineDestinationLocation` (`:121`). El template raíz las ensambla en `LineComponent` (`:137-161`).

**Qué sub-template usa cada componente** (importante para calcular el impacto de un `t-inherit`):
- `LineQuantity`, `LineDestinationLocation` → los tres (Line, Grouped, Package).
- `LineTitle` → Line + Grouped (Package tiene cabecera propia en `odoo_enterprise/18/stock_barcode/static/src/components/package_line.xml:9-20`, `stock_barcode.PackageLineComponent`, no lo t-llama).
- `LineLowerButtons` → Line + Grouped (Package lo oculta con `lowerButtons=false`).
- `LineOwner` → Line + Package (Grouped NO lo t-llama).
- `LineLot` → Line + Grouped (condicional a `!displayToggleBtn`).

**Indirección de botones incorporada**: `LineTitle` hace `t-call="{{upperButtons || 'stock_barcode.LineUpperButtons'}}"` protegido por `t-if="upperButtons !== false"` (`:18`); `LineQuantity` hace `t-call="{{lowerButtons || 'stock_barcode.LineLowerButtons'}}"` protegido por `t-if="lowerButtons !== false"` (`:53`). Poniendo `t-set="lowerButtons"`/`upperButtons` a otra plantilla (o a `false` para ocultarlas) **se sustituyen los grupos de botones sin xpath**. `PackageLineComponent` ya usa esto (`odoo_enterprise/18/stock_barcode/static/src/components/package_line.xml:21`, `stock_barcode.PackageLineComponent`: `t-set="lowerButtons" t-value="false"` antes del `t-call` a `LineQuantity` en `:23`).

### `GroupedLineComponent` (`odoo_enterprise/18/stock_barcode/static/src/components/grouped_line.js:5`, `GroupedLineComponent`, extiende `LineComponent`)
Línea agrupada (mismo producto con varias sublíneas/lotes).
- `static components = { LineComponent }` → renderiza sublíneas recursivamente (`odoo_enterprise/18/stock_barcode/static/src/components/grouped_line.xml:18-20`, `stock_barcode.GroupedLineComponent`).
- Overrides: `isComplete` (`:9`), `isSelected`→`virtual_ids.indexOf(selectedLineVirtualId) !== -1` (`:25-27`), `opened`→`groupKey===unfoldLineKey` (`:29`), `linesToDisplay` (filtra según `showReservedSns`, `:42`), `lotName` (`:51`), `displayToggleBtn`→`linesToDisplay.length>1` (`:69`).
- `sublineProps` (`:33`) construye las props de cada sublínea (`subline:true`, `editLine` heredado).
- `toggleSublines(ev)` → `env.model.toggleSublines(line)` (`:73`), que hace `trigger('update')` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::toggleSublines`).

### `PackageLineComponent` (`odoo_enterprise/18/stock_barcode/static/src/components/package_line.js:5`, `PackageLineComponent`, extiende `LineComponent`)
Línea que representa un paquete completo.
- **Props distintas**: `["displayUOM", "line", "openPackage"]` (`:6`) — **sin `subline`/`editLine`, con `openPackage`**.
- Overrides `isComplete`→`qtyDone==qtyDemand` (`:9`), `isSelected`→`package_id.id===lastScanned.packageId` (`:13`), `qtyDemand`→basado en `reservedPackage`/`result_package_id` (`:17-20`), `qtyDone`→suma sobre `line.lines`, **con guardia** `if (reservedQuantity > 0)` antes de dividir (`:22-29`; si es 0 devuelve 1 o 0, no divide).
- `select` override → `env.model.selectPackageLine(line)` + `trigger('update')` (`:31-35`).
- Botón "abrir paquete" → `props.openPackage(package_id.id)` (`odoo_enterprise/18/stock_barcode/static/src/components/package_line.xml:16`, `stock_barcode.PackageLineComponent`), cableado a `MainComponent.onOpenPackage` (`odoo_enterprise/18/stock_barcode/static/src/components/main.xml:81`, `stock_barcode.MainComponent` → `odoo_enterprise/18/stock_barcode/static/src/components/main.js::onOpenPackage`, cambia a vista `packagePage`).

### Selección de componente por línea (`odoo_enterprise/18/stock_barcode/static/src/components/main.xml:81-83`, `stock_barcode.MainComponent`)
```
PackageLineComponent  t-if="line.isPackageLine"   (openPackage.bind="onOpenPackage", displayUOM="false")
GroupedLineComponent  t-elif="line.lines"         (editLine.bind="onOpenProductPage")
LineComponent         t-else                       (editLine.bind="onOpenProductPage")
```
Datos de entrada: `env.model.groupedLinesByLocation` (`odoo_enterprise/18/stock_barcode/static/src/components/main.xml:67`, `stock_barcode.MainComponent`), getters `MainComponent.lines`/`packageLines` → `groupedLines`/`packageLines` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::lines-265`). Las clases se registran en `MainComponent.components` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:69-79`, `MainComponent`).

## 3. Diálogos (Dialog components)

Todos son `Dialog`/`ConfirmationDialog` de `@web/core`. Se abren de dos formas:
- Desde **componente** con `this.env.dialog.add(...)` (**solo** `ProductImageDialog`, desde `LineComponent.onClickImage`).
- Desde **modelo** con `this.dialogService.add(...)` (el resto; el modelo obtiene `dialogService` vía `useService('dialog')` en su constructor, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::constructor`).

| Diálogo | Fichero | Props que recibe | Quién lo abre |
|---|---|---|---|
| `ProductImageDialog` | `odoo_enterprise/18/stock_barcode/static/src/components/product_image_dialog.js:6`, `ProductImageDialog` | `{record, close}` | `LineComponent.onClickImage` (`odoo_enterprise/18/stock_barcode/static/src/components/line.js::onClickImage`) |
| `BackorderDialog` | `odoo_enterprise/18/stock_barcode/static/src/components/backorder_dialog.js:4`, `BackorderDialog` | `{displayUoM, uncompletedLines, onApply, close}` | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::_validate` (en `_validate`) |
| `ApplyQuantDialog` | `odoo_enterprise/18/stock_barcode/static/src/components/apply_quant_dialog.js:4`, `ApplyQuantDialog` | `{onApply, onApplyAll, close}` | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::checkBeforeApply` |
| `ExtraProductDialog` | `odoo_enterprise/18/stock_barcode/static/src/components/extra_product_dialog.js:7`, `ExtraProductDialog` | `...ConfirmationDialog.props, {products:Map, onCheckboxChange}` | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::createNewLine` (producto no reservado escaneado) |
| `ManualBarcodeScanner` / `BarcodeInput` | `odoo_enterprise/18/stock_barcode/static/src/components/manual_barcode.js:34, `ManualBarcodeScanner`/4` | `BarcodeInput: {onSubmit}` | `MainComponent` (`odoo_enterprise/18/stock_barcode/static/src/components/main.xml:113`, `stock_barcode.MainComponent`, `onSubmit.bind="onBarcodeSubmitted"`) |

Notas:
- **`BackorderDialog`** (`odoo_enterprise/18/stock_barcode/static/src/components/backorder_dialog.js::_onApply`): `_onApply` hace `await props.onApply()` y luego `props.close()`. El modelo le pasa `onApply: () => super._validate()` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::_validate`) → el diálogo es el gate del flujo de validación con backorder.
- **`ApplyQuantDialog`**: 3 botones en el XML (`odoo_enterprise/18/stock_barcode/static/src/components/apply_quant_dialog.xml:9-11`, `stock_barcode.ApplyQuantDialog`: Yes→`onApplyAll`, No→`onApply`, Discard→`props.close`). Body oculto (`bodyClass:'d-none'`, `:6`), solo footer. Los métodos `onApply`/`onApplyAll` (que llaman a `close`) están en `odoo_enterprise/18/stock_barcode/static/src/components/apply_quant_dialog.js::onApply-21`.
- **`ExtraProductDialog`** extiende `ConfirmationDialog` con `t-inherit="web.ConfirmationDialog" t-inherit-mode="primary"` (`odoo_enterprise/18/stock_barcode/static/src/components/extra_product_dialog.xml:4`, `template`), reemplaza el `<p>` por una lista de checkboxes. El modelo le pasa `products`, `cancel` (`:96`), `confirm` (`:97`), `close` (`:119`) y `onCheckboxChange: this.selectExtraProduct.bind(this)` (`:120`). La plantilla consume `productData.params[0].fieldsParams` (`:11`).
- **`BarcodeInput`** (`odoo_enterprise/18/stock_barcode/static/src/components/manual_barcode.js:4`, `BarcodeInput`): input con `useState({barcode})`, `onMounted` autofocus, `_onKeydown` dispara `props.onSubmit` con Enter. `ManualBarcodeScanner` (`:34`) extiende `web.BarcodeDialog` y mete `BarcodeInput` en el slot footer (`odoo_enterprise/18/stock_barcode/static/src/components/manual_barcode.xml:15-24`, `stock_barcode.ManualBarcodeScanner`), pasándole `onSubmit="props.onResult"`. Ojo: el `BarcodeInput` del client action (`odoo_enterprise/18/stock_barcode/static/src/components/main.xml:113`, `stock_barcode.MainComponent`) es un uso independiente con `onSubmit.bind="onBarcodeSubmitted"`.

## 4. Widgets de vista (`static/src/widgets/`)

Estos NO son parte del client action; se usan en los **form/kanban views** de `stock.move.line`, `stock.quant`, `stock.scrap` (registrados en `registry`, referenciados en `views/*.xml`).

- **`Digipad`** (`odoo_enterprise/18/stock_barcode/static/src/widgets/digipad.js:11`, `Digipad`) — `registry.category('view_widgets').add('digipad', digipad)` (`:182`). Props via `extractProps`: `attrs.field_to_edit`→`fieldToEdit`, `attrs.fulfilled_at`→`fulfilledAt` (`:175-180`). Teclado numérico +/−/erase/fulfill que escribe con `this.props.record.update({...})` (`:112,149,169`). Usa `useRecordObserver` para recargar botones de packaging al cambiar producto (`:37`). Referenciado en `odoo_enterprise/18/stock_barcode/views/stock_move_line_views.xml:52`, `stock_move_line_product_selector`, `odoo_enterprise/18/stock_barcode/views/stock_inventory_views.xml:43`, `stock_quant_barcode`, `odoo_enterprise/18/stock_barcode/views/stock_scrap_views.xml:63`, `scrap_product_selector`.
- **`SetReservedQuantityButton`** (`odoo_enterprise/18/stock_barcode/static/src/widgets/set_reserved_qty_button.js:8`, `SetReservedQuantityButton`) — `registry.category("fields").add("set_reserved_qty_button", ...)` (`:42`). Botón que copia la cantidad al campo destino (`fieldToSet`, de `attrs.field_to_set`) via `record.update` (`:28`).
- **`ImagePreviewField`** (`odoo_enterprise/18/stock_barcode/static/src/widgets/image_preview.js:16`, `ImagePreviewField`) extiende `ImageField`; añade `openImageFullScreen` → `dialog.add(FullScreenImage, {src})` (`:24-28`). `FullScreenImage` (`:8`). Registro `fields`/`image_preview` (`:35`). Usado en `odoo_enterprise/18/stock_barcode/views/stock_inventory_views.xml:30`, `stock_quant_barcode`, `odoo_enterprise/18/stock_barcode/views/stock_move_line_views.xml:39`, `stock_move_line_product_selector`.

## 5. Contrato modelo ↔ componente (métodos que la UI invoca)

En `barcode_model.js` (base). **Abstractos en la base** (lanzan `throw new Error('Not Implemented')`, hay que implementarlos/parchearlos en las subclases picking/quant): `getQtyDone` (`:74`), `getQtyDemand` (`:78`), `lineIsFaulty` (`:342`), `updateLineQty` (`:576`), `_updateLineQty` (`:1851`), `apply` (`:119`), getter `barcodeInfo` (`:123`).

**Con implementación en la base** (las subclases pueden sobreescribir): `selectLine` (`:493`), `selectPackageLine` (`:499`), `toggleSublines` (`:505`, con `trigger('update')` en `:511`), `deleteLine` (`:834`), `lineIsSelected` (`:1818`), `lineIsTracked` (`:346`), `lineCanBeDeleted` (`:338`, `return !getQtyDemand(line)`), `lineCanBeEdited` (`:1576`, `return true`), `lineCanBeSelected` (`:1572`, `return true`), `getIncrementQuantity` (`:101`), `getDisplayIncrementBtn` (`:82`, `return true`), `getDisplayIncrementBtnForSerial` (`:86`). `trigger('update')` es el pulso de refresco.

## 6. Cómo se re-renderiza / gotchas de reactividad
- No hay reactividad fina: **cualquier cambio = `trigger('update')` → `MainComponent.render(true)` (árbol entero)** (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart-237`).
- `t-key` de líneas = `line.virtual_id` (`odoo_enterprise/18/stock_barcode/static/src/components/main.xml:80`, `stock_barcode.MainComponent`, `odoo_enterprise/18/stock_barcode/static/src/components/grouped_line.xml:18`, `stock_barcode.GroupedLineComponent`). Cambiar identidad de línea sin conservar `virtual_id` provoca remount.
- `onPatched` en Main hace autoscroll a la línea seleccionada (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:139-141`, `MainComponent.onPatched`; `_scrollToSelectedLine` en `:506-550`). Usa selectores `.o_sublines .o_barcode_line.o_highlight`, `.o_barcode_line.o_highlight`, `.o_barcode_location_line`. Insertar/mover DOM de línea puede afectar el cálculo de scroll.
- `render` se salta si `state.uiBlocked` (durante procesado batch de barcodes).

## 7. Puntos de extensión (ver datos estructurados para detalle file:line y veredicto)
De menor a mayor invasividad:
1. **`t-inherit` de sub-templates** (`LineLowerButtons`, `LineTitle`, `LineQuantity`…) con `t-inherit-mode="extension"` + xpath. La forma más limpia de meter un botón/info en la línea sin tocar JS. Recuerda: cada sub-template lo usan distintos componentes (§2), así que el botón no aparece necesariamente en las tres líneas.
2. **Variable `lowerButtons`/`upperButtons`** vía `t-set` antes del `t-call` → sustituir/ocultar grupos de botones (seam incorporado, ya usado por package_line).
3. **`patch()` de `LineComponent`/`GroupedLineComponent`/`PackageLineComponent`** para getters o métodos. Parchear la base afecta a las subclases.
4. **`patch()` del modelo** (`BarcodeModel`/`BarcodePickingModel`/`BarcodeQuantModel`) para la lógica real. Muchos getters de visibilidad SÍ tienen base (parchear la base surte efecto); los realmente abstractos (getQtyDone/getQtyDemand/lineIsFaulty/updateLineQty) hay que implementarlos en la subclase correcta.
5. **`patch(MainComponent, {...})` de `static components`** o `t-inherit` de `main.xml` para **sustituir** el componente de línea.
6. **Nuevo Dialog propio** abierto con `env.dialog.add` desde un método del componente, o `dialogService.add` desde el modelo.
7. **Nuevo widget de vista** registrado en `registry.category('fields'|'view_widgets')` y referenciado por herencia en los form views.
8. **Ampliar `static props`** al parchear un componente si necesitas pasarle datos nuevos (OWL valida props estrictamente).

## 8. Riesgos transversales
- **Validación de props OWL**: `LineComponent.props` es lista cerrada (`odoo_enterprise/18/stock_barcode/static/src/components/line.js:7`, `LineComponent`); pasar props no declaradas revienta. Ampliar `static props` en el patch.
- **Herencia por prototipo**: `patch(LineComponent)` propaga a Grouped/Package salvo override; efecto deseado o bug silencioso.
- **UI vs modelo desacoplados**: un override de getter de visibilidad no cambia comportamiento si el método del modelo correspondiente no retorna lo esperado. Verifica en qué clase (base o picking/quant) vive el método.
- **Refresco**: mutar modelo sin `trigger('update')` = no refresca; y con `uiBlocked` el render se ignora.
- **Bundle glob**: nuevos ficheros bajo `static/src/**` entran automáticamente; el orden de patches dentro del glob no está garantizado (usa dependencia de módulo para asegurar carga posterior).


## Correcciones aplicadas en verificación

- GOTCHA CLAVE (refutado): el borrador afirma que los getters de visibilidad getDisplayIncrementBtn, lineCanBeDeleted (y por extensión lineCanBeEdited) 'viven en las subclases del modelo, NO en BarcodeModel base'. FALSO. En barcode_model.js SÍ tienen implementación base: getDisplayIncrementBtn → return true (:82), lineCanBeDeleted → return !this.getQtyDemand(line) (:338), lineCanBeEdited → return true (:1576), lineIsTracked (:346), lineIsSelected (:1818), getIncrementQuantity (:101). Las subclases pueden sobreescribirlos, pero existen en la base. Parchear la base SÍ tiene efecto sobre ambas subclases del modelo.
- Los métodos REALMENTE abstractos en la base (lanzan `throw new Error('Not Implemented')`) son: getQtyDone (:74), getQtyDemand (:78), lineIsFaulty (:342-343), updateLineQty (:576), _updateLineQty (:1851), apply (:119), y el getter barcodeInfo (:123). El borrador solo señalaba updateLineQty como abstracto; getQtyDone/getQtyDemand también lo son y no se advertía.
- §5 listaba getQtyDone/getQtyDemand como métodos base normales; en realidad son abstractos (throw) en BarcodeModel e implementados en picking/quant. Corregido.
- PackageLineComponent.qtyDone (odoo_enterprise/18/stock_barcode/static/src/components/package_line.js::qtyDone-28): el borrador (riesgo del extension point #4) advierte de 'divide por reservedQuantity: cuidado con divisiones'. La división está GUARDADA por `if (reservedQuantity > 0)`; si es 0 devuelve 1 o 0. No hay riesgo de división por cero. Corregido.
- Riesgo del extension point #1 (sub-templates): el borrador dice que un t-inherit de un sub-template 'aparecerá en las tres' líneas (Line/Grouped/Package). Es una generalización imprecisa. Verificado qué sub-template usa cada componente: LineQuantity y LineDestinationLocation → los tres; LineTitle → Line+Grouped (Package tiene su propia cabecera, no lo t-llama); LineLowerButtons → Line+Grouped (Package lo oculta con lowerButtons=false); LineOwner → Line+Package (Grouped NO lo t-llama); LineLot → Line+Grouped(condicional). El impacto de un t-inherit depende del sub-template concreto.
- Manifest (§1 y gotcha): la notación `static/src/**/*.{js,scss,xml}` es incorrecta; en __manifest__.py son TRES globs separados: `stock_barcode/static/src/**/*.js` (:37), `**/*.scss` (:38), `**/*.xml` (:39), dentro de web.assets_backend (:36). Además hay un `('remove', '**/*.dark.scss')` (:42). Line refs :36-40 correctos, notación corregida.
- Selección de componente por línea: el borrador cita odoo_enterprise/18/stock_barcode/static/src/components/main.xml:80-84; las líneas reales son :81 (PackageLineComponent t-if=line.isPackageLine, openPackage.bind=onOpenPackage), :82 (GroupedLineComponent t-elif=line.lines, editLine.bind=onOpenProductPage), :83 (LineComponent t-else, editLine.bind=onOpenProductPage). CONFIRMADO el cableado editLine→onOpenProductPage; ajustado el rango de líneas.
- componentClasses (odoo_enterprise/18/stock_barcode/static/src/components/line.js::componentClasses): el borrador lista 'o_line_completed/o_selected/o_highlight/o_faulty'. Completado: también emite o_line_not_completed (excluyente con o_line_completed) cuando la línea no está completa. Sin cambio funcional pero la lista estaba incompleta.
- GroupedLineComponent.isSelected (odoo_enterprise/18/stock_barcode/static/src/components/grouped_line.js::isSelected-27): usa `this.line.virtual_ids.indexOf(...) !== -1`, no `.includes()`. Equivalente semánticamente; corregido por precisión.
- ManualBarcodeScanner (odoo_enterprise/18/stock_barcode/static/src/components/manual_barcode.xml:15-24): el BarcodeInput del footer recibe `onSubmit="props.onResult"` (del BarcodeDialog padre), no un onSubmit directo. El BarcodeInput usado en el client action (odoo_enterprise/18/stock_barcode/static/src/components/main.xml:113) sí recibe `onSubmit.bind="onBarcodeSubmitted"`. Son dos usos distintos de BarcodeInput; matizado.
- apply_quant_dialog: los 3 botones (Yes→onApplyAll, No→onApply, Discard→close) están en odoo_enterprise/18/stock_barcode/static/src/components/apply_quant_dialog.xml:9-11; odoo_enterprise/18/stock_barcode/static/src/components/apply_quant_dialog.js::onApply-21 contiene los métodos onApply/onApplyAll. El borrador atribuía los botones al .js. Ref corregida (js define métodos, xml define botones y bodyClass='d-none' :6).
