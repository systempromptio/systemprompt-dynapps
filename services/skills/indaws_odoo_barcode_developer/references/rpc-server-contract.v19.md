# Contrato RPC cliente → servidor — stock_barcode v19 + Shop Floor

## Rutas HTTP (`controllers/stock_barcode.py`, `StockBarcodeController`)

En v19 hay **9 rutas jsonrpc + 1 http** (v18: 8 json + 1 http). NO "todas migraron"; 7 pasaron `json`→`jsonrpc`, 2 nacen jsonrpc, 1 sigue http.

| Ruta | Tipo | Línea | Propósito | Caller JS | ¿Extensión? |
|---|---|---|---|---|---|
| `/stock_barcode/scan_from_main_menu` | jsonrpc | :18 | Resolver escaneo desde menú; **inyecta `context.active_id`** (leído en odoo_enterprise/19/stock_barcode/static/src/components/main.js:100) | main_menu | Sí (override método) |
| `/stock_barcode/get_barcode_data` | jsonrpc | :92 | Carga inicial completa | `odoo_enterprise/19/stock_barcode/static/src/components/main.js::onWillStart` (onWillStart) | Sí — punto principal de carga |
| `/stock_barcode/save_barcode_data` | jsonrpc | :84 | Persistir x2many commands | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::save`; route string en overrides `_getSaveCommand` (picking `:1670`, quant `:482`) | Sí |
| `/stock_barcode/get_main_menu_data` | jsonrpc | :120 | Datos del kanban/menú | main_menu | Sí |
| `/stock_barcode/get_specific_barcode_data` | jsonrpc | :143 | Fetch on-scan de records por barcode | `odoo_enterprise/19/stock_barcode/static/src/lazy_barcode_cache.js:326,349,383`, `LazyBarcodeCache` | Sí — usa `odoo.fields.Domain` (import :10), NO listas; `nomenclature_id` en :163 |
| `/stock_barcode/get_quants` | jsonrpc | :232 | Quants para inventario | quant model | Sí |
| `/stock_barcode/rid_of_message_demo_barcodes` | jsonrpc | :237 | Housekeeping demo | — | raro |
| `/stock_barcode/get_epc` | jsonrpc | :467 | **NUEVO v19** (RFID EPC) | **SIN caller en static/src (grep=0)** | dudoso — presente pero no invocado por el cliente |
| `/stock_barcode/get_epc_sgtin` | jsonrpc | :472 | **NUEVO v19** (RFID SGTIN) | **SIN caller (grep=0)** | dudoso — idem |
| `/stock_barcode/print_inventory_commands` | http | :245 | Impresión comandos inventario | — | Sí (sigue http, sin cambio) |

**`get_specific_barcode_data_batch`: BORRADA en v19** (grep=0). Ya estaba **muerta en v18** (definida pero sin caller JS). No es punto de extensión: no reintroducir.

Overrides del controller: heredar `StockBarcodeController` y sobrescribir `_try_open_picking`/`_try_open_picking_type`/`_try_new_internal_picking`/`_try_open_product_location`/`_try_open_lot`/`_try_open_package`, `_get_barcode_field_by_model`, `_get_groups_data`, `_get_records_fields_stock_barcode` (def `:223`, hasattr-gate `:226`). Rutas nuevas propias → `type='jsonrpc'`.

## Llamadas ORM (orm.call desde el cliente)

| Modelo | Método | Propósito | Caller JS | ¿Extensión? |
|---|---|---|---|---|
| `this.resModel` (stock.picking) | `validateMethod` (button_validate) | Validar picking; firma `[recordIds]` | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::_validate` | Sí — reapuntar `validateMethod` en subclase |
| `stock.quant` | `action_validate` | Validar inventario; llamada directa `[quantIds]` + context | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js:99`, `BarcodeQuantModel.action` | Sí |
| `stock.picking` | `action_cancel` | Cancelar | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::_cancel`; `action_cancel_from_barcode` en `odoo_enterprise/19/stock_barcode/static/src/components/main.js::cancel` | Sí |
| `stock.picking` | `action_put_in_pack` | Put-in-pack | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js:2339`, `BarcodePickingModel._putInPack` | Sí (NUEVO uso anidado v19) |
| `stock.package` | `action_put_in_pack` | Pack-in-pack anidado | `:2398` | Sí — NUEVO v19 |
| `stock.move` | `post_barcode_process` | Persistir agregados al salir sin validar | `:1910` | Sí — NUEVO v19 (solo picking) |
| `this.resModel` | `get_stock_barcode_data_records` | Recarga de records | `:2018`, `:569` | Sí |
| `stock.quant` | `get_counted_quant_data_records` | Recarga quants contados | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::loadQuantsForLocation` | Sí |
| `stock.picking` | `action_open_new_picking` | Nuevo picking desde kanban | `kanban:24` | Sí (`@api.model`, `stock_barcode/models/stock_picking.py:27`, `StockPicking.action_open_new_picking`) |
| `stock.picking` | `filter_on_barcode` | Filtrar kanban por barcode | `kanban:44` | Sí (`@api.model`, `stock_barcode/models/stock_picking.py:201`, `StockPicking.filter_on_barcode`) |
| `stock.picking` | `action_create_return_picking` | Devolución | `:2424` | Sí |
| `stock.package` | `action_create_from_barcode` | Crear paquete desde barcode | `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::_processPackage,557` (Python `stock_barcode/models/stock_package.py:16`, `StockPackage.action_create_from_barcode`) | Sí |
| `hr.employee` | `set_employees_connected` | Empleados conectados (Shop Floor) | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/hooks/employee_hooks.js::useConnectedEmployee` | Sí — **NUEVO v19** (`mrp_workorder/models/hr_employee.py:135`, `HrEmployee.set_employees_connected`) |
| `mrp.workorder` | `get_shopfloor_limit` | Límite de paginación Shop Floor | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_action.js::setup` | Sí |
| MO record | `do_finish` / `button_mark_done` | Cerrar MO (nombre asignado :334/:345) | `odoo_enterprise/19/mrp_workorder/static/src/mrp_display/mrp_display_record.js::validate` (call único) | Sí |

