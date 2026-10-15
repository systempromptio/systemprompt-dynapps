## Modelo mental (aplica a las DOS apps)

Ambas apps de barcode de Odoo 18 EE son **client actions OWL de pantalla completa**, no vistas de formulario. El patrón compartido es:

**client action (registry) + estado 100% en cliente + cache local + contrato RPC batch con el servidor.**

- **stock_barcode** (recepciones/entregas/internos + ajuste de inventario): tag `stock_barcode_client_action`, componente raíz `MainComponent` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js:66`, `MainComponent`, `export default`, registrado en `odoo_enterprise/18/stock_barcode/static/src/components/main.js:620`, `stock_barcode_client_action`). La lógica vive en un **modelo JS** (`BarcodeModel` y subclases), NO en el componente.
- **mrp_display** (Shop Floor de fabricación, módulo `mrp_workorder`): tag `mrp_display`, componente raíz `MrpDisplayAction` → `MrpDisplay`. Orientado a tablet, batch de MOs/WOs con validación diferida.

El punto clave de arquitectura: **el componente OWL solo pinta; toda la lógica de negocio está en un objeto plano JS** (`BarcodeModel extends EventBus`, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:14`, `BarcodeModel`). Ese modelo **NO es reactivo**: cualquier mutación de estado debe terminar en `this.trigger('update')` (u otro evento suscrito) para que el componente re-renderice. Olvidarlo deja la UI desincronizada sin ningún error.

## Flujo de un escaneo end-to-end (stock_barcode)

1. Arranque: `MainComponent.setup()` → `_getModel()` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel-289`) instancia `BarcodePickingModel` (`:283`) o `BarcodeQuantModel` (`:285`) según `resModel`; si no es stock.picking/stock.quant, lanza (`:287`).
2. El modelo llama RPC `/stock_barcode/get_barcode_data` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::getActionRefresh`, definido en `getActionRefresh` `:94`). El controlador (`odoo_enterprise/18/stock_barcode/controllers/stock_barcode.py:89`, `StockBarcodeController`) compone un blob con `_get_stock_barcode_data` (picking `stock_barcode/models/stock_picking.py:85`, `StockPicking._get_stock_barcode_data`, quant `stock_barcode/models/stock_quant.py:75`, `StockQuant._get_stock_barcode_data`) — un dict `records` con **12 modelos** (picking) o los del quant + `res.company`.
3. `setData()` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::setData-52`) reconstruye en cada carga: cachea `records` en `LazyBarcodeCache` (indexado por modelo y por barcode vía `_getBarcodeField`), fija la nomenclatura (getRecord `barcode.nomenclature`, `:40`) y construye el parser `new BarcodeParser({nomenclature})` (`:45`).
4. Escaneo → `_parseBarcode` (`:1017`) usa el `BarcodeParser` (core `barcodes`) para producir `{encoding,type,code,base_code,value}`; `type` sale de `barcode.rule.type` (Selection FIJA en el core). Si el código casa un comando de `_getCommands` (`:860`), se ejecuta como `result.action` y **cortocircuita** el resto del procesado (`:1260-1262`).
5. Si no es comando → `_processBarcode` (`:1223`, bajo `actionMutex`) despacha a subprocesadores (`_processLocation:1534`, `_processPackage:1550`, GS1 `_processGs1Data:1154`), busca/crea línea (`_findLine:1699`, `createNewLine`), actualiza cantidades vía `updateLine`→`_markLineAsDirty` (`:566/:992`) y emite eventos (`update`, `flash`, `playSound`...). Múltiples early-return.
6. Validar → `_validate` (picking `:960`): agrega `uncompletedLines`, decide backorder/firma, y hace RPC. `save()` (`:484`) envía comandos x2many crudos (`_getSaveCommand`) al write; el diff se calcula comparando `currentState` vs `initialState` en `_getSaveLineCommand` (`:909`). `post_barcode_process` (`stock_barcode/models/stock_move.py:54`, `StockMove.post_barcode_process`) corre dentro de la validación con `context['barcode_trigger']=true` (`:602`).

## Árbol OWL — stock_barcode (verificado contra main.xml)

```
MainComponent (odoo_enterprise/18/stock_barcode/static/src/components/main.xml:10, .o_barcode_client_action)
├── .o_barcode_header (:13)
├── rama view==='barcodeLines' (:66)  ← estado por defecto
│   └── t-foreach env.model.groupedLinesByLocation (:67)  ← agrupado por UBICACIÓN
│       └── por grupo: t-foreach group.lines (:80-84)
│           ├── PackageLineComponent  t-if=line.isPackageLine (:81)  openPackage→onOpenPackage
│           ├── GroupedLineComponent   t-elif=line.lines   (:82)  editLine→onOpenProductPage
│           └── LineComponent          t-else              (:83)  editLine→onOpenProductPage
├── rama view==='actionsView' (:113)   ← destino del botón cog
│   └── BarcodeInput (onSubmit.bind=onBarcodeSubmitted) + botones Print/Operations
├── rama view==='infoFormView' (:109)  ← form embebido (productPage/scrapProductPage viven aquí)
│   └── Chatter (SOLO en esta rama)
├── rama view==='packagePage' (:94-100)  ← <View type="kanban"> sobre stock.quant
└── (otros estados: productPage, scrapProductPage)
```

Estados REALES de la máquina de vistas (`state.view`): `barcodeLines`, `actionsView`, `infoFormView`, `productPage`, `packagePage`, `scrapProductPage`. **`printMenu`/`editFormView` NO existen** (solo sobreviven en un comentario obsoleto `odoo_enterprise/18/stock_barcode/static/src/components/main.js:103`, `MainComponent.setup`).

Ojo doble getter: `this.lines` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::lines`) devuelve `env.model.groupedLines` (plano), pero la plantilla del listado consume `env.model.groupedLinesByLocation` (`odoo_enterprise/18/stock_barcode/static/src/components/main.xml:67`, `stock_barcode.MainComponent`), NO `this.lines`.

