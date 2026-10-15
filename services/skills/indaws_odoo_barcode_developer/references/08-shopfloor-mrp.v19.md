# Shop Floor / mrp_display — Odoo 19 Enterprise (`mrp_workorder`)

Bundle OWL de la app "Shop Floor" de fabricación. Ruta base: `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/`.

> **Corrección al enunciado**: en v19 NO se ha eliminado la carpeta `mrp_record_line/`. Sigue existiendo con `mrp_workorder.js`, `quality_check.js`, `stock_move.js`. Lo que sí desaparece: `mrp_record_line/mrp_worksheet.js/.xml`, los `.scss` de `quality_check`/`mrp_workorder` dentro de esa carpeta, la carpeta `mrp_employee_list/` entera y `dialog/mrp_quality_check_confirmation_dialog.*`. Se añaden `dialog/mrp_employee_dialog.*`, `dialog/mrp_quantity_dialog.*` y `dialog/mrp_select_quant_dialog.js`.
>
> **Matiz importante (corregido en v19)**: aunque la carpeta JS `mrp_employee_list/` y su uso desaparecen, la **acción server `mrp_workorder.action_open_employee_list` SIGUE existiendo** como `ir.actions.act_window` en `odoo_enterprise/19/mrp_workorder/views/mrp_workorder_views.xml:441`, `action_open_employee_list`. Lo que cambia es que el shop floor ya no la invoca (`doAction`); ahora usa `MrpEmployeeDialog` + `hr.employee.set_employees_connected`.

## 1. Registro y bootstrap

- Client action registrado en `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_action.js:196`, `mrp_display`: `registry.category("actions").add("mrp_display", MrpDisplayAction)`. Entrada = `ir.actions.client` con `tag="mrp_display"`.
- `MrpDisplayAction` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_action.js:11`, `MrpDisplayAction`) es un wrapper fino sobre `WithSearch`. En `onWillStart` (`:148`):
  1. Carga fields por modelo con `fieldService.loadFields` para 6 modelos declarados en el getter `fieldsStructure` (`:18`): `mrp.production`, `mrp.workorder`, `stock.move`, `stock.move.line`, `quality.check`, `stock.lot`. Cada field se decora con `defaultActiveField` (`:9`) y se le borra `context` (`:152-154`).
  2. Carga la search view (`viewService.loadViews`, `:158`).
  3. Fija `context.limit` llamando a `mrp.workorder.get_shopfloor_limit` (`orm.call` en `:170`).
  4. Construye `withSearchProps` (`:172`) con el dominio base montado en `setup` (`:139`): `state in [confirmed, progress, to_close]` + filtro BOM (`bom_id=false OR bom_id.type in [normal, phantom]`), y añade `picking_type_id` si `context.active_model === "stock.picking.type"` (`:145`).
- Assets: `stock_barcode/__manifest__.py:43-45` incluye globalmente `mrp_workorder/static/src/**/*.{scss,js,xml}` (con removes puntuales del gantt en `:46/:49`). No hay entradas por-fichero, así que cualquier `.js/.xml/.scss` nuevo bajo `static/src/` entra automáticamente en el bundle.

## 2. Componente raíz: `MrpDisplay` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display.js:33`, `MrpDisplay`)

