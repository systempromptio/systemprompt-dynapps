### Limpieza SQL de purchase.order (verificado)

```sql
BEGIN;
-- Variables: @po_ids = IDs de purchase_order a borrar
-- NOTA: Borrar facturas y recepciones asociadas PRIMERO (secciones account.move y stock.picking)

-- Paso 1: Limpiar FKs desde account.move.line
UPDATE account_move_line SET purchase_line_id = NULL
WHERE purchase_line_id IN (SELECT id FROM purchase_order_line WHERE order_id IN (@po_ids));

-- Paso 2: Limpiar auxiliares
DELETE FROM mail_message WHERE model = 'purchase.order' AND res_id IN (@po_ids);
DELETE FROM mail_followers WHERE res_model = 'purchase.order' AND res_id IN (@po_ids);
DELETE FROM mail_activity WHERE res_model = 'purchase.order' AND res_id IN (@po_ids);
DELETE FROM ir_attachment WHERE res_model = 'purchase.order' AND res_id IN (@po_ids);

-- Paso 3: Borrar purchase.order (cascade borra purchase_order_line — L47 ondelete=cascade)
DELETE FROM purchase_order WHERE id IN (@po_ids);

ROLLBACK;
```

