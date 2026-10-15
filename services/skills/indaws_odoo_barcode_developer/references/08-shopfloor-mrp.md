
# Shop Floor de fabricación (`mrp_display`) — Odoo 18 Enterprise

App OWL autónoma dentro de `mrp_workorder` (MRP II, licencia `OEEL-1`). NO es una vista Odoo estándar: es un **client action** (`ir.actions.client`, tag `mrp_display`) que monta un árbol de componentes OWL propio sobre `RelationalModel` de `@web/model`, sin `Controller`/`Renderer`/`ArchParser` de vista.

Diseñada para tablets en planta: multi-empleado con PIN, escaneo de códigos de barras, selección de puestos de trabajo (workcenters), validación diferida con animación, y consumo/producción de MOs y work orders.

Todas las citas `file:line` de este documento están verificadas contra la fuente en `odoo_enterprise/18/mrp_workorder/`.

---

## 1. Registro y arranque

- **Acción cliente**: `odoo_enterprise/18/mrp_workorder/views/mrp_production_views.xml:14`, `action_mrp_display` → `record id="action_mrp_display" model="ir.actions.client"`; el `field name="tag">mrp_display` está en `:17`.
- **Menú**: `odoo_enterprise/18/mrp_workorder/views/mrp_workorder_views_menus.xml:11`, `menu_shop_floor` (`action="action_mrp_display"`). App de primer nivel (Shop Floor). Además `stock_barcode/views/stock_picking_type_views.xml:7`, `stock_picking_type_kanban` lanza la misma acción desde el kanban de tipos de operación.
- **Entry point JS**: `odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_action.js:201`, `mrp_display` → `registry.category("actions").add("mrp_display", MrpDisplayAction)`.
- **Bundle** (`stock_barcode/__manifest__.py:40-48`): en `web.assets_backend` hay TRES globs separados — `static/src/**/*.scss`, `static/src/**/*.js`, `static/src/**/*.xml` — más `('remove', 'mrp_workorder/static/src/mrp_workorder_gantt_*')`; los gantt se re-añaden en `web.assets_backend_lazy`. NO hay bundle dedicado a shop_floor. Consecuencia: cualquier `.js/.xml/.scss` bajo `static/src/` del módulo (salvo gantt) entra al backend global.
- **PWA**: existe `controllers/webmanifest.py` y `this.pwaService = useService("pwa")` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display.js::setup`); la app es instalable como PWA.

### Cadena de componentes
```
MrpDisplayAction (mrp_display_action.js)   ← client action, carga fields + search view
  └─ WithSearch                            ← inyecta env.searchModel (MrpDisplaySearchModel)
       └─ MrpDisplay (mrp_display.js)      ← controlador raíz, dueño del RelationalModel
            ├─ Layout + MrpDisplaySearchBar + ControlPanelButtons
            ├─ MrpDisplayEmployeesPanel (panel lateral empleados)
            └─ MrpDisplayRecord  (uno por MO o por WO "relevante")
                 ├─ StockMove         (línea de consumo de componente)
                 ├─ MrpWorkorder      (línea de WO; extiende StockMove)
                 ├─ QualityCheck      (extiende MrpWorkorder)
                 └─ MrpWorksheet