- `static template = "mrp_workorder.MrpDisplay"` (`:34`). Subcomponentes (`:35`): `ControlPanelButtons`, `MrpDisplayRecord`, `MrpDisplayEmployeesPanel`, `PinPopup`, `MrpDisplaySearchBar`, `CheckboxItem`, `Pager`.
- Estado de datos: usa `useState(useModel(RelationalModel, params))` (`:95`). `_makeModelParams()` (`:446`) monta a mano el árbol de `activeFields` con relaciones anidadas: `move_raw_ids`/`move_byproduct_ids`/`move_finished_ids` → `stock.move` → `move_line_ids` → `stock.move.line`; `workorder_ids` → `mrp.workorder` (con sus propios `move_raw_ids` y `check_ids`); `check_ids` → `quality.check` (con `lot_ids` → `stock.lot` y `production_id` → `mrp.production`); `lot_producing_ids` → `stock.lot` con `onChange = true` (`:497`).
- `useSubEnv({ model, reload })` (`:96`): `reload` es la pieza central de refresco. Si recibe un `record` (y no `record.dismiss`), sube por `_parentRecord` hasta el `mrp.production` y hace `record.load()` + `model.notify()` reemplazando en `model.root.records` (`:99-109`); si no, recarga la raíz con offset/limit (`:111-114`). Siempre termina con `useEmployee.getConnectedEmployees()` (`:116`).
- **Workcenters como "pestañas"**: `defaultWorkcenterButtons` (`:28`) = `[{id:0, "Overview"}, {id:-1, "My WO"}]`. La selección activa se persiste en `localStorage` bajo la clave `mrp_workorder.db_<db>.user_<uid>` (subenv `localStorageName`, `:74`) y `...activeWC`. `_loadActiveWorkcenter` (`:567`) prioriza contexto → localStorage → primer WC.
- `state` (`:81`): `activeResModel` (`mrp.production` vs `mrp.workorder`), `activeWorkcenter`, `workcenters`, `canLoadSamples`, `offset`, `limit`, `firstLoad`.
- **`defineRelevantRecords()` (`:334`)** — llamado en `onWillRender` (`:160`). Es el corazón del render: decide qué tarjetas se muestran según `activeResModel`/`activeWorkcenter` (`-1` = "My WO" filtra por `adminWorkorderIds`), y ordena por caché (`recordCacheIds`, para estabilidad visual, con `showCaseId` primero) + estado del WO (`progress<ready<pending<waiting<finished`) con desempate por `date_start` (dates false al final). Resultado en `this._relevantRecords`, expuesto por el getter `relevantRecords` (`:402`).
- **Barcode** (`_onBarcodeScanned`, `:186`): resuelve en orden MO por nombre (`this.productions.find`) → WO por `barcode` → empleado (`mrp.workcenter.get_employee_barcode`, `:206`) → producto (`_onProductBarcodeScanned`, `:213`, que casa contra quality checks de consumo, moves manuales `manual_consumption` y byproducts). Ignora prefijos `OBT`/`OCD` y cuando hay un dialog abierto (overlay `DialogWrapper`).
- `onWillStart` (`:131`) carga grupos: `mrp.group_mrp_byproducts`, `uom.group_uom`, `mrp.group_mrp_routings` (→ `workorders`), `mrp_workorder.group_mrp_wo_tablet_timer` (→ `timer`).
- `popupAddEmployee()` (`:545`) abre el **nuevo** `MrpEmployeeDialog` pasándole `setConnectedEmployees` y `employees` (en v18 lanzaba un `doAction` a la lista de empleados).

## 3. Tarjeta: `MrpDisplayRecord` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_record.js:17`, `MrpDisplayRecord`)

Renderiza una tarjeta de MO o WO. `static template = "mrp_workorder.MrpDisplayRecord"` (`:41`).

- Getters de contenido: `checks` (`:155`, ordena la cadena de QC vía `previous_check_id`/`next_check_id`), `moves` (`:174`, filtra scrapped/bom_line/workorder), `byProducts` (`:136`), `workorders` (`:190`).
- `subRecordProps(subRecord)` (`:213`) es el dispatcher que construye props para cada línea hija según `resModel` y `test_type`. Para `quality.check` de tipo `register_consumed_materials`/`register_byproducts` **sustituye** el subRecord por el `stock.move` correspondiente (casado por `check_id.resIds.includes(...)`), es decir esos QC se renderizan como `StockMove`. Marca `isCurrent` comparando con `current_quality_check_id`.
- Acciones clave:
  - `registerProduction()` (`:71`): si `product_tracking=="serial"` + `product_qty>1` llama a `action_generate_serial` (`:79`, abre wizard); si no abre `MrpRegisterProductionDialog` (`:102`).
  - `validate()` (`:312`): flujo de cierre. Para WO: si `ready`+`qty_producing===0` setea `qty_producing` (`:318`), hace `stop_employee` del empleado validador si está conectado (`:327`), `record.save()` (`:333`), fija `methodName="do_finish"` con `context={no_start_next, mrp_display, employee_id}` (`:334-338`). Si es MO (o `picking_type_auto_close` + `is_last_unfinished_wo`) usa `button_mark_done` sobre `mrp.production` (`:345`). El `orm.call` único que ejecuta el método está en `:351`. Maneja backorders (`marked_as_done`, `:353`), quita la MO de `model.root.records` sin recargar todo (`:368-373`), y limpia el MO filter salvo `shouldKeepMoFilter` (`:384`).
  - `startWorking(shouldStop)` (`:422`): `button_start` (`:430`) / `stop_employee` (`:435`) sobre el WO con `context={mrp_display}`.
  - `onClickOpenMenu()` (`:293`) abre `MrpMenuDialog`; `editLogNote()` (`:105`) abre `MrpLogNoteDialog`.
