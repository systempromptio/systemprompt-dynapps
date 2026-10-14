## BarcodeQuantModel — el modelo cliente de ajuste de inventario (Odoo 18 EE) [VERIFICADO CONTRA FUENTE]

Fuente: `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js` (751 LOC). Extiende `BarcodeModel` (`.../models/barcode_model.js`). Es el hermano de `BarcodePickingModel`: mismo padre, dos especializaciones divergentes. Todo lo relativo a `stock.quant` / conteo / aplicar cantidades vive aquí. Todos los anclajes `file:line` de abajo han sido releídos y verificados contra el código.

### 1. Instanciación y punto de entrada
- El modelo NO se registra en ningún registry. Se instancia a mano en `MainComponent._getModel()` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel-289`): `else if (this.resModel === 'stock.quant') return new BarcodeQuantModel(this.resModel, this.resId, services)`. El discriminante es `this.resModel`; para pickings devuelve `BarcodePickingModel` (`:283`); cualquier otro modelo → `throw new Error('No JS model define')` (`:287`).
- El objeto `services` se arma en `odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel`: `{ rpc, orm, notification, action }`. La rama picking le añade `services.dialog = this.dialog` (`:282`); la rama quant (`:284-285`) NO pasa dialog.
- Constructor base `BarcodeModel(resModel, resId, services)` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::constructor-35`): `super()` de EventBus, luego `this.dialogService = useService('dialog')` (`:17`), `this.orm = services.orm`, `this.notificationService = services.notification`, `this.action = services.action`, `resId`, `resModel`, etc. Aunque el quant no reciba `services.dialog`, `this.dialogService` existe igualmente porque el base lo obtiene vía `useService('dialog')` (ejecutado dentro del setup del componente). El quant lo usa en `checkBeforeApply` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::checkBeforeApply`).
- **Constructor propio** (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::constructor-14`): CORRECCIÓN de precisión — la firma LITERAL es `constructor(params)` (un único parámetro, nombre engañoso). Llama `super(...arguments)` PRIMERO (`:9`), reenviando los 3 args reales (`resModel, resId, services`) al base, y después fija el contrato: `this.lineModel = this.resModel` (= 'stock.quant'), `this.validateMessage`, `this.validateMethod = "action_validate"`, y `this.deleteLineMethod = this.validateMethod` (borrar una línea contada a 0 = validarla, vía `deleteLine` base que hace `orm.call(this.lineModel, this.deleteLineMethod, [line.id])`, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::deleteLine`).
- `BarcodeModel extends EventBus` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js:14`, `BarcodeModel`): la UI OWL se comunica por eventos. Cualquier mutación de líneas DEBE terminar en `this.trigger('update')` (re-render) o `'refresh'` (recarga desde servidor) / `'history-back'`; si no, la pantalla no refleja el cambio.
- El kanban (`static/src/kanban/`) es SOLO para `stock.picking`: el js_class `stock_barcode_list_kanban` solo se aplica en `odoo_enterprise/18/stock_barcode/views/stock_picking_views.xml:98`, `stock_picking_view_kanban`. `odoo_enterprise/18/stock_barcode/static/src/kanban/stock_barcode_kanban_controller.js::openRecord` (`openRecord`) abre `stock_barcode_picking_client_action` y `createRecord` (`:23-28`) llama `stock.picking.action_open_new_picking`. NO participa en el flujo de inventario. El ajuste de inventario se abre por `stock.quant.action_client_action()` (`odoo_enterprise/18/stock_barcode/models/stock_quant.py:64-68`, `StockQuant.action_client_action`) → `stock_barcode_inventory_client_action`, que monta el mismo `MainComponent` con `resModel='stock.quant'`.

