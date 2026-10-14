# stock_barcode v19 — Contrato RPC servidor Python (verificado)

Superficie servidor que el cliente OWL invoca, releída y verificada contra `odoo_enterprise/19/stock_barcode` (todos los `.py` de `models/` y `controllers/stock_barcode.py`) y cruzada con los callers JS en `static/src`. Todo lo citado es v19; los diffs v18→v19 se confirmaron abriendo también `odoo_enterprise/18/stock_barcode`.

## 1. Arquitectura del contrato

Dos vías:

1. **Rutas HTTP del controlador** en `controllers/stock_barcode.py` — invocadas con `rpc(route, params)` de `@web/core/network/rpc`.
2. **ORM `this.orm.call/write/read/search/...`** — invocan métodos públicos de modelo directamente.

> **CAMBIO v18→v19 (corregido):** NO es cierto que "todas las rutas pasen de `type='json'` a `type='jsonrpc'`". Lo verificado:
> - **7 rutas migradas** `json`→`jsonrpc` (existían en v18 y v19): `scan_from_main_menu` (:18), `save_barcode_data` (:84), `get_barcode_data` (:92), `get_main_menu_data` (:120), `get_specific_barcode_data` (:143), `get_quants` (:232), `rid_of_message_demo_barcodes` (:237).
> - **2 rutas NUEVAS** ya nacen `jsonrpc`: `get_epc` (:467), `get_epc_sgtin` (:472).
> - **1 ruta sigue siendo `type='http'`**: `print_inventory_commands` (:245) — devuelve PDF, no cambió.
> - v18 tenía 8 rutas `json` + 1 `http` (incluía `get_specific_barcode_data_batch` :238, eliminada en v19).
> - **Total v19: 9 rutas `jsonrpc` + 1 `http`.** Una ruta custom en v19 debe declararse `type='jsonrpc'`.

Punto de entrada canónico de datos: **`_get_stock_barcode_data()`** (en `stock.picking` y `stock.quant`), que devuelve `{records: {model: [...]}, config, ...view_ids}`. Los campos por modelo los define **`_get_fields_stock_barcode()`** (uno por modelo). Estos dos son los hooks de extensión reales.

---

## 2. Inventario de rutas del controlador (`controllers/stock_barcode.py`)

