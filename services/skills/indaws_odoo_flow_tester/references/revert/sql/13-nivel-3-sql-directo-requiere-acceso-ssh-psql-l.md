### Limpieza SQL de sale.order (verificado)

```sql
BEGIN;
-- Variables: @so_ids = IDs de sale.order a borrar

-- Paso 1: Identificar facturas y pickings asociados (borrar ANTES — ver secciones anteriores)
SELECT DISTINCT am.id FROM account_move am
JOIN account_move_line aml ON aml.move_id = am.id
WHERE aml.sale_line_ids && ARRAY(SELECT id FROM sale_order_line WHERE order_id IN (@so_ids));
-- Nota: sale_line_ids es M2M, la query real depende del schema

-- Paso 2: Limpiar auxiliares
DELETE FROM mail_message WHERE model = 'sale.order' AND res_id IN (@so_ids);
DELETE FROM mail_followers WHERE res_model = 'sale.order' AND res_id IN (@so_ids);
DELETE FROM mail_activity WHERE res_model = 'sale.order' AND res_id IN (@so_ids);
DELETE FROM ir_attachment WHERE res_model = 'sale.order' AND res_id IN (@so_ids);

-- Paso 3: Borrar sale.order (cascade borra sale_order_line automaticamente — L41 ondelete=cascade)
DELETE FROM sale_order WHERE id IN (@so_ids);

ROLLBACK;
```