### 2. setData / carga inicial (`:188-196`)
`setData(data)` guarda tres flags de negocio ANTES de llamar a `super.setData`:
- `this.userId = data.data.user_id` (`:189`) — usuario dueño del conteo.
- `this.showQuantityCount = data.data.show_quantity_count` (`:190`) — grupo `stock_barcode.group_barcode_show_quantity_count` (backend `stock_barcode/models/stock_quant.py:120`, `StockQuant.data`); si off no se muestra la cantidad esperada.
- `this.countEntireLocation = data.data.count_entire_location` (`:191`) — grupo `stock_barcode.group_barcode_count_entire_location` (backend `stock_barcode/models/stock_quant.py:121`, `StockQuant.data`); cambia la semántica del "set" al contar.
Después: `super.setData` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::setData-70`) crea `this.cache = new LazyBarcodeCache(...)`, el parser de nomenclatura, `this.groups`, y llama `_createState()` (`:62`, que invoca `_createLinesState`). Luego el quant lee `companyIds` de `data.data.records['res.company']` (`:193-194`) y `lineFormViewId = data.data.line_view_id` (`:195`), apuntando a `stock_barcode.stock_quant_barcode`, servido en `odoo_enterprise/18/stock_barcode/models/stock_quant.py:93`, `StockQuant._get_stock_barcode_data`.

### 3. Construcción del estado de líneas — `_createLinesState` (`:696-720`)
Diferencia CLAVE con picking: las líneas del quant se construyen SOLO desde la cache de `stock.quant`, no de move.lines.
- Itera `this.cache.dbIdCache['stock.quant']` (`:699`).
- FILTRO DURO (`:701`): `if (quant.user_id !== this.userId || quant.inventory_date > today) continue`. Un usuario solo ve SUS conteos y no fechas futuras. En este punto `quant.user_id` es el id crudo (la hidratación de m2o empieza en `:711`). Se combina con el dominio servidor de `_get_stock_barcode_data` (`odoo_enterprise/18/stock_barcode/models/stock_quant.py:84`, `StockQuant._get_stock_barcode_data`: `search([('user_id','=?',uid),('location_id','in',...),('inventory_date','<=',today)])`).
- Preserva `virtual_id` entre recargas: `quant.dummy_id || previousVirtualId || this._uniqueVirtualId` (`:709-710`). `dummy_id` (campo Char compute/inverse en `stock.quant`, `odoo_enterprise/18/stock_barcode/models/stock_quant.py:7-15`, `StockQuant.dummy_id`) es el puente para reconocer un quant recién creado tras el round-trip al servidor.
- "Hidrata" los m2o a objetos de cache (`product_id`, `product_uom_id`, `location_id`, `lot_id`, `package_id`, `owner_id`) en `:711-716`.

### 4. Contar / actualizar cantidad
Campo de cantidad = `inventory_quantity` (no `qty_done`). Estado "contado" = booleano `inventory_quantity_set`.
- `getQtyDone(line)` (`:202-204`): devuelve `inventory_quantity` SOLO si `inventory_quantity_set`, si no 0.
- `getQtyDemand(line)` (`:206-208`): `quantity` (stock teórico) solo si `showQuantityCount`, si no 0.
- `updateLineQty(virtualId, qty=1)` (`:271-277`): entrada del usuario (botones UI). Corre bajo `this.actionMutex.exec` para serializar. Llama `updateLine(line, {inventory_quantity: qty})` (SET absoluto vía `_updateLineQty`) y `trigger('update')`.
- `_updateLineQty(line, args)` (`:638-668`): corazón del incremento al escanear.
  - `args.quantity` → set del stock teórico `line.quantity` (`:640`).
  - `args.inventory_quantity` → INCREMENTA `line.inventory_quantity += args.inventory_quantity` (`:659`). Maneja conversión de UoM (`:643-657`): categoría distinta → notifica y aborta (`:653`); misma categoría distinta UoM → convierte por factor (`:656`).
  - Auto-set (`:660-663`): si `line.inventory_quantity > 0` pone `args.inventory_quantity_set = true`; luego `line.inventory_quantity_set = this.countEntireLocation ? args.inventory_quantity_set : true`. Con `countEntireLocation` respeta el flag; sin él SIEMPRE fuerza `true`. Bifurcación sutil.
  - Serial (`:664-666`): si producto serial con `lot_name`/`lot_id`, clampa `inventory_quantity` a [0,1] con `Math.max(0, Math.min(1, ...))`.
- `toggleAsCounted(line)` (`:264-269`): pone `inventory_quantity=0`, togglea `inventory_quantity_set`, marca dirty y trigger update. Es cómo se marca "presente = 0" (missing).
- `_lineIsNotComplete(line)` (`:463-465`): completa ⇔ `inventory_quantity !== 0`. Dirige los mensajes de `barcodeInfo`.

### 5. Aplicar el ajuste (validate) — flujo `apply` → `checkBeforeApply` → `_apply`
Mayor divergencia con picking. `_validate()` (`:16-18`) delega en `apply()`.
- `apply()` (`:24-28`): si `checkBeforeApply()` → `_apply()`.
- `checkBeforeApply()` (`:35-68`):
  - `applyOn === 0` (getter `:99-101`, = nº de `pageLines` con `inventory_quantity_set`) → notifica "There is nothing to apply", false.
  - Detecta SN NO contados que conviven con SN contados del mismo producto vía `this.groupedLines` (`:42-52`). Si los hay, abre `ApplyQuantDialog` (`components/apply_quant_dialog.js`) con dos callbacks: `onApply = this._apply.bind(this)`; `onApplyAll` = marca los SN faltantes como contados (`toggleAsCounted`, a 0) y luego `_apply()` (`:57-63`). Devuelve false.
- `_apply()` (`:74-90`):
  1. `await this.save()` (persiste primero, ver §6).
  2. Filtra `this.pageLines` por `inventory_quantity_set` (`:76`) y toma sus `id` (`:77`).
  3. `await this.orm.call("stock.quant", "action_validate", [quantIds])` (`:78`, RPC directa; NO usa `this.validateMethod`/`lineModel` pese a estar definidos en el constructor — esos solo se usan para `deleteLine`).
  4. Si la acción devuelta trae `res_model` → `this.action.doAction(action, {onClose: notifyAndGoAhead})` (`:86-87`). Si `res.special` (wizard descartado) → solo `trigger('refresh')` (`:80`). Si no, notifica éxito y `trigger('history-back')` (`:83-84`).
- Backend `action_validate` (`odoo_enterprise/18/stock_barcode/models/stock_quant.py:56-62`, `StockQuant.action_validate`): `with_context(inventory_mode=True)`, `filtered(lambda q: q.inventory_quantity_set)`, `_compute_inventory_diff_quantity()`, y `action_apply_inventory()` (core stock). Puede devolver una acción que el JS abre.
- Comando de barcode `OBTAPPLY` mapeado a `apply` (`:283-287`).

### 6. Guardado — `save` → `_getSaveCommand` → ruta → `barcode_write`
- `save()` (base, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::save-491`): pide `_getSaveCommand()`, si hay `route` hace `rpc(route, params)` y `refreshCache(res.records)`; vacía `linesToSave`.
- `_getSaveCommand()` (quant, `:438-452`): si hay comandos, ruta `/stock_barcode/save_barcode_data` con `params = {model:'stock.quant', res_id:false, write_field:false, write_vals: commands}`. Si no hay cambios, `{}`.
- Comandos por el base `_getSaveLineCommand()` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_getSaveLineCommand-937`): por cada `virtualId` en `linesToSave`, con `id` → `[1, id, changedValues]` (diff campo a campo contra `initialState`, comparando `.id` para objetos, `:918-928`); sin `id` → `[0, 0, this._createCommandVals(line)]` (`:933`). Campos comparados de `_getFieldToWrite()`.
- `_getFieldToWrite()` (quant, `:424-436`): `inventory_date, inventory_quantity, inventory_quantity_set, user_id, location_id, lot_name, lot_id, package_id, owner_id`.
- `_createCommandVals(line)` (quant, `:307-325`): vals de creación (incluye `dummy_id: line.virtual_id`, `user_id: this.userId`), aplanando m2o a id vía `_fieldToValue`.
- Controller (`odoo_enterprise/18/stock_barcode/controllers/stock_barcode.py:81-87`, `StockBarcodeController.save_barcode_data`): si `not res_id` → `request.env['stock.quant'].barcode_write(write_vals)` (`:84`).
- `barcode_write(vals)` (`odoo_enterprise/18/stock_barcode/models/stock_quant.py:17-54`, `StockQuant.barcode_write`): corre con `inventory_mode=True` (`:24`). Crea `stock.lot` on-the-fly si hay `lot_name` sin `lot_id` (`:29-34`). Aplica `[1,id,vals]` = write, `[0,0,vals]` = create; tras crear escribe `dummy_id` (`:46-47`), `inventory_date` (`:48`) y `user_id` si falta (`:49-52`) por separado (parche para que la línea no "desaparezca"). Devuelve `_get_stock_barcode_data()`. Se usa `barcode_write` en vez de override de `write` porque los pickings también escriben quants (comentario `:19-23`).

### 7. Creación de líneas al escanear — `_createNewLine` (`:327-387`)
Muy distinto del base porque un producto puede tener VARIOS quants (ubicación/lote/paquete/owner):
- Rechaza no almacenables: `if (! product.is_storable)` notifica y devuelve false (`:331-339`).
- Busca quants existentes vía `this.cache.getQuants(product, location_id[, quantParams])` (`:343-350`).
- Si hay quants, crea UNA línea por quant (`:359-382`) llamando a `super._createNewLine` por cada uno, y mete cada quant en `initialState.lines` con valores originales (`:374-381`) para que el diff de save no detecte cambios falsos. Selecciona la primera creada.
- Si no hay quants, crea línea vacía (`super._createNewLine`, `:384`).
- `_getNewLineDefaultValues` (`:407-422`): fija `inventory_date=hoy`, `inventory_quantity=0`, `quantity`, `user_id`. Auto-marca `inventory_quantity_set=true` SOLO si `fieldsParams.quantity === undefined` (no es quant existente) o ya trae `inventory_quantity` (`:418-420`) — salvaguarda anti-aplicado accidental en multi-serial.
- `_getNewLineDefaultContext` (`:295-305`): contexto del form embebido (`default_company_id`, `default_location_id`, `default_inventory_quantity:1`, `default_user_id`, `inventory_mode:true`, `hide_qty_to_count: !showQuantityCount`).
- `_convertDataToFieldsParams` (`:389-405`): traduce barcodeData a params (`inventory_quantity`, `lot_id`, `lot_name`, `owner_id`, `package_id`, `product_id`, `product_uom_id`, `packaging`).

### 8. Paquetes y ubicación entera
- `_processPackage(barcodeData)` (`:467-591`): cambia tipo de paquete (`orm.write('stock.quant.package', ...)` `:480`), crea paquete (`action_create_from_barcode` `:499` y `:511`), carga quants del paquete (`stock.quant.get_stock_barcode_data_records` `:527`), crea/actualiza una línea por quant. Marca `barcodeData.stopped=true` y `lastScanned.packageId` (`:587-589`).
- `_processLocation` (`:593-598`): llama super y, si `countEntireLocation`, `loadQuantsForLocation`.
- `loadQuantsForLocation(barcodeData)` (`:600-636`): RPC `stock.location.get_counted_quant_data_records` (`odoo_enterprise/18/stock_barcode/models/stock_location.py:20`, `Location.get_counted_quant_data_records`), crea líneas para quants no presentes con `inventory_quantity_set=false` (`:629`) (presentes pero sin contar todavía).

### 9. Presentación (getters usados por componentes line/main)
- `displayApplyButton`=true (`:198-200`); `displayByUnitButton`=true (`:178-180`); `displaySetButton(line)` (`:182-186`).
- `highlightValidateButton` (`:219-221`): resalta cuando todas las líneas de la página están "set".
- `lineIsFaulty(line)` (`:231-236`): solo si `showQuantityCount` y `inventory_quantity !== quantity`.
- `lineCanBeDeleted` (`:227-229`): set y `getQtyDone===0`. `IsNotSet` (`:223-225`). `zeroQtyClass` (`:744-746`): "text-danger" si set a 0.
- `lineIsTracked` (`:238-245`): trata seriales sin SN y qty<=1 como no-tracked (agrupación).
- `barcodeInfo` (`:103-176`): mensajes de guía según estado de línea y multi-locations.
- `printButtons` (`:247-253`): "Print Inventory" → `stock.action_report_inventory`. `_getPrintOptions` (`:726-734`) pasa `active_ids` = líneas set.
- `isValidForBarcodeLookup` (`:92-97`): true solo si `resModel==='stock.quant'`.

### 10. Diferencias clave vs. BarcodePickingModel
- Campo cantidad: quant `inventory_quantity` (SET absoluto por usuario, incrementa al escanear) vs picking `qty_done`.
- Estado hecho: quant booleano `inventory_quantity_set` vs picking comparación reservado/hecho.
- lineModel: quant `stock.quant` (=resModel) vs picking `stock.move.line`.
- Validar: quant `apply` → `stock.quant.action_validate` (RPC directa) + `ApplyQuantDialog` para SN faltantes; picking `action_validate` sobre picking + backorders.
- Guardar: quant ruta → `barcode_write` (comandos [0/1]); picking write sobre move.lines.
- Estado líneas: quant `_createLinesState` desde cache de quants filtrado por user_id/inventory_date; picking desde move.lines.
- Multi-quant por producto: quant sí (una línea por quant existente); picking una por move.line.
- Entrada: quant `stock_barcode_inventory_client_action` (no kanban); picking kanban + client action de picking.

### 11. Métodos del padre relevantes (para override informado)
`updateLine` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::updateLine-567`) hidrata m2o y llama `_updateLineQty` + `_markLineAsDirty` (`:566`). `_markLineAsDirty` (`:992-997`) empuja el `virtual_id` a `scannedLinesVirtualId` y a `linesToSave`. `_createNewLine` base (`:795-832`) valida UoM (`:796-808`), arma la línea con `_getNewLineDefaultValues`, gestiona sortIndex y hace push a `currentState.lines` (`:830`). `_getSaveLineCommand` (`:909-937`) compara contra `initialState` (crash si el campo objeto no está en initialState → `initialValue.id` de undefined, `:923`). `getActionRefresh` base (`:94-98`) usa la ruta `/stock_barcode/get_barcode_data`; el quant la sobreescribe (`:210-217`) para pasar la LISTA de ids de líneas como `res_id`.

