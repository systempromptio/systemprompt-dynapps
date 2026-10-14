## stock_barcode v18 EE — Contrato RPC servidor (superficie extensible en Python)

Fuente verificada releyendo `odoo_enterprise/18/stock_barcode/models/*.py` y `odoo_enterprise/18/stock_barcode/controllers/stock_barcode.py`, cruzada con `static/src/**/*.js`. Todas las citas `file:line` han sido re-verificadas contra el código.

### Cómo habla el cliente con el servidor

Dos mecanismos, ambos overridables desde un módulo Python:

1. **Rutas HTTP `type='json'`** en `controllers/stock_barcode.py`, clase `StockBarcodeController(http.Controller)` (`:13`). Se llaman con `rpc("/stock_barcode/...", params)` desde JS.
2. **`orm.call(model, method, args, kwargs)`** contra métodos de modelo. ORM crudo: cualquier método público del modelo es invocable.

El servidor devuelve SIEMPRE los registros vía `record.read(model._get_fields_stock_barcode(), load=False)`. **`load=False` significa que los Many2one llegan como id desnudo (int), no como tupla `[id, name]`.** El cliente asume ese formato; esto condiciona toda extensión de campos.

---

### A. Rutas del controlador (`controllers/stock_barcode.py`)

| Ruta | Método | Recibe | Devuelve | Caller JS |
|---|---|---|---|---|
| `/stock_barcode/scan_from_main_menu` | `main_menu(barcode)` `:15` | `barcode` str | `{action}` \| `{warning}` | `odoo_enterprise/18/stock_barcode/static/src/main_menu/main_menu.js::_onBarcodeScanned` |
| `/stock_barcode/save_barcode_data` | `save_barcode_data(model, res_id, write_field, write_vals)` `:81` | modelo, id, campo, comandos x2many | `record._get_stock_barcode_data()` o `barcode_write(write_vals)` si `res_id` falsy | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::_getSaveCommand`, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::_getSaveCommand` |
| `/stock_barcode/get_barcode_data` | `get_barcode_data(model, res_id)` `:89` | modelo + id (o `False`) | `{data, groups}` | `odoo_enterprise/18/stock_barcode/static/src/components/main.js::onWillStart`, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::getActionRefresh` (getActionRefresh) |
| `/stock_barcode/get_main_menu_data` | `get_main_menu_data()` `:117` | — | `{groups, play_sound, quant_count}` | `odoo_enterprise/18/stock_barcode/static/src/main_menu/main_menu.js::setup` |
| `/stock_barcode/get_quants` | `get_existing_quant_and_related_data(domain)` `:147` | domain | `quants.get_stock_barcode_data_records()` | `odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js:128`, `LazyBarcodeCache.getQuants` |
| `/stock_barcode/get_specific_barcode_data` | `get_specific_barcode_data(**kwargs)` `:152` | `barcode`/`barcodes`/`barcodes_by_model`/`domains_by_model`/`fetch_quants`/`context` | `defaultdict(list)` model→records | `odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js:308,331,362`, `LazyBarcodeCache` |
| `/stock_barcode/get_specific_barcode_data_batch` | `get_specific_barcode_data_batch(kwargs)` `:238` | dict model→barcodes | model→records | **SIN caller en v18** (ruta muerta/reservada) |
| `/stock_barcode/rid_of_message_demo_barcodes` | `rid_of_message_demo_barcodes(**kw)` `:283` | — | edita client action | `odoo_enterprise/18/stock_barcode/static/src/main_menu/main_menu.js::removeDemoMessage` |
| `/stock_barcode/print_inventory_commands` | `type='http'` `:291` | `barcode_type` | PDF response | menú (print) |

**Helpers del controlador (overridables por herencia Python de la clase):**
- `_get_groups_data()` `:475` — dict de 7 `has_group` que el cliente usa para mostrar/ocultar UI. **Punto clave** para exponer un grupo propio al front.
- `_get_barcode_field_by_model()` `:494` — lista HARDCODEADA de 7 modelos escaneables + su `_barcode_field` (filtra con `hasattr(_barcode_field)`).
- `_get_records_fields_stock_barcode(records)` `:138` — envuelve `read(_get_fields_stock_barcode(), load=False)` + fusiona `_get_stock_barcode_specific_data()` vía `hasattr` (`:141-144`).
- `_get_barcode_nomenclature()` `:486`, `_get_allowed_company_ids()` `:433`, `_get_picking_type_domain()` `:445`, `_get_barcode_pdfs()` `:452`.
- `_try_open_*(barcode)` `:316-431`: `_try_open_lot`, `_try_open_product_location`, `_try_open_picking_type`, `_try_open_picking`, `_try_open_package`, `_try_new_internal_picking`. Override para añadir tipos de resolución.

`get_barcode_data` `:97-115` es el **payload maestro**: llama `target_record._get_stock_barcode_data()`, actualiza `data['records']` con la nomenclatura, añade `data['precision']` (top-level) y rellena `data['config']` con `play_sound`, `barcode_separator_regex`, `barcode_rfid_batch_time`, `delay_between_scan` (todos desde `ir.config_parameter`). Devuelve `{data, groups}`.

---

### B. Métodos ORM llamados vía `orm.call`

| Modelo | Método | Definido en | Caller JS |
|---|---|---|---|
| `stock.picking` | `button_validate` (validateMethod) | core stock | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_validate` (context `barcode_trigger=True` fijado en `:602`) |
| `stock.quant` | `action_validate` (validateMethod) | `stock_barcode/models/stock_quant.py:56`, `StockQuant.action_validate` | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::_apply` |
| `stock.picking` | `action_cancel` | core stock | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::_cancel` |
| `stock.picking` | `action_cancel_from_barcode` | `stock_barcode/models/stock_picking.py:12`, `StockPicking.action_cancel_from_barcode` | `odoo_enterprise/18/stock_barcode/static/src/components/main.js::cancel` |
| `stock.picking` | `action_put_in_pack` | core stock | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js:1823`, `BarcodePickingModel._putInPack` |
| `stock.picking` | `action_create_return_picking` | `stock_barcode/models/stock_picking.py:65`, `StockPicking.action_create_return_picking` | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::_returnProducts` |
| `stock.picking` | `action_open_new_picking` | `stock_barcode/models/stock_picking.py:26`, `class StockPicking` | `stock_barcode/static/src/kanban/stock_barcode_kanban_controller.js:24`, `createRecord` |
| `stock.picking` | `filter_on_barcode` | `stock_barcode/models/stock_picking.py:197`, `StockPicking.filter_on_barcode` | `stock_barcode/static/src/kanban/stock_barcode_kanban_controller.js:46`, `_onBarcodeScannedHandler` |
| `stock.picking` | `do_print_picking` | **core stock** (`odoo18/addons/stock/models/stock_picking.py:1158`, `Picking.do_print_picking`) | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::printButtons,1296` |
| `stock.picking` | `action_print_delivery_slip` `:79`, `action_print_barcode` `:76`, `action_print_packges` `:82` | `stock_picking.py` | `barcode_picking_model.js` (printButtons) |
| `stock.move` | `post_barcode_process` | `stock_barcode/models/stock_move.py:54`, `StockMove.post_barcode_process` | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::_onExit` |
| `stock.quant.package` | `action_create_from_barcode` | `stock_barcode/models/stock_quant_package.py:17`, `class QuantPackage` | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::_processPackage,513` |
| `stock.quant` | `get_stock_barcode_data_records` | `stock_barcode/models/stock_quant.py:96`, `StockQuant.get_stock_barcode_data_records` | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::_processPackage`, `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::_processPackage` |
| `stock.location` | `get_counted_quant_data_records` | `stock_barcode/models/stock_location.py:20`, `Location.get_counted_quant_data_records` | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::loadQuantsForLocation` |
| `product.packaging` | `searchRead` | ORM | `odoo_enterprise/18/stock_barcode/static/src/widgets/digipad.js::_fetchPackagingButtons` |

---

### C. Los métodos-gancho de datos (el corazón extensible)

Todos viven en `models/*.py` como `_inherit`. Patrón: override + `super()`.

**`_get_fields_stock_barcode(self)`** — lista de campos leídos y enviados al cliente. Un método por modelo:
- `stock.picking` `:171` (**17 campos**), `stock.quant` `:126`, `stock.move` `stock_barcode/models/stock_move.py:8`, `StockMove._get_fields_stock_barcode`, `stock.move.line` `stock_barcode/models/stock_move_line.py:93`, `StockMoveLine._get_fields_stock_barcode`, `stock.location` `:17`, `stock.picking.type` `:115`, `product.product` `:26`, `product.packaging` `:19`, `stock.lot` `:25`, `stock.quant.package` `:27`, `stock.package.type` `:12`, `uom.uom` `:10`, `res.partner` `:11`.
- **Es EL punto de extensión para añadir un campo al cliente.** Docstring en `stock_barcode/models/stock_picking.py:172-174`, `StockPicking._get_fields_stock_barcode` lo dice explícitamente.

**`_get_stock_barcode_data(self)`** — payload completo por modelo raíz:
- `stock.picking` `:85-145` — construye `records` con **12 modelos**, `config` (vía `picking_type_id._get_barcode_config()`, con `create_backorder='never'` si `_should_ignore_backorders()`), `nomenclature_id`, `source_location_ids`, `destination_locations_ids`, `line_view_id`, `form_view_id`, `scrap_view_id`, `package_view_id`.
- `stock.quant` `:75-94` — inventario; delega en `get_stock_barcode_data_records()` y añade `stock.location`/`stock.package.type`/`line_view_id`.

**`get_stock_barcode_data_records(self)`** `stock_barcode/models/stock_quant.py:96-124`, `StockQuant.get_stock_barcode_data_records` — dict `records` (incluye `res.company` con `read(['name'])`) + flags `show_quantity_count`, `count_entire_location`, `user_id`.

**`_get_stock_barcode_specific_data(self)`** — registros relacionados extra (opcional, invocado solo si `hasattr` en `:141`): `product.product` (`sale_amazon/models/product_product.py:39`, `ProductProduct._restore_data_product`), `product.packaging` (`stock_barcode/models/product_packaging.py:11`, `ProductPackaging._get_stock_barcode_specific_data`), `stock.quant` (`stock_barcode/models/stock_quant.py:147`, `StockQuant._get_stock_barcode_specific_data`), `stock.lot` (`stock_barcode/models/stock_lot.py:17`, `StockLot._get_stock_barcode_specific_data`).

**`stock.picking.type._get_barcode_config(self)`** `stock_barcode/models/stock_picking_type.py:86-113`, `StockPickingType._get_barcode_config` — traduce campos de config del tipo de operación a flags que el cliente consume (`restrict_scan_*`, `barcode_validation_*`, `create_backorder`, `restrict_put_in_pack`, `lines_need_to_be_packed`). Para exponer una regla de escaneo nueva al front, se amplía aquí.

**`_barcode_field`** (atributo de clase): `stock.picking='name'`, `stock.location='barcode'`, `product.product='barcode'`, `stock.lot='name'`, `stock.quant.package='name'`, `product.packaging='barcode'`, `stock.package.type='barcode'`. Gobierna la búsqueda en `get_specific_barcode_data` (`:185`).

---

### D. Gotchas críticos para extender

1. **`load=False` en todos los `read`**: los relacionales llegan como id int. Si añades un Many2one en `_get_fields_stock_barcode`, hay que garantizar que el modelo relacionado también se envía (añadirlo al `records` de `_get_stock_barcode_data` **y** darle su propio `_get_fields_stock_barcode`). Un campo huérfano rompe `cache.getRecord`.
2. **Para hacer escaneable un modelo nuevo** no basta el `_barcode_field`: hay que sobreescribir `_get_barcode_field_by_model` `:494` (lista hardcodeada de 7 modelos) e incluirlo en el payload. `get_specific_barcode_data :185` hace `env[model]._barcode_field` SIN guardia → `AttributeError` si el modelo no lo define.
3. **Override del controlador**: `StockBarcodeController(http.Controller)` NO usa `_inherit`; se extiende con herencia Python clásica (`class MyCtrl(StockBarcodeController)`) redefiniendo métodos con el mismo `@http.route`. Gana el último registrado.
4. **`save_barcode_data` `:86`** hace `write({write_field: write_vals})` con comandos x2many crudos de JS (`_getSaveCommand`). `barcode_write` en quant (`stock_barcode/models/stock_quant.py:18`, `StockQuant.barcode_write`) es el camino paralelo del inventario (formato posicional `[[1,id,{}],[0,0,{}]]`, `inventory_mode=True`, crea `stock.lot` on-the-fly).
5. **`_get_stock_barcode_data` del picking es caro**: `stock.location search child_of` (x3), `uom.uom search([])` completo si `group_uom`, `_get_usable_packages`. Ampliarlo con más queries degrada el arranque de cada operación. `usable_packages_limit` (config param) limita packages.
6. **`post_barcode_process` `:54`** se ejecuta en validación (context `barcode_trigger=True`): `split_uncompleted_moves` + `_truncate_overreserved_moves` (con `unreserve_unpicked_only`). Es el sitio correcto para reglas de negocio sobre cantidades procesadas por barcode, no `button_validate`.
7. **`filter_on_barcode` `:197` y `main_menu` `:15`** dependen de la nomenclatura GS1 (`parse_barcode`, `is_gs1_nomenclature`). Toda extensión de resolución de barcodes debe cubrir ambas ramas (GS1 y no-GS1).
8. **`config` es un dict plano** que viaja a JS; los flags nuevos en `_get_barcode_config` no hacen nada solos: el OWL debe leerlos (`this.config.<flag>`), lo que casi siempre obliga a un `patch()` JS complementario.
9. **`stock.quant.dummy_id`** es un compute/inverse no-op incluido en `_get_inventory_fields_write` `:144` para identificar líneas nuevas del front antes de tener id. No lo elimines al overridear. (Existe un `dummy_id` análogo en `stock.move.line`, `stock_barcode/models/stock_move_line.py:13`, `class StockMoveLine`.)
10. **`get_specific_barcode_data_batch` `:238` es código muerto en v18**: la ruta existe pero ningún JS de enterprise/base la invoca. No apoyarse en ella como contrato estable.
11. **`do_print_picking` es del core `stock`**, no de stock_barcode; para overridearlo se hereda `stock.picking` del core. Los tres botones de impresión propios de stock_barcode son `action_print_barcode`/`action_print_delivery_slip`/`action_print_packges` (`:76-83`).


## Correcciones aplicadas en verificación

- stock.picking._get_fields_stock_barcode (stock_picking.py:171) devuelve 17 campos, NO 15. Lista real: company_id, location_dest_id, location_id, move_ids, move_line_ids, name, note, partner_id, picking_type_code, picking_type_entire_packs, picking_type_id, return_id, signature, state, use_create_lots, use_existing_lots, user_id.
- stock.picking._get_stock_barcode_data (stock_picking.py:85-145) construye el dict 'records' con 12 modelos, NO 11: stock.picking, stock.picking.type, stock.move, stock.move.line, product.product, product.packaging, res.partner, stock.location, stock.package.type, stock.quant.package, stock.lot, uom.uom.
- do_print_picking NO está definido en odoo_enterprise/18/stock_barcode/models/stock_picking.py:76-83. Ese fichero solo define action_print_barcode (:76), action_print_delivery_slip (:79) y action_print_packges (:82). do_print_picking es un método del módulo core 'stock' (odoo18/addons/stock/models/stock_picking.py:1158). Sigue siendo un target RPC válido (JS odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::printButtons y :1296), pero para overridearlo se hereda stock.picking del core, no la extensión de stock_barcode.
- get_specific_barcode_data_batch (odoo_enterprise/18/stock_barcode/controllers/stock_barcode.py:238) NO tiene ningún caller JS en v18 — ni en stock_barcode ni en el resto de odoo_enterprise/18 ni en addons core. Es una ruta muerta/reservada, no un endpoint de 'batch flows' en uso. El borrador la presentaba como usada por flujos batch; en v18 no la llama nadie.
- Drifts de línea en callers JS (el borrador iba 2-4 líneas desfasado): save() está en odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::save (llama _getSaveCommand en :485), no :487; la llamada a validateMethod está en odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_validate con context['barcode_trigger']=true fijado en :602 (no :603); action_put_in_pack en odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js:1823 (no :1821); action_create_return_picking en :1837 (no :1835); action_cancel_from_barcode en odoo_enterprise/18/stock_barcode/static/src/components/main.js::cancel (no :335); tercer get_specific_barcode_data en odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js:362 (no :361); la ruta save_barcode_data del quant está en odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::_getSaveCommand (el :438 es _getSaveCommand) y la del picking en :1361 (el :1357 es _getSaveCommand).
- Etiquetado incorrecto de _get_stock_barcode_specific_data: el borrador escribió 'product.product product_packaging.py:11'. El método en product_packaging.py:11 pertenece al modelo product.packaging (no product.product). Los 4 modelos que implementan _get_stock_barcode_specific_data son: product.product (product_product.py:39), product.packaging (product_packaging.py:11), stock.quant (stock_quant.py:147) y stock.lot (stock_lot.py:17).
- Ajuste menor de líneas de definición: stock.location._get_fields_stock_barcode está en la def de la línea 17 (decorador @api.model en :16); filter_on_barcode está en la def de la línea 197 (@api.model en :196). El borrador citó :16 y :196 (la línea del decorador), no la del def.
- Complemento (no error): stock.quant.get_stock_barcode_data_records (stock_quant.py:96) además de los flags show_quantity_count/count_entire_location/user_id incluye en 'records' la clave res.company (companies.read(['name'])).
