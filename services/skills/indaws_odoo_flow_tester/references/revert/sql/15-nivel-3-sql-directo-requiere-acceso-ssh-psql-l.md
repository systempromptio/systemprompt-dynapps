### Limpieza SQL de stock.picking (verificado)

```sql
BEGIN;
-- Variables: @picking_ids = IDs de stock_picking a borrar

-- Paso 1: Obtener stock_move asociados
-- @move_ids = SELECT id FROM stock_move WHERE picking_id IN (@picking_ids);

-- Paso 2: Limpiar valuation layers de los moves (auto-ref primero)
UPDATE stock_valuation_layer SET stock_valuation_layer_id = NULL
WHERE stock_move_id IN (@move_ids) AND stock_valuation_layer_id IS NOT NULL;
DELETE FROM stock_valuation_layer WHERE stock_move_id IN (@move_ids);

-- Paso 3: Borrar stock_move_line
DELETE FROM stock_move_line WHERE move_id IN (@move_ids);

-- Paso 4: Limpiar M2M de move origins/destinations
DELETE FROM stock_move_move_rel WHERE move_id IN (@move_ids) OR move_orig_id IN (@move_ids);

-- Paso 5: Borrar stock_move
DELETE FROM stock_move WHERE id IN (@move_ids);

-- Paso 6: Borrar package levels
DELETE FROM stock_package_level WHERE picking_id IN (@picking_ids);

-- Paso 7: Borrar scrap asociados
DELETE FROM stock_scrap WHERE picking_id IN (@picking_ids);

-- Paso 8: Limpiar auto-referencias de picking (backorder_id)
UPDATE stock_picking SET backorder_id = NULL WHERE backorder_id IN (@picking_ids);

-- Paso 9: Limpiar auxiliares
DELETE FROM mail_message WHERE model = 'stock.picking' AND res_id IN (@picking_ids);
DELETE FROM mail_followers WHERE res_model = 'stock.picking' AND res_id IN (@picking_ids);
DELETE FROM ir_attachment WHERE res_model = 'stock.picking' AND res_id IN (@picking_ids);

-- Paso 10: Borrar pickings
DELETE FROM stock_picking WHERE id IN (@picking_ids);

ROLLBACK;
```