### 12. Cómo enganchar (mecanismos reales)
- Patrón estándar: `patch(BarcodeQuantModel.prototype, {...})` importando el default export desde `@stock_barcode/models/barcode_quant_model` en un asset JS. Requiere actualizar el módulo (assets).
- Subclase completa: `class X extends BarcodeQuantModel` + patch de `MainComponent._getModel` (`odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel-285`) para instanciarla. OJO: el constructor del quant es `constructor(params)` con `super(...arguments)`; si tu subclase declara constructor propio, reenvía `...arguments` (o declara `(resModel, resId, services)`). Más frágil ante updates.
- Backend: overrides Python de `stock.quant.action_validate` / `barcode_write` / `get_stock_barcode_data_records` / `_get_fields_stock_barcode` (`models/stock_quant.py`).

### 13. RPC del cliente (verificado)
- `stock.quant.barcode_write` (vía ruta `/stock_barcode/save_barcode_data`): `save` → `_getSaveCommand` (`:438-452`); controller (`stock_barcode/controllers/stock_barcode.py:84`, `StockBarcodeController.save_barcode_data`). Persiste líneas [1,id,vals]/[0,0,vals]; crea stock.lot; escribe dummy_id/inventory_date/user_id aparte.
- `stock.quant.action_validate`: `_apply` (`:78`). Filtra inventory_quantity_set, computa diff, `action_apply_inventory`; puede devolver acción-wizard.
- `stock.location.get_counted_quant_data_records`: `loadQuantsForLocation` (`:601`). Precarga quants de la ubicación con `count_entire_location`.
- `stock.quant.get_stock_barcode_data_records`: `_processPackage` (`:527`, sobre `recPackage.quant_ids`). Carga quants de un paquete.
- `stock.quant.package.action_create_from_barcode`: `_processPackage` (`:499` y `:511`). Crea paquete nuevo.
- `stock.quant.package.write` (package_type_id): `_processPackage` (`:480`). Cambia tipo de paquete.
- Ruta `/stock_barcode/get_barcode_data` → `_get_stock_barcode_data`/`get_stock_barcode_data_records`: `getActionRefresh` (`:210-217`), `res_id` = LISTA de ids de líneas.