- `disabled` (`:269`) / `active` (`:263`) gobiernan interactividad según el empleado sesión (`sessionOwner`) y `allowed_employees`/`all_employees_allowed`.

## 4. Líneas hijas (`mrp_record_line/`) — jerarquía de herencia

Cadena de herencia OWL (clave para extender): **`MrpWorkorder` → `QualityCheck` → `StockMove`**.

- **`MrpWorkorder` (`mrp_record_line/mrp_workorder.js:5`)**: línea base de un WO dentro de una MO. `onClick`→`clicked()`→`selectWorkcenter(workcenter.id, resId)`. Template `mrp_workorder.WorkOrder`.
- **`QualityCheck` (`mrp_record_line/quality_check.js:8`, `QualityCheck`) extends `MrpWorkorder`**: aquí vive el **flujo de calidad inline** (ver §6). `clicked()` (`:116`) despacha por `test_type` (`instructions`/`print_label`/`picture`/`register_production`); `doActionAndNext(action, stateToSet="pass")` (`:147`) llama al server (`action_next`/`action_print`) con `context={from_shopfloor}`, actualiza `quality_state` y `current_quality_check_id` en cliente (`:155-156`) y encadena `startWorking` (`:166`); `onFileUploaded` (`:140`) sube foto vía `FileUploader` y avanza; `showWorksheet()` (`:169`) abre `MrpWorksheetDialog`. Getters `icon` (`:61`) y `showQty` (`:98`). Template `mrp_workorder.QualityCheck`.
- **`StockMove` (`mrp_workorder/static/src/mrp_display/mrp_record_line/stock_move.js:9`, `StockMove`) extends `QualityCheck`**: línea de consumo/byproduct. `clicked()` (`:171`) decide botón según tracking: tracked+prefilled (`displayCheck`)→`markAsDone()` (`action_next`, `:150`); tracked+byproduct→`createQuant()` (`:198`); tracked normal→`addMoveLine()` (`:122`)→`MrpSelectQuantDialog`; untracked completo→`undo()` (`action_undo`, `:160`); untracked→`editUntrackedMove()` (`:164`)→`MrpQuantityDialog`. `selectQuant` (`:192`) llama `action_add_from_quant` (`:194`); `createQuant` usa `useOpenMany2XRecord` sobre `stock.quant` (`:31`). Getter `icon` (`:48`). Template `mrp_workorder.StockMove`.

## 5. Diálogos (`dialog/`)