```

---

## 2. `MrpDisplayAction` — carga de datos (`mrp_display_action.js`)

Componente ligero. En `onWillStart` (`:149`):
1. Para cada modelo de `fieldsStructure` (`:20`) llama `fieldService.loadFields(resModel, {fieldNames})` (`:151`) y fusiona con `defaultActiveField` (`:11`). Modelos precargados con listas explícitas de campos: **`mrp.production`, `mrp.workorder`, `stock.move`, `quality.check`** (`:22-122`). Añadir un campo al Shop Floor requiere añadirlo aquí.
2. `fieldsManuallyFetched` (`:125`): campos declarados a mano en vez de vía loadFields (`operation_note: html` en `mrp.workorder`) — patrón para campos pesados/html.
3. Carga la search view de `mrp.production` (`viewService.loadViews`, `:167`) y arma `withSearchProps` con `SearchModel: MrpDisplaySearchModel` (`:193`) y un `orderBy` fijo por prioridad/estado/fecha/id/name (`:186-192`).
4. **Dominio base** (`:140-145`): `state in [confirmed, progress, to_close]` **AND** (`bom_id = False` **OR** `bom_id.type in [normal, phantom]`). Si viene de un `stock.picking.type` (context `active_model/active_id`), añade `picking_type_id = active_id` (`:146-148`).

`static props = { "*": true }` (`:16`) — acepta cualquier prop de la acción cliente.

---

## 3. `MrpDisplay` — controlador raíz (`mrp_display.js`)

### 3.1 Modelo de datos
- `useState(useModel(RelationalModel, params))` (`:87`). `_makeModelParams()` (`:482`) construye a mano la config de **campos relacionados anidados**: `mrp.production` es la raíz, con related `workorder_ids` → (`move_raw_ids`, `check_ids`), y `move_raw_ids/move_byproduct_ids/move_finished_ids` + `check_ids` a nivel MO. Un único grafo cargado en una raíz; NO hay un modelo por WO.
- `get productions()` = `model.root.records` (`:258`). `get workorders()` = flatMap de `workorder_ids.records` de todas las MO (`:266`).
- **Auto-refresh**: `setInterval(() => this.env.reload(), 600000)` (10 min), armado en `onWillStart` (`:162`) y re-armado dentro de `env.reload()` (`:104`). Se limpia en `onWillDestroy` (`:172`) y al inicio de cada `env.reload()` sin args (`:99`).

### 3.2 `env` compartido (useSubEnv)
- `env.localStorageName` (`:69`, useSubEnv aparte): `mrp_workorder.db_{db}.user_{userId}`.
- `env.model` — el RelationalModel; `env.reload(record?)` (`:91`); `env.loadWorkcenters()` (`:110`) — useSubEnv en `:89`.
- `env.reload(record)`: sube al padre `mrp.production` (`while record.resModel !== "mrp.production"`) y hace `record.load()` + `model.notify()` (recarga parcial, `:92-97`); sin argumento recarga la raíz completa con offset/limit y reinicia el intervalo (`:98-107`). **Mecanismo central de refresco tras cualquier operación.**
- `env.loadWorkcenters()`: searchRead de `mrp.workcenter` + inyecta pseudo-workcenters `{id:0 "All MO"}` y `{id:-1 "My WO"}` (`:122-126`).
- `env.searchModel` — el `MrpDisplaySearchModel`.

### 3.3 Estado (`useState`, `:74`)
`activeResModel` (mrp.production | mrp.workorder), `activeWorkcenter` (id, o 0 = All MO, o -1 = My WO), `workcenters` (los fichados en esta estación, en localStorage), `showEmployeesPanel`, `canLoadSamples`, `offset`, `limit` (default `action.context.limit` o 40, `:81`).

### 3.4 Selección de registros relevantes — `defineRelevantRecords()` (`:350`)
Corre en `onWillRender` (`:167`):
- `activeResModel === mrp.production` → todas las MO.
- si no → `filteredWorkorders` filtradas por: `activeWorkcenter === -1` → **My WO** (`myWorkordersFilter`), o por workcenter (`workcenterFilter`).
- **Cache de orden** (`recordCacheIds`, `:83`): mantiene orden estable entre renders. WOs nuevas se ordenan por estado (progress<ready<pending<waiting<finished) y `date_start` (`:378-392`). `invalidateRecordIdsCache()` (`:181`) fuerza reordenar.
- `filteredWorkorders` (`:271`) aplica los filtros de estado del `MrpDisplaySearchModel` (`workorderFilters`).

### 3.5 Barcode (`_onBarcodeScanned`, `:203`)
`useService("barcode")` + `useBus` (`:132`). Orden de resolución:
1. Prefijos `OBT`/`OCD` → ignora (lotes internos, `:204`).
2. `barcode === MO.name` → filtra por esa MO (`_onProductionBarcodeScanned`, `:225`; añade autocompletion al searchModel).
3. `barcode === wo.barcode` → abre la WO (`_onWorkorderBarcodeScanned` → `component.onClickHeader()` + reload, `:237`).
4. `mrp.workcenter.get_employee_barcode(barcode)` (`:217`) → login de empleado (`setSessionOwner`).

`barcodeTargetRecord` (`:242`) resuelve a qué tarjeta dirigir el scan según el admin conectado y sus WO en curso.

### 3.6 Validación diferida (validation stack)
- `validationStack = {"mrp.production": [], "mrp.workorder": []}` (`:57`).
- Al pulsar Validar, `MrpDisplayRecord.validate()` mete el record en el stack con callback y activa animación fadeout; la validación REAL (`realValidation`) se dispara al terminar la animación (`onAnimationEnd`) o al procesar el stack.
- `processValidationStack()` (`:302`): ejecuta callbacks de WO primero, luego agrupa MOs y llama **`mrp.production.button_mark_done`** en lote (`:325`), gestionando serial/backorder vía context (`skip_redirection`, `skip_backorder`, `mo_ids_to_backorder`). Se ejecuta en `onWillDestroy` (`:173`), al cambiar de workcenter (`selectWorkcenter`, `:422`) y en refresh (`onClickRefresh`, `:525`). **Salir de la app valida lo pendiente.**

### 3.7 Empleados
`useConnectedEmployee("mrp_display", ...)` (`:130`, hook en `hooks/employee_hooks.js`). `adminWorkorderIds` (`:405`) = WO del empleado admin (session owner) conectado.

---

## 4. `MrpDisplayRecord` — tarjeta de MO o WO (`mrp_display_record.js`)

Componente polimórfico: la misma clase renderiza MO o WO según `props.record.resModel`. **Se auto-registra en el record**: `this.props.record.component = this` (`:60`) — así `MrpDisplay` invoca métodos del componente desde el record (p.ej. `workorder.component.onClickHeader()` en barcode, `:238`).

Getters que definen las sub-líneas:
- `checks` (`:181`): ordena los `quality.check` siguiendo la cadena `previous_check_id`/`next_check_id` (lista enlazada, NO por secuencia).
- `moves` (`:200-222`): componentes de consumo manual (`manual_consumption && !scrapped`); si el record es WO también concatena los raw de la MO sin operación asignada. **Excluye los move cuyo `product_id` figura como `component_id` en cualquier check** (no solo checks "register").
- `byProducts` (`:167`), `workorders` (`:225`: WO no pending/waiting).

Acciones principales (todas terminan en `env.reload`):
- `onClickHeader` (`:447`): WO → `startWorking`; MO → `action_start`.
- `startWorking` (`:623`): `button_start` con context `{mrp_display: true}`, ficha al empleado y abre el primer QC pendiente.
- `registerProduction`/`quickRegisterProduction` (`:86`/`:108`): fija `qty_producing`, llama `set_qty_producing`; abre `MrpRegisterProductionDialog`.
- `generateSerialNumber` (`:120`): `action_generate_serial`.
- `validate` (`:486`) / `realValidation` (`:530`): WO → `pre_record_production` + `do_finish`; MO → `pre_button_mark_done`/`action_mass_produce` + `button_mark_done`. Diferencia serial vs mass_produce vía `trackingMode` (`:425`).
- `onClickCloseProduction` (`:669`): fast-forward — valida WO y MO saltándose la animación, y elimina la MO de `model.root.records` a mano (`:699`) para evitar recarga completa.
- `displayInstruction` (`:335`) / `qualityCheckDone` (`:389`): abre `MrpQualityCheckConfirmationDialog` y navega la cadena de QC (next/previous).
- `onClickOpenMenu` (`:458`): abre `MrpMenuDialog`.

`disabled`/`active` (`:405-423`): control de acceso por empleado — si el grupo `mrp.group_mrp_routings` está activo y no hay session owner, o el empleado no está en `allowed_employees`, la tarjeta se deshabilita.

---

## 5. Componentes de línea (`mrp_record_line/`) — jerarquía de herencia
```
StockMove (stock_move.js)              ← línea de consumo componente
  └─ MrpWorkorder (mrp_workorder.js)   ← línea de WO (extends StockMove)
       └─ QualityCheck (quality_check.js) ← línea de QC (extends MrpWorkorder)
