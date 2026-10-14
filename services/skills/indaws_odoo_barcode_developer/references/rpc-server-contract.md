Contrato RPC cliente→servidor consolidado (Odoo 18 EE). "Ext. server-side" = si es punto de extensión por herencia Python. Líneas de caller JS verificadas (con drifts del draft corregidos).

## stock_barcode — controlador (`controllers/stock_barcode.py`, `http.Controller` clásico, se extiende por herencia y mismo `@http.route`, gana el último registrado)

| Route / método | Def | Propósito | Caller JS | Ext. server-side |
|---|---|---|---|---|
| `/stock_barcode/get_barcode_data` → `get_barcode_data` | `:89`/`:90` | Blob inicial: llama `_get_stock_barcode_data` del modelo | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::getActionRefresh` (`getActionRefresh` def `:94`) | SÍ (override controlador + `_get_stock_barcode_data`) |
| `/stock_barcode/get_quants` → `get_existing_quant_and_related_data` | `:147`/`:148` | Quants + datos relacionados (NO se llama "get_quants") | `barcode_quant_model.js` (loadQuants) | SÍ |
| `/stock_barcode/get_specific_barcode_data` → `get_specific_barcode_data` | `:152`/`:153` | Fetch on-demand de un barcode/modelo; usa `env[model]._barcode_field` (`:185`) | `lazy_barcode_cache.js` (`:362` 3ª llamada) | SÍ |
| `get_specific_barcode_data_batch` | `:238` | **RUTA MUERTA en v18** — sin ningún caller JS en stock_barcode ni EE ni core | ninguno | Reservada, no usar |
| `save_barcode_data` | `:81` | write batch: `write({write_field: write_vals})` (`:86`) con comandos x2many crudos de `_getSaveCommand` | picking `odoo_enterprise/18/stock_barcode/static/src/models/barcode_model.js::_processBarcode` (save `:484`) | SÍ (o override write del modelo destino) |
| `_get_groups_data` | `:475` | dict de 7 grupos (has_group) que condicionan ramas del modelo | vía blob → `this.groups.*` | SÍ |
| `_get_barcode_field_by_model` | `:494` (return `:504`, filtra hasattr) | dict modelo→campo barcode (7 modelos hardcodeados) | vía blob → LazyBarcodeCache | SÍ (pareja con `barcodeFieldByModel` cliente) |
| `_get_records_fields_stock_barcode` | `:141-144` | invoca `_get_stock_barcode_specific_data` vía hasattr | interno | indirecto |
| `main_menu` (`/stock_barcode/scan_from_main_menu`) + `_try_open_*` | `:15`, `:316-431` | Escaneo desde el menú principal (lote/producto/ubicación/tipo/picking/paquete) | MainMenu | SÍ (encadenar `_try_open_*`, cubrir GS1/no-GS1) |

## stock_barcode — métodos de modelo (RPC vía `orm.call`)

| Modelo.método | Fichero | Propósito | Caller JS | Ext. |
|---|---|---|---|---|
| `stock.picking._get_stock_barcode_data` | `stock_barcode/models/stock_picking.py:85`, `StockPicking._get_stock_barcode_data` | Compone `records` con 12 modelos | (dentro de get_barcode_data) | SÍ |
| `stock.quant._get_stock_barcode_data` / `get_stock_barcode_data_records` | `stock_barcode/models/stock_quant.py:75`, `StockQuant._get_stock_barcode_data` / `:96` | Payload quant + flags + `res.company` | idem | SÍ |
| `<modelo>._get_fields_stock_barcode` | 13 modelos (picking:171, quant:126, move:8, move.line:93, location:17, picking.type:115, product:26, packaging:19, lot:25, quant.package:27, package.type:12, uom:10, partner:11) | Whitelist de campos al front (docstring "to be overridden") | vía payload | SÍ (el punto de extensión canónico) |
| `<modelo>._get_stock_barcode_specific_data` | product.product:39, product.packaging:11, stock.quant:147, stock.lot:17 | Datos específicos on-demand `{model: read(...)}` | vía `_get_records_fields_stock_barcode` | SÍ |
| `stock.picking.type._get_barcode_config` | `stock_barcode/models/stock_picking_type.py:86`, `StockPickingType._get_barcode_config` | Flags de config (barcode_allow_extra_product, restrict_scan_*, create_backorder, etc.) | `this.config.*` | SÍ (flag JSON-safe; requiere patch JS para comportamiento) |
| `stock.move.post_barcode_process` | `stock_barcode/models/stock_move.py:54`, `StockMove.post_barcode_process` (`split_uncompleted_moves:16`, `_truncate_overreserved_moves:43`) | Post-proceso de moves en validación (context `barcode_trigger`) | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::_onExit` (`orm.call('stock.move','post_barcode_process',[moveIds, quantitiesByMove])`) | SÍ (alto riesgo) |
| `stock.picking.action_put_in_pack` | core | Empaquetado | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js:1823`, `BarcodePickingModel._putInPack` | SÍ (core) |
| `stock.picking.action_create_return_picking` | `stock_barcode/models/stock_picking.py:65`, `StockPicking.action_create_return_picking` | Devolución | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::_returnProducts` | SÍ |
| `stock.picking._create_new_picking` | `stock_barcode/models/stock_picking.py:148`, `StockPicking._create_new_picking` | Nuevo picking desde barcode | main_menu flow | SÍ |
| `stock.picking.action_cancel_from_barcode` | — | Cancelar | `odoo_enterprise/18/stock_barcode/static/src/components/main.js::cancel` | SÍ |
| `stock.picking.do_print_picking` | **core** `stock/models/stock_picking.py:1158`, `Picking.do_print_picking` | Imprimir (NO es de stock_barcode) | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_picking_model.js::printButtons,1296` | SÍ (heredar stock.picking core) |
| `stock.picking.action_print_barcode` / `action_print_delivery_slip` / `action_print_packges` | `stock_barcode/models/stock_picking.py:76`, `StockPicking.action_print_barcode`/`:79`/`:82` | Reports de stock_barcode | getter `printButtons` (JS) | SÍ (+ patch JS del getter para botón nuevo) |
| `stock.picking.filter_on_barcode` | `stock_barcode/models/stock_picking.py:197`, `StockPicking.filter_on_barcode` (def; `@api.model` :196) | Filtrado por barcode | main_menu | SÍ |
| `stock.quant.barcode_write` | `stock_barcode/models/stock_quant.py:17`, `class StockQuant` (`@api.model`) | write inventario `inventory_mode=True`, formato posicional, crea `stock.lot` on-the-fly | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::_getSaveCommand` | SÍ |
| `stock.quant.action_validate` | `stock_barcode/models/stock_quant.py:56`, `StockQuant.action_validate` | Aplicar ajuste; puede devolver acción-wizard | `odoo_enterprise/18/stock_barcode/static/src/models/barcode_quant_model.js::_apply-87` (onClose `:79-89`) | SÍ |
| `stock.quant.package.action_create_from_barcode` | `stock_barcode/models/stock_quant_package.py:17`, `class QuantPackage` (`@api.model`) | Crea paquete; retorno `{'stock.quant.package': recordset.read(...)}` inyectado directo en cache (`:501,:513`) | barcode_quant_model | SÍ (shape exacto `:22-24`) |
| `stock.picking.action_open_new_picking` | core | Nuevo picking desde kanban | `odoo_enterprise/18/stock_barcode/static/src/kanban/stock_barcode_kanban_controller.js::createRecord` | SÍ |

