### Limpieza SQL completa de flujo Sale-to-Cash (verificado)

```sql
BEGIN;
-- FLUJO COMPLETO: Venta → Factura → Pago → Albaran
-- Variables: @so_ids = IDs de sale.order origen

-- A. Identificar todos los registros relacionados
WITH so_lines AS (
    SELECT id FROM sale_order_line WHERE order_id IN (@so_ids)
),
invoices AS (
    SELECT DISTINCT am.id FROM account_move am
    JOIN account_move_line aml ON aml.move_id = am.id
    WHERE aml.sale_line_ids && ARRAY(SELECT id FROM so_lines)
),
pickings AS (
    SELECT DISTINCT sm.picking_id FROM stock_move sm
    WHERE sm.sale_line_id IN (SELECT id FROM so_lines) AND sm.picking_id IS NOT NULL
)
SELECT 'invoices' as tipo, id FROM invoices
UNION ALL
SELECT 'pickings', picking_id FROM pickings;

-- B. Ejecutar en orden:
-- 1. Limpiar pagos y reconciliaciones (seccion account.move pasos 1-3 para @invoice_ids)
-- 2. Limpiar facturas (seccion account.move pasos 4-8 para @invoice_ids)
-- 3. Limpiar albaranes (seccion stock.picking para @picking_ids)
-- 4. Borrar sale.order (seccion sale.order)

ROLLBACK;
```

---

