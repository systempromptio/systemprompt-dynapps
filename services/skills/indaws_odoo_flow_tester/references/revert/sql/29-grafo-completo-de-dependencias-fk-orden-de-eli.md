## Grafo Completo de Dependencias FK — Orden de Eliminacion (verificado contra Odoo 18.0)

Orden correcto de eliminacion (28 pasos verificados) para todos los modelos comunes.
Eliminar de arriba a abajo (los que dependen de otros primero, luego los independientes).
Este orden coincide con la seccion "Orden de dependencias FK" del Nivel 1.

```
Orden de Eliminacion (delete de arriba a abajo):
 1. account.partial.reconcile   (depende de account.move.line — debit/credit_move_id)
 2. account.full.reconcile      (auto-borrado al borrar partials — NUNCA borrar directo)
 3. stock.valuation.layer       (depende de stock.move + account.move, auto-ref primero)
 4. account.payment             (depende de account.move — move_id, set null)
 5. account.move.line           (depende de account.move — move_id, cascade en ORM)
 6. account.move                (facturas, asientos)
 7. stock.move.line             (depende de stock.move — move_id, set null)
 8. stock.move                  (depende de stock.picking — picking_id, set null)
      8a: DELETE stock_move_move_rel (M2M move_orig/dest_ids)
      8b: DELETE stock_move
 9. stock.scrap                 (depende de stock.picking, mrp.production)
10. stock.package_level         (depende de stock.picking)
11. stock.picking               (auto-ref: backorder_id — UPDATE NULL primero)
12. mrp.workcenter.productivity (depende de mrp.workorder)
13. mrp.workorder               (depende de mrp.production — restrict)
14. mrp.unbuild                 (depende de mrp.production — mo_id)
15. mrp.production              (indirecta via stock.move)
16. pos.payment                 (depende de pos.order — cascade)
17. pos.order.line              (depende de pos.order — cascade)
18. pos.order                   (depende de stock.picking, account.move)
19. sale.order.line             (depende de sale.order — cascade en PostgreSQL)
20. sale.order
21. purchase.order.line         (depende de purchase.order — cascade en PostgreSQL)
22. purchase.order
23. hr.expense                  (depende de hr.expense.sheet — set null)
24. hr.expense.sheet            (depende de account.move)
25. hr.payslip                  (depende de account.move — move_id)
26. hr.leave                    (afecta resource.calendar.leaves)
27. mailing.trace               (depende de mailing.mailing)
28. mailing.mailing
```

**Modelos adicionales** (sin FK criticas, borrar despues):
```
29. hr.contract, account.asset, maintenance.request, helpdesk.ticket
30. quality.check, fleet.vehicle.log.services, crm.lead, project.task
```

**Relaciones cascade vs set null (resumen):**
- **CASCADE:** sale_order_line→sale_order, purchase_order_line→purchase_order, account_move_line→account_move, pos_order_line→pos_order, pos_payment→pos_order
- **SET NULL:** stock_move→stock_picking, stock_move_line→stock_move, stock_valuation_layer→stock_move, account_payment→account_move, hr_expense→hr_expense_sheet
- **RESTRICT:** mrp_workorder→mrp_production

### Consultas SQL por nivel de dependencia (con prefijo [TEST-%)