| Fichero | Clase / base | Rol |
|---|---|---|
| `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_employee_dialog.js:6`, `MrpEmployeeDialog` **(NUEVO v19)** | `MrpEmployeeDialog extends ConfirmationDialog` | Multi-selección de empleados conectados con checkboxes + barcode; en `confirm()` (`:33`) llama `setConnectedEmployees(this.selected.ids)`. Sustituye a la vieja `mrp_employee_list/`. |
| `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_quantity_dialog.js:4`, `MrpQuantityDialog` **(NUEVO v19)** | `MrpQuantityDialog extends ConfirmationDialog` | Edita cantidad de un `stock.move` untracked (FloatField). `apply()` (`:15`) guarda y llama `action_pass`; `remove()` (`:24`) hace `record.delete()`; `_dismiss()` (`:29`) hace `record.discard()`. |
| `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_select_quant_dialog.js:5`, `MrpSelectQuantDialog` | `MrpSelectQuantDialog extends SelectCreateDialog` | Selector de `stock.quant` para añadir move line, con barcode (`get_quant_from_barcode`, `:24`). **Nota**: el marcador `// TODO remove in master` NO está en este fichero, sino en `stock_move.js` (import en `:7` y método `addMoveLine` en `:121`). |
| `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_register_production_dialog.js:8`, `MrpRegisterProductionDialog` | `MrpRegisterProductionDialog extends ConfirmationDialog` **(cambió base en v19)** | Registrar producción: qty + lot/serial. `validate()` (`:43`) guarda y llama `set_qty_producing` (`:49`). También `actionGenerateSerial()` (`:54`) y `actionClearLotProducing()` (`:71`). En v18 heredaba de `MrpQualityCheckConfirmationDialog` (verificado). |
| `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_menu_dialog.js:9`, `MrpMenuDialog` | `MrpMenuDialog extends Component` (template `mrp_workorder.MrpDisplayMenuDialog`) | Menú "Options": scrap, quality alert, add component/byproduct, move WC, openMO, block/unblock, y propuestas de cambio de ruta (add/update/remove step, set picture). Submenús vía `state.menu` (`:32`). `callAction` usa `context={from_shop_floor}` (`:41`). |
| `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_check_select_dialog.js:3`, `MrpQualityCheckSelectDialog` | `MrpQualityCheckSelectDialog extends ConfirmationDialog` | Selector de step de QC para las propuestas de cambio (`proposeChange`). |
| `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_worksheet_dialog.js:5`, `MrpWorksheetDialog` | `MrpWorksheetDialog extends ConfirmationDialog` | Muestra worksheet/documento del QC (PDF/imagen). |
| `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_workcenter_dialog.js:6`, `MrpWorkcenterDialog` | `MrpWorkcenterDialog extends ConfirmationDialog` | Activar work centers / mover a WC. |
| `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_log_note_dialog.js:4`, `MrpLogNoteDialog` | `MrpLogNoteDialog extends ConfirmationDialog` | Editar log note de la MO. |

## 6. Flujo de calidad reestructurado (v18 → v19) — CAMBIO ESTRUCTURAL

En v18 existía `dialog/mrp_quality_check_confirmation_dialog.*` (verificado presente en v18, ausente en v19): un gran diálogo modal que centralizaba TODOS los tipos de quality check. En v18 `QualityCheck` (record line) tenía props `displayInstruction`/`quantityToProduce` (verificado, `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/quality_check.js:16-17`, `QualityCheck` de v18) y un `pass()` monolítico.

En **v19 ese diálogo se elimina** y el flujo pasa a ser **inline en las propias líneas**:
- `QualityCheck.clicked()` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/quality_check.js::clicked`) resuelve el check directamente contra el server (`doActionAndNext` → `action_next`/`action_print`) sin abrir el modal confirmatorio.
- La foto se sube inline con `FileUploader` + `onFileUploaded` (`:140`).
- El worksheet/instrucciones se muestran en su propio `MrpWorksheetDialog` (`showWorksheet`, `:169`).
- `register_production` delega en `MrpDisplayRecord.registerProduction()` → `MrpRegisterProductionDialog`.
- El consumo de componentes (`register_consumed_materials`/`register_byproducts`) se renderiza como `StockMove` (ver `subRecordProps`) y se gestiona con los nuevos `MrpQuantityDialog` (untracked) y `MrpSelectQuantDialog` (tracked).
- `MrpRegisterProductionDialog` ya no hereda del confirmation dialog sino de `ConfirmationDialog` estándar (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_register_production_dialog.js:8`, `MrpRegisterProductionDialog`).

Resultado: la lógica de calidad está distribuida entre `quality_check.js`, `stock_move.js` y diálogos pequeños en vez de un único componente. Extender un `test_type` ya no es tocar un switch en el confirmation dialog sino el `clicked()`/`icon`/`showQty` de `QualityCheck`.

## 7. Flujo empleado/cantidad reestructurado (v18 → v19) — CAMBIO ESTRUCTURAL