### 14. Gotchas (verificados)
1. Campo de cantidad = `inventory_quantity` (SET absoluto), NO `qty_done`. `updateLineQty` hace set; `_updateLineQty` al escanear hace `+=` (`:659`). Confundirlos rompe el conteo.
2. `inventory_quantity_set` es el gatillo de "se aplicará". `_updateLineQty` lo fuerza a true si qty>0 salvo con `countEntireLocation` (`:660-663`). `action_validate` aplica SOLO líneas con `inventory_quantity_set=true` (`stock_barcode/models/stock_quant.py:57`, `StockQuant.action_validate`).
3. `_createLinesState` (`:701`) filtra por `user_id === this.userId` y `inventory_date <= today`: un usuario NO ve conteos de otros ni futuros. Relajarlo en cliente sin ajustar el dominio servidor (`stock_barcode/models/stock_quant.py:84`, `StockQuant.self`) deja quants huérfanos en cache.
4. `_apply` llama `orm.call('stock.quant','action_validate',...)` DIRECTO (`:78`), ignorando `this.validateMethod`/`this.lineModel`. Esos atributos solo se usan para `deleteLine`/`deleteLineMethod` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::deleteLine`). Cambiar `validateMethod` NO cambia el validate del inventario.
5. El guardado NO usa write estándar: ruta `/stock_barcode/save_barcode_data` → `stock.quant.barcode_write`, con `inventory_mode=True` y creación de `stock.lot` on-the-fly (`stock_barcode/models/stock_quant.py:29-34`, `StockQuant.barcode_write`). Override `write` no intercepta esto.
6. `dummy_id` es el puente virtual_id↔quant nuevo tras el round-trip. `barcode_write` lo re-escribe aparte (`stock_barcode/models/stock_quant.py:46-47`, `StockQuant.barcode_write`) y `_createLinesState` lo usa (`:709`). Tocar dummy_id descuadra la reconciliación.
7. Un producto con varios quants genera VARIAS líneas de golpe en `_createNewLine` (`:359-382`), y cada quant existente se inyecta en `initialState` (`:381`) para que el diff de save no vea cambios falsos. Extender `_createNewLine` sin replicar ese push a `initialState` provoca escrituras espurias.
8. El kanban de stock_barcode es exclusivo de `stock.picking` (js_class solo en `odoo_enterprise/18/stock_barcode/views/stock_picking_views.xml:98`, `stock_picking_view_kanban`); NO aplica al inventario. Entrada del quant = `action_client_action` → `stock_barcode_inventory_client_action`.
9. `services.dialog` NO se pasa al `BarcodeQuantModel` en `odoo_enterprise/18/stock_barcode/static/src/components/main.js::_getModel-285` (solo a picking, `:282`), pero `this.dialogService` funciona porque `BarcodeModel` hace `useService('dialog')` (`odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::constructor`). Al recrear services en una subclase, replica el ensamblado `{rpc, orm, notification, action}` sin intentar "arreglar" dialog.
10. Si `action_validate` devuelve una acción con `res_model`, el flujo continúa SOLO en `onClose` (`notifyAndGoAhead`, `:86-89`); si el wizard se descarta (`res.special`) solo hace refresh, no history-back (`:80`). Un wizard backend mal devuelto deja al usuario atascado.
11. Diff de save (`_getSaveLineCommand`, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_getSaveLineCommand`): para campos objeto compara `fieldValue.id !== initialValue.id`; si el campo custom no viaja en la carga inicial (`_get_fields_stock_barcode`), `initialValue` es undefined y revienta. Al añadir campos custom, aparéalos en carga + `_getFieldToWrite` + `_createCommandVals`.


