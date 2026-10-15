## Convenciones

### Deteccion de Registros Cascade

Cuando un paso genera registros secundarios automaticamente, se detectan via API:

| Accion | Modelo Origen | Modelo Generado | Como Detectar |
|--------|--------------|-----------------|---------------|
| sale.order.action_confirm | sale.order | stock.picking | `search_records.sh stock.picking '[["origin","=","<SO_name>"]]'` |
| sale.order._create_invoices | sale.order | account.move | `search_records.sh account.move '[["invoice_origin","=","<SO_name>"]]'` |
| purchase.order.button_confirm | purchase.order | stock.picking | `search_records.sh stock.picking '[["origin","=","<PO_name>"]]'` |
| account.move.action_post | account.move | account.move.line | Ya incluidas en el move |
| account.payment.action_post | account.payment | account.move | Campo move_id en el payment |
| stock.picking.button_validate | stock.picking | stock.move.line | Ya incluidos en el picking |
| mrp.production.action_confirm | mrp.production | stock.move | `search_records.sh stock.move '[["raw_material_production_id","=",<MO_id>]]'` |
| mrp.production.action_confirm | mrp.production | mrp.workorder | `search_records.sh mrp.workorder '[["production_id","=",<MO_id>]]'` |
| hr.expense.sheet.action_sheet_move_post | hr.expense.sheet | account.move | Campo account_move_ids en el sheet (**action_sheet_move_create NO EXISTE en 18.0**) |
| repair.order.action_validate | repair.order | stock.move | `search_records.sh stock.move '[["repair_id","=",<repair_id>]]'` |
| sale.order.action_confirm (dropship) | sale.order | purchase.order | `search_records.sh purchase.order '[["origin","=","<SO_name>"]]'` |
| sale.order.action_confirm (intercompany) | sale.order (Cia A) | purchase.order (Cia B) | `search_records.sh purchase.order '[["origin","=","<SO_name>"],["company_id","=",<company_b_id>]]'` |
| hr.leave.allocation.action_validate | hr.leave.allocation | — | allocation.state -> 'validate' |
| hr.leave.action_approve | hr.leave | — | leave.state -> 'validate' |
| maintenance.request (stage done) | maintenance.request | — | stage_id.done == True |
| hr.applicant.create_employee_from_applicant | hr.applicant | hr.employee | Campo employee_id en el applicant |
| stock.scrap.action_validate | stock.scrap | stock.move | Campo move_ids en el scrap |
| account.asset.validate | account.asset | account.move | Campo depreciation_move_ids en el asset |
| repair.order.action_create_sale_order | repair.order | sale.order | Campo sale_order_id en el repair |

### Tabla Completa de Deteccion de Registros Hijos

| Modelo Padre | Modelo Hijo | Campo de Relacion | Ejemplo de Busqueda |
|-------------|-------------|-------------------|---------------------|
| sale.order | stock.picking | origin = SO name | `[["origin","=","SO001"]]` |
| sale.order | account.move | invoice_origin = SO name | `[["invoice_origin","=","SO001"]]` |
| sale.order | purchase.order | origin = SO name (dropship/MTO) | `[["origin","=","SO001"]]` |
| purchase.order | stock.picking | origin = PO name | `[["origin","=","PO001"]]` |
| purchase.order | account.move | ref = PO name (vendor bill) | `[["ref","=","PO001"]]` |
| mrp.production | stock.move | raw_material_production_id = MO id | `[["raw_material_production_id","=",MO_id]]` |
| mrp.production | stock.move | production_id = MO id (finished) | `[["production_id","=",MO_id]]` |
| mrp.production | mrp.workorder | production_id = MO id | `[["production_id","=",MO_id]]` |
| account.move | account.payment | reconciled_invoice_ids | execute_method / campo en payment |
| account.move | account.move (reversal) | reversed_entry_id | `[["reversed_entry_id","=",move_id]]` |
| hr.expense.sheet | account.move | expense_sheet_id = sheet id | `[["expense_sheet_id","=",sheet_id]]` |
| hr.payslip | account.move | move_id en payslip | Campo directo move_id |
| pos.session | account.move | ref like session name | `[["ref","like","POS/001"]]` |
| pos.order | pos.payment | pos_order_id | `[["pos_order_id","=",order_id]]` |
| repair.order | stock.move | repair_id = repair id | `[["repair_id","=",repair_id]]` |
| repair.order | account.move | repair_ids en move | `[["repair_ids","in",[repair_id]]]` |
| crm.lead | sale.order | opportunity_id = lead id | `[["opportunity_id","=",lead_id]]` |
| stock.warehouse.orderpoint | purchase.order | origin like orderpoint name | `[["origin","like","OP/00001"]]` |
| maintenance.request | stock.picking | (via modulo custom) | Depende de implementacion |
| hr.leave.allocation | hr.leave | holiday_status_id (mismo tipo) | `[["holiday_status_id","=",type_id],["employee_id","=",emp_id]]` |
| hr.applicant | hr.employee | employee_id en applicant | Campo directo employee_id |
| stock.scrap | stock.move | scrap_id = scrap id | `[["scrap_id","=",scrap_id]]` |
| account.asset | account.move | asset_id = asset id | `[["asset_id","=",asset_id]]` |
| helpdesk.ticket | repair.order | ticket_id = ticket id (helpdesk_repair) | `[["ticket_id","=",ticket_id]]` |

### Paso de Variables Entre Flujos

- `{chain.SO_id}` — ID del sale.order creado en paso anterior
- `{chain.SO_name}` — Nombre (referencia) del sale.order
- `{chain.PO_id}` — ID del purchase.order
- `{chain.PO_name}` — Nombre del purchase.order
- `{chain.picking_id}` — ID del picking detectado
- `{chain.invoice_id}` — ID de la factura creada/detectada
- `{chain.payment_id}` — ID del pago creado
- `{chain.MO_id}` — ID de la orden de fabricacion
- `{chain.MO_name}` — Referencia de la orden de fabricacion
- `{chain.bom_id}` — ID de la lista de materiales
- `{chain.repair_id}` — ID de la orden de reparacion
- `{chain.expense_id}` — ID del gasto
- `{chain.sheet_id}` — ID del informe de gastos
- `{chain.employee_id}` — ID del empleado
- `{chain.leave_id}` — ID de la ausencia
- `{chain.allocation_id}` — ID de la asignacion de ausencia
- `{chain.applicant_id}` — ID del candidato
- `{chain.maintenance_id}` — ID de la solicitud de mantenimiento
- `{chain.lead_id}` — ID de la oportunidad CRM
- `{chain.ticket_id}` — ID del ticket de helpdesk
- `{chain.scrap_id}` — ID del desguace (stock.scrap)
- `{chain.asset_id}` — ID del activo fijo (account.asset)
- `{chain.website_id}` — ID del website (website.website)

---