**Empleado**: v18 tenía la carpeta `mrp_employee_list/` (componente `MrpEmployeeList`, verificado presente en v18) y el hook exponía `popupAddEmployee` (verificado `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js::useConnectedEmployee/214` de v18) que abría `doAction("mrp_workorder.action_open_employee_list")`. En **v19**:
- Se elimina la carpeta JS `mrp_employee_list/` y el shop floor deja de invocar `action_open_employee_list`. **OJO (corregido)**: el `ir.actions.act_window` `action_open_employee_list` **sigue existiendo** en `odoo_enterprise/19/mrp_workorder/views/mrp_workorder_views.xml:441`, `action_open_employee_list`; lo eliminado es el código JS y su uso, no la acción server.
- El hook `employee_hooks.js` reemplaza `popupAddEmployee` por **`setConnectedEmployees(ids)`** (`:195`), que hace logout del admin si sale de la lista (`:196-199`), llama al **nuevo método server `hr.employee.set_employees_connected`** (verificado nuevo en v19 — `odoo_enterprise/19/mrp_workorder/models/hr_employee.py:135`, `HrEmployee.set_employees_connected`; ausente en v18) (`:200`) y refresca.
- `MrpDisplay.popupAddEmployee` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display.js::popupAddEmployee`) abre `MrpEmployeeDialog` (multi-check) pasándole ese callback.
- El orden de la lista de conectados cambió: v19 introduce el algoritmo "MGMSort" (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js::useConnectedEmployee-116`) que mantiene orden estable y usa `emp.workcenters.length` (v18 usaba `emp.workorder.length`, verificado `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js::useConnectedEmployee/89` de v18).