| Ruta | Método Python | file:line | Caller JS (verificado) | Salida |
|---|---|---|---|---|
| `/stock_barcode/scan_from_main_menu` | `main_menu(barcode)` | `:18` | `odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js::_onBarcodeScanned` | `{action}` / `{warning}` |
| `/stock_barcode/save_barcode_data` | `save_barcode_data(model,res_id,write_field,write_vals)` | `:84` | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::save` (vía `_getSaveCommand`; route en `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_getSaveCommand` y `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::_getSaveCommand`) | `_get_stock_barcode_data()` (write) o `barcode_write()` (sin res_id) |
| `/stock_barcode/get_barcode_data` | `get_barcode_data(model,res_id)` | `:92` | `odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart` (onWillStart) y `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::getActionRefresh` (`getActionRefresh`) | `{data, groups}` |
| `/stock_barcode/get_main_menu_data` | `get_main_menu_data()` | `:120` | `odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js::setup` | `{groups, play_sound, quant_count}` |
| `/stock_barcode/get_specific_barcode_data` | `get_specific_barcode_data(**kwargs)` | `:143` | `odoo_enterprise/19/stock_barcode/static/src/lazy_barcode_cache.js:326,349,383`, `LazyBarcodeCache` | `{model: [records...]}` |
| `/stock_barcode/get_quants` | `get_existing_quant_and_related_data(domain)` | `:232` | `odoo_enterprise/19/stock_barcode/static/src/lazy_barcode_cache.js:132`, `LazyBarcodeCache.getQuants` | `quants.get_stock_barcode_data_records()` |
| `/stock_barcode/rid_of_message_demo_barcodes` | `rid_of_message_demo_barcodes(**kw)` | `:237` | `odoo_enterprise/19/stock_barcode/static/src/main_menu/main_menu.js::removeDemoMessage` | None (side-effect) |
| `/stock_barcode/print_inventory_commands` | `print_inventory_commands(barcode_type)` | `:245` | `<a>` HTTP link (`type='http'`) | PDF |
| `/stock_barcode/get_epc` | `get_epc(scheme_name, element_string_list, ...)` | `:467` | **sin caller en `stock_barcode/static/src`** (grep=0; lo consume RFID/EPC externo) | `{string: epc}` |
| `/stock_barcode/get_epc_sgtin` | `get_epc_sgtin(gtin, tracking_number_list, ...)` | `:472` | **sin caller en `stock_barcode/static/src`** (grep=0) | `{tracking: epc}` |

### 2.1 Cambios v18→v19 en rutas (verificados)

- **`get_specific_barcode_data_batch` ELIMINADA.** v18: `odoo_enterprise/18/stock_barcode/controllers/stock_barcode.py:238`, `StockBarcodeController.get_specific_barcode_data_batch` (`type='json'`), sin caller JS (grep=0 en `v18/static`) → confirmada ruta muerta en v18. v19: grep en todo el módulo = 0. Confirmado borrada.
- **`get_epc` / `get_epc_sgtin` NUEVAS en v19.** v18: `grep get_epc` en el controlador = 0. Sirven al encoder EPC/RFID (`models/epc_encoder.py`, clase `EpcScheme`, importada en el controlador `:13`). Sin caller dentro de `stock_barcode/static/src`.
- **Dominios con objeto `Domain`.** v19 `get_specific_barcode_data` importa `from odoo.fields import Domain` (controlador `:10`) y usa `Domain(field,'in',barcodes)` (`:180`), `Domain.OR(...)` (`:195`), `Domain.TRUE` (`:161`), operadores `&=`/`|=`/`&` (`:197,201,202,203`). Overrides con listas de tuplas rompen la composición.
- **`_get_groups_data` (`:436`):** en v19 **desaparece `group_stock_packaging`** (v18 lo tenía en `:482`: `product.group_stock_packaging`). Confirmado abriendo v18. Resto de grupos idéntico (`group_stock_multi_locations`, `group_tracking_owner`, `group_tracking_lot`, `group_production_lot`, `group_uom`, `group_stock_sign_delivery`).
- **`_get_barcode_field_by_model` (`:454`):** lista v19 = `stock.location, product.product, uom.uom, product.uom, stock.picking, stock.lot, stock.package, stock.package.type`, filtrada con `hasattr(env[model], '_barcode_field')` (`:465`). El filtro es necesario porque `uom.uom` y `res.partner` NO definen `_barcode_field` en este módulo.
- Config de `get_barcode_data` (`barcode_separator_regex`, `barcode_rfid_batch_time`, `delay_between_scan`): **ya existía en v18**, NO es cambio v19. Confirmado.

---

## 3. Métodos de modelo invocados por ORM (`this.orm.call/write/...`)

| Modelo | Método | file:line (py) | Caller JS (verificado) | Notas |
|---|---|---|---|---|
| `stock.picking` | `button_validate` | (stock core) | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::_validate` vía `this.validateMethod` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::constructor`) | dispatch genérico con `[this.recordIds]` |
| `stock.quant` | `action_validate` | `stock_barcode/models/stock_quant.py:62`, `StockQuant.action_validate` | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js:99`, `BarcodeQuantModel._apply` (directo, `[quantIds]`) y `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::_validate` (genérico) | `validateMethod="action_validate"` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::constructor`) |
| `stock.move.line` | `unlink` | (ORM) | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::deleteLine,891` vía `deleteLineMethod` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::constructor`) | |
| `stock.picking` | `action_cancel_from_barcode` | `stock_barcode/models/stock_picking.py:12`, `StockPicking.action_cancel_from_barcode` | `odoo_enterprise/19/stock_barcode/static/src/components/main.js::cancel` | abre wizard cancelación |
| `stock.picking` | `action_cancel` | (stock core) | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_cancel` | |
| `stock.picking` | `action_put_in_pack` | (stock core) | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js:2339`, `BarcodePickingModel._putInPack` | sobre `this.resModel` |
| `stock.package` | `action_put_in_pack` | (stock core) | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js:2398`, `BarcodePickingModel.getPackageToPackIds` | sobre `packageIds` |
| `stock.picking` | `action_create_return_picking` | `stock_barcode/models/stock_picking.py:65`, `StockPicking.action_create_return_picking` | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_returnProducts` | devolución + abre en barcode |
| `stock.picking` | `action_open_new_picking` | `stock_barcode/models/stock_picking.py:27`, `StockPicking.action_open_new_picking` | `odoo_enterprise/19/stock_barcode/static/src/kanban/stock_barcode_kanban_controller.js::createRecord` | `@api.model` |
| `stock.picking` | `filter_on_barcode` | `stock_barcode/models/stock_picking.py:201`, `StockPicking.filter_on_barcode` | `odoo_enterprise/19/stock_barcode/static/src/kanban/stock_barcode_kanban_controller.js::_onBarcodeScannedHandler` | `@api.model`, filtra pickings ready |
| `stock.move` | `post_barcode_process` | `stock_barcode/models/stock_move.py:56`, `StockMove.post_barcode_process` | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_onExit` | split + truncate tras validar |
| `stock.quant` | `get_stock_barcode_data_records` | `stock_barcode/models/stock_quant.py:110`, `StockQuant.get_stock_barcode_data_records` | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_processPackage`, `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::_processPackage` | cache-fill de quants |
| `stock.location` | `get_counted_quant_data_records` | `stock_barcode/models/stock_location.py:46`, `StockLocation.get_counted_quant_data_records` | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::loadQuantsForLocation` | quants de una ubicación |
| `stock.package` | `action_create_from_barcode` | `stock_barcode/models/stock_package.py:16`, `StockPackage.action_create_from_barcode` (deco `:15`) | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::_processPackage,557` | **CAMBIO v18→v19: era `stock.quant.package`** |

Propiedades JS reapuntables (override en JS sin tocar el core): `this.resModel` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::constructor`), `this.lineModel` / `this.validateMethod` / `this.deleteLineMethod` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::constructor,18,19`; `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::constructor,11,12`). El core llama `orm.call(this.resModel, this.validateMethod, [this.recordIds])` genéricamente en `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::_validate`.

---

## 4. Renombres de modelo v18→v19 (impacto en el contrato)

> **Matiz importante:** son renombres/fusiones del **core de Odoo** (packaging fusionado en UoM; `stock.quant.package`→`stock.package`), no modelos que este módulo cree. `stock_barcode` solo hace `_inherit` de los nombres nuevos. Los ficheros del módulo reflejan el cambio.

- **`product.packaging` → `product.uom`**: v18 `models/product_packaging.py`; v19 `models/product_uom.py` (`_inherit='product.uom'`, `_barcode_field='barcode'`, `_get_fields_stock_barcode = ['barcode','product_id','uom_id']` en `:9`, más `_get_stock_barcode_specific_data` en `:16`). Clave de datos ahora `"product.uom"`.
- **`stock.quant.package` → `stock.package`**: v18 `models/stock_quant_package.py`; v19 `models/stock_package.py`. Aquí viven `action_create_from_barcode` (deco `:15` / def `:16`) y `_get_usable_packages` (deco `:39` / def `:40`).
- **`stock.move.line._get_fields_stock_barcode` (`stock_barcode/models/stock_move_line.py:120`, `StockMoveLine._get_fields_stock_barcode`)** añade en v19: `outermost_result_package_id`, `is_entire_pack`, `packaging_uom_id`, `packaging_uom_qty`, `product_category_name`. Verificado.
- **`stock.move._get_fields_stock_barcode` (`stock_barcode/models/stock_move.py:8`, `StockMove._get_fields_stock_barcode`)** usa `packaging_uom_id`/`packaging_uom_qty` (no `product_packaging_*`). Verificado.
- **`stock.package._get_fields_stock_barcode` (`stock_barcode/models/stock_package.py:25`, `StockPackage._get_fields_stock_barcode`)** incluye `dest_complete_name`, `package_dest_id`, `outermost_package_id`, `contained_quant_ids`, `parent_package_id`, `complete_name`, `location_id`, `location_dest_id`, `package_type_id`, `name`. Verificado.

---

## 5. Puntos de extensión reales

1. **`_get_fields_stock_barcode()`** (por modelo). Docstring explícita en `stock_barcode/models/stock_picking.py:175-177`, `StockPicking._get_fields_stock_barcode`: *"The purpose of this function is to be overridden in order to inject new fields to the client action."* Se lee con `read(..., load=False)` ⇒ los relacionales viajan como ID crudo; para *renderizar* hay que tocar el modelo OWL/template.
2. **`_get_stock_barcode_data()`** — `stock.picking` (`:85`) y `stock.quant` (`:76`). Diccionario raíz que devuelve `get_barcode_data`. Override para inyectar claves nuevas (config, view_ids, colecciones extra).
3. **`_get_stock_barcode_specific_data()`** — `hasattr`-gated en `odoo_enterprise/19/stock_barcode/controllers/stock_barcode.py:226`, `StockBarcodeController._get_records_fields_stock_barcode`. Presente en `product.product` (`:37`), `product.uom` (`:16`), `stock.lot` (`:17`), `stock.quant` (`:158`). Añadirlo a un modelo lo integra en `get_specific_barcode_data` sin tocar el controlador.
4. **`_get_barcode_config()`** — `stock_barcode/models/stock_picking_type.py:86`, `StockPickingType._get_barcode_config`. Dict `config` de comportamiento por tipo de operación.
5. **`_get_fields_stock_barcode` de `stock.picking.type`** (`:122`) + campos de config del tipo (`barcode_allow_extra_product`, `restrict_scan_*`, `restrict_put_in_pack`, `show_reserved_sns`, `is_barcode_picking_type`), todos definidos en `stock_picking_type.py`.
6. **Propiedades JS `resModel`/`lineModel`/`validateMethod`/`deleteLineMethod`** — reapuntar el contrato a un modelo custom desde JS.
7. **`filter_on_barcode` / `action_open_new_picking`** (`@api.model`, `stock_barcode/models/stock_picking.py:201`, `StockPicking.filter_on_barcode`, y `stock_barcode/models/stock_picking.py:27`, `StockPicking.action_open_new_picking`) — lógica de escaneo desde el kanban.
8. **Herencia de `StockBarcodeController`** — override de `_try_*` / `_get_barcode_field_by_model` / `_get_groups_data`. Rutas nuevas: `type='jsonrpc'`.

---

## 6. Gotchas / riesgos (verificados)

- **`auth='user'`, sin sudo general:** las rutas corren como el usuario. `sudo()` solo puntual en `ir.config_parameter.get_param(...)` (`:107,110,111,112,135`) y en el `write` de la acción demo (`:243`).
- **`get_specific_barcode_data` muta el contexto** (`request.update_env(context=...)` `:158`) y fuerza `display_default_code=False` (`:157`). Cuidado al sobreescribir.
- **`read(..., load=False)` en todos los `_get_fields_stock_barcode`** ⇒ relacionales como ID crudo, sin `(id,name)`; un campo nuevo relacional necesita precarga aparte.
- **Docstring obsoleta** en `get_barcode_data` (`:96`) menciona `_get_barcode_data_prefetch`, método **inexistente**. El real es `_get_stock_barcode_data`.
- **`barcode_write` (`stock_barcode/models/stock_quant.py:24`, `StockQuant.barcode_write`)** usa comandos legacy `[[1,id,vals],[0,0,vals],...]`, NO `Command.*`. Se dispara cuando `save_barcode_data` recibe `res_id` falso (`:86-87`).
- **`_search` con `_preprocess_gs1_search_args`** en `stock_barcode/models/product_product.py:18`, `class Product`, `stock_barcode/models/stock_location.py:38`, `stock_barcode/models/stock_package.py:11`, `StockPackage._search`, `stock_barcode/models/stock_lot.py:12`, `StockLot._search` (product y lot lo hacen con `.sudo()` para usuarios externos). Respetar `super()._search` para no romper GS1.
- **Objetos `Domain`** en `get_specific_barcode_data`: override con listas de tuplas rompe `&=`/`|=`/`Domain.OR`.
- **`get_epc`/`get_epc_sgtin`** sin caller en `static/src`: este subsistema no los cruza con JS local (RFID externo).


## Correcciones aplicadas en verificación (v19)

- §1 y changedVsV18[0] IMPRECISO->CORREGIDO: no 'todas las rutas' pasan de type='json' a 'jsonrpc'. Verificado: 7 rutas migradas json->jsonrpc (scan_from_main_menu:18, save_barcode_data:84, get_barcode_data:92, get_main_menu_data:120, get_specific_barcode_data:143, get_quants:232, rid_of_message_demo_barcodes:237); 2 NUEVAS ya nacen jsonrpc (get_epc:467, get_epc_sgtin:472); print_inventory_commands:245 sigue type='http'. Total v19 = 9 jsonrpc + 1 http. v18 tenia 8 json + 1 http.
- save_barcode_data caller CORREGIDO: odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::save (no :506). El route string vive en los overrides _getSaveCommand: odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_getSaveCommand y odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::_getSaveCommand.
- get_specific_barcode_data callers CORREGIDO: odoo_enterprise/19/stock_barcode/static/src/lazy_barcode_cache.js:326,349,383 (el borrador decia 382; la linea del route string es 383).
- §4 _get_usable_packages CORREGIDO: stock_package.py:40 (def) / :39 (deco), NO :38 como decia el borrador.
- action_create_from_barcode caller JS PRECISADO: odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::_processPackage,557 (lineas del string del metodo); el borrador citaba 543,555 (inicio de la llamada orm.call) — aceptable pero afinado. El metodo Python esta en stock_package.py deco :15 / def :16.
- MATIZ anadido en §4: product.uom y stock.package son renombres/fusiones del CORE de Odoo (packaging->uom; quant.package->package), no modelos que stock_barcode cree. El modulo solo hace _inherit; los ficheros del modulo reflejan el cambio. El borrador era correcto sobre los ficheros pero omitia el origen core.
- CONFIRMADO (no eran errores): _get_stock_barcode_data stock_picking.py:85 y stock_quant.py:76; docstring _get_fields_stock_barcode stock_picking.py:175-177; hasattr gate controllers:226; _get_barcode_config stock_picking_type.py:86; barcode_write comandos legacy stock_quant.py:24; docstring obsoleta _get_barcode_data_prefetch :96; get_specific_barcode_data_batch borrada en v19 (grep=0) y muerta en v18 (sin caller JS); get_epc nuevo (v18 grep=0); _get_groups_data pierde group_stock_packaging (v18 lo tenia en :482); Domain objects (import :10); mutacion de context :157-158; get_epc/get_epc_sgtin sin caller en static/src (grep=0); todos los file:line de _get_fields_stock_barcode por modelo; callers ORM action_validate:99/639, action_cancel:1241, action_put_in_pack:2339/2398, post_barcode_process:1910, get_stock_barcode_data_records:2018/569, get_counted_quant_data_records:648, action_open_new_picking kanban:24, filter_on_barcode kanban:44, action_cancel_from_barcode odoo_enterprise/19/stock_barcode/static/src/components/main.js::cancel, action_create_return_picking:2424.