## Correcciones aplicadas en verificación

- Firma del constructor (odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::constructor): el borrador implicaba que el constructor propio recibe `(resModel, resId, services)`. FALSO en la letra: la firma literal es `constructor(params)` — un ÚNICO parámetro con nombre engañoso (`params`). Reenvía los 3 argumentos reales al base con `super(...arguments)` (línea 9), por lo que el COMPORTAMIENTO que describía el borrador (recibe resModel/resId/services) es correcto, pero quien override el constructor no debe fiarse del nombre `params`: debe usar `...arguments` o declarar `(resModel, resId, services)`. El base sí es `constructor(resModel, resId, services)` (odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::constructor).
- Kanban (§1): `action_open_new_picking` NO está en la línea :24. `openRecord` hace doAction de `stock_barcode.stock_barcode_picking_client_action` en odoo_enterprise/18/stock_barcode/static/src/kanban/stock_barcode_kanban_controller.js::openRecord, y `createRecord` llama `stock.picking.action_open_new_picking` en el bloque :23-28 (nombre del método en :26). El resto de la afirmación (kanban exclusivo de picking) SÍ se confirma: el js_class `stock_barcode_list_kanban` solo se aplica en odoo_enterprise/18/stock_barcode/views/stock_picking_views.xml:98 (a stock.picking); no hay vista kanban registrada para stock.quant. El renderer chequea `resModel === 'stock.picking'` de forma defensiva (renderer:17) pero en la práctica solo picking usa esa vista.
- §11/§12: `_parseBarcode` del base está DEFINIDO en odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_parseBarcode (el borrador citaba :1022; la resolución de comandos ocurre dentro de ese método, alrededor de :1022, pero la definición es :1017).
- §11: `getActionRefresh` base ocupa odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::getActionRefresh-98 (el borrador decía 94-99). Nit de una línea, sin impacto.