El resto del hook `useConnectedEmployee` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js::useConnectedEmployee`) es estable: `login`/`logout`/`pin_validation` vía `hr.employee`, `PinPopup` envuelto en `DialogWrapper` (`:24-40`), `admin` = session owner, `start_employee`/`stop_employee` sobre `mrp.workorder` (`:42-48`).

**Cantidad**: el long-press + animación de v18 (`toggleQuantityDone`, `onAnimationEnd`, `isLongPressable`/`longPressed`, verificados en `stock_move.js` de v18) desaparece. v19 usa botones explícitos con iconos (`icon` getter, `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/stock_move.js::icon`) y los diálogos `MrpQuantityDialog`/`MrpSelectQuantDialog`.

## 8. Otros componentes

- `control_panel.js` (`ControlPanelButtons`, clase `:4`, `setup:18`): botonera de workcenters (`workcenterButtons:35`, `moveWorkcenterButtons:70`), overview/My WO, refresh, add employee.
- `search_bar.js` / `search_model.js` (`MrpDisplaySearchModel`, `:5`): search custom con filtros de workorder por estado (`workorderFilters:9`), `setWorkcenterFilter:59`, `removeMOFilter:75`. `addAutoCompletionValues` es método heredado del `SearchModel` base (usado desde `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display.js::_onProductionBarcodeScanned`), no custom.
- `employees_panel.js` (`MrpDisplayEmployeesPanel`, `:6`): panel lateral de empleados conectados.

## 9. Modelos/estado (cliente)

No hay modelos JS propios más allá del `RelationalModel` estándar parametrizado en `_makeModelParams` (§2). El "estado" es: `MrpDisplay.state` (useState) + `model.root.records` (MOs) + `employees` (useState del hook) + `localStorage` (workcenters activos). No hay store OWL ni servicio propio registrado; todo cuelga del subenv (`model`, `reload`, `localStorageName`).

## 10. RPC cliente → servidor (verificadas)

| Método server | Modelo | Caller JS | Propósito |
|---|---|---|---|
| `get_shopfloor_limit` | `mrp.workorder` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_action.js::setup` | Límite de paginación inicial. |
| `can_load_samples` / `action_load_samples` | `mrp.production` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display.js::setup` / `:527` | Datos de muestra. |
| `get_employee_barcode` | `mrp.workcenter` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display.js::_onBarcodeScanned`, `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_employee_dialog.js::_onBarcodeScanned` | Resolver barcode de empleado. |
| `button_start` | `mrp.workorder` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_record.js:430`, `MrpDisplayRecord.admin_id` | Iniciar WO (context `mrp_display`). |
| `stop_employee` / `start_employee` | `mrp.workorder` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_record.js::validate/435`; `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js::useConnectedEmployee/43` | Timer de empleado en WO. |
| `do_finish` | `mrp.workorder` | `mrp_display_record.js` (methodName `:334`, call `:351`) | Cerrar WO (context `no_start_next, mrp_display, employee_id`). |
| `button_mark_done` | `mrp.production` | `mrp_display_record.js` (methodName `:345`, call `:351`) | Validar MO, backorders. |
| `set_qty_producing` | `mrp.production` / `mrp.workorder` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_register_production_dialog.js::validate` | Fijar cantidad producida. |
| `action_generate_serial` | `mrp.production` / `mrp.workorder` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_record.js::registerProduction`; `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_register_production_dialog.js::actionGenerateSerial` | Generar serie (abre wizard). |
| `action_clear_lot_producing_ids` | `mrp.production` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_register_production_dialog.js::actionClearLotProducing` | Limpiar lotes en curso. |
| `action_next` | `quality.check` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/quality_check.js:149`, `QualityCheck.doActionAndNext`; `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/stock_move.js::markAsDone` | Siguiente QC / confirmar prefilled; devuelve `next_check_id`. |
| `action_print` | `quality.check` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/quality_check.js:149`, `QualityCheck.doActionAndNext` (branch `print_label`, `:126`) | Imprimir etiqueta. |
| `action_pass` | `stock.move` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_quantity_dialog.js::apply` | Confirmar consumo untracked (`mrp_workorder/models/stock_move.py:56`, `StockMove.action_pass`). |
| `action_undo` | `stock.move` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/stock_move.js::undo` | Deshacer consumo untracked. |
| `action_add_from_quant` | `stock.move` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/stock_move.js::selectQuant` | Crear move line desde quant. |
| `get_quant_from_barcode` | `stock.move` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_select_quant_dialog.js::_onBarcodeScanned` | Resolver quant por barcode. |
| `action_add_step` / `action_propose_change` / `action_add_component` / `action_add_byproduct` | `mrp.workorder` / `mrp.production` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_menu_dialog.js::proposeChangeForCheck/206/62/70` | Cambios de ruta y añadir componentes/subproductos. |
| `set_employees_connected` | `hr.employee` | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js::useConnectedEmployee` | **NUEVO v19**: fijar lista de empleados conectados. |
| `get_all_employees` / `login` / `logout` / `pin_validation` / `remove_session_owner` / `stop_all_workorder_from_employee` | `hr.employee` | `employee_hooks.js` (varios) | Login/PIN y session owner. |

## 11. Gotchas

- `mrp_record_line/` NO fue eliminada en v19. Solo desaparece `mrp_worksheet` dentro de ella (y sus `.scss`).
- La acción server `action_open_employee_list` sigue existiendo en `odoo_enterprise/19/mrp_workorder/views/mrp_workorder_views.xml:441`, `action_open_employee_list` aunque el JS que la usaba (`mrp_employee_list/`) se eliminó. No asumas que la acción desapareció.
- El árbol de `activeFields` se construye a MANO en `_makeModelParams` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display.js::_makeModelParams`). Un campo relacional anidado nuevo exige extender ESE método además de `fieldsStructure` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_action.js::fieldsStructure`).
- `fieldsStructure` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_action.js::fieldsStructure`) es un getter con listas hardcodeadas por modelo. Cualquier campo a leer en cliente debe estar ahí.
- `reload()` en el subenv (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display.js::setup`) sube por `_parentRecord` hasta `mrp.production`. Si el record no tiene esa jerarquía, pasa `false` para recargar la raíz.
- Cadena `StockMove → QualityCheck → MrpWorkorder`: `StockMove` hereda getters de calidad (`super.label`, `super.isComplete`, `super.hasInstruction`) condicionados a `this.check`. Override sin `super` rompe el modo "QC renderizado como move".
- `subRecordProps` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_record.js::subRecordProps`) SUSTITUYE el `quality.check` por su `stock.move` para `register_consumed_materials`/`register_byproducts`. El QC original llega como prop `check`, no como `record`.
- Workcenters persistidos en `localStorage` con clave por db+user (`mrp_workorder.db_<db>.user_<uid>` y `.activeWC`). En tests/multi-tab el estado inicial depende de ese storage.
- Dos variantes de contexto coexisten: calidad usa `from_shopfloor` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/quality_check.js:151`, `QualityCheck.result`), el menú usa `from_shop_floor` (`odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_menu_dialog.js:41`, `MrpMenuDialog.callAction`). Confundirlas rompe la activación del modo Shop Floor en server.
- `MrpSelectQuantDialog` y `StockMove.addMoveLine` marcados `// TODO remove in master` (el marcador está en `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/stock_move.js:7` y `:121`, no en el fichero del diálogo): no construir features encima.
- El bundle carga TODO `static/src/**/*.{js,xml,scss}` por glob (`stock_barcode/__manifest__.py:43-45`). Un `.js` con error de sintaxis tumba toda la app Shop Floor.
- Se retira `/** @odoo-module **/` de las cabeceras JS (import maps nativos; verificado: ningún fichero de `mrp_display/` lo lleva)."