```sql
BEGIN;

-- ============================================================
-- NIVEL 1: Reconciliaciones (depende de account.move.line)
-- ============================================================
DELETE FROM account_partial_reconcile WHERE id IN (
    SELECT apr.id FROM account_partial_reconcile apr
    JOIN account_move_line aml ON (apr.debit_move_id = aml.id OR apr.credit_move_id = aml.id)
    JOIN account_move am ON aml.move_id = am.id
    WHERE am.ref LIKE '[TEST-%' OR am.name LIKE '[TEST-%'
);

DELETE FROM account_full_reconcile WHERE id NOT IN (
    SELECT DISTINCT full_reconcile_id
    FROM account_partial_reconcile
    WHERE full_reconcile_id IS NOT NULL
);

-- ============================================================
-- NIVEL 2: Pagos (depende de account.move)
-- ============================================================
DELETE FROM account_payment WHERE id IN (
    SELECT ap.id FROM account_payment ap
    JOIN account_move am ON ap.move_id = am.id
    WHERE am.ref LIKE '[TEST-%' OR am.name LIKE '[TEST-%'
);

-- ============================================================
-- NIVEL 3: Extractos bancarios (depende de account.move)
-- ============================================================
DELETE FROM account_bank_statement_line WHERE id IN (
    SELECT absl.id FROM account_bank_statement_line absl
    JOIN account_move am ON absl.move_id = am.id
    WHERE am.ref LIKE '[TEST-%' OR am.name LIKE '[TEST-%'
);

-- ============================================================
-- NIVEL 4: Lineas de asiento (depende de account.move)
-- ============================================================
DELETE FROM account_move_line WHERE move_id IN (
    SELECT id FROM account_move
    WHERE ref LIKE '[TEST-%' OR name LIKE '[TEST-%'
);

-- ============================================================
-- NIVEL 5: Asientos contables
-- ============================================================
DELETE FROM account_move WHERE ref LIKE '[TEST-%' OR name LIKE '[TEST-%';

-- ============================================================
-- NIVEL 6: Lineas de venta (depende de sale.order)
-- ============================================================
DELETE FROM sale_order_line WHERE order_id IN (
    SELECT id FROM sale_order
    WHERE name LIKE '[TEST-%' OR client_order_ref LIKE '[TEST-%'
);

-- ============================================================
-- NIVEL 7: Pedidos de venta
-- ============================================================
DELETE FROM sale_order WHERE name LIKE '[TEST-%' OR client_order_ref LIKE '[TEST-%';

-- ============================================================
-- NIVEL 8: Lineas de compra (depende de purchase.order)
-- ============================================================
DELETE FROM purchase_order_line WHERE order_id IN (
    SELECT id FROM purchase_order
    WHERE name LIKE '[TEST-%' OR partner_ref LIKE '[TEST-%'
);

-- ============================================================
-- NIVEL 9: Pedidos de compra
-- ============================================================
DELETE FROM purchase_order WHERE name LIKE '[TEST-%' OR partner_ref LIKE '[TEST-%';

-- ============================================================
-- NIVEL 10: Movimientos de stock (depende de stock.picking)
-- ============================================================
DELETE FROM stock_move_line WHERE picking_id IN (
    SELECT id FROM stock_picking
    WHERE origin LIKE '[TEST-%' OR name LIKE '[TEST-%'
);

DELETE FROM stock_move WHERE picking_id IN (
    SELECT id FROM stock_picking
    WHERE origin LIKE '[TEST-%' OR name LIKE '[TEST-%'
);

-- ============================================================
-- NIVEL 11: Albaranes
-- ============================================================
DELETE FROM stock_picking WHERE origin LIKE '[TEST-%' OR name LIKE '[TEST-%';

-- ============================================================
-- NIVEL 12: Capas de valoracion (depende de stock.move)
-- ============================================================
UPDATE stock_valuation_layer SET stock_valuation_layer_id = NULL
WHERE stock_valuation_layer_id IN (
    SELECT svl.id FROM stock_valuation_layer svl
    JOIN stock_move sm ON svl.stock_move_id = sm.id
    JOIN stock_picking sp ON sm.picking_id = sp.id
    WHERE sp.origin LIKE '[TEST-%' OR sp.name LIKE '[TEST-%'
);

DELETE FROM stock_valuation_layer WHERE stock_move_id IN (
    SELECT sm.id FROM stock_move sm
    JOIN stock_picking sp ON sm.picking_id = sp.id
    WHERE sp.origin LIKE '[TEST-%' OR sp.name LIKE '[TEST-%'
);

-- ============================================================
-- NIVEL 13: Tareas de proyecto (depende de project.project)
-- ============================================================
DELETE FROM account_analytic_line WHERE task_id IN (
    SELECT id FROM project_task WHERE name LIKE '[TEST-%'
);
UPDATE project_task SET parent_id = NULL
WHERE parent_id IN (SELECT id FROM project_task WHERE name LIKE '[TEST-%');
DELETE FROM project_task WHERE name LIKE '[TEST-%';

-- ============================================================
-- NIVEL 14: Informes de gastos (depende de hr.expense)
-- ============================================================
DELETE FROM hr_expense WHERE sheet_id IN (
    SELECT id FROM hr_expense_sheet WHERE name LIKE '[TEST-%'
);
DELETE FROM hr_expense_sheet WHERE name LIKE '[TEST-%';

-- ============================================================
-- NIVEL 15: Gastos individuales
-- ============================================================
DELETE FROM hr_expense WHERE name LIKE '[TEST-%' AND sheet_id IS NULL;

ROLLBACK; -- Cambiar a COMMIT solo tras verificacion
```

---