## Contrato de datos por modelo (inyección de campos)

- **`_get_fields_stock_barcode`** (por modelo): define qué campos van a `data.records[model]`. Overridable en: `stock.picking` (`stock_barcode/models/stock_picking.py:174`, `StockPicking._get_fields_stock_barcode`, docstring `:175-177`), `stock.quant` (`stock_barcode/models/stock_quant.py:138`, `StockQuant._get_fields_stock_barcode`), `stock.move` (`:8`), `stock.move.line` (`:120`), `product.product` (`:24`), `stock.picking.type` (`:122`), `stock.location` (`:43`), `stock.package` (`:25`), `stock.lot` (`:25`), `uom.uom` (`:10`), `product.uom` (`:9`), `res.partner` (`:11`), `stock.package.type` (`:12`). Todos leen con `read(load=False)` → **relacionales viajan como ID crudo, sin display** (requiere precarga + tocar OWL/template).
- **`_get_stock_barcode_data`** (`stock_barcode/models/stock_picking.py:85`, `StockPicking._get_stock_barcode_data`, `stock_barcode/models/stock_quant.py:76`, `StockQuant._get_stock_barcode_data`): dict raíz de la carga inicial. Override con `super()` + claves. Método pesado (muchos search + has_group gating) → añadir searches degrada el arranque.
- **`_get_stock_barcode_specific_data`** (`stock_barcode/models/product_product.py:37`, `Product._get_fields_stock_barcode`, `stock_barcode/models/product_uom.py:16`, `ProductUom._get_stock_barcode_specific_data`, `stock_barcode/models/stock_lot.py:17`, `StockLot._get_stock_barcode_specific_data`, `stock_barcode/models/stock_quant.py:159`, `StockQuant._get_stock_barcode_specific_data`, v19): precarga de relacionados on-scan; invocado por `hasattr` en `controllers:226`.
- **`_get_barcode_config`** (`stock_barcode/models/stock_picking_type.py:86`, `StockPickingType._get_barcode_config`): flags de config por tipo de operación. El cliente OWL debe conocer las claves o solo-Python no cambia comportamiento.

## Propiedades JS reapuntables (contrato de método servidor)

`this.resModel` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_model.js::constructor`), `this.lineModel`/`this.validateMethod`/`this.deleteLineMethod` (`odoo_enterprise/19/stock_barcode/static/src/models/barcode_picking_model.js::constructor,18,19`; `odoo_enterprise/19/stock_barcode/static/src/models/barcode_quant_model.js::constructor,11,12`). El método destino debe respetar la firma (validate genérico `[recordIds]`; quant directo `[quantIds]`) y devolver acción/None compatible. `_get_groups_data` **pierde `group_stock_packaging` en v19** (v18 lo tenía en `:482`).