## Correcciones aplicadas en verificación (v19)

- REFUTADO (changedVsV18 + §7 + extension point empleados): la acción server 'mrp_workorder.action_open_employee_list' NO fue eliminada en v19. El act_window sigue definido en odoo_enterprise/19/mrp_workorder/views/mrp_workorder_views.xml:441. Lo que se elimina es la CARPETA JS mrp_employee_list/ y su USO desde el shop floor (ya no se hace doAction sobre esa acción). El borrador afirmaba que la acción server desaparecía: falso.
- CORREGIDO (§5 tabla): mrp_select_quant_dialog.js NO lleva en su interior el comentario '// TODO remove in master'. El marcador TODO está en stock_move.js: en el import (línea 7) y sobre el método addMoveLine (línea 121). El fichero del diálogo en sí no está anotado. La deuda técnica es real pero vive en stock_move.js, no en el dialog.
- PRECISADO (§1 punto 3 y rpcCalls): la llamada orm.call('mrp.workorder','get_shopfloor_limit') está en odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_action.js::setup (la asignación context['limit'] arranca en :169). El rpcCalls ya decía :170 (correcto); el texto de §1 decía :169.
- PRECISADO (rpcCalls do_finish/button_mark_done): en validate() el methodName se asigna en `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_record.js::validate` (string literal `"do_finish"`) y `:345` (string literal `"button_mark_done"`) — son nombres de método de servidor (`mrp.workorder`/`mrp.production`), NO símbolos JS de este fichero; el orm.call real que los ejecuta es único y está en la línea 351. El borrador anclaba la RPC en 334/345 (líneas de asignación), no en el call site.
- PRECISADO (rpcCalls menú): action_add_byproduct se dispara vía callAction dentro de callAddByProductAction; la definición del método está en odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_menu_dialog.js::callAddByProductAction y la llamada callAction('action_add_byproduct') en :70. action_add_component en :62 (call en :62). action_add_step en :202, action_propose_change en :206. Todo confirmado.
- CONFIRMADO todo lo demás contra fuente v19 y baseline v18: jerarquía StockMove→QualityCheck→MrpWorkorder; eliminación real de dialog/mrp_quality_check_confirmation_dialog.* (presente en v18, ausente en v19); eliminación de mrp_record_line/mrp_worksheet.js/.xml (presente en v18); MrpRegisterProductionDialog cambió de MrpQualityCheckConfirmationDialog (v18) a ConfirmationDialog (v19); set_employees_connected NUEVO en v19 (hr_employee.py:135, ausente en v18); MGMSort usa emp.workcenters.length (v19) vs emp.workorder.length (v18); desaparición de isLongPressable/onAnimationEnd/toggleQuantityDone (v18) en stock_move.js v19; retirada de /** @odoo-module **/; ambas variantes de contexto from_shopfloor (odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_record_line/quality_check.js:151) y from_shop_floor (odoo_enterprise/19/mrp_workorder/static/src/mrp_display/dialog/mrp_menu_dialog.js:41) coexisten; action_pass en models/stock_move.py:56.
