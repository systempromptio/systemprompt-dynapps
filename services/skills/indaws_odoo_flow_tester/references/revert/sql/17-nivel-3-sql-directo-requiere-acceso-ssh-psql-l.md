### Limpieza SQL de pos.order (verificado)

```sql
BEGIN;
-- Variables: @pos_ids = IDs de pos_order a borrar

-- Paso 1: Borrar pagos POS (cascade en ORM, explicito por seguridad — pos_payment.py L20)
DELETE FROM pos_payment WHERE pos_order_id IN (@pos_ids);

-- Paso 2: Borrar lineas POS (cascade en ORM — pos_order.py L1401)
DELETE FROM pos_order_line WHERE order_id IN (@pos_ids);

-- Paso 3: Limpiar auxiliares
DELETE FROM mail_message WHERE model = 'pos.order' AND res_id IN (@pos_ids);
DELETE FROM mail_followers WHERE res_model = 'pos.order' AND res_id IN (@pos_ids);

-- Paso 4: Borrar pos_order
-- NOTA: Borrar account.move y stock.picking asociados PRIMERO
DELETE FROM pos_order WHERE id IN (@pos_ids);

ROLLBACK;
```

