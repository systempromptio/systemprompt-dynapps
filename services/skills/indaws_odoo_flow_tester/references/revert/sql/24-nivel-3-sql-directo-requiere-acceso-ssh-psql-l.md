### Limpieza masiva por prefijo [TEST-*]

```sql
BEGIN;

-- ============================================================
-- ORDEN OBLIGATORIO: Inverso a la cadena de dependencias FK
-- 1. Reconciliaciones → 2. Pagos → 3. Facturas → 4. Albaranes
-- → 5. MRP → 6. SO/PO → 7. CRM → 8. Proyecto → 9. Gastos
-- → 10. POS → 11. RRHH → 12. Otros modelos
-- ============================================================

-- ---- 0. RECONCILIACIONES (NUEVO — debe ir PRIMERO) ----
-- Borrar reconciliaciones de moves de test ANTES de borrar los moves
CREATE TEMP TABLE _test_move_line_ids AS
SELECT aml.id FROM account_move_line aml
JOIN account_move am ON aml.move_id = am.id
WHERE am.ref LIKE '[TEST-%' OR am.name LIKE '[TEST-%';

DELETE FROM account_partial_reconcile
WHERE debit_move_id IN (SELECT id FROM _test_move_line_ids)
   OR credit_move_id IN (SELECT id FROM _test_move_line_ids);

-- Limpiar fulls huerfanos
DELETE FROM account_full_reconcile
WHERE id NOT IN (
    SELECT DISTINCT full_reconcile_id
    FROM account_partial_reconcile
    WHERE full_reconcile_id IS NOT NULL
);

-- Resetear full_reconcile_id en move lines afectadas
UPDATE account_move_line SET full_reconcile_id = NULL, matching_number = NULL
WHERE id IN (SELECT id FROM _test_move_line_ids) AND full_reconcile_id IS NOT NULL;

-- ---- 1. PAGOS ----
CREATE TEMP TABLE _test_payments AS
SELECT ap.id, ap.move_id FROM account_payment ap
JOIN account_move am ON ap.move_id = am.id
WHERE am.ref LIKE '[TEST-%' OR am.name LIKE '[TEST-%';

DELETE FROM mail_message WHERE model = 'account.payment' AND res_id IN (SELECT id FROM _test_payments);
DELETE FROM mail_followers WHERE res_model = 'account.payment' AND res_id IN (SELECT id FROM _test_payments);
DELETE FROM ir_attachment WHERE res_model = 'account.payment' AND res_id IN (SELECT id FROM _test_payments);
DELETE FROM account_payment WHERE id IN (SELECT id FROM _test_payments);

-- ---- 2. FACTURAS/ASIENTOS ----
CREATE TEMP TABLE _test_moves AS
SELECT id FROM account_move WHERE name LIKE '[TEST-%' OR ref LIKE '[TEST-%';

-- Limpiar FKs desde otros modelos que apuntan a account_move
UPDATE hr_expense_sheet SET account_move_id = NULL WHERE account_move_id IN (SELECT id FROM _test_moves);
UPDATE hr_payslip SET move_id = NULL WHERE move_id IN (SELECT id FROM _test_moves);
UPDATE pos_order SET account_move = NULL WHERE account_move IN (SELECT id FROM _test_moves);
UPDATE account_asset SET disposal_move_id = NULL WHERE disposal_move_id IN (SELECT id FROM _test_moves);

-- Limpiar valuation layers
DELETE FROM stock_valuation_layer WHERE account_move_id IN (SELECT id FROM _test_moves);

-- Limpiar auxiliares
DELETE FROM mail_message WHERE model = 'account.move' AND res_id IN (SELECT id FROM _test_moves);
DELETE FROM mail_followers WHERE res_model = 'account.move' AND res_id IN (SELECT id FROM _test_moves);
DELETE FROM mail_activity WHERE res_model = 'account.move' AND res_id IN (SELECT id FROM _test_moves);
DELETE FROM ir_attachment WHERE res_model = 'account.move' AND res_id IN (SELECT id FROM _test_moves);

-- Borrar lineas y moves
DELETE FROM account_move_line WHERE move_id IN (SELECT id FROM _test_moves);
DELETE FROM account_move WHERE id IN (SELECT id FROM _test_moves);

-- ---- 3. ALBARANES ----
CREATE TEMP TABLE _test_pickings AS
SELECT id FROM stock_picking WHERE origin LIKE '[TEST-%' OR name LIKE '[TEST-%';

-- Guardar productos afectados para recalculo posterior de quants
CREATE TEMP TABLE _affected_stock_products AS
SELECT DISTINCT sml.product_id
FROM stock_move_line sml
WHERE sml.picking_id IN (SELECT id FROM _test_pickings);

DELETE FROM stock_valuation_layer WHERE stock_move_id IN (
    SELECT id FROM stock_move WHERE picking_id IN (SELECT id FROM _test_pickings));
DELETE FROM stock_move_line WHERE picking_id IN (SELECT id FROM _test_pickings);
-- Romper auto-referencias M2M de stock.move
DELETE FROM stock_move_move_rel WHERE move_orig_id IN (
    SELECT id FROM stock_move WHERE picking_id IN (SELECT id FROM _test_pickings))
    OR move_dest_id IN (
    SELECT id FROM stock_move WHERE picking_id IN (SELECT id FROM _test_pickings));
DELETE FROM stock_scrap WHERE picking_id IN (SELECT id FROM _test_pickings);
DELETE FROM stock_move WHERE picking_id IN (SELECT id FROM _test_pickings);
DELETE FROM stock_package_level WHERE picking_id IN (SELECT id FROM _test_pickings);
DELETE FROM mail_message WHERE model = 'stock.picking' AND res_id IN (SELECT id FROM _test_pickings);
DELETE FROM mail_followers WHERE res_model = 'stock.picking' AND res_id IN (SELECT id FROM _test_pickings);
DELETE FROM ir_attachment WHERE res_model = 'stock.picking' AND res_id IN (SELECT id FROM _test_pickings);
DELETE FROM stock_picking WHERE id IN (SELECT id FROM _test_pickings);

-- ---- 4. MRP ----
CREATE TEMP TABLE _test_productions AS
SELECT id FROM mrp_production WHERE name LIKE '[TEST-%' OR origin LIKE '[TEST-%';

DELETE FROM stock_valuation_layer WHERE stock_move_id IN (
    SELECT id FROM stock_move
    WHERE raw_material_production_id IN (SELECT id FROM _test_productions)
       OR production_id IN (SELECT id FROM _test_productions));
DELETE FROM stock_scrap WHERE production_id IN (SELECT id FROM _test_productions);
DELETE FROM mrp_workcenter_productivity WHERE workorder_id IN (
    SELECT id FROM mrp_workorder WHERE production_id IN (SELECT id FROM _test_productions));
DELETE FROM mrp_workorder WHERE production_id IN (SELECT id FROM _test_productions);
DELETE FROM stock_move_line WHERE move_id IN (
    SELECT id FROM stock_move
    WHERE raw_material_production_id IN (SELECT id FROM _test_productions)
       OR production_id IN (SELECT id FROM _test_productions));
-- Romper auto-referencias M2M
DELETE FROM stock_move_move_rel WHERE move_orig_id IN (
    SELECT id FROM stock_move
    WHERE raw_material_production_id IN (SELECT id FROM _test_productions)
       OR production_id IN (SELECT id FROM _test_productions))
    OR move_dest_id IN (
    SELECT id FROM stock_move
    WHERE raw_material_production_id IN (SELECT id FROM _test_productions)
       OR production_id IN (SELECT id FROM _test_productions));
DELETE FROM stock_move
WHERE raw_material_production_id IN (SELECT id FROM _test_productions)
   OR production_id IN (SELECT id FROM _test_productions);
DELETE FROM mrp_unbuild WHERE mo_id IN (SELECT id FROM _test_productions);
DELETE FROM mail_message WHERE model = 'mrp.production' AND res_id IN (SELECT id FROM _test_productions);
DELETE FROM mail_followers WHERE res_model = 'mrp.production' AND res_id IN (SELECT id FROM _test_productions);
DELETE FROM ir_attachment WHERE res_model = 'mrp.production' AND res_id IN (SELECT id FROM _test_productions);
DELETE FROM mrp_production WHERE id IN (SELECT id FROM _test_productions);

-- ---- 5. PEDIDOS DE VENTA ----
CREATE TEMP TABLE _test_so AS
SELECT id FROM sale_order WHERE name LIKE '[TEST-%' OR client_order_ref LIKE '[TEST-%';

DELETE FROM mail_message WHERE model = 'sale.order' AND res_id IN (SELECT id FROM _test_so);
DELETE FROM mail_followers WHERE res_model = 'sale.order' AND res_id IN (SELECT id FROM _test_so);
DELETE FROM ir_attachment WHERE res_model = 'sale.order' AND res_id IN (SELECT id FROM _test_so);
DELETE FROM sale_order_line WHERE order_id IN (SELECT id FROM _test_so);
DELETE FROM sale_order WHERE id IN (SELECT id FROM _test_so);

-- ---- 5b. PEDIDOS DE COMPRA ----
CREATE TEMP TABLE _test_po AS
SELECT id FROM purchase_order WHERE name LIKE '[TEST-%' OR partner_ref LIKE '[TEST-%';

UPDATE account_move_line SET purchase_line_id = NULL
WHERE purchase_line_id IN (SELECT id FROM purchase_order_line WHERE order_id IN (SELECT id FROM _test_po));
DELETE FROM mail_message WHERE model = 'purchase.order' AND res_id IN (SELECT id FROM _test_po);
DELETE FROM mail_followers WHERE res_model = 'purchase.order' AND res_id IN (SELECT id FROM _test_po);
DELETE FROM ir_attachment WHERE res_model = 'purchase.order' AND res_id IN (SELECT id FROM _test_po);
DELETE FROM purchase_order_line WHERE order_id IN (SELECT id FROM _test_po);
DELETE FROM purchase_order WHERE id IN (SELECT id FROM _test_po);

-- ---- 6. CRM ----
UPDATE calendar_event SET opportunity_id = NULL
WHERE opportunity_id IN (SELECT id FROM crm_lead WHERE name LIKE '[TEST-%');
DELETE FROM mail_message WHERE model = 'crm.lead' AND res_id IN (SELECT id FROM crm_lead WHERE name LIKE '[TEST-%');
DELETE FROM mail_followers WHERE res_model = 'crm.lead' AND res_id IN (SELECT id FROM crm_lead WHERE name LIKE '[TEST-%');
DELETE FROM mail_activity WHERE res_model = 'crm.lead' AND res_id IN (SELECT id FROM crm_lead WHERE name LIKE '[TEST-%');
DELETE FROM ir_attachment WHERE res_model = 'crm.lead' AND res_id IN (SELECT id FROM crm_lead WHERE name LIKE '[TEST-%');
DELETE FROM crm_lead WHERE name LIKE '[TEST-%';

-- ---- 7. PROYECTO ----
DELETE FROM account_analytic_line WHERE task_id IN (SELECT id FROM project_task WHERE name LIKE '[TEST-%');
DELETE FROM project_task_stage_personal WHERE task_id IN (SELECT id FROM project_task WHERE name LIKE '[TEST-%');
UPDATE project_task SET parent_id = NULL WHERE parent_id IN (SELECT id FROM project_task WHERE name LIKE '[TEST-%');
DELETE FROM mail_message WHERE model = 'project.task' AND res_id IN (SELECT id FROM project_task WHERE name LIKE '[TEST-%');
DELETE FROM mail_followers WHERE res_model = 'project.task' AND res_id IN (SELECT id FROM project_task WHERE name LIKE '[TEST-%');
DELETE FROM mail_activity WHERE res_model = 'project.task' AND res_id IN (SELECT id FROM project_task WHERE name LIKE '[TEST-%');
DELETE FROM ir_attachment WHERE res_model = 'project.task' AND res_id IN (SELECT id FROM project_task WHERE name LIKE '[TEST-%');
DELETE FROM project_task WHERE name LIKE '[TEST-%';

-- ---- 8. GASTOS ----
CREATE TEMP TABLE _test_sheets AS
SELECT id FROM hr_expense_sheet WHERE name LIKE '[TEST-%';

UPDATE account_move SET expense_sheet_id = NULL WHERE expense_sheet_id IN (SELECT id FROM _test_sheets);
DELETE FROM mail_message WHERE model = 'hr.expense' AND res_id IN (SELECT id FROM hr_expense WHERE sheet_id IN (SELECT id FROM _test_sheets));
DELETE FROM ir_attachment WHERE res_model = 'hr.expense' AND res_id IN (SELECT id FROM hr_expense WHERE sheet_id IN (SELECT id FROM _test_sheets));
DELETE FROM hr_expense WHERE sheet_id IN (SELECT id FROM _test_sheets);
DELETE FROM mail_message WHERE model = 'hr.expense.sheet' AND res_id IN (SELECT id FROM _test_sheets);
DELETE FROM mail_followers WHERE res_model = 'hr.expense.sheet' AND res_id IN (SELECT id FROM _test_sheets);
DELETE FROM ir_attachment WHERE res_model = 'hr.expense.sheet' AND res_id IN (SELECT id FROM _test_sheets);
DELETE FROM hr_expense_sheet WHERE id IN (SELECT id FROM _test_sheets);

-- ---- 9. POS ----
CREATE TEMP TABLE _test_pos AS
SELECT id FROM pos_order WHERE name LIKE '[TEST-%' OR pos_reference LIKE '[TEST-%';

UPDATE pos_order SET account_move = NULL WHERE id IN (SELECT id FROM _test_pos);
DELETE FROM pos_payment WHERE pos_order_id IN (SELECT id FROM _test_pos);
DELETE FROM pos_pack_operation_lot WHERE pos_order_line_id IN (
    SELECT id FROM pos_order_line WHERE order_id IN (SELECT id FROM _test_pos));
DELETE FROM pos_order_line WHERE order_id IN (SELECT id FROM _test_pos);
DELETE FROM stock_move_line WHERE picking_id IN (
    SELECT id FROM stock_picking WHERE pos_order_id IN (SELECT id FROM _test_pos));
DELETE FROM stock_move WHERE picking_id IN (
    SELECT id FROM stock_picking WHERE pos_order_id IN (SELECT id FROM _test_pos));
DELETE FROM stock_picking WHERE pos_order_id IN (SELECT id FROM _test_pos);
DELETE FROM mail_message WHERE model = 'pos.order' AND res_id IN (SELECT id FROM _test_pos);
DELETE FROM mail_followers WHERE res_model = 'pos.order' AND res_id IN (SELECT id FROM _test_pos);
DELETE FROM pos_order WHERE id IN (SELECT id FROM _test_pos);

-- ---- 10. RRHH ----
-- Nominas
DELETE FROM mail_message WHERE model = 'hr.payslip' AND res_id IN (SELECT id FROM hr_payslip WHERE name LIKE '[TEST-%');
DELETE FROM hr_payslip WHERE name LIKE '[TEST-%';

-- Ausencias
DELETE FROM mail_message WHERE model = 'hr.leave' AND res_id IN (SELECT id FROM hr_leave WHERE name LIKE '[TEST-%');
DELETE FROM hr_leave WHERE name LIKE '[TEST-%';

-- Contratos
DELETE FROM mail_message WHERE model = 'hr.contract' AND res_id IN (SELECT id FROM hr_contract WHERE name LIKE '[TEST-%');
DELETE FROM hr_contract WHERE name LIKE '[TEST-%';

-- ---- 11. OTROS MODELOS ----
-- Activos fijos
DELETE FROM mail_message WHERE model = 'account.asset' AND res_id IN (SELECT id FROM account_asset WHERE name LIKE '[TEST-%');
DELETE FROM account_asset WHERE name LIKE '[TEST-%';

-- Mailing
DELETE FROM mailing_trace WHERE mass_mailing_id IN (SELECT id FROM mailing_mailing WHERE subject LIKE '[TEST-%');
DELETE FROM mailing_mailing WHERE subject LIKE '[TEST-%';

-- Mantenimiento
DELETE FROM maintenance_request WHERE name LIKE '[TEST-%';

-- Helpdesk
DELETE FROM account_analytic_line WHERE helpdesk_ticket_id IN (SELECT id FROM helpdesk_ticket WHERE name LIKE '[TEST-%');
DELETE FROM helpdesk_ticket WHERE name LIKE '[TEST-%';

-- Quality
DELETE FROM quality_check WHERE title LIKE '[TEST-%';

-- Fleet
DELETE FROM fleet_vehicle_log_services WHERE description LIKE '[TEST-%';

-- Stock scrap (sueltos, no vinculados a MRP)
DELETE FROM stock_scrap WHERE name LIKE '[TEST-%' AND production_id IS NULL;

-- ============================================================
-- LIMPIEZA DE TABLAS TEMPORALES
-- ============================================================
DROP TABLE IF EXISTS _test_move_line_ids, _test_payments, _test_moves,
    _test_pickings, _affected_stock_products, _test_productions,
    _test_so, _test_po, _test_sheets, _test_pos;

-- ============================================================
-- VERIFICACION FINAL
-- ============================================================
SELECT 'sale_order' AS tabla, COUNT(*) FROM sale_order WHERE name LIKE '[TEST-%' OR client_order_ref LIKE '[TEST-%'
UNION ALL SELECT 'purchase_order', COUNT(*) FROM purchase_order WHERE name LIKE '[TEST-%' OR partner_ref LIKE '[TEST-%'
UNION ALL SELECT 'account_move', COUNT(*) FROM account_move WHERE name LIKE '[TEST-%' OR ref LIKE '[TEST-%'
UNION ALL SELECT 'stock_picking', COUNT(*) FROM stock_picking WHERE origin LIKE '[TEST-%' OR name LIKE '[TEST-%'
UNION ALL SELECT 'crm_lead', COUNT(*) FROM crm_lead WHERE name LIKE '[TEST-%'
UNION ALL SELECT 'project_task', COUNT(*) FROM project_task WHERE name LIKE '[TEST-%'
UNION ALL SELECT 'mrp_production', COUNT(*) FROM mrp_production WHERE name LIKE '[TEST-%' OR origin LIKE '[TEST-%'
UNION ALL SELECT 'pos_order', COUNT(*) FROM pos_order WHERE name LIKE '[TEST-%' OR pos_reference LIKE '[TEST-%'
UNION ALL SELECT 'hr_payslip', COUNT(*) FROM hr_payslip WHERE name LIKE '[TEST-%'
UNION ALL SELECT 'hr_leave', COUNT(*) FROM hr_leave WHERE name LIKE '[TEST-%'
UNION ALL SELECT 'hr_contract', COUNT(*) FROM hr_contract WHERE name LIKE '[TEST-%'
UNION ALL SELECT 'account_asset', COUNT(*) FROM account_asset WHERE name LIKE '[TEST-%'
UNION ALL SELECT 'mailing_mailing', COUNT(*) FROM mailing_mailing WHERE subject LIKE '[TEST-%'
UNION ALL SELECT 'maintenance_request', COUNT(*) FROM maintenance_request WHERE name LIKE '[TEST-%'
UNION ALL SELECT 'helpdesk_ticket', COUNT(*) FROM helpdesk_ticket WHERE name LIKE '[TEST-%'
UNION ALL SELECT 'quality_check', COUNT(*) FROM quality_check WHERE title LIKE '[TEST-%'
UNION ALL SELECT 'partial_reconcile', COUNT(*) FROM account_partial_reconcile apr
    JOIN account_move_line aml ON apr.debit_move_id = aml.id OR apr.credit_move_id = aml.id
    JOIN account_move am ON aml.move_id = am.id WHERE am.ref LIKE '[TEST-%' OR am.name LIKE '[TEST-%';

-- Todos deben ser 0
ROLLBACK; -- Cambiar a COMMIT solo tras verificacion
```