## mrp_display (mrp_workorder) — métodos RPC

| Modelo.método | Fichero | Propósito | Caller / punto JS | Ext. |
|---|---|---|---|---|
| `mrp.production.button_mark_done` | `mrp/models/mrp_production.py:2117`, `MrpProduction.button_mark_done` | Validar MO (en lote) | `processValidationStack` (`odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display.js::processValidationStack`) | SÍ (core mrp) |
| `mrp.workorder.do_finish` / `pre_record_production` / `pre_button_mark_done` | mrp_workorder (`pre_button_mark_done:98`) | Cierre de WO / pre-validación | validationStack | SÍ |
| `mrp.production.set_qty_producing` | `mrp:1443` | Fijar cantidad produciendo | dialogs | SÍ (core) |
| `mrp.production.action_mass_produce` | `mrp:2409` | Producción en masa | UI | SÍ (core) |
| `mrp.production.action_start` | `mrp:2979` | Iniciar (base mrp, NO mrp_workorder) | UI | SÍ (core) |
| `mrp.production.action_generate_serial` | `mrp:1519` | Generar serial | UI | SÍ (core) |
| `quality.check.action_pass_and_next` / `action_fail_and_next` | `quality_mrp_workorder/models/quality.py:82`, `QualityCheck.action_pass_and_next`/`:87` | Pasar/fallar check | `QualityCheck.pass()` / dialog | SÍ (multi-módulo) |
| `quality.check.do_measure` | `quality_control` (override `quality_mrp_workorder:44`) | Medida | dialog `validate()` (`:71`) | SÍ |
| `quality.check.action_worksheet_check` | `quality_control_worksheet:88` + `quality_mrp_workorder_worksheet:11` | Check de worksheet | UI | SÍ |
| `quality.check.action_open_quality_check_wizard` | `quality_control(_worksheet)` | Abrir wizard | UI | SÍ |
| `quality.check.action_print`/`action_next`/`action_continue`/`action_generate_serial_number_and_pass` | `odoo_enterprise/18/mrp_workorder/models/quality.py:312, `QualityCheck.action_print`/357/361/455` | Acciones de check | UI/dialog | SÍ |
| `quality.check.action_add_step` / `action_propose_change` | vía `proposeChangeForCheck` | Añadir paso / proponer cambio | `odoo_enterprise/18/mrp_workorder/static/src/mrp_display/dialog/mrp_menu_dialog.js::proposeChangeForCheck/173` | SÍ |
| `mrp.production.action_add_component` | vía `callAddComponentAction` | Añadir componente | `odoo_enterprise/18/mrp_workorder/static/src/mrp_display/dialog/mrp_menu_dialog.js::callAddComponentAction` | SÍ |
| `mrp.workcenter.get_employee_barcode` | `mrp_workorder/models/mrp_workcenter.py:36`, `MrpWorkcenter.get_employee_barcode` | Resolver empleado por barcode | `odoo_enterprise/18/mrp_workorder/static/src/mrp_display/mrp_display.js::_onBarcodeScanned` | SÍ |
| `hr.employee.get_all_employees` | `mrp_workorder/models/hr_employee.py:155`, `HrEmployee.get_all_employees` | Lista empleados (hook) | `useConnectedEmployee` | SÍ |
| `hr.employee.login`/`logout`/`pin_validation`/`remove_session_owner`/`stop_all_workorder_from_employee` | `mrp_workorder/models/hr_employee.py:28`, `HrEmployee.login`; `:35`, `HrEmployee.logout`; `:21`, `HrEmployee.pin_validation`; `:51`, `HrEmployee.remove_session_owner`; `:116`, `HrEmployee.stop_all_workorder_from_employee`. `start_employee`/`stop_employee` NO están aquí: son de `mrp.workorder` (`mrp_workorder/models/mrp_workorder.py:788`, `MrpProductionWorkcenterLine.start_employee`, y `:798`, `MrpProductionWorkcenterLine.stop_employee`) | Sesión/PIN empleado | hook | SÍ |

**Nota transversal RPC:** en todos los `read` del contrato los Many2one llegan como **id crudo** (`load=False`). Para que un campo relacional sea usable en cliente hay que (a) declararlo en `_get_fields_stock_barcode` y (b) enviar también el modelo relacionado en `_get_stock_barcode_data` con sus fields, o `cache.getRecord` rompe. Todo valor debe ser JSON-serializable (copia profunda `JSON.parse(JSON.stringify())` en `odoo_enterprise/18/stock_barcode/static/src/lazy_barcode_cache.js:114`, `LazyBarcodeCache.getRecord`).