```
- `StockMove.clicked()` (`:79`): abre `action_show_details` del move. `toggleQuantityDone` (`:93`): marca `picked` con `save({reload:false})` — actualización optimista sin recarga servidor.
- `MrpWorkorder` sobreescribe `clicked()` a vacío (`:93`) y añade `longPress()` → `button_finish` (`:87`). `checksInfo` (`:51`) muestra "hechos/total".
- `QualityCheck.pass()` (`:33`) despacha por `test_type`: `instructions`/`passfail` → `_pass`; `register_production`; `print_label`; `picture` (no-op); **todo lo demás (incl. `measure`) → `this.clicked()`** que abre el diálogo. El getter `icon` (`:86`) sí mapea `measure`, `picture`, `register_consumed_materials`/`register_byproduct`, etc. **Para un nuevo test_type de shop floor hay que tocar `pass()`/`icon` aquí, `MrpQualityCheckConfirmationDialog.validate()` y el backend, que puede estar repartido en varios módulos `quality_*` (ver §9).**

---

## 6. Diálogos (`dialog/`)

- `MrpQualityCheckConfirmationDialog` (`mrp_quality_check_confirmation_dialog.js`): **extiende `ConfirmationDialog` de web** (`:14`). Corazón del procesamiento de QC. `validate()` (`:67`) despacha por test_type a métodos server (`action_print`, `do_measure`, `action_worksheet_check`, `action_next`). `doActionAndClose` (`:108`) es el helper genérico (guarda record, llama ORM, gestiona action devuelto, recarga). Escucha barcode (`_onBarcodeScanned`, `:137`) para asignar lotes.
- `MrpRegisterProductionDialog` (`mrp_register_production_dialog.js`): **extiende el anterior** (`:5`); sobreescribe `doActionAndClose`(:22, usa `set_qty_producing` en :27)/`qtyDoneInfo`/`lotInfo`/`actionGenerateSerial` para el registro de producción sobre `mrp.production`.
- `MrpMenuDialog` (`mrp_menu_dialog.js`, `extends Component`): menú contextual (mover WC, abrir MO, bloquear/desbloquear WC, proponer cambios de paso vía `action_add_step`/`action_propose_change`, añadir componente vía `action_add_component`). SOLO lanza `MrpWorkcenterDialog` y `MrpQualityCheckSelectDialog`. (`MrpLogNoteDialog` lo abre `MrpDisplayRecord.editLogNote`; `MrpWorksheetDialog` es componente aparte.)

---

## 7. Search model (`search_model.js`)

`MrpDisplaySearchModel extends SearchModel`. Añade `state.workorderFilters` (ready/progress/waiting/pending/done, `:9-37`) — NO son filtros de dominio sino filtros client-side aplicados en `MrpDisplay.filteredWorkorders`. Sobreescribe `_getFacets`, `_getIrFilterDescription`, `toggleSearchItem`, `deleteFavorite` para persistir en favoritos (context `wo_active_filters`). `setWorkcenterFilter` (`:105`) reescribe el dominio del filtro `shop_floor_this_station` según los workcenters fichados (con guard si el filtro no existe, `:109`).

---

## 8. Empleados y PIN (`hooks/employee_hooks.js`)

`useConnectedEmployee` (`:10`) devuelve un objeto reactivo `employees {connected, all, admin}` (`:15-19`) + funciones. Backend `hr.employee`: `get_all_employees` (`odoo_enterprise/18/mrp_workorder/models/hr_employee.py:155`, `HrEmployee.get_all_employees`), `login`(:28)/`logout`(:35)/`pin_validation`(:21), `remove_session_owner`(:51), `stop_all_workorder_from_employee`(:116). Concepto **admin/session owner**: el empleado activo cuyas acciones se registran. Multi-empleado real: varios fichados, uno es el owner. PIN vía `PinPopup` (`@mrp_workorder/components/pin_popup`).

---

## 9. Backend Python que respalda la app (métodos RPC)

**En `mrp_workorder/models/`:**
- `mrp.workorder` (`mrp_workorder.py`): `button_start` (`:266`), `button_finish` (`:299`), `action_propose_change` (`:314`), `action_add_step` (`:329`), `pre_record_production` (`:445`), `record_production` (`:453`), `do_finish` (`:561`), `start_employee` (`:788`), `stop_employee` (`:798`).
- `mrp.production` (`mrp_production.py`): `pre_button_mark_done` (`:98`), `can_load_samples` (`:119`), `action_load_samples` (`:122`).
- `mrp.workcenter` (`mrp_workcenter.py`): `get_employee_barcode` (`:36`).
- `quality.check` (`quality.py`): `action_print` (`:312`), `action_next` (`:357`), `action_continue` (`:361`), `action_generate_serial_number_and_pass` (`:455`).
- `hr.employee` (`hr_employee.py`): `pin_validation` (`:21`), `login` (`:28`), `logout` (`:35`), `remove_session_owner` (`:51`), `stop_all_workorder_from_employee` (`:116`), `get_all_employees` (`:155`).

**En módulos base/hermanos (NO en mrp_workorder):**
- `mrp` base (`mrp/models/mrp_production.py`): `set_qty_producing` (`:1443`), `action_generate_serial` (`:1519`), `button_mark_done` (`:2117`), `action_mass_produce` (`:2409`), `action_start` (`:2979`).
- `quality_mrp_workorder/models/quality.py`: `action_pass_and_next` (`:82`), `action_fail_and_next` (`:87`), `do_measure` (`:44`, override).
- `quality_control` / `quality_control_worksheet`: `do_measure` base, `action_open_quality_check_wizard`, `action_worksheet_check` (`quality_control_worksheet:88`), `action_quality_worksheet` (`quality_control_worksheet:70`).
- `quality_mrp_workorder_worksheet/models/quality.py`: `action_worksheet_check` (`:11`).

> Nota: la cita del borrador "pre_button_mark_done en base mrp:2239" no se ha verificado; en mrp_workorder sí está en `:98`.

---

## 10. Diferencias arquitectónicas con `stock_barcode`

| | `mrp_display` (Shop Floor) | `stock_barcode` |
|---|---|---|
| Tipo | `ir.actions.client` tag `mrp_display` | `ir.actions.client` tag `stock_barcode_client_action` |
| Modelo de datos | `RelationalModel` estándar de `@web/model` con related fields anidados; records reactivos `.update()/.save()/.load()` | Modelo/cache **propio** (`barcode_model.js` + line grouping) sobre RPC, NO usa RelationalModel |
| Foco | Multi-MO/WO simultáneas tipo kanban de tarjetas; validación diferida en lote | Un picking a la vez, flujo lineal de escaneo línea a línea |
| Barcode | Servicio `barcode` global + resolución JS contra records ya cargados | Parser dedicado (`barcode.nomenclature`), núcleo del flujo |
| Estado UI | `useState` + `RelationalModel` reactivo; orden estable vía `recordCacheIds` | máquina de estados propia |
| Extensión | Herencia OWL + `patch()` prototype + añadir campos en `fieldsStructure` | `patch()` sobre `barcode_model`/`barcode_handler` + registro de parsers |

**mrp_display se apoya en el framework de modelo relacional de Odoo ("ORM en el cliente")**; stock_barcode reimplementa su propia capa de datos.

---

## 11. Gotchas críticos para extender

1. **`fieldsStructure` es la única fuente de campos** (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_action.js::fieldsStructure`). Campo usado en un getter y ausente aquí llega `undefined` sin error → tarjeta rota en silencio. HTML/pesados → `fieldsManuallyFetched` (`:125`).
2. **Validación diferida**: pulsar Validar NO llama al servidor de inmediato; va al `validationStack` y se ejecuta con la animación / al destruir / al cambiar WC / al refrescar. Engánchate a `addToValidationStack`/`processValidationStack`, no dispares `button_mark_done` por tu cuenta.
3. **`recordCacheIds`** (`:83`) cachea el orden de tarjetas; filtrados que no llamen `invalidateRecordIdsCache` producen orden inconsistente/parpadeo.
4. **`record.component = this`** (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_record.js::setup`): acoplamiento record→componente. Frágil si el record se recrea; comprueba existencia antes de `record.component.*`.
5. **`env.reload(record)` (parcial, sube al padre MO) vs `env.reload()` (total)**: elegir mal deja datos stale o lagea la tablet. `qualityCheckDone` depende de recarga parcial + búsqueda manual del next check en `env.model.root`.
6. **Cadena de QC por punteros** `previous_check_id`/`next_check_id` (lista enlazada), NO por sequence. Insertar/reordenar requiere mantener punteros (backend `action_add_step`).
7. **Bundle global**: todo `static/src/**` (salvo gantt) va a `web.assets_backend`. Un patch/CSS mal aislado afecta a todo el backend.
8. **Herencia de diálogos**: `MrpRegisterProductionDialog extends MrpQualityCheckConfirmationDialog extends ConfirmationDialog`. `patch(MrpQualityCheckConfirmationDialog.prototype)` afecta también al de registro de producción.
9. **`QualityCheck.pass()` NO cubre todos los test_type**: solo instructions/passfail/register_production/print_label/picture; el resto (incl. measure) va a `clicked()`→diálogo. El dispatch está repartido en JS `pass()` + JS `validate()` del diálogo + **varios módulos backend `quality_*`** (mrp_workorder, quality_mrp_workorder, quality_control(_worksheet)...). Un test_type nuevo puede requerir tocar varias capas y módulos.
10. **localStorage por db+user** (`env.localStorageName mrp_workorder.db_{db}.user_{userId}`, `:69`): workcenters activos y WC activo persisten por usuario/db → estado residual en la misma tablet.
11. **Auto-refresh cada 10 min** (`setInterval 600000`, `:104`/`:162`) reinicia `model.root.load`; operaciones largas pueden colisionar. Se limpia en `onWillDestroy` y al arrancar cada `env.reload()`.
12. **`activeWorkcenter` con ids especiales**: 0 = 'All MO' (muestra `mrp.production`), -1 = 'My WO' (WO del admin conectado), >0 = workcenter real. No son ids reales de `mrp.workcenter`.


## Correcciones aplicadas en verificación

- §1 Bundle: el manifest NO usa un glob con brace-expansion `**/*.{scss,js,xml}`. Son TRES globs separados (uno por extensión) en web.assets_backend (`stock_barcode/__manifest__.py:40-45`), un `('remove', 'mrp_workorder/static/src/mrp_workorder_gantt_*')` (:44) y los gantt re-añadidos en web.assets_backend_lazy (:46-48). El resto de la afirmación (todo static/src va al backend salvo gantt) es correcto.
- §1 El tag `mrp_display` está en `odoo_enterprise/18/mrp_workorder/views/mrp_production_views.xml:17`, `action_mrp_display` (field name=tag), no en :14; en :14 está el `record id=action_mrp_display`. El menú en `odoo_enterprise/18/mrp_workorder/views/mrp_workorder_views_menus.xml:11`, `menu_shop_floor` es correcto. Además `stock_barcode/views/stock_picking_type_views.xml:7`, `stock_picking_type_kanban` lanza la misma acción desde el kanban de tipos de operación.
- §2/§11 Dominio base incompleto: es `state in [confirmed,progress,to_close]` AND (`bom_id = False` OR `bom_id.type in [normal,phantom]`) — el borrador omite la rama OR `bom_id = False` que deja pasar MOs sin lista de materiales (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_action.js::setup-145`).
- §4 Getter `moves` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_record.js::moves-222`): NO excluye 'los que ya tienen un QC de tipo register'. Excluye los move cuyo `product_id` aparece como `component_id` en CUALQUIER quality.check de la WO/MO (independiente del test_type). También concatena los move_raw de la MO sin operación asignada cuando el record es una WO.
- §5/§11/EP3 `QualityCheck.pass()` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_record_line/quality_check.js::pass`): el switch maneja explícitamente `instructions`/`passfail` (→_pass), `register_production`, `print_label`, `picture` (no-op); TODO lo demás —incluido `measure`— cae al default `this.clicked()` que abre el diálogo. `measure` NO es una rama de `pass()`; se procesa en `MrpQualityCheckConfirmationDialog.validate()` (:71 do_measure). El getter `icon` (:86) sí mapea measure y también register_consumed_materials/register_byproduct→barcode.
- §9 Localización de métodos quality.check CORREGIDA: solo `action_print` (:312), `action_next` (:357), `action_continue` (:361) y `action_generate_serial_number_and_pass` (:455) están en `mrp_workorder/models/quality.py`. `action_pass_and_next`/`action_fail_and_next` viven en `odoo_enterprise/18/quality_mrp_workorder/models/quality.py:82,87`, `QualityCheck`; `do_measure` en `quality_control` (base) con override en `quality_mrp_workorder:44`; `action_worksheet_check` en `quality_control_worksheet:88` y `quality_mrp_workorder_worksheet:11`; `action_open_quality_check_wizard` en `quality_control(_worksheet)`; `action_quality_worksheet` en `quality_control_worksheet:70`. El borrador los daba todos en mrp_workorder.
- §9 Métodos base mrp verificados: `action_start` es de mrp base (`odoo18/addons/mrp/models/mrp_production.py:2979`, `MrpProduction.action_start`), no de mrp_workorder. `set_qty_producing` mrp:1443 ✓, `action_mass_produce` mrp:2409 ✓, `button_mark_done` mrp:2117 (el borrador no daba línea), `action_generate_serial` mrp:1519. La cita 'pre_button_mark_done base mrp:2239' NO se pudo verificar y queda como no confirmada; en mrp_workorder pre_button_mark_done sí está en :98.
- §6 `MrpMenuDialog` (mrp_menu_dialog.js) SOLO importa/lanza `MrpWorkcenterDialog` y `MrpQualityCheckSelectDialog`. `MrpLogNoteDialog` lo abre `MrpDisplayRecord.editLogNote` y `MrpWorksheetDialog` es un componente aparte; no son 'menús secundarios' del MrpMenuDialog. Además callAddComponentAction (:50) llama `action_add_component` (no citado en el borrador).
- EP2 Los sub-templates de extensión existen con nombre completo `mrp_workorder.MrpDisplayProductionHeader/WorkorderHeader/ProductionFooter/WorkorderFooter` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display_record.xml:105`, `mrp_workorder.MrpDisplayProductionHeader` (y `:122` WorkorderHeader, `:133` ProductionFooter, `:144` WorkorderFooter)), invocados vía t-call en :10-11 y :98-99 — confirmado como puntos de extensión por t-inherit.