## Árbol OWL — mrp_display (Shop Floor)

```
MrpDisplayAction (fieldsStructure getter :20-122, fieldsManuallyFetched :125-131)
└── MrpDisplay (withSearchProps: MrpDisplaySearchModel, odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_action.js::setup)
    ├── MrpDisplaySearchModel (search_model.js) → workorderFilters CLIENT-SIDE (:9-37)
    ├── filteredWorkorders (:271) / defineRelevantRecords (:350, recordCacheIds)
    ├── MrpDisplayRecord (odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_record.js:21, record.component=this :60)
    │   ├── sub-templates: MrpDisplayProductionHeader/WorkorderHeader/Footer (xml:105-144)
    │   ├── validate (:486) → realValidation (:530); moves (:200); checks (:181)
    │   └── QualityCheck (quality_check.js): pass() switch (:33), icon (:86)
    │       └── MrpQualityCheckConfirmationDialog.validate() (:67)  ← measure, etc.
    ├── validationStack: addToValidationStack(:187) → processValidationStack(:302, button_mark_done en lote :325) → removeFromValidationStack(:453)
    ├── MrpMenuDialog (odoo_enterprise/18/mrp_workorder/static/src/mrp_display/dialog/mrp_menu_dialog.js:11, callAction :32 con context from_shop_floor)
    └── useConnectedEmployee hook (odoo_enterprise/18/mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js::useConnectedEmployee) → {connected, all, admin}
```

Dominio base de MOs cargadas: `state in [confirmed,progress,to_close]` AND (`bom_id=False` OR `bom_id.type in [normal,phantom]`) (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_action.js::setup-145`). `processValidationStack` se ejecuta en `onWillDestroy`, `selectWorkcenter` y `onClickRefresh`.

## Diferencias de arquitectura entre las dos apps

| Aspecto | stock_barcode | mrp_display (Shop Floor) |
|---|---|---|
| Modelo de estado | `BarcodeModel extends EventBus` (imperativo, trigger manual) | Relational-style + validationStack + SearchModel client-side |
| Carga | blob único `/stock_barcode/get_barcode_data` + LazyBarcodeCache | `fieldsStructure` getter + read batch, filtros client-side |
| Unidad de trabajo | picking / quant, líneas agrupadas por ubicación | MOs/WOs, tarjetas, batch async con animación |
| Validación | `save()` + `_validate` + backorder/firma | `processValidationStack` en lote (button_mark_done) |
| Empleados/PIN | n/a | `useConnectedEmployee` + validación PIN server-side |

## Manifest / assets (verificado, notación corregida)

**stock_barcode `__manifest__.py`**: `web.assets_backend` en `:36`; TRES globs separados (NO brace-expansion): `**/*.js` (`:37`), `**/*.scss` (`:38`), `**/*.xml` (`:39`); `('remove', **/*.dark.scss)` (`:42`); `web.assets_web_dark` (`:44-45`); tests unit `web.qunit_suite_tests` (`:47-48`); tours `web.assets_tests` (`:50-51`).

**mrp_workorder `__manifest__.py`**: TRES globs separados en `web.assets_backend` (`:40-45`), `('remove', mrp_workorder_gantt_*)` (`:44`), y los gantt re-añadidos en `web.assets_backend_lazy` (`:46-48`). Todo static/src va al backend salvo el gantt.
